use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::world::environment::EnvironmentAttributes;
use crate::world::game_event::GameEventContext;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        ranged_attack::RangedAttackGoal, wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity, RangedAttackMob},
    player::Player,
    projectile::snowball::SnowballEntity,
};

const SNOW_GOLEM_HUNTS: &[Hunt] = &[Hunt::new(1, TargetMatch::Enemy, true)];
const PUMPKIN_FLAG: i8 = 16;
const DEFAULT_PUMPKIN: bool = true;
const AMBIENT_SOUND_INTERVAL: i32 = 120;
const SHEAR_SNOW_GOLEM_LOOT_TABLE: &str = "minecraft:shearing/snow_golem";

pub struct SnowGolemEntity {
    pub mob_entity: MobEntity,
    pub has_pumpkin: AtomicBool,
}

impl SnowGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let snow_golem = Self {
            mob_entity,
            has_pumpkin: AtomicBool::new(DEFAULT_PUMPKIN),
        };
        let mob_arc = Arc::new(snow_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(
                1,
                Box::new(RangedAttackGoal::new(ranged_weak, 1.25, 20, 10.0)),
            );
            goal_selector.add_goal(
                2,
                Box::new(WanderAroundGoal::water_avoiding_with_probability(
                    1.0,
                    1.000_000_1E-5,
                )),
            );
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));

            hunting::apply(target_selector, &mob_arc.mob_entity, SNOW_GOLEM_HUNTS);
        };

        mob_arc
    }

    #[must_use]
    pub fn has_pumpkin(&self) -> bool {
        self.has_pumpkin.load(Ordering::Relaxed)
    }

    pub fn set_has_pumpkin(&self, has_pumpkin: bool) {
        self.has_pumpkin.store(has_pumpkin, Ordering::Relaxed);
        let entity = self.get_entity();
        let flags = if has_pumpkin { PUMPKIN_FLAG } else { 0 };
        entity.set_synced_data(
            pumpkin_data::tracked_data::snow_golem::DATA_PUMPKIN_ID,
            flags,
        );
    }

    pub fn throw_snowball(&self, target: &Arc<dyn EntityBase>) {
        let entity = self.get_entity();
        let world = entity.world.load_full();

        let snowball_entity = Entity::new(world.clone(), entity.pos.load(), &EntityType::SNOWBALL);
        let snowball = SnowballEntity::new_shot(snowball_entity, entity);

        let mob_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let dx = target_pos.x - mob_pos.x;
        let dy = target_pos.y + target_entity.get_eye_height() - f64::from(1.1f32);
        let dz = target_pos.z - mob_pos.z;
        #[expect(
            clippy::imprecise_flops,
            reason = "preserve SnowGolem.performRangedAttack rounding"
        )]
        let horizontal_distance = (dx * dx + dz * dz).sqrt();
        let yo = horizontal_distance * f64::from(0.2f32);

        snowball.thrown.set_velocity(
            dx,
            dy + yo - snowball.get_entity().pos.load().y,
            dz,
            f64::from(1.6f32),
            12.0,
        );

        let snowball = Arc::new(snowball);
        if world.spawn_entity(snowball.clone()) {
            crate::entity::projectile::apply_on_projectile_spawned(
                snowball.get_entity(),
                &ItemStack::new(1, &Item::SNOWBALL),
                None,
                None,
            );
        }

        self.mob_entity.living_entity.make_sound(
            Sound::EntitySnowGolemShoot,
            self.sound_source(),
            1.0,
            0.4 / (self.get_random().random::<f32>() * 0.4 + 0.8),
        );
    }

    pub fn ready_for_shearing(&self) -> bool {
        self.has_pumpkin()
    }

    pub fn shear(&self, category: SoundCategory, tool: &ItemStack) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntitySnowGolemShear, category, &pos);
        self.set_has_pumpkin(false);
        if let Some(table) = world.get_loot_table(SHEAR_SNOW_GOLEM_LOOT_TABLE) {
            let params = LootContextParameters {
                world: Some(&world),
                position: Some(pos),
                this_entity: Some(self),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            for stack in generate_loot_from_handle(&table, self.get_random().random(), &params) {
                let item = ItemEntity::new(
                    Entity::new(
                        world.clone(),
                        pos.add_raw(0.0, entity.get_eye_height(), 0.0),
                        &EntityType::ITEM,
                    ),
                    stack,
                );
                world.spawn_entity(Arc::new(item));
            }
        }
    }

    fn interact_with_shears(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: Hand,
    ) -> bool {
        if item_stack.get_item() == &Item::SHEARS && self.ready_for_shearing() {
            let entity = self.get_entity();
            let world = entity.world.load();
            if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                    player: player.clone(),
                    entity_id: entity.entity_id,
                    hand: match hand {
                        Hand::Right => 0,
                        Hand::Left => 1,
                    },
                    cancelled: false,
                };
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            self.shear(SoundCategory::Players, item_stack);
            world.emit_game_event_with_context(
                GameEvent::Shear.name(),
                entity.pos.load(),
                &GameEventContext {
                    source_entity: Some(player.as_ref()),
                },
            );
            let _ = player.damage_item_stack(item_stack, 1);
            return true;
        }
        false
    }
}

impl Mob for SnowGolemEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Pumpkin", self.has_pumpkin());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_has_pumpkin(nbt.get_bool("Pumpkin").unwrap_or(DEFAULT_PUMPKIN));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        AMBIENT_SOUND_INTERVAL
    }

    fn is_sensitive_to_water(&self) -> bool {
        true
    }

    fn mob_ai_step(&self) {
        let entity = self.get_entity();
        if entity.is_in_water_or_rain() {
            self.damage(self, 1.0, DamageType::DROWN);
        }
        let world = entity.world.load();
        if EnvironmentAttributes::new(&world).get_value_bool(
            EnvironmentAttribute::GameplaySnowGolemMelts,
            &entity.block_pos.load(),
        ) {
            self.damage(self, 1.0, DamageType::ON_FIRE);
        }

        if !world.level_info.load().game_rules.mob_griefing {
            return;
        }

        let pos = entity.pos.load();
        let snow = Block::SNOW.default_state;
        for i in 0..4 {
            let snow_pos = BlockPos::floored(
                pos.x + f64::from(i % 2 * 2 - 1) * 0.25,
                pos.y,
                pos.z + f64::from(i / 2 % 2 * 2 - 1) * 0.25,
            );
            if world.get_block_state(&snow_pos).is_air()
                && world.block_registry.can_place_at(
                    None,
                    Some(&world),
                    world.as_ref(),
                    None,
                    &Block::SNOW,
                    snow,
                    &snow_pos,
                    None,
                    None,
                )
            {
                let mut new_state = snow.id;
                if let Some(server) = world.server.upgrade()
                    && let Some(golem) = world.get_entity_by_id(entity.entity_id)
                {
                    let mut event = crate::plugin::api::events::block::entity_block_form::EntityBlockFormEvent::new(
                        golem, snow_pos, world.clone(), new_state,
                    );
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        continue;
                    }
                    new_state = event.new_state_id;
                }
                world.set_block_state(&snow_pos, new_state, BlockFlags::NOTIFY_ALL);
                world.emit_game_event_with_context(
                    GameEvent::BlockPlace.name(),
                    snow_pos.to_centered_f64(),
                    &GameEventContext {
                        source_entity: Some(self),
                    },
                );
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags = if self.has_pumpkin() { PUMPKIN_FLAG } else { 0 };
        entity.set_synced_data(
            pumpkin_data::tracked_data::snow_golem::DATA_PUMPKIN_ID,
            flags,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.mob_interact_with_hand(player, item_stack, Hand::Right)
            .unwrap_or(false)
    }

    fn mob_interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: Hand,
    ) -> Option<bool> {
        if !self.mob_entity.living_entity.is_alive() {
            return Some(false);
        }
        let is_named_name_tag = item_stack.item == &Item::NAME_TAG
            && item_stack
                .get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
                .is_some();
        if !is_named_name_tag
            && item_stack.item != &Item::SHEARS
            && item_stack.item != &Item::LEAD
            && self
                .get_entity()
                .get_leash_holder()
                .is_none_or(|holder| holder.get_entity().entity_id != player.entity_id())
        {
            return None;
        }
        let before = item_stack.clone();
        let interacted = if is_named_name_tag {
            // Mob.checkAndHandleImportantInteractions precedes common leash interactions.
            let entity = self.get_entity();
            let world = entity.world.load();
            let server = world.server.upgrade()?;
            let golem = world.get_entity_by_id(entity.entity_id)?;
            server
                .item_registry
                .get_pumpkin_item(item_stack.item.id)?
                .use_on_entity(item_stack, player, golem);
            true
        } else if item_stack.get_item() != &Item::SHEARS {
            self.mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed())
        } else if let Some(sheared) = self
            .get_entity()
            .shear_off_all_leash_connections(self, Some(player))
        {
            if sheared {
                let _ = player.damage_item_stack(item_stack, 1);
            }
            sheared
        } else if self.can_shear_equipment(player)
            && !player.get_entity().is_sneaking()
            && let Some(sheared) =
                self.attempt_to_shear_equipment_with_hand(player, item_stack, hand)
        {
            sheared
        } else {
            self.interact_with_shears(player, item_stack, hand)
        };
        if interacted {
            self.get_entity().world.load().emit_game_event_with_context(
                GameEvent::EntityInteract.name(),
                self.get_entity().pos.load(),
                &GameEventContext {
                    source_entity: Some(player.as_ref()),
                },
            );
        }
        if !item_stack.are_equal(&before) {
            player.increment_stat(
                crate::entity::player::statistics::StatisticCategory::Used,
                i32::from(before.item.id),
                1,
            );
        }
        if before.is_damageable() && item_stack.is_empty() {
            let world = self.get_entity().world.load();
            player.increment_stat(
                crate::entity::player::statistics::StatisticCategory::Broken,
                i32::from(before.item.id),
                1,
            );
            let slot = match hand {
                Hand::Right => EquipmentSlot::MAIN_HAND,
                Hand::Left => EquipmentSlot::OFF_HAND,
            };
            world.send_entity_status(
                player.get_entity(),
                crate::entity::equipment_break_status(&slot),
                pumpkin_protocol::bedrock::server::actor_event::ActorEventID::None,
            );
        }
        Some(interacted)
    }
}

impl RangedAttackMob for SnowGolemEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.throw_snowball(target);
    }
}
