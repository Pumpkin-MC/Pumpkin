use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::{calculate_ray_intersection, is_projectile};
use crate::entity::util::RandomExt;
use crate::world::World;
use crate::world::loot::LootContextParameters;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::Block;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::Hand;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::plugin::api::events::player::fish::{PlayerFishEvent, PlayerFishState};
use pumpkin_util::random::RandomImpl;
use pumpkin_util::random::legacy_rand::LegacyRand;

/// Vanilla `FishingHook.FishHookState`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HookState {
    Flying,
    HookedIn(i32),
    Bobbing { out_of_water_time: u8 },
}

impl HookState {
    /// Vanilla `FishingHook.MAX_OUT_OF_WATER_TIME`.
    const MAX_OUT_OF_WATER_TIME: u8 = 10;
}

pub struct FishingBobberEntity {
    pub entity: Entity,
    pub owner_id: i32,
    pub owner_uuid: uuid::Uuid,
    pub state: AtomicCell<HookState>,
    pub left_owner: AtomicBool,
    pub life: AtomicI32,
    pub wait_countdown: AtomicI32,
    pub bite_countdown: AtomicI32,
    pub hook_countdown: AtomicI32,
    pub fish_angle: AtomicCell<f32>,
    pub luck_bonus: i32,
    pub wait_time_reduction_ticks: i32,
}

impl FishingBobberEntity {
    const INERTIA: f64 = 0.92;
    const GRAVITY: f64 = 0.03;

    /// Creates a bobber owned by `owner`, thrown along the given `yaw` and `pitch`.
    pub fn new(
        entity: Entity,
        owner: &Player,
        yaw: f32,
        pitch: f32,
        luck_bonus: i32,
        wait_time_reduction_ticks: i32,
    ) -> Self {
        let owner_id = owner.living_entity.entity.entity_id;
        let owner_entity = &owner.living_entity.entity;

        // Vanilla draws `RandomSource.triangle(0.5, 0.0103365)` once per axis.
        let mut rng = rand::rng();
        let triangle = Vector3::new(
            rng.triangle(0.5, 0.010_336_5),
            rng.triangle(0.5, 0.010_336_5),
            rng.triangle(0.5, 0.010_336_5),
        );
        let (origin, velocity) = throw_setup(yaw, pitch, triangle);

        let owner_pos = owner_entity.pos.load();
        // Move the whole entity, so the bounding box follows the eye position; writing `pos`
        // directly leaves the box behind and the next move resolves against the wrong place.
        entity.set_pos(Vector3::new(
            owner_pos.x + origin.x,
            owner_pos.y + owner_entity.get_eye_height(),
            owner_pos.z + origin.z,
        ));
        entity.update_last_pos();
        // Vanilla derives the hook's rotation from its launch movement, not the player's look.
        entity.set_rotation(
            (velocity.x.atan2(velocity.z) as f32).to_degrees(),
            (velocity.y.atan2(velocity.horizontal_length()) as f32).to_degrees(),
        );
        // Vanilla `FishingHook.getAddEntityPacket` sends the owner id as the spawn data,
        // which the client needs to render the line.
        entity.data.store(owner_id, Ordering::Relaxed);
        entity.velocity.store(velocity);

        let luck_bonus = luck_bonus.max(0);
        let wait_time_reduction_ticks = wait_time_reduction_ticks.max(0);
        // Vanilla `FishingBobberEntity.catchingFish` rolls `100..=600` minus the lure speed.
        let wait_countdown = rand::random_range(100..=600) - wait_time_reduction_ticks;

        Self {
            entity,
            owner_id,
            owner_uuid: owner.gameprofile.id,
            state: AtomicCell::new(HookState::Flying),
            left_owner: AtomicBool::new(false),
            life: AtomicI32::new(0),
            wait_countdown: AtomicI32::new(wait_countdown),
            bite_countdown: AtomicI32::new(0),
            hook_countdown: AtomicI32::new(0),
            fish_angle: AtomicCell::new(0.0),
            luck_bonus,
            wait_time_reduction_ticks,
        }
    }

    /// Matches vanilla `FishingBobberEntity.use(ItemStack usedItem)`.
    #[expect(clippy::too_many_lines)]
    pub fn reel_in(&self, player: &Player, used_item: &ItemStack, hand: Hand) -> i32 {
        // Vanilla `FishingHook.retrieve`: a hook whose owner stopped fishing yields nothing.
        if self.should_stop_fishing(player) {
            return 0;
        }

        let world = self.entity.world.load();
        let mut damage = 0;

        if let HookState::HookedIn(hooked_id) = self.state.load()
            && let Some(hooked) = world.get_entity_by_id(hooked_id)
        {
            // Bukkit `PlayerFishEvent`: reeling in a hooked entity is the `CAUGHT_ENTITY` outcome.
            if self
                .fire_fish_event(
                    &world,
                    PlayerFishState::CaughtEntity,
                    Some(&*hooked),
                    hand,
                    0,
                )
                .is_none()
            {
                return 0;
            }

            let player_pos = player.get_entity().pos.load();
            let bobber_pos = self.entity.pos.load();
            let delta = player_pos - bobber_pos;
            let motion = delta * 0.1;
            hooked.get_entity().add_velocity(motion);
            world.send_entity_status(&self.entity, EntityStatus::FishingRodReelIn, None);

            damage = if hooked.get_entity().entity_type == &EntityType::ITEM {
                3
            } else {
                5
            };
        } else if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            let bobber_pos = self.entity.pos.load();
            let player_luck = player.living_entity.get_attribute_value(&Attributes::LUCK) as f32;
            let total_luck = self.luck_bonus as f32 + player_luck;

            let params = LootContextParameters {
                tool: Some(used_item.clone()),
                position: Some(bobber_pos),
                this_entity: Some(&EntityType::FISHING_BOBBER),
                luck: total_luck,
                ..Default::default()
            };

            let items = world
                .get_loot_table("gameplay/fishing")
                .map_or_else(Vec::new, |table| {
                    let seed = rand::random::<i64>();
                    table.generate_loot_with_context(seed, &params)
                });

            // Fallback while the shared loot engine can't resolve the nested
            // `gameplay/fishing` subtables.
            let loot = if items.is_empty() {
                vec![ItemStack::new(1, &Item::COD)]
            } else {
                items
            };

            for item_stack in loot {
                let player_pos = player.position();
                let delta = player_pos - bobber_pos;
                let dist_sq = delta.length_squared();
                let velocity = Vector3::new(
                    delta.x * 0.1,
                    delta.y * 0.1 + dist_sq.sqrt().sqrt() * 0.08,
                    delta.z * 0.1,
                );

                let entity = Entity::new(world.clone(), bobber_pos, &EntityType::ITEM);

                // Bukkit `PlayerFishEvent`: each caught item is a `CAUGHT_FISH` outcome.
                let Some(xp) = self.fire_fish_event(
                    &world,
                    PlayerFishState::CaughtFish,
                    Some(&entity as &dyn EntityBase),
                    hand,
                    rand::random_range(1..=6),
                ) else {
                    continue;
                };

                let mut item_event =
                    crate::plugin::api::events::entity::item_spawn::ItemSpawnEvent::new(
                        entity.entity_id,
                        bobber_pos,
                        item_stack.item.registry_key.to_string(),
                    );
                if let Some(server) = world.server.upgrade() {
                    server
                        .plugin_manager
                        .fire_blocking(&server, &mut item_event);
                }
                if !item_event.cancelled {
                    let item_entity = Arc::new(ItemEntity::new_with_velocity(
                        entity,
                        item_stack.clone(),
                        velocity,
                        0,
                    ));
                    world.spawn_entity(item_entity);
                }

                let mut xp_pos = player_pos;
                xp_pos.y += 0.5;
                xp_pos.z += 0.5;
                ExperienceOrbEntity::spawn(&world, xp_pos, xp.max(0) as u32);

                if item_stack.item.has_tag(&tag::Item::MINECRAFT_FISHES) {
                    player.increment_stat(
                        pumpkin_data::statistic::StatisticCategory::Custom,
                        pumpkin_data::statistic::CustomStatistic::FishCaught as i32,
                        1,
                    );
                }

                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::FishedItem {
                        item_id: format!("minecraft:{}", item_stack.item.registry_key),
                    },
                );
            }

            damage = 1;
        }

        if self.entity.on_ground.load(Ordering::Relaxed) {
            damage = 2;
        }

        damage
    }

    /// Vanilla `Projectile.tick`: the hook cannot hit the owner until it has left the owner's
    /// vehicle, so it does not hook the caster at spawn.
    fn update_left_owner(&self, world: &World, owner: &dyn EntityBase, velocity: Vector3<f64>) {
        if self.left_owner.load(Ordering::Relaxed) {
            return;
        }

        let owner_root = root_vehicle_uuid(owner);
        let search = self
            .entity
            .bounding_box
            .load()
            .stretch(velocity)
            .expand(1.0, 1.0, 1.0);
        let still_riding = world
            .get_entities_at_box(&search)
            .iter()
            .any(|found| root_vehicle_uuid(found.as_ref()) == owner_root);

        if !still_riding {
            self.left_owner.store(true, Ordering::Relaxed);
        }
    }

    /// Vanilla `FishingHook.tick`: the bite motion uses a random reseeded from the hook UUID and
    /// game time, so the client predicts the same motion instead of jittering.
    fn synced_random(&self, world: &World) -> LegacyRand {
        let (_, uuid_low) = self.entity.entity_uuid.as_u64_pair();
        let game_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age;
        LegacyRand::from_seed(uuid_low ^ game_time as u64)
    }

    /// Fires the Bukkit `PlayerFishEvent` for `state`. Returns `None` when a plugin cancelled it,
    /// otherwise the experience to drop.
    fn fire_fish_event(
        &self,
        world: &World,
        state: PlayerFishState,
        caught: Option<&dyn EntityBase>,
        hand: Hand,
        exp_to_drop: i32,
    ) -> Option<i32> {
        let Some(owner) = world.get_player_by_uuid(self.owner_uuid) else {
            return Some(exp_to_drop);
        };
        let Some(server) = world.server.upgrade() else {
            return Some(exp_to_drop);
        };

        let mut event = PlayerFishEvent::new(
            owner,
            caught.map(|entity| entity.get_entity().entity_uuid),
            self.entity.entity_uuid,
            caught.map_or_else(String::new, |entity| {
                entity.get_entity().entity_type.registry_key().to_string()
            }),
            state,
            hand,
            exp_to_drop,
        );
        server.plugin_manager.fire_blocking(&server, &mut event);

        if event.cancelled {
            None
        } else {
            Some(event.exp_to_drop)
        }
    }

    /// Advances the bobber a tick, moving it and running the fishing cycle.
    #[expect(clippy::too_many_lines)]
    pub fn process_tick(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();

        // Vanilla `FishingBobberEntity.tick`: drop the hook once the owner stops fishing.
        let Some(owner) = world.get_entity_by_id(self.owner_id) else {
            entity.remove();
            return;
        };
        self.update_left_owner(&world, &*owner, entity.velocity.load());
        let Some(player) = owner.cast_any().downcast_ref::<Player>() else {
            entity.remove();
            return;
        };
        if self.should_stop_fishing(player) {
            self.discard(player);
            return;
        }

        if entity.on_ground.load(Ordering::Relaxed) {
            let life = self.life.fetch_add(1, Ordering::Relaxed) + 1;
            if life >= 1200 {
                self.discard(player);
            }
            return;
        }
        self.life.store(0, Ordering::Relaxed);

        let block_pos = entity.block_pos.load();
        let (fluid, fluid_state) = world.get_fluid_and_fluid_state(&block_pos);
        let water_height = if fluid.matches_type(&Fluid::WATER) {
            f64::from(world.get_fluid_height(&block_pos, fluid, &fluid_state))
        } else {
            0.0
        };
        let in_water = water_height > 0.0;

        let mut velocity = entity.velocity.load();
        let start_pos = entity.pos.load();

        let mut state = self.state.load();
        match state {
            HookState::HookedIn(hooked_id) => {
                if let Some(hooked) = world.get_entity_by_id(hooked_id)
                    && can_interact_with_level(&*hooked)
                    && hooked.get_entity().world.load().dimension
                        == self.entity.world.load().dimension
                {
                    let mut hooked_pos = hooked.get_entity().pos.load();
                    hooked_pos.y += f64::from(hooked.get_entity().height()) * 0.8;
                    entity.set_pos(hooked_pos);
                } else {
                    self.set_hooked_entity(None);
                }
                return;
            }
            // Vanilla `FishingBobberEntity.tick`: entering water turns the flying
            // bobber into a bobbing one and kills most of its momentum.
            HookState::Flying if in_water => {
                entity
                    .velocity
                    .store(velocity * Vector3::new(0.3, 0.2, 0.3));
                self.state.store(HookState::Bobbing {
                    out_of_water_time: 0,
                });
                return;
            }
            HookState::Bobbing { out_of_water_time } => {
                velocity = bob_velocity(
                    velocity,
                    start_pos.y,
                    block_pos.0.y,
                    water_height,
                    rand::random::<f64>(),
                );

                if in_water {
                    // Vanilla `FishingBobberEntity.tick`: a biting bobber is pulled down every tick.
                    if self.bite_countdown.load(Ordering::Relaxed) > 0 {
                        let mut synced = self.synced_random(&world);
                        velocity.y -= 0.1 * f64::from(synced.next_f32() * synced.next_f32());
                    }
                    self.catching_fish(
                        &world,
                        &block_pos,
                        &mut velocity,
                        crate::item::hand_holding(player, &Item::FISHING_ROD),
                    );
                    state = HookState::Bobbing {
                        out_of_water_time: out_of_water_time.saturating_sub(1),
                    };
                } else {
                    state = HookState::Bobbing {
                        out_of_water_time: out_of_water_time
                            .saturating_add(1)
                            .min(HookState::MAX_OUT_OF_WATER_TIME),
                    };
                }
            }
            HookState::Flying => {}
        }
        self.state.store(state);

        // Vanilla `FishingBobberEntity.tick`: gravity while airborne and unhooked.
        if !in_water && !entity.on_ground.load(Ordering::Relaxed) {
            velocity.y -= Self::GRAVITY;
        }

        // Vanilla `FishingHook.checkCollision`: ray-cast the movement and hook the nearest entity.
        if matches!(state, HookState::Flying) {
            self.check_collision(&world, &*owner, caller, start_pos, velocity);
        }

        // Vanilla moves the hook through the engine's collision resolution, so it comes to rest on
        // block surfaces instead of hovering above them.
        entity.move_entity(caller, velocity);

        // Vanilla zeroes the hook's momentum when it comes to rest while flying.
        if matches!(state, HookState::Flying)
            && (entity.on_ground.load(Ordering::Relaxed)
                || entity.horizontal_collision.load(Ordering::Relaxed))
        {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
        }
        entity
            .velocity
            .store(entity.velocity.load() * Self::INERTIA);
    }

    /// Vanilla `FishingHook.checkCollision`: ray-cast the movement and hook the nearest entity.
    fn check_collision(
        &self,
        world: &World,
        owner: &dyn EntityBase,
        caller: &dyn EntityBase,
        start_pos: Vector3<f64>,
        velocity: Vector3<f64>,
    ) {
        let new_pos = start_pos.add(&velocity);
        let search_box = BoundingBox::new(
            Vector3::new(
                start_pos.x.min(new_pos.x),
                start_pos.y.min(new_pos.y),
                start_pos.z.min(new_pos.z),
            ),
            Vector3::new(
                start_pos.x.max(new_pos.x),
                start_pos.y.max(new_pos.y),
                start_pos.z.max(new_pos.z),
            ),
        )
        .expand(0.3, 0.3, 0.3);

        let mut closest_t = 1.0;
        let mut hooked = None;

        // A block in front of an entity stops the hook.
        let (block_cols, _) = world.get_block_collisions(search_box, caller);
        for shape in &block_cols {
            if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, shape)
                && t < closest_t
            {
                closest_t = t;
                hooked = None;
            }
        }

        let left_owner = self.left_owner.load(Ordering::Relaxed);
        let owner_root = root_vehicle_uuid(owner);
        for cand in world.get_entities_at_box(&search_box) {
            let candidate = cand.get_entity();
            // Vanilla `Projectile.canHitEntity`: not the hook, not a spectator, not a projectile,
            // alive, and not the owner's vehicle chain until the hook has left it.
            if candidate.entity_id == self.entity.entity_id
                || cand.is_spectator()
                || is_projectile(candidate.entity_type)
                || (!left_owner && root_vehicle_uuid(cand.as_ref()) == owner_root)
                || cand
                    .get_living_entity()
                    .is_some_and(|living| living.health.load() <= 0.0)
            {
                continue;
            }

            let entity_box = candidate.bounding_box.load().expand(0.3, 0.3, 0.3);
            if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, &entity_box)
                && t < closest_t
            {
                closest_t = t;
                hooked = Some(candidate.entity_id);
            }
        }

        if let Some(id) = hooked {
            self.set_hooked_entity(Some(id));
        }
    }

    /// Vanilla `FishingHook.setHookedEntity`: records the hooked entity and syncs it to the client.
    fn set_hooked_entity(&self, hooked_id: Option<i32>) {
        self.state
            .store(hooked_id.map_or(HookState::Flying, HookState::HookedIn));
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::fishing_bobber::HOOKED_ENTITY,
            hooked_id.map_or(0, |id| id + 1),
        );
    }

    /// Vanilla `FishingBobberEntity.shouldStopFishing(Player)`.
    fn should_stop_fishing(&self, player: &Player) -> bool {
        let owner = player.get_entity();
        // Vanilla `Entity.canInteractWithLevel` gates the checks below.
        if can_interact_with_level(player) {
            let inventory = player.inventory();
            let holding_rod = inventory.held_item().item.id == Item::FISHING_ROD.id
                || inventory.get_stack_in_hand(Hand::Left).item.id == Item::FISHING_ROD.id;
            if holding_rod {
                let owner_pos = owner.pos.load();
                let bobber_pos = self.entity.pos.load();
                return owner_pos.squared_distance_to_vec(&bobber_pos) > 1024.0;
            }
        }
        true
    }

    /// Vanilla `FishingBobberEntity.remove`: clears `owner.fishing` and drops the hook.
    fn discard(&self, player: &Player) {
        player.fishing_bobber.store(-1, Ordering::Relaxed);
        self.entity.remove();
    }

    /// Matches vanilla `FishingBobberEntity.catchingFish(BlockPos)`.
    #[expect(clippy::too_many_lines)]
    fn catching_fish(
        &self,
        world: &World,
        block_pos: &BlockPos,
        velocity: &mut Vector3<f64>,
        hand: Hand,
    ) {
        let entity = self.get_entity();
        let pos = entity.pos.load();

        let mut fishing_speed = 1;
        let above = block_pos.up();
        if rand::random::<f32>() < 0.25 && world.is_raining_at(&above) {
            fishing_speed += 1;
        }
        if rand::random::<f32>() < 0.5 && !world.can_see_sky(&above) {
            fishing_speed -= 1;
        }

        let nibble = self.bite_countdown.load(Ordering::Relaxed);
        if nibble > 0 {
            let next = nibble - 1;
            self.bite_countdown.store(next, Ordering::Relaxed);
            if next <= 0 {
                self.wait_countdown.store(0, Ordering::Relaxed);
                self.hook_countdown.store(0, Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::DATA_BITING,
                    false,
                );
                // Bukkit `PlayerFishEvent`: the bite window closed without a reel.
                self.fire_fish_event(world, PlayerFishState::FailedAttempt, None, hand, 0);
            }
        } else if self.hook_countdown.load(Ordering::Relaxed) > 0 {
            let hooked = self.hook_countdown.load(Ordering::Relaxed) - fishing_speed;
            self.hook_countdown.store(hooked, Ordering::Relaxed);
            if hooked > 0 {
                // Vanilla `RandomSource.triangle(0.0, 9.188)`: the approaching fish wanders.
                let angle = self.fish_angle.load() + rand::rng().triangle(0.0, 9.188) as f32;
                self.fish_angle.store(angle);
                let radians = f64::from(angle).to_radians();
                let angle_sin = radians.sin();
                let angle_cos = radians.cos();
                let fish_x = pos.x + angle_sin * f64::from(hooked) * 0.1;
                let fish_y = f64::from(block_pos.0.y) + 1.0;
                let fish_z = pos.z + angle_cos * f64::from(hooked) * 0.1;
                let splash_pos = BlockPos::floored(fish_x, fish_y - 1.0, fish_z);
                if world.get_block(&splash_pos) == &Block::WATER {
                    if rand::random::<f32>() < 0.15 {
                        world.spawn_particle(
                            Vector3::new(fish_x, fish_y - 0.1, fish_z),
                            Vector3::new(angle_sin as f32, 0.1, angle_cos as f32),
                            0.0,
                            1,
                            Particle::Bubble,
                        );
                    }

                    let x_movement = angle_sin as f32 * 0.04;
                    let z_movement = angle_cos as f32 * 0.04;
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(z_movement, 0.01, -x_movement),
                        1.0,
                        0,
                        Particle::Fishing,
                    );
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(-z_movement, 0.01, x_movement),
                        1.0,
                        0,
                        Particle::Fishing,
                    );
                }
            } else {
                world.play_sound_fine(
                    Sound::EntityFishingBobberSplash,
                    SoundCategory::Neutral,
                    &pos,
                    0.25,
                    rand::rng().triangle(1.0, 0.4) as f32,
                );

                let width = entity.width();
                let count = (1.0 + width * 20.0) as i32;
                let particle_pos = Vector3::new(pos.x, pos.y + 0.5, pos.z);
                let particle_offset = Vector3::new(width, 0.0, width);
                world.spawn_particle(particle_pos, particle_offset, 0.2, count, Particle::Bubble);
                world.spawn_particle(particle_pos, particle_offset, 0.2, count, Particle::Fishing);

                self.bite_countdown
                    .store(rand::random_range(20..=40), Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::DATA_BITING,
                    true,
                );
                // Bukkit `PlayerFishEvent`: a fish bit the hook.
                self.fire_fish_event(world, PlayerFishState::Bite, None, hand, 0);
                // Vanilla applies the bite's downward kick in `onSyncedDataUpdated`.
                let mut synced = self.synced_random(world);
                velocity.y = -0.4 * (f64::from(synced.next_f32()) * 0.4 + 0.6);
            }
        } else if self.wait_countdown.load(Ordering::Relaxed) > 0 {
            let wait = self.wait_countdown.load(Ordering::Relaxed) - fishing_speed;
            self.wait_countdown.store(wait, Ordering::Relaxed);
            let mut tease_chance = 0.15;
            if wait < 20 {
                tease_chance += f64::from(20 - wait) * 0.05;
            } else if wait < 40 {
                tease_chance += f64::from(40 - wait) * 0.02;
            } else if wait < 60 {
                tease_chance += f64::from(60 - wait) * 0.01;
            }

            if rand::random::<f64>() < tease_chance {
                let angle = f64::from(rand::random::<f32>() * 360.0).to_radians();
                let distance = 25.0 + rand::random::<f64>() * 35.0;
                let fish_x = pos.x + angle.sin() * distance * 0.1;
                let fish_y = f64::from(block_pos.0.y) + 1.0;
                let fish_z = pos.z + angle.cos() * distance * 0.1;
                let splash_pos = BlockPos::floored(fish_x, fish_y - 1.0, fish_z);
                if world.get_block(&splash_pos) == &Block::WATER {
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(0.1, 0.0, 0.1),
                        0.0,
                        2 + rand::random_range(0..2),
                        Particle::Splash,
                    );
                }
            }

            if wait <= 0 {
                self.fish_angle.store(rand::random::<f32>() * 360.0);
                self.hook_countdown
                    .store(rand::random_range(20..=80), Ordering::Relaxed);
            }
        } else {
            let wait = rand::random_range(100..=600) - self.wait_time_reduction_ticks;
            self.wait_countdown.store(wait, Ordering::Relaxed);
        }
    }
}

/// Vanilla `Entity.getRootVehicle`: the UUID of the entity at the top of the vehicle chain.
fn root_vehicle_uuid(entity: &dyn EntityBase) -> uuid::Uuid {
    let mut uuid = entity.get_entity().entity_uuid;
    let mut current = entity.get_entity().get_vehicle();
    while let Some(vehicle) = current {
        uuid = vehicle.get_entity().entity_uuid;
        current = vehicle.get_entity().get_vehicle();
    }
    uuid
}

/// Vanilla `Entity.canInteractWithLevel`: alive, not removed and not a spectator.
fn can_interact_with_level(entity: &dyn EntityBase) -> bool {
    entity.get_entity().is_alive()
        && !entity.is_spectator()
        && entity
            .get_living_entity()
            .is_none_or(|living| living.health.load() > 0.0)
}

/// Vanilla `FishHookState.BOBBING`: damp the bobber toward the water surface.
fn bob_velocity(
    velocity: Vector3<f64>,
    y: f64,
    block_y: i32,
    water_height: f64,
    random: f64,
) -> Vector3<f64> {
    let mut force = y + velocity.y - f64::from(block_y) - water_height;
    if force.abs() < 0.01 {
        force += force.signum() * 0.1;
    }
    Vector3::new(
        velocity.x * 0.9,
        velocity.y - force * random * 0.2,
        velocity.z * 0.9,
    )
}

/// Vanilla `FishingBobberEntity(Player, Level, int, int)`: the throw origin offset (X/Z,
/// relative to the owner) and launch velocity for the owner's yaw and pitch.
fn throw_setup(yaw: f32, pitch: f32, triangle: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let yaw_rad = f64::from(yaw).to_radians();
    let pitch_rad = f64::from(pitch).to_radians();
    let y_sin = (-yaw_rad - std::f64::consts::PI).sin();
    let y_cos = (-yaw_rad - std::f64::consts::PI).cos();
    let x_cos = -(-pitch_rad).cos();
    let x_sin = (-pitch_rad).sin();

    let origin = Vector3::new(-y_sin * 0.3, 0.0, -y_cos * 0.3);

    let mut movement = Vector3::new(-y_sin, (-(x_sin / x_cos)).clamp(-5.0, 5.0), -y_cos);
    let scale = 0.6 / movement.length();
    movement = movement * triangle.add_raw(scale, scale, scale);
    (origin, movement)
}

impl EntityBase for FishingBobberEntity {
    fn get_owner_id(&self) -> Option<i32> {
        Some(self.owner_id)
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.process_tick(caller);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bob_settles_at_the_water_surface() {
        let block_y = 62;
        let water_height = 0.9;
        let surface = f64::from(block_y) + water_height;

        // From above and below the surface the bobber must converge instead of rising
        // out of the water and falling back in.
        for start in [surface + 2.0, surface - 1.0] {
            let mut y = start;
            let mut velocity = Vector3::new(0.0, 0.0, 0.0);
            for _ in 0..200 {
                velocity = bob_velocity(velocity, y, block_y, water_height, 0.5);
                velocity = velocity * FishingBobberEntity::INERTIA;
                y += velocity.y;
                assert!(
                    y - surface < 2.1,
                    "bobber rose {:.3} above the surface from {start}",
                    y - surface
                );
            }
            assert!(
                (y - surface).abs() < 0.2,
                "bobber settled at {y}, expected the surface at {surface}"
            );
        }
    }

    #[test]
    fn throw_uses_the_look_direction() {
        // Due +Z at the default rotation, spawned 0.3 blocks in front of the eye.
        let (origin, forward) = throw_setup(0.0, 0.0, Vector3::new(0.5, 0.5, 0.5));
        assert!((origin.z - 0.3).abs() < 1e-9 && origin.x.abs() < 1e-9);
        assert!(
            forward.x.abs() < 1e-9 && (forward.z - 1.1).abs() < 1e-9 && forward.y.abs() < 1e-9,
            "expected due +Z at 1.1, got {forward:?}"
        );

        // Looking east throws east and offsets the origin to -X.
        let (origin, east) = throw_setup(90.0, 0.0, Vector3::new(0.5, 0.5, 0.5));
        assert!((origin.x + 0.3).abs() < 1e-9 && origin.z.abs() < 1e-9);
        assert!(
            (east.x + 1.1).abs() < 1e-9 && east.z.abs() < 1e-9,
            "expected due -X at 1.1, got {east:?}"
        );
    }

    #[test]
    fn throw_clamps_steep_pitch() {
        let (_, down) = throw_setup(0.0, 90.0, Vector3::new(0.0, 0.0, 0.0));
        assert!(down.y < 0.0 && down.z > 0.0);
        assert!(
            (down.y / down.z + 5.0).abs() < 1e-9,
            "expected the -Y clamp, got {down:?}"
        );

        let (_, up) = throw_setup(0.0, -90.0, Vector3::new(0.0, 0.0, 0.0));
        assert!(up.y > 0.0 && up.z > 0.0);
        assert!(
            (up.y / up.z - 5.0).abs() < 1e-9,
            "expected the +Y clamp, got {up:?}"
        );
    }

    #[test]
    fn throw_scales_each_axis_independently() {
        // A yaw/pitch where every base component is non-zero, so a per-axis scale is visible.
        let (yaw, pitch) = (45.0, 30.0);
        let (_, uniform) = throw_setup(yaw, pitch, Vector3::new(0.0, 0.0, 0.0));
        let (_, varied) = throw_setup(yaw, pitch, Vector3::new(0.1, 0.2, 0.3));

        // Vanilla multiplies each axis by its own `0.6 / dist + triangle` value; a single shared
        // scale would leave these ratios equal.
        let ratio_x = varied.x / uniform.x;
        let ratio_y = varied.y / uniform.y;
        let ratio_z = varied.z / uniform.z;
        assert!((ratio_x - ratio_y).abs() > 1e-6);
        assert!((ratio_y - ratio_z).abs() > 1e-6);
        assert!((ratio_x - ratio_z).abs() > 1e-6);
    }
}
