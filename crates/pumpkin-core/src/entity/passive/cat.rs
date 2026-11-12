use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use pumpkin_data::block_properties::WhiteBedLikeProperties;
use pumpkin_data::cat_sound_variant::CatSoundVariant;
use pumpkin_data::cat_variant::{CatSpawnCondition, CatVariant};
use pumpkin_data::data_component_impl::DyeImpl;
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::custom_sound::CustomSound;
use crate::entity::item::ItemEntity;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::behavior::{
        feline::{self, SPRINT_SPEED, TEMPT_SPEED},
        fleeing::{self, Flee},
        hunting::{self, Hunt},
    },
    ai::goal::{
        Controls, Goal, GoalReach, active_target::TargetCondition,
        cat_lie_on_block::CatLieOnBlockGoal, cat_sit_on_block::CatSitOnBlockGoal,
        escape_danger::EscapeDangerGoal, follow_owner::FollowOwnerGoal,
        sit_when_ordered_to::SitWhenOrderedToGoal, swim::SwimGoal, tempt::TemptGoal,
    },
    ai::pathfinder::NavigatorGoal,
    ai::target_match::TargetMatch,
    mob::{
        Mob, MobEntity, drop_from_gift_loot_table,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::{
        animal::{Animal, get_mixed_color},
        tamable::{TamableAnimal, TamableData, is_untamed},
    },
    player::Player,
};
use crate::world::World;

const CAT_FLEES: &[Flee] = &[feline::flee_players(4, is_untamed)];

const CAT_HUNTS: &[Hunt] = &[
    Hunt::new(1, TargetMatch::Type(&EntityType::RABBIT), false).when(TargetCondition::Untamed),
    hunting::baby_turtles(1, false).when(TargetCondition::Untamed),
];

const AMBIENT_SOUND_INTERVAL: i32 = 120;
const BEG_FOR_FOOD_INTERVAL: i32 = 100;
const LIE_DOWN_PURR_INTERVAL: i32 = 5;

pub struct CatEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
    pub collar_color: AtomicU8,
    pub tamable_data: TamableData,
    pub ageable_data: AgeableData,
    pub is_lying: AtomicBool,
    pub relax_state_one: AtomicBool,
}

impl CatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cat = Self {
            mob_entity,
            variant: AtomicU8::new(CatVariant::Black.id()),
            sound_variant: AtomicU8::new(0), // Default to classic
            collar_color: AtomicU8::new(14), // Default to red
            tamable_data: TamableData::default(),
            ageable_data: AgeableData::default(),
            is_lying: AtomicBool::new(false),
            relax_state_one: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(cat);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            // Goal 1: SwimGoal (FloatGoal)
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Goal 1: TamableAnimalPanicGoal (EscapeDangerGoal)
            goal_selector.add_goal(1, EscapeDangerGoal::tamable(1.5));
            // Goal 2: SitWhenOrderedToGoal
            goal_selector.add_goal(2, Box::new(SitWhenOrderedToGoal::new()));
            goal_selector.add_goal(
                3,
                Box::new(CatRelaxOnOwnerGoal::new(Arc::downgrade(&mob_arc))),
            );
            // Goal 4: CatTemptGoal
            goal_selector.add_goal(4, Box::new(CatTemptGoal::new(Arc::downgrade(&mob_arc))));
            // Goal 4: CatAvoidEntityGoal (when untamed)
            fleeing::apply(goal_selector, CAT_FLEES);
            goal_selector.add_goal(5, CatLieOnBlockGoal::new(Arc::downgrade(&mob_arc), 1.1, 8));
            // Goal 6: FollowOwnerGoal
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 5.0));
            goal_selector.add_goal(7, CatSitOnBlockGoal::new(Arc::downgrade(&mob_arc), 0.8));
            // Goal 8-12: LeapAtTarget, OcelotAttack, Breed, Stroll, LookAtPlayer
            feline::apply_tail(goal_selector, mob_weak, 8);
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            // NonTameRandomTargetGoal for Rabbit and Turtle
            hunting::apply(target_selector, &mob_arc.mob_entity, CAT_HUNTS);
        };

        mob_arc
    }

    pub fn get_tame_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_in_sitting_pose() {
            flags |= 0x01;
        }
        if self.is_tame() {
            flags |= 0x04;
        }
        flags
    }

    pub fn is_lying(&self) -> bool {
        self.is_lying.load(Ordering::Relaxed)
    }

    pub fn is_relax_state_one(&self) -> bool {
        self.relax_state_one.load(Ordering::Relaxed)
    }

    pub fn get_collar_color(&self) -> u8 {
        self.collar_color.load(Ordering::Relaxed)
    }

    pub fn set_collar_color(&self, color: u8) {
        self.collar_color.store(color, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_COLLAR_COLOR,
            VarInt(color as i32),
        );
    }

    pub fn set_variant(&self, variant: u8) {
        self.variant.store(variant, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_VARIANT,
            VarInt(variant as i32),
        );
    }

    pub fn set_lying(&self, lying: bool) {
        self.is_lying.store(lying, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::cat::IS_LYING, lying);
    }

    pub fn set_relax_state_one(&self, relax: bool) {
        self.relax_state_one.store(relax, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::cat::RELAX_STATE_ONE, relax);
    }

    pub fn set_sound_variant(&self, sound_variant: CatSoundVariant) {
        self.sound_variant
            .store(sound_variant as u8, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::SOUND_VARIANT,
            VarInt(sound_variant as u8 as i32),
        );
    }

    pub fn hiss(&self) {
        self.make_sound(
            self.get_sound_variant()
                .hiss_sound(self.mob_entity.living_entity.is_baby()),
        );
    }

    fn get_sound_variant(&self) -> CatSoundVariant {
        CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed)).unwrap_or_default()
    }

    fn handle_lie_down(&self) {
        if (self.is_lying() || self.is_relax_state_one())
            && self.get_entity().tick_count.load(Ordering::Relaxed) % LIE_DOWN_PURR_INTERVAL == 0
        {
            let mut random = self.get_random();
            let volume = 0.6 + 0.4 * (random.random::<f32>() - random.random::<f32>());
            self.play_entity_sound(
                self.get_sound_variant()
                    .purr_sound(self.mob_entity.living_entity.is_baby()),
                volume,
                1.0,
            );
        }
    }

    fn try_to_tame(&self, player: &Arc<Player>) {
        let entity = self.get_entity();
        let world = entity.world.load();
        if self.get_random().random_range(0..3) == 0 && self.approve_taming(player) {
            TamableAnimal::tame(self, player.gameprofile.id);
            self.set_ordered_to_sit(true);
            world.send_entity_status(
                entity,
                EntityStatus::TamingSucceeded,
                ActorEventID::TamingSucceeded,
            );
        } else {
            world.send_entity_status(
                entity,
                EntityStatus::TamingFailed,
                ActorEventID::TamingFailed,
            );
        }
    }

    fn select_variant_to_spawn(&self, world: &World) -> Option<CatVariant> {
        let pos = self.get_entity().block_pos.load();
        let mut entries: Vec<_> = CatVariant::all()
            .iter()
            .flat_map(|variant| {
                variant
                    .spawn_selectors()
                    .iter()
                    .map(move |selector| (*variant, selector))
            })
            .collect();
        entries.sort_by_key(|(_, selector)| std::cmp::Reverse(selector.priority));
        let mut highest_matched_priority = i32::MIN;
        entries.retain(|(_, selector)| {
            if selector.priority < highest_matched_priority {
                return false;
            }
            let matches = match selector.condition {
                None => true,
                Some(CatSpawnCondition::Structure(structures)) => {
                    world.has_structure_piece_at(pos, structures)
                }
                Some(CatSpawnCondition::MoonBrightness { min, max }) => {
                    let brightness = f64::from(world.get_moon_brightness(&pos));
                    min.is_none_or(|min| brightness >= min)
                        && max.is_none_or(|max| brightness <= max)
                }
            };
            if matches {
                highest_matched_priority = selector.priority;
            }
            matches
        });
        if entries.is_empty() {
            return None;
        }
        Some(entries[self.get_random().random_range(0..entries.len())].0)
    }
}

impl CustomSound for CatEntity {
    fn death_sound(&self) -> Option<pumpkin_data::sound::Sound> {
        Some(
            self.get_sound_variant()
                .death_sound(self.mob_entity.living_entity.is_baby()),
        )
    }

    fn hurt_sound(&self) -> Option<pumpkin_data::sound::Sound> {
        Some(
            self.get_sound_variant()
                .hurt_sound(self.mob_entity.living_entity.is_baby()),
        )
    }

    fn sound_category(&self) -> SoundCategory {
        self.sound_source()
    }
}

impl Animal for CatEntity {
    fn play_eating_sound(&self) {
        self.play_entity_sound(
            self.get_sound_variant()
                .eat_sound(self.mob_entity.living_entity.is_baby()),
            1.0,
            1.0,
        );
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_CAT_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        self.is_tame()
            && partner
                .cast_any()
                .downcast_ref::<Self>()
                .is_some_and(TamableAnimal::is_tame)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let baby = Self::new(Entity::new(
            world.clone(),
            entity.pos.load(),
            &EntityType::CAT,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            if self.get_random().random::<bool>() {
                baby.set_variant(self.variant.load(Ordering::Relaxed));
            } else {
                baby.set_variant(partner.variant.load(Ordering::Relaxed));
            }
            if self.is_tame() {
                baby.set_owner(self.get_owner());
                baby.set_tame(true);
                let color = get_mixed_color(
                    &world,
                    DyeColor::from(self.get_collar_color()),
                    DyeColor::from(partner.get_collar_color()),
                );
                baby.set_collar_color(color.id());
            }
        }
        baby
    }
}

impl AgeableMob for CatEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl TamableAnimal for CatEntity {
    fn get_tamable_data(&self) -> &TamableData {
        &self.tamable_data
    }
}

impl Mob for CatEntity {
    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self.finalize_spawn_super(group_data);
        if let Some(variant) = self.select_variant_to_spawn(world) {
            self.set_variant(variant.id());
        }
        let sound_variants = CatSoundVariant::ALL;
        self.set_sound_variant(
            sound_variants[self.get_random().random_range(0..sound_variants.len())],
        );
        group_data
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_tamable(&self) -> Option<&dyn TamableAnimal> {
        Some(self)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_cat(&self) -> Option<&CatEntity> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        let sound_variant = self.get_sound_variant();
        let is_baby = self.mob_entity.living_entity.is_baby();
        Some(if self.is_tame() {
            if self.mob_entity.is_in_love() {
                sound_variant.purr_sound(is_baby)
            } else if self.get_random().random_range(0..4) == 0 {
                sound_variant.purreow_sound(is_baby)
            } else {
                sound_variant.ambient_sound(is_baby)
            }
        } else {
            sound_variant.stray_ambient_sound(is_baby)
        })
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        AMBIENT_SOUND_INTERVAL
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if !self.is_tame()
            && self.get_entity().tick_count.load(Ordering::Relaxed) % BEG_FOR_FOOD_INTERVAL == 0
            && self.mob_entity.goals_selector.is_running::<CatTemptGoal>()
        {
            self.play_entity_sound(
                self.get_sound_variant()
                    .beg_for_food_sound(self.mob_entity.living_entity.is_baby()),
                1.0,
                1.0,
            );
        }
        self.handle_lie_down();
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let (pose, sprinting) = {
            let steered = self.mob_entity.steered();
            let move_control = steered.move_control();
            if move_control.has_wanted() {
                let speed = move_control.get_speed_modifier();
                if speed == TEMPT_SPEED {
                    (EntityPose::Crouching, false)
                } else if speed == SPRINT_SPEED {
                    (EntityPose::Standing, true)
                } else {
                    (EntityPose::Standing, false)
                }
            } else {
                (EntityPose::Standing, false)
            }
        };
        let entity = self.get_entity();
        entity.set_pose(pose);
        if entity.is_sprinting() != sprinting {
            entity.set_sprinting(sprinting);
        }
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let mut meta = crate::entity::mob::MobMetadata::new();
        if self.get_entity().age.load(Ordering::Relaxed) < 0 {
            meta = meta.baby(true);
        }
        if self.is_tame() {
            meta = meta.tamed(true);
        }
        if self.is_in_sitting_pose() {
            meta = meta.sitting(true);
        }
        meta = meta.collar_color(
            pumpkin_data::tracked_data::cat::CAT_COLLAR_COLOR,
            pumpkin_data::dye_color::DyeColor::from(self.collar_color.load(Ordering::Relaxed)),
        );
        Some(meta)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let variant_id = u32::from(self.variant.load(Ordering::Relaxed));
        let variant_str =
            self.resolve_dynamic_entry_id("minecraft:cat_variant", variant_id, |id| {
                CatVariant::all()
                    .get(id as usize)
                    .map_or("minecraft:tabby", CatVariant::asset_id)
                    .to_string()
            });
        nbt.put_string("variant", variant_str);
        let sound_variant = CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        nbt.put_string(
            "sound_variant",
            format!("minecraft:{}", sound_variant.to_name()),
        );
        nbt.put_byte(
            "CollarColor",
            self.collar_color.load(Ordering::Relaxed) as i8,
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("variant")
            && let Some(id) =
                self.resolve_dynamic_raw_id("minecraft:cat_variant", variant_str, |n| {
                    CatVariant::from_name(n).map(|v| u32::from(v.id()))
                })
            && let Ok(variant_id) = u8::try_from(id)
        {
            self.set_variant(variant_id);
        }
        if let Some(sound_str) = nbt.get_string("sound_variant")
            && let Some(sound_variant) = CatSoundVariant::from_name(sound_str)
        {
            self.sound_variant
                .store(sound_variant as u8, Ordering::Relaxed);
        }
        if let Some(collar) = nbt.get_byte("CollarColor") {
            self.collar_color.store(collar as u8, Ordering::Relaxed);
        } else if let Some(collar_int) = nbt.get_int("CollarColor") {
            self.collar_color.store(collar_int as u8, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(id) = self.resolve_dynamic_raw_id("minecraft:cat_variant", name, |n| {
            CatVariant::from_name(n).map(|v| u32::from(v.id()))
        }) && let Ok(variant_id) = u8::try_from(id)
        {
            self.set_variant(variant_id);
        }
    }

    fn mob_set_sound_variant_name(&self, name: &str) {
        if let Some(v) = CatSoundVariant::from_name(name) {
            self.set_sound_variant(v);
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::cat::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::TAMEABLE_FLAGS,
            self.get_tame_flags(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::OWNER_UUID,
            self.get_owner(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_VARIANT,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::IS_LYING,
            self.is_lying.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::RELAX_STATE_ONE,
            self.relax_state_one.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_COLLAR_COLOR,
            VarInt(self.collar_color.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::SOUND_VARIANT,
            VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if self.is_tame() {
            if self.is_owned_by(&player.gameprofile.id) {
                if item_stack
                    .get_item()
                    .has_tag(&tag::Item::MINECRAFT_CAT_COLLAR_DYES)
                {
                    if let Some(dye) = item_stack.get_data_component::<DyeImpl>()
                        && dye.color.id() != self.get_collar_color()
                    {
                        self.set_collar_color(dye.color.id());
                        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                        self.mob_entity.set_persistence_required();
                        return true;
                    }
                } else if self.is_food(item_stack)
                    && self.mob_entity.living_entity.health.load()
                        < self.mob_entity.living_entity.get_max_health()
                {
                    self.feed(player, item_stack, 1.0, 1.0);
                    return true;
                }

                let parent_interaction = self.animal_interact(player, item_stack);
                if !parent_interaction {
                    self.order_to_sit(!self.is_ordered_to_sit());
                    return true;
                }
                return parent_interaction;
            }
        } else if self.is_food(item_stack) {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.try_to_tame(player);
            self.mob_entity.set_persistence_required();
            self.play_eating_sound();
            return true;
        }

        let interact = self.animal_interact(player, item_stack);
        if interact {
            self.mob_entity.set_persistence_required();
        }
        interact
    }

    fn has_global_tick_reach(&self) -> bool {
        self.mob_entity
            .goals_selector
            .is_running::<CatRelaxOnOwnerGoal>()
    }

    fn mob_get_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        use pumpkin_data::{
            data_component::DataComponent,
            data_component_impl::{CatCollarImpl, CatSoundVariantImpl, CatVariantImpl},
        };
        match component {
            DataComponent::CatVariant => {
                let variant_id = u32::from(self.variant.load(Ordering::Relaxed));
                Some(Box::new(CatVariantImpl {
                    value: self
                        .resolve_dynamic_entry_id("minecraft:cat_variant", variant_id, |id| {
                            CatVariant::all()
                                .get(id as usize)
                                .map_or("minecraft:tabby", CatVariant::asset_id)
                                .to_string()
                        })
                        .into(),
                }))
            }
            DataComponent::CatSoundVariant => Some(Box::new(CatSoundVariantImpl {
                value: format!("minecraft:{}", self.get_sound_variant().to_name()).into(),
            })),
            DataComponent::CatCollar => Some(Box::new(CatCollarImpl {
                value: DyeColor::from(self.get_collar_color()).name().into(),
            })),
            _ => None,
        }
    }

    fn mob_is_stepping_carefully(&self) -> bool {
        let entity = self.get_entity();
        entity.pose.load() == EntityPose::Crouching || entity.is_sneaking()
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_tame() && self.get_entity().tick_count.load(Ordering::Relaxed) > 2400
    }
}

const CAT_MORNING_GIFT_LOOT_TABLE: &str = "minecraft:gameplay/cat_morning_gift";
const RELAX_MAX_OWNER_DISTANCE_SQ: f64 = 100.0;
const RELAX_ON_BED_DISTANCE_SQ: f64 = 2.5;
const RELAX_SPEED: f64 = 1.1;
const RELAX_LIE_DOWN_TICKS: i32 = 16;

struct CatRelaxOnOwnerGoal {
    cat: Weak<CatEntity>,
    owner_player: Option<Weak<Player>>,
    goal_pos: Option<BlockPos>,
    on_bed_ticks: i32,
}

impl CatRelaxOnOwnerGoal {
    const fn new(cat: Weak<CatEntity>) -> Self {
        Self {
            cat,
            owner_player: None,
            goal_pos: None,
            on_bed_ticks: 0,
        }
    }

    fn space_is_occupied(cat: &CatEntity, goal_pos: &BlockPos) -> bool {
        let entity = cat.get_entity();
        let bounding_box = BoundingBox::from_block(goal_pos).expand_all(2.0);
        entity
            .world
            .load()
            .get_entities_at_box(&bounding_box)
            .iter()
            .any(|other| {
                other.get_entity().entity_id != entity.entity_id
                    && other
                        .get_mob()
                        .and_then(Mob::as_cat)
                        .is_some_and(|other| other.is_lying() || other.is_relax_state_one())
            })
    }

    fn move_to_goal(cat: &CatEntity, goal_pos: &BlockPos) {
        cat.mob_entity
            .steered()
            .navigator()
            .set_progress(NavigatorGoal {
                current_progress: cat.get_entity().pos.load(),
                destination: Vector3::new(
                    f64::from(goal_pos.0.x),
                    f64::from(goal_pos.0.y),
                    f64::from(goal_pos.0.z),
                ),
                speed: RELAX_SPEED,
            });
    }

    fn give_morning_gift(cat: &CatEntity) {
        let entity = cat.get_entity();
        let world = entity.world.load_full();
        let mut rng = cat.get_random();
        let origin = entity.get_leash_holder().map_or_else(
            || entity.block_pos.load(),
            |holder| holder.get_entity().block_pos.load(),
        );
        let target = Vector3::new(
            f64::from(origin.0.x + rng.random_range(0..11) - 5),
            f64::from(origin.0.y + rng.random_range(0..5) - 2),
            f64::from(origin.0.z + rng.random_range(0..11) - 5),
        );
        cat.mob_entity
            .living_entity
            .random_teleport(cat, target, false, |block| {
                block.has_tag(&tag::Block::MINECRAFT_CAT_DOES_NOT_TELEPORT_TO)
            });
        drop(rng);
        let cat_pos = entity.block_pos.load();
        let body_yaw = entity.body_yaw.load().to_radians();
        drop_from_gift_loot_table(cat, CAT_MORNING_GIFT_LOOT_TABLE, |item_stack| {
            let drop_pos = Vector3::new(
                f64::from(cat_pos.0.x) - f64::from(body_yaw.sin()),
                f64::from(cat_pos.0.y),
                f64::from(cat_pos.0.z) + f64::from(body_yaw.cos()),
            );
            let item_entity = ItemEntity::new(
                Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                item_stack,
            );
            world.spawn_entity(Arc::new(item_entity));
        });
    }
}

impl Goal for CatRelaxOnOwnerGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cat) = self.cat.upgrade() else {
            return false;
        };
        if !cat.is_tame() || cat.is_ordered_to_sit() {
            return false;
        }
        let entity = cat.get_entity();
        let world = entity.world.load();
        let Some(owner) = cat.get_owner().and_then(|id| world.get_player_by_uuid(id)) else {
            return false;
        };
        self.owner_player = Some(Arc::downgrade(&owner));
        if !owner.is_sleeping()
            || entity
                .pos
                .load()
                .squared_distance_to_vec(&owner.get_entity().pos.load())
                > RELAX_MAX_OWNER_DISTANCE_SQ
        {
            return false;
        }
        let owner_pos = owner.get_entity().block_pos.load();
        let (block, state) = world.get_block_and_state(&owner_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_CATS_CAN_LIE_ON) {
            return false;
        }
        let goal_pos = if block.has_tag(&tag::Block::MINECRAFT_BEDS) {
            let facing = WhiteBedLikeProperties::from_state_id(state.id).facing;
            owner_pos.offset(facing.opposite().to_offset())
        } else {
            owner_pos
        };
        self.goal_pos = Some(goal_pos);
        !Self::space_is_occupied(&cat, &goal_pos)
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cat) = self.cat.upgrade() else {
            return false;
        };
        cat.is_tame()
            && !cat.is_ordered_to_sit()
            && self
                .owner_player
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|owner| owner.is_sleeping())
            && self
                .goal_pos
                .is_some_and(|goal_pos| !Self::space_is_occupied(&cat, &goal_pos))
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(cat) = self.cat.upgrade()
            && let Some(goal_pos) = self.goal_pos
        {
            cat.set_in_sitting_pose(false);
            Self::move_to_goal(&cat, &goal_pos);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        let Some(cat) = self.cat.upgrade() else {
            return;
        };
        cat.set_lying(false);
        if let Some(owner) = self.owner_player.as_ref().and_then(Weak::upgrade)
            && owner.get_sleep_timer() >= Player::FULL_SLEEP_TICKS
        {
            let world = cat.get_entity().world.load();
            let gift_chance = world.environment_attributes().get_value_f32(
                EnvironmentAttribute::GameplayCatWakingUpGiftChance,
                &cat.get_entity().block_pos.load(),
            );
            if cat.get_random().random::<f32>() < gift_chance {
                Self::give_morning_gift(&cat);
            }
        }
        self.on_bed_ticks = 0;
        cat.set_relax_state_one(false);
        cat.mob_entity.steered().navigator().stop();
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let (Some(cat), Some(owner), Some(goal_pos)) = (
            self.cat.upgrade(),
            self.owner_player.as_ref().and_then(Weak::upgrade),
            self.goal_pos,
        ) else {
            return;
        };
        cat.set_in_sitting_pose(false);
        Self::move_to_goal(&cat, &goal_pos);
        let owner_entity = owner.get_entity();
        if cat
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&owner_entity.pos.load())
            < RELAX_ON_BED_DISTANCE_SQ
        {
            self.on_bed_ticks += 1;
            if self.on_bed_ticks > self.get_tick_count(RELAX_LIE_DOWN_TICKS) {
                cat.set_lying(true);
                cat.set_relax_state_one(false);
            } else {
                cat.mob_entity.look_at(owner_entity, 45.0, 45.0);
                cat.set_relax_state_one(true);
            }
        } else {
            cat.set_lying(false);
        }
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

const SELECT_PLAYER_TICKS: i32 = 600;
const FORGET_PLAYER_TICKS: i32 = 500;

struct CatTemptGoal {
    cat: Weak<CatEntity>,
    tempt_goal: TemptGoal,
    selected_player: Option<i32>,
}

impl CatTemptGoal {
    fn new(cat: Weak<CatEntity>) -> Self {
        Self {
            cat,
            tempt_goal: TemptGoal::with_tag(TEMPT_SPEED, &tag::Item::MINECRAFT_CAT_FOOD, true),
            selected_player: None,
        }
    }

    fn is_untamed(&self) -> bool {
        self.cat.upgrade().is_some_and(|cat| !cat.is_tame())
    }
}

impl Goal for CatTemptGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.tempt_goal.can_start(mob) && self.is_untamed()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let selected_is_tempting = self.selected_player.is_some()
            && self.selected_player == self.tempt_goal.target_player().map(|p| p.entity_id());
        self.tempt_goal.set_can_scare(!selected_is_tempting);
        self.tempt_goal.should_continue(mob) && self.is_untamed()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.tempt_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.tempt_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.tempt_goal.tick(mob);
        let mut random = mob.get_random();
        if self.selected_player.is_none()
            && random.random_range(0..self.get_tick_count(SELECT_PLAYER_TICKS)) == 0
        {
            self.selected_player = self.tempt_goal.target_player().map(|p| p.entity_id());
        } else if random.random_range(0..self.get_tick_count(FORGET_PLAYER_TICKS)) == 0 {
            self.selected_player = None;
        }
    }

    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.tempt_goal.tick_reach(mob, running)
    }

    fn controls(&self) -> Controls {
        self.tempt_goal.controls()
    }
}
