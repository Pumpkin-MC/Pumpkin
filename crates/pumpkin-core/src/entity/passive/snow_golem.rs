use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::world::environment::EnvironmentAttributes;
use crate::world::game_event::GameEventContext;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        ranged_attack::RangedAttackGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, RangedAttackMob},
    player::Player,
    projectile::snowball::SnowballEntity,
    shearable::{Shearable, shear_by_player_in_hand, shearing_loot},
};

const SNOW_GOLEM_HUNTS: &[Hunt] = &[Hunt::new(1, TargetMatch::Enemy, true)];
const PUMPKIN_FLAG: i8 = 16;
const DEFAULT_PUMPKIN: bool = true;
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

        self.play_entity_sound(
            Sound::EntitySnowGolemShoot,
            1.0,
            0.4 / (self.get_random().random::<f32>() * 0.4 + 0.8),
        );
    }
}

impl Shearable for SnowGolemEntity {
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack) -> bool {
        if !self.has_pumpkin.swap(false, Ordering::Relaxed) {
            return false;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntitySnowGolemShear, sound_category, &pos);
        self.set_has_pumpkin(false);

        for stack in shearing_loot(self, SHEAR_SNOW_GOLEM_LOOT_TABLE, tool) {
            entity.spawn_at_location_with_offset(
                stack,
                Vector3::new(0.0, entity.get_eye_height(), 0.0),
            );
        }
        true
    }

    fn ready_for_shearing(&self) -> bool {
        self.has_pumpkin()
    }
}

impl Mob for SnowGolemEntity {
    fn as_shearable(&self) -> Option<&dyn Shearable> {
        Some(self)
    }

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
        super::abstract_golem::AMBIENT_SOUND_INTERVAL
    }

    fn is_sensitive_to_water(&self) -> bool {
        true
    }

    fn mob_ai_step(&self) {
        let entity = self.get_entity();
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
                if pumpkin_data::BlockState::from_id(new_state).is_air() {
                    continue;
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

    fn mob_interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: Hand,
    ) -> Option<bool> {
        if item_stack.get_item() == &Item::SHEARS && self.ready_for_shearing() {
            return Some(shear_by_player_in_hand(self, player, item_stack, hand));
        }
        None
    }
}

impl RangedAttackMob for SnowGolemEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.throw_snowball(target);
    }
}
