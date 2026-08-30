use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
use uuid::Uuid;

use pumpkin_data::{
    Block,
    damage::DamageType,
    entity::EntityType,
    item::Item,
    item_stack::ItemStack,
    tag::{self, Taggable},
    tracked_data,
    world::WorldEvent,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
use pumpkin_util::{
    Difficulty,
    math::{position::BlockPos, vector3::Vector3},
    text::TextComponent,
};
use pumpkin_world::world::BlockFlags;

use crate::{
    entity::{
        Entity, EntityBase,
        ai::goal::{
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
            revenge::RevengeGoal,
        },
        mob::{Mob, MobEntity},
        projectile::wither_skull::WitherSkullEntity,
    },
    world::{
        ExplosionInteraction,
        bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags},
    },
};

const INVULNERABLE_TICKS: i32 = 220;

pub struct WitherEntity {
    pub mob_entity: MobEntity,
    pub invulnerable_ticks: AtomicI32,
    pub destroy_blocks_tick: AtomicI32,
    pub next_head_update: [AtomicI32; 2],
    pub idle_head_updates: [AtomicI32; 2],
    pub alternative_targets: [AtomicI32; 3],
    pub main_attack_timer: AtomicI32,
    pub bossbar_uuid: Uuid,
    pub bossbar_players: Mutex<Vec<Uuid>>,
    pub dropped_loot: AtomicBool,
}

impl WitherEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wither = Self {
            mob_entity,
            invulnerable_ticks: AtomicI32::new(0),
            destroy_blocks_tick: AtomicI32::new(0),
            next_head_update: [AtomicI32::new(0), AtomicI32::new(0)],
            idle_head_updates: [AtomicI32::new(0), AtomicI32::new(0)],
            alternative_targets: [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)],
            main_attack_timer: AtomicI32::new(0),
            bossbar_uuid: Uuid::new_v4(),
            bossbar_players: Mutex::new(Vec::new()),
            dropped_loot: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(wither);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_invulnerable_ticks(&self) -> i32 {
        self.invulnerable_ticks.load(Ordering::Relaxed)
    }

    pub fn set_invulnerable_ticks(&self, ticks: i32) {
        self.invulnerable_ticks.store(ticks, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                tracked_data::wither::DATA_ID_INV,
                VarInt(ticks),
            )],
            None,
        );
    }

    #[must_use]
    pub fn get_alternative_target(&self, head: usize) -> i32 {
        if head < 3 {
            self.alternative_targets[head].load(Ordering::Relaxed)
        } else {
            0
        }
    }

    pub fn set_alternative_target(&self, head: usize, entity_id: i32) {
        if head < 3 {
            let old = self.alternative_targets[head].swap(entity_id, Ordering::Relaxed);
            if old != entity_id {
                let tracker_id = match head {
                    0 => tracked_data::wither::DATA_TARGET_A,
                    1 => tracked_data::wither::DATA_TARGET_B,
                    _ => tracked_data::wither::DATA_TARGET_C,
                };
                self.mob_entity
                    .living_entity
                    .entity
                    .send_meta_data(&[Metadata::new(tracker_id, VarInt(entity_id))], None);
            }
        }
    }

    #[must_use]
    pub fn is_powered(&self) -> bool {
        let living = &self.mob_entity.living_entity;
        living.health.load() <= living.get_max_health() / 2.0
    }

    pub fn make_invulnerable(&self) {
        self.set_invulnerable_ticks(INVULNERABLE_TICKS);
        self.mob_entity
            .living_entity
            .set_health(self.mob_entity.living_entity.get_max_health() / 3.0);
    }

    #[must_use]
    pub fn can_destroy(block: &Block) -> bool {
        block != &Block::AIR && !block.has_tag(&tag::Block::MINECRAFT_WITHER_IMMUNE)
    }

    #[must_use]
    pub fn get_head_x(&self, head: usize) -> f64 {
        if head == 0 {
            return self.mob_entity.living_entity.entity.pos.load().x;
        }
        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        let angle = (yaw + 180.0 * (head as f32 - 1.0)).to_radians();
        self.mob_entity.living_entity.entity.pos.load().x + (angle.cos() as f64) * 1.3
    }

    #[must_use]
    pub fn get_head_y(&self, head: usize) -> f64 {
        let base_y = self.mob_entity.living_entity.entity.pos.load().y;
        if head == 0 {
            base_y + 3.0
        } else {
            base_y + 2.2
        }
    }

    #[must_use]
    pub fn get_head_z(&self, head: usize) -> f64 {
        if head == 0 {
            return self.mob_entity.living_entity.entity.pos.load().z;
        }
        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        let angle = (yaw + 180.0 * (head as f32 - 1.0)).to_radians();
        self.mob_entity.living_entity.entity.pos.load().z + (angle.sin() as f64) * 1.3
    }

    pub fn perform_ranged_attack(
        &self,
        head: usize,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        dangerous: bool,
    ) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();

        if !entity.silent.load(Ordering::Relaxed) {
            world.sync_world_event(WorldEvent::SoundWitherBossShoot, entity.block_pos.load(), 0);
        }

        let hx = self.get_head_x(head);
        let hy = self.get_head_y(head);
        let hz = self.get_head_z(head);
        let head_pos = Vector3::new(hx, hy, hz);

        let dir = Vector3::new(target_x - hx, target_y - hy, target_z - hz);
        let normalized_dir = if dir.length_squared() > 1e-6 {
            dir.normalize()
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };

        let skull_entity = Entity::from_uuid(
            Uuid::new_v4(),
            (*world).clone(),
            head_pos,
            &EntityType::WITHER_SKULL,
        );
        let skull = Arc::new(WitherSkullEntity::new_shot(
            skull_entity,
            entity,
            dangerous,
            normalized_dir,
        ));
        world.spawn_entity(skull);
    }

    fn make_bossbar(&self) -> Bossbar {
        let title = self
            .mob_entity
            .living_entity
            .entity
            .custom_name
            .load()
            .as_ref()
            .clone()
            .unwrap_or_else(|| {
                TextComponent::translate_cross(
                    "entity.minecraft.wither",
                    "entity.minecraft.wither",
                    [],
                )
            });

        Bossbar {
            uuid: self.bossbar_uuid,
            title,
            health: 1.0,
            color: BossbarColor::Purple,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::DARKEN_SKY,
        }
    }

    fn update_bossbar(&self, world: &Arc<crate::world::World>, progress: f32) {
        let pos = self.mob_entity.living_entity.entity.pos.load();
        let tracking_radius_sq = 50.0 * 50.0;
        let players = world.players.load();

        let current: Vec<Uuid> = players
            .iter()
            .filter(|p| {
                let p_pos = p.living_entity.entity.pos.load();
                (p_pos - pos).length_squared() < tracking_radius_sq
            })
            .map(|p| p.gameprofile.id)
            .collect();

        let mut bossbar_players = self
            .bossbar_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        for &uid in &current {
            if !bossbar_players.contains(&uid) {
                if let Some(p) = players.iter().find(|p| p.gameprofile.id == uid) {
                    let mut bar = self.make_bossbar();
                    bar.health = progress;
                    p.send_bossbar(&bar);
                }
                bossbar_players.push(uid);
            }
        }

        let to_remove: Vec<Uuid> = bossbar_players
            .iter()
            .filter(|uid| !current.contains(uid))
            .copied()
            .collect();

        for uid in &to_remove {
            if let Some(p) = players.iter().find(|p| &p.gameprofile.id == uid) {
                p.remove_bossbar(self.bossbar_uuid);
            }
            bossbar_players.retain(|u| u != uid);
        }

        for player in players.iter() {
            if bossbar_players.contains(&player.gameprofile.id) {
                player.update_bossbar_health(&self.bossbar_uuid, progress);
            }
        }
    }

    fn remove_all_bossbar(&self, world: &Arc<crate::world::World>) {
        let mut bossbar_players = self
            .bossbar_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let players = world.players.load();
        for player in players.iter() {
            if bossbar_players.contains(&player.gameprofile.id) {
                player.remove_bossbar(self.bossbar_uuid);
            }
        }
        bossbar_players.clear();
    }

    #[expect(clippy::too_many_lines)]
    fn tick_wither(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();

        if world.level_info.load().difficulty == Difficulty::Peaceful {
            self.remove_all_bossbar(&world);
            entity.remove();
            return;
        }

        if !entity.is_alive() || self.mob_entity.living_entity.health.load() <= 0.0 {
            self.remove_all_bossbar(&world);
            if !self.dropped_loot.swap(true, Ordering::SeqCst) {
                let pos = entity.block_pos.load();
                world.drop_stack(&pos, ItemStack::new(1, &Item::NETHER_STAR));
            }
            return;
        }

        let invul = self.get_invulnerable_ticks();
        let tick_count = entity.age.load(Ordering::Relaxed);

        if invul > 0 {
            let new_count = invul - 1;
            let progress = (1.0 - (new_count as f32) / 220.0).clamp(0.0, 1.0);
            self.update_bossbar(&world, progress);

            if new_count <= 0 {
                let pos = entity.pos.load();
                let eye_y = pos.y + entity.get_eye_height();
                world.explode(
                    Vector3::new(pos.x, eye_y, pos.z),
                    7.0,
                    ExplosionInteraction::Mob,
                );

                if !entity.silent.load(Ordering::Relaxed) {
                    world.sync_world_event(
                        WorldEvent::SoundWitherBossSpawn,
                        entity.block_pos.load(),
                        0,
                    );
                }
            }

            self.set_invulnerable_ticks(new_count);
            if tick_count % 10 == 0 {
                self.mob_entity.living_entity.heal(10.0);
            }
        } else {
            let living = &self.mob_entity.living_entity;
            let max_health = living.get_max_health();
            let health = living.health.load();
            let progress = if max_health > 0.0 {
                (health / max_health).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.update_bossbar(&world, progress);

            if tick_count % 20 == 0 {
                living.heal(1.0);
            }

            // AI step - movement towards main target
            let mut delta_movement = entity.velocity.load().multiply(1.0, 0.6, 1.0);
            let target_opt = self.mob_entity.get_target();

            if let Some(ref target) = target_opt {
                if target.get_entity().is_alive() {
                    let target_pos = target.get_entity().pos.load();
                    let wither_pos = entity.pos.load();
                    let mut yd = delta_movement.y;

                    if wither_pos.y < target_pos.y
                        || (!self.is_powered() && wither_pos.y < target_pos.y + 5.0)
                    {
                        yd = yd.max(0.0);
                        yd += 0.3 - yd * 0.6;
                    }

                    delta_movement.y = yd;
                    let delta = Vector3::new(
                        target_pos.x - wither_pos.x,
                        0.0,
                        target_pos.z - wither_pos.z,
                    );

                    if delta.horizontal_length_squared() > 9.0 {
                        let scale = delta.normalize();
                        delta_movement.x += scale.x * 0.3 - delta_movement.x * 0.6;
                        delta_movement.z += scale.z * 0.3 - delta_movement.z * 0.6;
                    }

                    self.set_alternative_target(0, target.get_entity().entity_id);

                    // Main head attack
                    let dist_sq = (wither_pos - target_pos).length_squared();
                    if dist_sq <= 400.0 {
                        let attack_timer = self.main_attack_timer.load(Ordering::Relaxed);
                        if attack_timer <= 0 {
                            self.main_attack_timer.store(40, Ordering::Relaxed);
                            let dangerous = rand::random_range(0.0..1.0) < 0.001;
                            let eye_h = target.get_entity().get_eye_height();
                            self.perform_ranged_attack(
                                0,
                                target_pos.x,
                                target_pos.y + eye_h * 0.5,
                                target_pos.z,
                                dangerous,
                            );
                        } else {
                            self.main_attack_timer
                                .store(attack_timer - 1, Ordering::Relaxed);
                        }
                    }
                } else {
                    self.set_alternative_target(0, 0);
                    self.mob_entity.set_target(None);
                }
            } else {
                self.set_alternative_target(0, 0);
            }

            entity.velocity.store(delta_movement);
            if delta_movement.horizontal_length_squared() > 0.05 {
                let yaw = (delta_movement.z.atan2(delta_movement.x).to_degrees() as f32) - 90.0;
                entity.set_rotation(yaw, entity.pitch.load());
            }

            // Side heads attack logic
            let difficulty = world.level_info.load().difficulty;
            let wither_pos = entity.pos.load();

            for i in 1..=2 {
                let next_update = self.next_head_update[i - 1].load(Ordering::Relaxed);
                if tick_count >= next_update {
                    let rand_delay = rand::random_range(0..10);
                    self.next_head_update[i - 1]
                        .store(tick_count + 10 + rand_delay, Ordering::Relaxed);

                    if (difficulty == Difficulty::Normal || difficulty == Difficulty::Hard)
                        && self.idle_head_updates[i - 1].fetch_add(1, Ordering::Relaxed) > 15
                    {
                        let (xt, yt, zt) = (
                            wither_pos.x + rand::random_range(-10.0..10.0),
                            wither_pos.y + rand::random_range(-5.0..5.0),
                            wither_pos.z + rand::random_range(-10.0..10.0),
                        );
                        self.perform_ranged_attack(i, xt, yt, zt, true);
                        self.idle_head_updates[i - 1].store(0, Ordering::Relaxed);
                    }

                    let head_target_id = self.get_alternative_target(i);
                    if head_target_id > 0 {
                        let head_target = world.get_entity_by_id(head_target_id);
                        if let Some(target) = head_target {
                            let t_pos = target.get_entity().pos.load();
                            if target.get_entity().is_alive()
                                && (wither_pos - t_pos).length_squared() <= 900.0
                            {
                                let eye_h = target.get_entity().get_eye_height();
                                self.perform_ranged_attack(
                                    i,
                                    t_pos.x,
                                    t_pos.y + eye_h * 0.5,
                                    t_pos.z,
                                    false,
                                );
                                let next_delay = 40 + rand::random_range(0..20);
                                self.next_head_update[i - 1]
                                    .store(tick_count + next_delay, Ordering::Relaxed);
                                self.idle_head_updates[i - 1].store(0, Ordering::Relaxed);
                            } else {
                                self.set_alternative_target(i, 0);
                            }
                        } else {
                            self.set_alternative_target(i, 0);
                        }
                    } else {
                        let search_box = entity.bounding_box.load().expand(20.0, 8.0, 20.0);
                        let entities = world.get_entities_at_box(&search_box);
                        let candidates: Vec<_> = entities
                            .into_iter()
                            .filter(|e| {
                                e.get_entity().entity_id != entity.entity_id
                                    && e.get_living_entity().is_some()
                                    && e.get_entity().is_alive()
                                    && !e
                                        .get_entity()
                                        .entity_type
                                        .has_tag(&tag::EntityType::MINECRAFT_WITHER_FRIENDS)
                            })
                            .collect();

                        if !candidates.is_empty() {
                            let idx = rand::random_range(0..candidates.len());
                            self.set_alternative_target(i, candidates[idx].get_entity().entity_id);
                        }
                    }
                }
            }

            // Block destruction
            let destroy_tick = self.destroy_blocks_tick.load(Ordering::Relaxed);
            if destroy_tick > 0 {
                let next_destroy = destroy_tick - 1;
                self.destroy_blocks_tick
                    .store(next_destroy, Ordering::Relaxed);

                if next_destroy == 0 && world.level_info.load().game_rules.mob_griefing {
                    let bb = entity.bounding_box.load();
                    let bb_width = bb.max.x - bb.min.x;
                    let bb_height = bb.max.y - bb.min.y;
                    let width = (bb_width as f32 / 2.0 + 1.0).floor() as i32;
                    let height = (bb_height as f32).floor() as i32;
                    let min_pos = entity.block_pos.load();
                    let mut destroyed = false;

                    for dx in -width..=width {
                        for dy in 0..=height {
                            for dz in -width..=width {
                                let bpos = BlockPos::new(
                                    min_pos.0.x + dx,
                                    min_pos.0.y + dy,
                                    min_pos.0.z + dz,
                                );
                                let block = world.get_block(&bpos);
                                if Self::can_destroy(block) {
                                    world.set_block_state(
                                        &bpos,
                                        Block::AIR.default_state.id,
                                        BlockFlags::NOTIFY_ALL,
                                    );
                                    destroyed = true;
                                }
                            }
                        }
                    }

                    if destroyed && !entity.silent.load(Ordering::Relaxed) {
                        world.sync_world_event(
                            WorldEvent::SoundWitherBlockBreak,
                            entity.block_pos.load(),
                            0,
                        );
                    }
                }
            }
        }
    }
}

impl Mob for WitherEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.send_meta_data(
            &[
                Metadata::new(
                    tracked_data::wither::DATA_TARGET_A,
                    VarInt(self.get_alternative_target(0)),
                ),
                Metadata::new(
                    tracked_data::wither::DATA_TARGET_B,
                    VarInt(self.get_alternative_target(1)),
                ),
                Metadata::new(
                    tracked_data::wither::DATA_TARGET_C,
                    VarInt(self.get_alternative_target(2)),
                ),
                Metadata::new(
                    tracked_data::wither::DATA_ID_INV,
                    VarInt(self.get_invulnerable_ticks()),
                ),
            ],
            None,
        );
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.tick_wither();
    }

    fn pre_damage(&self, damage_type: DamageType, source: Option<&dyn EntityBase>) -> bool {
        if damage_type.has_tag(&tag::DamageType::MINECRAFT_WITHER_IMMUNE_TO) {
            return false;
        }

        if let Some(src) = source {
            let src_type = src.get_entity().entity_type;
            if src_type == &EntityType::WITHER {
                return false;
            }
            if src_type.has_tag(&tag::EntityType::MINECRAFT_WITHER_FRIENDS) {
                return false;
            }
            if self.is_powered()
                && (src_type == &EntityType::ARROW
                    || src_type == &EntityType::SPECTRAL_ARROW
                    || src_type == &EntityType::WIND_CHARGE
                    || src_type == &EntityType::BREEZE_WIND_CHARGE)
            {
                return false;
            }
        }

        if self.get_invulnerable_ticks() > 0
            && !damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY)
        {
            return false;
        }

        true
    }

    fn on_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) {
        if self.destroy_blocks_tick.load(Ordering::Relaxed) <= 0 {
            self.destroy_blocks_tick.store(20, Ordering::Relaxed);
        }

        for idle in &self.idle_head_updates {
            idle.fetch_add(3, Ordering::Relaxed);
        }
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() || self.mob_entity.living_entity.health.load() <= 0.0 {
            let world = entity.world.load_full();
            self.remove_all_bossbar(&world);
            if !self.dropped_loot.swap(true, Ordering::SeqCst) {
                let pos = entity.get_entity().block_pos.load();
                world.drop_stack(&pos, ItemStack::new(1, &Item::NETHER_STAR));
            }
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Invul", self.get_invulnerable_ticks());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(invul) = nbt.get_int("Invul") {
            self.set_invulnerable_ticks(invul);
        }
    }
}
