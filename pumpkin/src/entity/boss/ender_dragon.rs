use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;

use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::{chunk::ChunkHeightmapType, world::BlockFlags};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity, mob::MobEntity,
    player::Player,
};
use crate::server::Server;

pub mod flight_history;
pub mod phase;

use flight_history::DragonFlightHistory;
use phase::{EnderDragonPhase, PhaseManager};

#[derive(Clone, Debug, Default)]
pub struct DragonPath {
    pub nodes: Vec<usize>,
    pub index: usize,
}

impl DragonPath {
    #[must_use]
    pub const fn new(nodes: Vec<usize>) -> Self {
        Self { nodes, index: 0 }
    }

    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.index >= self.nodes.len()
    }

    #[must_use]
    pub fn current_node(&self) -> Option<usize> {
        self.nodes.get(self.index).copied()
    }

    pub const fn advance(&mut self) {
        self.index += 1;
    }

    pub const fn finish(&mut self) {
        self.index = self.nodes.len();
    }
}

/// Wraps an angle in degrees to the range [-180, 180].
/// Equivalent to Java's `MathHelper.wrapDegrees(float)`.
#[must_use]
#[inline]
pub fn wrap_degrees(deg: f32) -> f32 {
    let mut d = deg % 360.0;
    if d < -180.0 {
        d += 360.0;
    }
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

pub const NODE_COUNT: usize = 24;
pub const NODE_Y: i32 = 105;
pub const DEATH_TIMER_MAX: i32 = 200;
const GROWL_COOLDOWN_BASE: i32 = 200;
const ARENA_RADIUS: f64 = 192.0;

const NODE_ADJACENCY: [u32; NODE_COUNT] = [
    6146, 8197, 8202, 16404, 32808, 32848, 65696, 131392, 131712, 263424, 526848, 525313, 1581057,
    3166214, 2138120, 6373424, 4358208, 12910976, 9044480, 9706496, 15216640, 13688832, 11763712,
    8257536,
];

#[derive(Clone, Copy, Default, Debug)]
pub struct DragonNode {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl DragonNode {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    #[must_use]
    pub fn dist_sq(&self, other: &Self) -> f64 {
        let (dx, dy, dz) = (self.x - other.x, self.y - other.y, self.z - other.z);
        dx * dx + dy * dy + dz * dz
    }

    #[inline]
    #[must_use]
    pub fn dist_sq_vec(&self, v: Vector3<f64>) -> f64 {
        let (dx, dy, dz) = (self.x - v.x, self.y - v.y, self.z - v.z);
        dx * dx + dy * dy + dz * dz
    }

    #[inline]
    #[must_use]
    pub const fn as_vec3(&self) -> Vector3<f64> {
        Vector3::new(self.x, self.y, self.z)
    }
}

#[must_use]
pub fn find_path(
    nodes: &[Option<DragonNode>; NODE_COUNT],
    from: usize,
    to: usize,
    final_node: Option<DragonNode>,
    minimum_node_index: usize,
) -> Vec<usize> {
    if from == to && final_node.is_none() {
        return vec![];
    }

    let mut g = [f64::MAX; NODE_COUNT];
    let mut came_from = [NODE_COUNT; NODE_COUNT];
    let mut closed = [false; NODE_COUNT];
    let mut open: std::collections::BinaryHeap<(ordered_float::OrderedFloat<f64>, usize)> =
        std::collections::BinaryHeap::new();

    let h = |n: usize| -> f64 {
        if let Some(target) = final_node
            && let Some(node) = nodes[n]
        {
            return node.dist_sq(&target).sqrt();
        }
        match (nodes[n], nodes[to]) {
            (Some(a), Some(b)) => a.dist_sq(&b).sqrt(),
            _ => 0.0,
        }
    };

    g[from] = 0.0;
    open.push((ordered_float::OrderedFloat(-h(from)), from));

    'outer: while let Some((_, cur)) = open.pop() {
        if closed[cur] {
            continue;
        }
        if cur == to {
            break 'outer;
        }
        closed[cur] = true;

        let adj = NODE_ADJACENCY[cur];
        // Java: `minimumNodeIndex` filters out neighbors below this index when
        // no alive crystals exist.  Outer ring (0-11) is excluded, only nodes
        // 12-23 are reachable.
        for nbr in minimum_node_index..NODE_COUNT {
            if (adj >> nbr) & 1 == 0 || closed[nbr] {
                continue;
            }
            let (Some(cn), Some(nn)) = (nodes[cur], nodes[nbr]) else {
                continue;
            };
            let ng = g[cur] + cn.dist_sq(&nn).sqrt();
            if ng < g[nbr] {
                g[nbr] = ng;
                came_from[nbr] = cur;
                open.push((ordered_float::OrderedFloat(-(ng + h(nbr))), nbr));
            }
        }
    }

    let mut path = Vec::new();
    let mut cur = to;
    while cur != NODE_COUNT {
        path.push(cur);
        cur = came_from[cur];
    }
    path.reverse();
    if path.first() == Some(&from) {
        path.remove(0);
    }
    path
}

pub struct EnderDragonPart {
    pub entity: Entity,
    pub dragon_uuid: uuid::Uuid,
    pub part_index: usize,
}

impl EnderDragonPart {
    pub const fn new(entity: Entity, dragon_uuid: uuid::Uuid, part_index: usize) -> Self {
        Self {
            entity,
            dragon_uuid,
            part_index,
        }
    }
}

impl NBTStorage for EnderDragonPart {}

impl EntityBase for EnderDragonPart {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn damage<'a>(
        &'a self,
        source: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let world = self.entity.world.load();
            if let Some(dragon_base) = world
                .entities
                .load()
                .iter()
                .find(|e| e.get_entity().entity_uuid == self.dragon_uuid)
            {
                // Delegate to dragon's damage_part
                if let Some(dragon) = dragon_base.cast_any().downcast_ref::<EnderDragonEntity>() {
                    return dragon
                        .damage_part(
                            dragon_base.as_ref(),
                            self.part_index,
                            amount,
                            damage_type,
                            source,
                        )
                        .await;
                }
                return dragon_base.damage(source, amount, damage_type).await;
            }
            false
        })
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        // EnderDragonPart.get_living_entity() returns None, so the default
        // damage_with_context() silently fails. Route through damage() instead,
        // which correctly delegates to EnderDragonEntity::damage_part().
        self.damage(source.unwrap_or(caller), amount, damage_type)
    }
}

pub struct EnderDragonEntity {
    pub mob_entity: MobEntity,
    pub parts: Vec<Arc<EnderDragonPart>>,

    pub flight_history: Mutex<DragonFlightHistory>,
    pub flap_time: Mutex<f32>,
    pub o_flap_time: Mutex<f32>,

    pub dragon_death_time: Mutex<i32>,
    pub damage_during_sitting: Mutex<f32>,

    pub nodes_initialized: Mutex<bool>,

    pub fight_origin: Mutex<BlockPos>,

    pub phase_manager: PhaseManager,
    pub phase: Mutex<EnderDragonPhase>,
    pub growl_time: Mutex<i32>,
    pub yaw_rot_accel: Mutex<f32>,
    pub holding_pattern_clockwise: Mutex<bool>,
    pub target_location: Mutex<Option<Vector3<f64>>>,

    pub nodes: Mutex<[Option<DragonNode>; NODE_COUNT]>,
    pub path: Mutex<Option<DragonPath>>,

    pub strafe_target: Mutex<Option<Vector3<f64>>>,
    pub target_player: Mutex<Option<uuid::Uuid>>,
    pub ticks_sitting: Mutex<i32>,
    pub sit_attack_timer: Mutex<i32>,
    pub breathing_timer: Mutex<i32>,
    pub sit_scanning_ticks: Mutex<i32>,

    pub slowed_down_by_block: Mutex<bool>,
    pub connected_crystal: Mutex<Option<uuid::Uuid>>,
    pub sitting_flaming_times_run: Mutex<i32>,
    pub seen_target_times: Mutex<i32>,
    pub should_find_new_path: Mutex<bool>,
    /// `StrafePlayerPhase`'s own copy of the direction-toggle flag - Java gives every
    /// phase its own private field of this name; sharing one field between
    /// `TakeoffPhase` and `StrafePlayerPhase` would leak state across phase switches.
    pub strafe_should_find_new_path: Mutex<bool>,
    pub charging_ticks: Mutex<i32>,
    /// UUID of the dragon breath area-effect cloud spawned during `SittingFlaming`.
    pub breath_cloud: Mutex<Option<uuid::Uuid>>,
    /// Last velocity actually broadcast to clients, so we only resend it when it
    /// meaningfully changes (matching vanilla's `EntityTrackerEntry.tick()` dedup),
    /// instead of every tick.
    pub last_sent_velocity: Mutex<Vector3<f64>>,
}

impl EnderDragonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Java EnderDragonPart sizes: head(1,1), neck(3,3), body(5,3),
        // tail1(2,2), tail2(2,2), tail3(2,2), rightWing(4,2), leftWing(4,2)
        const PART_DIMS: [(f32, f32); 8] = [
            (1.0, 1.0),
            (3.0, 3.0),
            (5.0, 3.0),
            (2.0, 2.0),
            (2.0, 2.0),
            (2.0, 2.0),
            (4.0, 2.0),
            (4.0, 2.0),
        ];

        entity.no_clip.store(true, Ordering::Relaxed);
        let base_id = entity.entity_id;
        let dragon_uuid = entity.entity_uuid;
        let world = entity.world.load();

        let _ = Entity::reserve_ids(8);

        let mut parts = Vec::new();
        for i in 1..=8 {
            let part_entity = Entity::from_uuid_with_id(
                base_id + i,
                uuid::Uuid::new_v4(),
                world.clone(),
                entity.pos.load(),
                &EntityType::ENDER_DRAGON,
            );
            let (w, h) = PART_DIMS[(i - 1) as usize];
            let dims = pumpkin_util::math::boundingbox::EntityDimensions {
                width: w,
                height: h,
                eye_height: h,
            };
            part_entity.entity_dimension.store(dims);
            part_entity.bounding_box.store(
                pumpkin_util::math::boundingbox::BoundingBox::new_from_pos(
                    entity.pos.load().x,
                    entity.pos.load().y,
                    entity.pos.load().z,
                    &dims,
                ),
            );
            let part = Arc::new(EnderDragonPart::new(
                part_entity,
                dragon_uuid,
                (i - 1) as usize,
            ));
            // Registered into `world.entities` (silently - no spawn packet) by
            // whoever constructs the dragon, once it knows the world; see
            // `DragonFight::create_new_dragon`.
            parts.push(part);
        }

        let mob_entity = MobEntity::new(entity);
        mob_entity
            .living_entity
            .skip_travel
            .store(true, Ordering::Relaxed);

        Arc::new(Self {
            mob_entity,
            parts,
            flight_history: Mutex::new(DragonFlightHistory::default()),
            flap_time: Mutex::new(0.0),
            o_flap_time: Mutex::new(0.0),
            dragon_death_time: Mutex::new(0),
            damage_during_sitting: Mutex::new(0.0),
            nodes_initialized: Mutex::new(false),
            fight_origin: Mutex::new(BlockPos::new(0, 128, 0)),
            phase_manager: PhaseManager::new(),
            phase: Mutex::new(EnderDragonPhase::Hover),
            growl_time: Mutex::new(GROWL_COOLDOWN_BASE),
            yaw_rot_accel: Mutex::new(0.0),
            holding_pattern_clockwise: Mutex::new(true),
            target_location: Mutex::new(None),
            nodes: Mutex::new([None; NODE_COUNT]),
            path: Mutex::new(None),
            strafe_target: Mutex::new(None),
            target_player: Mutex::new(None),
            ticks_sitting: Mutex::new(0),
            sit_attack_timer: Mutex::new(0),
            breathing_timer: Mutex::new(0),
            sit_scanning_ticks: Mutex::new(0),
            slowed_down_by_block: Mutex::new(false),
            connected_crystal: Mutex::new(None),
            sitting_flaming_times_run: Mutex::new(0),
            seen_target_times: Mutex::new(0),
            should_find_new_path: Mutex::new(false),
            strafe_should_find_new_path: Mutex::new(false),
            charging_ticks: Mutex::new(0),
            breath_cloud: Mutex::new(None),
            last_sent_velocity: Mutex::new(Vector3::new(0.0, 0.0, 0.0)),
        })
    }

    pub async fn set_fight_origin(&self, pos: BlockPos) {
        let mut initialized = self.nodes_initialized.lock().await;
        let mut origin = self.fight_origin.lock().await;
        if *origin != pos {
            *origin = pos;
            *initialized = false;
        }
    }

    fn send_phase_meta_data(&self, phase_type: EnderDragonPhase) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[
                // v1.21.x
                Metadata::new(
                    TrackedData::PHASE_TYPE,
                    MetaDataType::INTEGER,
                    VarInt(phase_type.network_id()),
                ),
                // v26.x
                Metadata::new(
                    TrackedData::PHASE,
                    MetaDataType::INT,
                    VarInt(phase_type.network_id()),
                ),
            ],
            None,
        );
    }

    pub async fn set_phase(&self, phase_type: EnderDragonPhase) {
        // Don't hold `self.phase`'s guard across `end()`/`begin()`: those hooks run
        // arbitrary phase logic on `self` (including, transitively, further
        // `set_phase` calls or reads of the current phase), which would deadlock on
        // this same non-reentrant mutex if held here.
        let old_phase_type = {
            let mut phase_lock = self.phase.lock().await;
            if *phase_lock == phase_type {
                return;
            }
            let old = *phase_lock;
            *phase_lock = phase_type;
            old
        };

        self.phase_manager.get_phase(old_phase_type).end(self).await;

        self.send_phase_meta_data(phase_type);

        self.phase_manager.get_phase(phase_type).begin(self).await;
    }

    async fn ensure_nodes_initialized(&self) {
        let mut initialized = self.nodes_initialized.lock().await;
        if *initialized {
            return;
        }

        let world = self.mob_entity.living_entity.entity.world.load();

        let mut nodes = self.nodes.lock().await;
        for i in 0..NODE_COUNT {
            let mut y_adjustment = 5;
            let node_x;
            let node_z;

            // Java: nodes are at ABSOLUTE world positions centered at (0, 0),
            // NOT offset by fight_origin.  `Mth.floor(60.0F * cos(...))` gives
            // values in [-60, 60] range.  fight_origin only affects heightmap
            // queries and portal_top, not node patrol positions.
            if i < 12 {
                node_x = (60.0 * ((i as f32) * std::f32::consts::TAU / 12.0).cos()).floor() as i32;
                node_z = (60.0 * ((i as f32) * std::f32::consts::TAU / 12.0).sin()).floor() as i32;
            } else if i < 20 {
                let multiplier = i - 12;
                node_x = (40.0 * ((multiplier as f32) * std::f32::consts::TAU / 8.0).cos()).floor()
                    as i32;
                node_z = (40.0 * ((multiplier as f32) * std::f32::consts::TAU / 8.0).sin()).floor()
                    as i32;
                y_adjustment += 10;
            } else {
                let multiplier = i - 20;
                node_x = (20.0 * ((multiplier as f32) * std::f32::consts::TAU / 4.0).cos()).floor()
                    as i32;
                node_z = (20.0 * ((multiplier as f32) * std::f32::consts::TAU / 4.0).sin()).floor()
                    as i32;
            }

            let height = world.get_heightmap_height(
                ChunkHeightmapType::MotionBlockingNoLeaves,
                node_x,
                node_z,
            );
            let node_y = 73.max(height + y_adjustment);
            nodes[i] = Some(DragonNode::new(node_x as f64, node_y as f64, node_z as f64));
        }

        let pos = self.mob_entity.living_entity.entity.pos.load();
        let nearest = Self::nearest_node_in(&nodes, pos);
        let dest = Self::random_node_idx();
        let new_path = find_path(&nodes, nearest, dest, None, 0);
        drop(nodes);

        *self.path.lock().await = Some(DragonPath::new(new_path));
        *initialized = true;
    }

    pub async fn find_closest_node(&self) -> usize {
        let pos = self.mob_entity.living_entity.entity.pos.load();
        self.find_closest_node_to(pos).await
    }

    pub async fn find_closest_node_to(&self, pos: Vector3<f64>) -> usize {
        self.ensure_nodes_initialized().await;
        let nodes = self.nodes.lock().await;

        let world = self.mob_entity.living_entity.entity.world.load();
        let j = if let Some(ref fight) = world.dragon_fight {
            if fight.lock().await.alive_crystals() == 0 {
                12
            } else {
                0
            }
        } else {
            0
        };

        Self::nearest_node_in_range(&nodes, pos, j)
    }

    fn nearest_node_in(nodes: &[Option<DragonNode>; NODE_COUNT], pos: Vector3<f64>) -> usize {
        Self::nearest_node_in_range(nodes, pos, 0)
    }

    fn nearest_node_in_range(
        nodes: &[Option<DragonNode>; NODE_COUNT],
        pos: Vector3<f64>,
        start: usize,
    ) -> usize {
        nodes
            .iter()
            .enumerate()
            .skip(start)
            .filter_map(|(i, n)| n.map(|n| (i, n.dist_sq_vec(pos))))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map_or(0, |(i, _)| i)
    }

    fn random_node_idx() -> usize {
        rand::random_range(0..NODE_COUNT)
    }

    #[inline]
    #[must_use]
    pub fn yaw_toward(from: Vector3<f64>, to: Vector3<f64>) -> f32 {
        (-(to.z - from.z)).atan2(to.x - from.x).to_degrees() as f32 - 90.0
    }

    /// Single source of truth for "the top of the end portal platform", matching Java's
    /// `Vec3d.ofBottomCenter(world.getTopPosition(Heightmap.Type.MOTION_BLOCKING_NO_LEAVES,
    /// EndPortalFeature.offsetOrigin(fightOrigin)))` (`offsetOrigin` is the identity
    /// function). Every phase that used to compute this with its own ad-hoc formula
    /// should call this instead.
    ///
    /// Pumpkin's `World::get_heightmap_height` returns the Y of the top *solid* block
    /// (one less than vanilla's `getTopY`, which returns the first non-solid Y above it),
    /// hence the `+ 1.0` here.
    pub async fn portal_top(&self) -> Vector3<f64> {
        let origin = self.fight_origin.lock().await.0;
        let world = self.mob_entity.living_entity.entity.world.load();

        // Scan from the top of the world downward to find the highest non-air block,
        // instead of relying on the heightmap which may not be accurate for the End
        // fountain structure.
        let top_y = world.get_top_y();
        let mut result_y = world.min_y as f64;
        for y in (world.min_y..=top_y).rev() {
            let pos = pumpkin_util::math::position::BlockPos::new(origin.x, y, origin.z);
            if !world.get_block(&pos).is_air() {
                result_y = y as f64 + 1.0;
                break;
            }
        }

        Vector3::new(origin.x as f64 + 0.5, result_y, origin.z as f64 + 0.5)
    }

    /// Returns the rotation vector adjusted for the current phase, matching
    /// Java's `EnderDragonEntity.getRotationVectorFromPhase(float)`.
    pub async fn get_rotation_vector_from_phase(&self, _tick_progress: f32) -> Vector3<f64> {
        let phase_type = *self.phase.lock().await;
        if phase_type == EnderDragonPhase::Landing || phase_type == EnderDragonPhase::Takeoff {
            let portal_center = self.portal_top().await;
            let entity_pos = self.mob_entity.living_entity.entity.pos.load();
            let dist = entity_pos.distance_squared(portal_center).sqrt();
            let g = (dist / 4.0).max(1.0) as f32;
            let h = 6.0f32 / g;
            let old_pitch = self.mob_entity.living_entity.entity.pitch.load();
            self.mob_entity
                .living_entity
                .entity
                .pitch
                .store(-h * 1.5 * 5.0);
            let result = Vector3::rotation_vector(
                self.mob_entity.living_entity.entity.pitch.load() as f64,
                self.mob_entity.living_entity.entity.yaw.load() as f64,
            );
            self.mob_entity.living_entity.entity.pitch.store(old_pitch);
            result
        } else if phase_type.is_sitting_or_hovering() {
            let old_pitch = self.mob_entity.living_entity.entity.pitch.load();
            self.mob_entity.living_entity.entity.pitch.store(-45.0);
            let result = Vector3::rotation_vector(
                self.mob_entity.living_entity.entity.pitch.load() as f64,
                self.mob_entity.living_entity.entity.yaw.load() as f64,
            );
            self.mob_entity.living_entity.entity.pitch.store(old_pitch);
            result
        } else {
            Vector3::rotation_vector(
                self.mob_entity.living_entity.entity.pitch.load() as f64,
                self.mob_entity.living_entity.entity.yaw.load() as f64,
            )
        }
    }

    pub fn move_relative(&self, speed: f32, relative_movement: Vector3<f64>) {
        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        let movement = Self::get_relative_movement(relative_movement, speed, yaw);
        let entity = &self.mob_entity.living_entity.entity;
        let vel = entity.velocity.load();
        entity.velocity.store(vel + movement);
    }

    /// Java: `EnderDragonEntity.isFlapping()` — detects the exact moment the
    /// wing passes through the bottom of its arc (flap cycle crossing).
    /// Used by `processFlappingMovement()` to trigger the flap sound.
    pub async fn is_flapping(&self) -> bool {
        let flap = *self.flap_time.lock().await;
        let o_flap = *self.o_flap_time.lock().await;
        let old_cos = (o_flap * std::f32::consts::TAU).cos();
        let new_cos = (flap * std::f32::consts::TAU).cos();
        old_cos <= -0.3 && new_cos >= -0.3
    }

    /// Java: `EnderDragonEntity.processFlappingMovement()` — plays the flap
    /// sound when `isFlapping()` returns true.  Called at the start of `aiStep()`.
    pub async fn process_flapping_movement(&self) {
        if self.is_flapping().await {
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            world.play_sound(
                pumpkin_data::sound::Sound::EntityEnderDragonFlap,
                pumpkin_data::sound::SoundCategory::Hostile,
                &entity.pos.load(),
            );
        }
    }

    fn get_relative_movement(
        relative_movement: Vector3<f64>,
        speed: f32,
        yaw: f32,
    ) -> Vector3<f64> {
        let dist = relative_movement.length_squared();
        if dist < 1.0E-7 {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        let vec = if dist > 1.0 {
            relative_movement.normalize()
        } else {
            relative_movement
        } * (speed as f64);
        let sin = (yaw * (std::f32::consts::PI / 180.0)).sin() as f64;
        let cos = (yaw * (std::f32::consts::PI / 180.0)).cos() as f64;
        Vector3::new(vec.x * cos - vec.z * sin, vec.y, vec.z * cos + vec.x * sin)
    }

    pub async fn steer_toward(
        &self,
        pos: Vector3<f64>,
        target: Vector3<f64>,
        max_y_acceleration: f32,
        yaw_acceleration: f32,
    ) {
        let xdd = target.x - pos.x;
        let mut ydd = target.y - pos.y;
        let zdd = target.z - pos.z;
        let dist_sq = xdd * xdd + ydd * ydd + zdd * zdd;

        let horizontal_dist = xdd.hypot(zdd);
        if horizontal_dist > 0.0 {
            ydd = (ydd / horizontal_dist)
                .clamp(-(max_y_acceleration as f64), max_y_acceleration as f64);
        }

        let entity = &self.mob_entity.living_entity.entity;
        let mut vel = entity.velocity.load();
        vel.y += ydd * 0.01;
        entity.velocity.store(vel);

        let yaw = entity.yaw.load();
        let aim_diff = target - pos;
        let aim = if aim_diff.length_squared() > 1e-6 {
            aim_diff.normalize()
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        let dir_vec = Vector3::new(
            (yaw * (std::f32::consts::PI / 180.0)).sin() as f64,
            vel.y,
            -(yaw * (std::f32::consts::PI / 180.0)).cos() as f64,
        );
        let dir = if dir_vec.length_squared() > 1e-6 {
            dir_vec.normalize()
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        let dot = (dir.dot(&aim) as f32 + 0.5).max(0.0) / 1.5;
        if xdd.abs() > 1.0E-5 || zdd.abs() > 1.0E-5 {
            let mut y_rot_d = wrap_degrees(180.0 - (xdd.atan2(zdd).to_degrees() as f32) - yaw);
            y_rot_d = y_rot_d.clamp(-50.0, 50.0);

            let mut y_rot_a = self.yaw_rot_accel.lock().await;
            *y_rot_a *= 0.8;
            *y_rot_a += y_rot_d * yaw_acceleration;
            entity.yaw.store(wrap_degrees(yaw + *y_rot_a * 0.1));
        }

        let span = (2.0 / (dist_sq + 1.0)) as f32;
        self.move_relative(
            0.06 * (dot * span + (1.0 - span)),
            Vector3::new(0.0, 0.0, -1.0),
        );

        // Apply slowed_down_by_block multiplier
        let slowed = *self.slowed_down_by_block.lock().await;
        let actual_vel = entity.velocity.load();
        let final_vel = if slowed {
            actual_vel.multiply(0.8, 0.8, 0.8)
        } else {
            actual_vel
        };

        // move_entity to apply velocity to position (no_clip=true so this just moves pos)
        entity.move_pos(final_vel);

        // Apply damping
        let actual_vel = entity.velocity.load();
        let actual_dir = if actual_vel.length_squared() > 1e-6 {
            actual_vel.normalize()
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };
        let slide = 0.8 + 0.15 * (actual_dir.dot(&dir) + 1.0) / 2.0;
        entity.velocity.store(Vector3::new(
            actual_vel.x * slide,
            actual_vel.y * 0.91,
            actual_vel.z * slide,
        ));
    }

    /// Sends a velocity packet only when the velocity meaningfully changed since the last
    /// send, matching vanilla's `EntityTrackerEntry.tick()` (`alwaysUpdateVelocity`) instead
    /// of blindly resending every tick.
    async fn sync_velocity(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let vel = entity.velocity.load();
        let mut last = self.last_sent_velocity.lock().await;

        let just_stopped = last.length_squared() > 0.0 && vel.length_squared() == 0.0;
        if (vel - *last).length_squared() > 1.0e-7 || just_stopped {
            entity.send_velocity();
            *last = vel;
        }
    }

    async fn tick_growl(&self) {
        if self.phase.lock().await.is_sitting_or_hovering() {
            return;
        }
        let mut t = self.growl_time.lock().await;
        if *t > 0 {
            *t -= 1;
        } else {
            *t = GROWL_COOLDOWN_BASE + rand::random_range(0..100);
        }
    }

    /// Matches Java's `EnderDragonEntity.canSee(Entity)` / `LivingEntity.canSee`: an
    /// unobstructed ray between eye positions.
    pub async fn can_see(&self, target_eye_pos: Vector3<f64>) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let eye_pos = entity.get_eye_pos();
        let world = entity.world.load();
        world
            .raycast(eye_pos, target_eye_pos, async |block_pos, w| {
                let state = w.get_block_state(block_pos);
                state.is_solid()
            })
            .await
            .is_none()
    }

    /// Java: `level.getNearestPlayer(TargetingConditions.forCombat(), ...)` —
    /// combat targeting checks `canBeSeenAsEnemy()`, which for Player returns
    /// `!abilities.invulnerable`. Creative players have `invulnerable = true`,
    /// so they are excluded.
    pub fn find_nearest_player(&self) -> Option<Arc<Player>> {
        let world = self.mob_entity.living_entity.entity.world.load();
        let pos = self.mob_entity.living_entity.entity.pos.load();
        world
            .players
            .load()
            .iter()
            .filter(|p| !p.is_spectator() && !p.is_creative())
            .filter(|p| {
                let p_pos = p.living_entity.entity.pos.load();
                p_pos.distance_squared(pos) < ARENA_RADIUS * ARENA_RADIUS
            })
            .min_by(|a, b| {
                let a_pos = a.living_entity.entity.pos.load();
                let b_pos = b.living_entity.entity.pos.load();
                a_pos
                    .distance_squared(pos)
                    .partial_cmp(&b_pos.distance_squared(pos))
                    .unwrap()
            })
            .cloned()
    }

    /// Java: `this.hurtTime == 0` — the dragon checks the 10-tick hurt animation
    /// timer, NOT the 20-tick invulnerability timer. Pumpkin's `hurt_cooldown`
    /// starts at 20 and counts down, so the equivalent of vanilla's
    /// `hurtTime > 0` (still in hurt animation) is `hurt_cooldown > 10`.
    fn recently_hurt(&self) -> bool {
        self.mob_entity
            .living_entity
            .hurt_cooldown
            .load(Ordering::Relaxed)
            > 10
    }

    /// Pushes/damages players caught in the wings, matching
    /// `EnderDragonEntity.launchLivingEntities` called right after body/wing placement.
    async fn handle_wing_collisions(&self, sitting_or_hovering: bool) {
        if self.recently_hurt() {
            return;
        }

        let world = self.mob_entity.living_entity.entity.world.load();
        let body_bbox = self.parts[2].entity.bounding_box.load();
        let body_center_x = f64::midpoint(body_bbox.min.x, body_bbox.max.x);
        let body_center_z = f64::midpoint(body_bbox.min.z, body_bbox.max.z);

        for wing_idx in [6, 7] {
            let wing_bbox = {
                let expanded = self.parts[wing_idx]
                    .entity
                    .bounding_box
                    .load()
                    .expand(4.0, 2.0, 4.0);
                let offset_box =
                    BoundingBox::new(Vector3::new(0.0, -2.0, 0.0), Vector3::new(0.0, -2.0, 0.0));
                expanded.offset(offset_box)
            };

            for player in world.players.load().iter() {
                if player.is_spectator() || player.is_creative() {
                    continue;
                }
                if player
                    .living_entity
                    .entity
                    .bounding_box
                    .load()
                    .intersects(&wing_bbox)
                {
                    let player_pos = player.get_entity().pos.load();
                    let xd = player_pos.x - body_center_x;
                    let zd = player_pos.z - body_center_z;
                    let dd = (xd * xd + zd * zd).max(0.1);

                    if !sitting_or_hovering {
                        let dragon_tick = self
                            .mob_entity
                            .living_entity
                            .entity
                            .age
                            .load(Ordering::Relaxed);
                        let last_hurt_by_mob = player
                            .living_entity
                            .last_hurt_by_mob_tick
                            .load(Ordering::Relaxed);
                        if last_hurt_by_mob < dragon_tick - 2 {
                            player.get_entity().add_velocity(Vector3::new(
                                xd / dd * 4.0,
                                0.2,
                                zd / dd * 4.0,
                            ));
                            player.damage(self, 5.0, DamageType::MOB_ATTACK).await;
                        }
                    }
                }
            }
        }
    }

    /// Damages players caught in the head/neck, matching
    /// `EnderDragonEntity.damageLivingEntities` called right after head/neck placement.
    async fn handle_head_collisions(&self) {
        if self.recently_hurt() {
            return;
        }

        let world = self.mob_entity.living_entity.entity.world.load();
        for head_idx in [0, 1] {
            let part_bbox = self.parts[head_idx]
                .entity
                .bounding_box
                .load()
                .expand(1.0, 1.0, 1.0);

            for player in world.players.load().iter() {
                if player.is_spectator() || player.is_creative() {
                    continue;
                }
                if player
                    .living_entity
                    .entity
                    .bounding_box
                    .load()
                    .intersects(&part_bbox)
                {
                    player.damage(self, 10.0, DamageType::MOB_ATTACK).await;
                }
            }
        }
    }

    async fn tick_crystal_healing(&self) {
        let world = self.mob_entity.living_entity.entity.world.load();
        let pos = self.mob_entity.living_entity.entity.pos.load();
        let age = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed);

        // Check if connected crystal still exists
        {
            let mut connected = self.connected_crystal.lock().await;
            if let Some(crystal_uuid) = *connected {
                let still_exists = world.entities.load().iter().any(|e| {
                    e.get_entity().entity_uuid == crystal_uuid
                        && e.get_entity().entity_type == &EntityType::END_CRYSTAL
                });
                if !still_exists {
                    *connected = None;
                }
            }
        }

        // Every 10 ticks, heal from connected crystal
        if age % 10 == 0 {
            let connected = self.connected_crystal.lock().await;
            if connected.is_some() {
                let living = &self.mob_entity.living_entity;
                if living.health.load() < living.get_max_health() {
                    living.heal(1.0);
                }
            }
        }

        // Randomly find nearest crystal
        if rand::random_range(0..10) == 0 {
            let mut nearest_crystal = None;
            let mut min_dist_sq = 1024.0; // 32 blocks

            for entity in world.entities.load().iter() {
                if entity.get_entity().entity_type == &EntityType::END_CRYSTAL {
                    let crystal_pos = entity.get_entity().pos.load();
                    let dist_sq = pos.distance_squared(crystal_pos);
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                        nearest_crystal = Some(entity.get_entity().entity_uuid);
                    }
                }
            }

            *self.connected_crystal.lock().await = nearest_crystal;
        }
    }

    /// Java: `destroyBlocks(head) | destroyBlocks(neck) | destroyBlocks(body)`, called
    /// right after tail placement in `tickMovement`.
    async fn tick_block_breaking(&self) {
        if self.phase.lock().await.is_sitting_or_hovering() {
            *self.slowed_down_by_block.lock().await = false;
            return;
        }

        let world = self.mob_entity.living_entity.entity.world.load();
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        let mut any_slowed = false;

        // Check per-part (head, neck, body)
        for part_idx in [0, 1, 2] {
            let bbox = self.parts[part_idx].entity.bounding_box.load();
            let min = bbox.min_block_pos();
            let max = bbox.max_block_pos();

            for pos in BlockPos::iterate(min, max) {
                let block = world.get_block(&pos);
                if block.is_air() {
                    continue;
                }

                // DRAGON_TRANSPARENT blocks are ignored (light, fire, soul_fire)
                if block
                    .id
                    .has_tag(pumpkin_data::tag::Block::MINECRAFT_DRAGON_TRANSPARENT)
                {
                    continue;
                }

                // Match vanilla: if mob griefing is on and block is NOT dragon immune, break it.
                // Otherwise (griefing off OR dragon immune), mark as slowed.
                if mob_griefing
                    && !block
                        .id
                        .has_tag(pumpkin_data::tag::Block::MINECRAFT_DRAGON_IMMUNE)
                {
                    world
                        .set_block_state(&pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL)
                        .await;
                } else {
                    any_slowed = true;
                }
            }
        }

        *self.slowed_down_by_block.lock().await = any_slowed;
    }

    async fn tick_parts(&self) {
        let history: tokio::sync::MutexGuard<'_, DragonFlightHistory> =
            self.flight_history.lock().await;
        let p5 = history.get(5);
        let p10 = history.get(10);
        let p0 = history.get(0);

        let tilt = (p5.y - p10.y) as f32 * 10.0 * (std::f32::consts::PI / 180.0);
        let cc_tilt = tilt.cos() as f64;
        let ss_tilt = tilt.sin() as f64;

        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        let rot1 = yaw * (std::f32::consts::PI / 180.0);
        let ss1 = rot1.sin() as f64;
        let cc1 = rot1.cos() as f64;

        let pos = self.mob_entity.living_entity.entity.pos.load();

        // Body
        self.parts[2]
            .entity
            .set_pos(Vector3::new(pos.x + ss1 * 0.5, pos.y, pos.z - cc1 * 0.5));

        // Wings
        self.parts[6].entity.set_pos(Vector3::new(
            pos.x + cc1 * 4.5,
            pos.y + 2.0,
            pos.z + ss1 * 4.5,
        ));
        self.parts[7].entity.set_pos(Vector3::new(
            pos.x - cc1 * 4.5,
            pos.y + 2.0,
            pos.z - ss1 * 4.5,
        ));

        let sitting_or_hovering = self.phase.lock().await.is_sitting_or_hovering();

        // Java: wing collision damage/push happens right after body/wing placement,
        // before the head and neck are placed.
        self.handle_wing_collisions(sitting_or_hovering).await;

        let head_y_offset = if sitting_or_hovering {
            -1.0
        } else {
            (p5.y - p0.y) as f64
        };

        let yaw_accel = *self.yaw_rot_accel.lock().await;
        let rot2 = (yaw - yaw_accel * 0.01) * (std::f32::consts::PI / 180.0);
        let ss2 = rot2.sin() as f64;
        let cc2 = rot2.cos() as f64;

        // Head & Neck
        self.parts[0].entity.set_pos(Vector3::new(
            pos.x + ss2 * 6.5 * cc_tilt,
            pos.y + head_y_offset + ss_tilt * 6.5,
            pos.z - cc2 * 6.5 * cc_tilt,
        ));
        self.parts[1].entity.set_pos(Vector3::new(
            pos.x + ss2 * 5.5 * cc_tilt,
            pos.y + head_y_offset + ss_tilt * 5.5,
            pos.z - cc2 * 5.5 * cc_tilt,
        ));

        // Java: head/neck contact damage happens right after they're placed, before tails.
        self.handle_head_collisions().await;

        // Tails
        for i in 0..3 {
            let pi = history.get(12 + i * 2);
            let rot = yaw * (std::f32::consts::PI / 180.0)
                + wrap_degrees(pi.y_rot - p5.y_rot).to_radians();
            let ss = rot.sin() as f64;
            let cc = rot.cos() as f64;
            let dd = (i + 1) as f64 * 2.0;

            self.parts[3 + i as usize].entity.set_pos(Vector3::new(
                pos.x - (ss1 * 1.5 + ss * dd) * cc_tilt,
                pos.y + (pi.y - p5.y) - (dd + 1.5) * ss_tilt + 1.5,
                pos.z + (cc1 * 1.5 + cc * dd) * cc_tilt,
            ));
        }

        drop(history);

        // Java: block-breaking (`destroyBlocks`) happens right after tail placement.
        self.tick_block_breaking().await;

        // Parts are not networked entities: vanilla clients derive their positions
        // locally from the dragon's own spawn packet/frame history, so no per-part
        // position packet is sent here - `set_pos` above only keeps the server-side
        // bounding boxes (hit detection, block breaking, collisions) in sync.
    }

    pub async fn ai_step(&self) {
        self.mob_entity.living_entity.entity.update_last_pos();
        self.ensure_nodes_initialized().await;

        let is_dead = self.mob_entity.living_entity.health.load() <= 0.0;

        // Java: `oFlapTime = flapTime` always runs, but `flapTime += flapSpeed` only
        // when NOT dead (`EnderDragon.java:174-191`).
        {
            let mut o_flap = self.o_flap_time.lock().await;
            let mut flap = self.flap_time.lock().await;
            *o_flap = *flap;

            if !is_dead {
                let sitting = self.phase.lock().await.is_sitting_or_hovering();
                let slowed = *self.slowed_down_by_block.lock().await;
                let vel = self.mob_entity.living_entity.entity.velocity.load();

                let g = if sitting {
                    0.1
                } else {
                    let hspeed = vel.horizontal_length() as f32;
                    let base = 0.2 / (hspeed * 10.0 + 1.0);
                    let g = base * 2.0f32.powf(vel.y as f32);
                    if slowed { g * 0.5 } else { g }
                };

                // Java does NOT wrap flapTime — it lets it grow unbounded.
                // The client reads cos(flapTime * 2*PI), and cos is periodic.
                *flap += g;
            }
        }

        // Java: `processFlappingMovement()` — plays the wing-flap sound when
        // the wing crosses the bottom of its arc.
        self.process_flapping_movement().await;

        // Wrap yaw to [-180, 180] every tick, matching Java's MathHelper.wrapDegrees
        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        self.mob_entity
            .living_entity
            .entity
            .yaw
            .store(wrap_degrees(yaw));

        let y = self.mob_entity.living_entity.entity.pos.load().y;
        let yaw = self.mob_entity.living_entity.entity.yaw.load();
        self.flight_history.lock().await.record(y, yaw);

        self.tick_growl().await;

        // Java: checkCrystals() only runs when !isDeadOrDying()
        if !is_dead {
            self.tick_crystal_healing().await;
        }

        {
            let world = self.mob_entity.living_entity.entity.world.load();
            if let Some(ref fight_mutex) = world.dragon_fight {
                let living = &self.mob_entity.living_entity;
                fight_mutex
                    .lock()
                    .await
                    .update_dragon(&world, living.health.load(), living.get_max_health())
                    .await;
            }
        }

        let phase_before: EnderDragonPhase = *self.phase.lock().await;

        if phase_before.is_sitting_or_hovering() {
            *self.ticks_sitting.lock().await += 1;
        }

        // Java: `if (this.isDead())` (health <= 0) skips collisions, block-breaking,
        // phase ticking, movement and part placement entirely for the tick.
        // EXCEPTION: the Dying phase must still tick even when dead, because the
        // death timer (dragonDeathTime 0→200) is driven by DyingPhase.tick(), not
        // by LivingEntity.tick(). Without this, the dragon would be removed after
        // only 20 ticks instead of 200.
        if !is_dead || phase_before == EnderDragonPhase::Dying {
            let mut phase = self.phase_manager.get_phase(phase_before);
            phase.tick(self).await;

            // Java: if the phase changed mid-tick, the new phase is ticked again in the
            // same tick (`EnderDragonEntity.java:206-212`), so acceleration parameters
            // below are always read from the phase that's actually active now.
            let phase_after: EnderDragonPhase = *self.phase.lock().await;
            if phase_after != phase_before {
                phase = self.phase_manager.get_phase(phase_after);
                phase.tick(self).await;
            }

            // Skip movement, steering, and part placement when dead — only the
            // DyingPhase tick above should have run.
            let is_dead_now = self.mob_entity.living_entity.health.load() <= 0.0;
            if !is_dead_now {
                // Java: `this.yBodyRot = this.getYRot()` — sync body rotation
                // to match yaw every tick.
                let yaw = self.mob_entity.living_entity.entity.yaw.load();
                self.mob_entity.living_entity.entity.body_yaw.store(yaw);

                let target_location = *self.target_location.lock().await;
                if let Some(target) = target_location {
                    let pos = self.mob_entity.living_entity.entity.pos.load();
                    let yaw_accel = phase.get_yaw_acceleration(self).await;
                    self.steer_toward(pos, target, phase.get_fly_speed(), yaw_accel)
                        .await;
                }

                self.mob_entity.living_entity.entity.send_pos_rot();
                self.tick_parts().await;
            }
        } else {
            // Java: EnderDragonEntity.aiStep() when isDeadOrDying() — spawns
            // per-tick EXPLOSION particles (separate from the EXPLOSION_EMITTER
            // particles spawned in tickDeath at ticks 180-200).
            let entity = &self.mob_entity.living_entity.entity;
            let pos = entity.pos.load();
            let xo = (rand::random::<f32>() - 0.5) * 8.0;
            let yo = (rand::random::<f32>() - 0.5) * 4.0;
            let zo = (rand::random::<f32>() - 0.5) * 8.0;
            let world = entity.world.load();
            world.spawn_particle(
                Vector3::new(
                    pos.x + xo as f64,
                    pos.y + 2.0 + yo as f64,
                    pos.z + zo as f64,
                ),
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::Explosion,
            );
        }

        self.sync_velocity().await;
    }

    pub async fn damage_part(
        &self,
        self_dyn: &dyn EntityBase,
        part_index: usize,
        amount: f32,
        damage_type: DamageType,
        source: &dyn EntityBase,
    ) -> bool {
        let phase_type = *self.phase.lock().await;
        if phase_type == EnderDragonPhase::Dying {
            return false;
        }

        let phase = self.phase_manager.get_phase(phase_type);
        let amount = phase.modify_damage_taken(amount);

        // Non-head parts take reduced damage
        let amount = if part_index != 0 {
            amount / 4.0 + amount.min(1.0)
        } else {
            amount
        };

        if amount < 0.01 {
            return false;
        }

        // Only player attacks or ALWAYS_HURTS_ENDER_DRAGONS damage applies
        let is_player = source.get_player().is_some();
        // NOTE: Using a direct enum check because DamageType `has_tag` is broken —
        // DamageType tag u16 ID arrays are always empty in the generated data, so
        // `has_tag()` always returns false for DamageType. This workaround can be
        // removed once the tag code generation is fixed to populate DamageType IDs.
        let is_always_hurts = matches!(
            damage_type,
            DamageType::FIREWORKS
                | DamageType::EXPLOSION
                | DamageType::PLAYER_EXPLOSION
                | DamageType::BAD_RESPAWN_POINT
        );

        if is_player || is_always_hurts {
            let old_health = self.mob_entity.living_entity.health.load();
            self.mob_entity
                .living_entity
                .damage_with_context(
                    self_dyn,
                    amount,
                    damage_type,
                    None,
                    Some(source),
                    Some(source),
                )
                .await;

            let new_health = self.mob_entity.living_entity.health.load();

            // Java: handleKillingBlow() transitions to DYING when NOT sitting.
            // When sitting, vanilla does nothing here — but tickDeath() still
            // runs from LivingEntity.tick().  Since Rust has no tickDeath()
            // (the death timer lives in DyingPhase.tick()), we MUST transition
            // to Dying regardless of sitting state so the death animation plays.
            if new_health <= 0.0 {
                self.mob_entity.living_entity.set_health(1.0);
                self.set_phase(EnderDragonPhase::Dying).await;
            } else if phase.is_sitting_or_hovering() {
                let mut dmg_sitting = self.damage_during_sitting.lock().await;
                *dmg_sitting += old_health - new_health;
                let max_health = self.mob_entity.living_entity.get_max_health();
                if *dmg_sitting > 0.25 * max_health {
                    *dmg_sitting = 0.0;
                    drop(dmg_sitting);
                    self.set_phase(EnderDragonPhase::Takeoff).await;
                }
            }
        }

        true
    }

    /// Called when an end crystal is destroyed, matching
    /// `EnderDragonEntity.crystalDestroyed(ServerWorld, EndCrystalEntity, BlockPos, DamageSource)`.
    pub async fn crystal_destroyed(
        &self,
        self_dyn: &dyn EntityBase,
        crystal_uuid: uuid::Uuid,
        crystal_pos: Vector3<f64>,
        attacker: Option<&Player>,
    ) {
        // Java: CRYSTAL_DESTROY_TARGETING has range 64.0.
        const CRYSTAL_DESTROY_RANGE: f64 = 64.0;

        let world = self.mob_entity.living_entity.entity.world.load();

        // Prefer the actual attacker; otherwise fall back to the closest player to the
        // crystal (Java: `world.getNearestPlayer(CRYSTAL_DESTROY_TARGETING, pos.x, pos.y, pos.z)`).
        let player = if let Some(p) = attacker {
            world
                .players
                .load()
                .iter()
                .find(|wp| wp.gameprofile.id == p.gameprofile.id)
                .cloned()
        } else {
            world
                .players
                .load()
                .iter()
                .filter(|p| !p.is_spectator() && !p.is_creative())
                .filter(|p| {
                    let p_pos = p.living_entity.entity.pos.load();
                    p_pos.distance_squared(crystal_pos)
                        < CRYSTAL_DESTROY_RANGE * CRYSTAL_DESTROY_RANGE
                })
                .min_by(|a, b| {
                    let a_pos = a.living_entity.entity.pos.load();
                    let b_pos = b.living_entity.entity.pos.load();
                    a_pos
                        .distance_squared(crystal_pos)
                        .partial_cmp(&b_pos.distance_squared(crystal_pos))
                        .unwrap()
                })
                .cloned()
        };

        if *self.connected_crystal.lock().await == Some(crystal_uuid) {
            let source: &dyn EntityBase = attacker.map_or(self_dyn, |p| p as &dyn EntityBase);
            self.damage_part(self_dyn, 0, 10.0, DamageType::EXPLOSION, source)
                .await;
        }

        let phase_type = *self.phase.lock().await;
        self.phase_manager
            .get_phase(phase_type)
            .crystal_destroyed(self, player)
            .await;
    }
}

impl NBTStorage for EnderDragonEntity {}

/// The dragon does not go through the generic `Mob` tick pipeline: vanilla's
/// `EnderDragonEntity.tickMovement()` never calls `super.tickMovement()`, so there is no
/// `travel()`, gravity, goal-selector AI, navigator, or `LookControl`/`MoveControl` for it.
/// All of that would double-integrate position/velocity on top of `ai_step`'s own
/// `steer_toward` physics. `LivingEntity::tick` is still reused for the bookkeeping the
/// dragon does need (status effects, hurt cooldown, death timer, ...) via the
/// `skip_travel` flag, which makes it skip only the movement/AI portion.
impl EntityBase for EnderDragonEntity {
    fn get_entity(&self) -> &Entity {
        &self.mob_entity.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.mob_entity.living_entity)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    /// Only the dragon's parts (head/neck/body/tail/wings) are hittable; the main entity
    /// itself is not (`EnderDragonEntity.java:761`).
    fn can_hit(&self) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.0
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let phase = *self.phase.lock().await;
            self.send_phase_meta_data(phase);
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Body part, matching `EnderDragonEntity.damage()` (`:497-499`).
            self.damage_part(caller, 2, amount, damage_type, source.unwrap_or(caller))
                .await
        })
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.tick(caller, server).await;

            // Java: the dragon has its OWN 200-tick death timer (dragonDeathTime)
            // handled by DyingPhase.tick(), NOT by LivingEntity's 20-tick removal.
            // Reset death_time to 0 after LivingEntity::tick() so it never reaches
            // 20 and triggers the generic entity removal.
            if self.mob_entity.living_entity.health.load() <= 0.0 {
                self.mob_entity
                    .living_entity
                    .death_time
                    .store(0, Ordering::Relaxed);
            }

            self.ai_step().await;
        })
    }
}

pub trait Vector3Ext {
    fn distance_squared(self, other: Self) -> f64;
}

impl Vector3Ext for Vector3<f64> {
    fn distance_squared(self, other: Self) -> f64 {
        let (dx, dy, dz) = (self.x - other.x, self.y - other.y, self.z - other.z);
        dx * dx + dy * dy + dz * dz
    }
}
