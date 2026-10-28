use super::BlockEntity;
use pumpkin_data::Block;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use pumpkin_world::world::BlockFlags;
use std::sync::{Arc, Mutex};

use crate::world::World;

pub struct EndGatewayBlockEntity {
    pub position: BlockPos,
    pub age: Mutex<i64>,
    pub teleport_cooldown: Mutex<i32>,
    pub exact_teleport: Mutex<bool>,
    pub exit_portal: Mutex<Option<BlockPos>>,
}

impl BlockEntity for EndGatewayBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let age = nbt.get_long("Age").unwrap_or(0);
        let exact_teleport = nbt.get_bool("ExactTeleport").unwrap_or(false);
        let exit_portal = nbt
            .get_int_array("exit_portal")
            .and_then(|arr| (arr.len() == 3).then(|| BlockPos::new(arr[0], arr[1], arr[2])))
            .or_else(|| {
                nbt.get_compound("exit_portal")
                    .or_else(|| nbt.get_compound("ExitPortal"))
                    .map(|c| {
                        BlockPos::new(
                            c.get_int("X").or_else(|| c.get_int("x")).unwrap_or(0),
                            c.get_int("Y").or_else(|| c.get_int("y")).unwrap_or(0),
                            c.get_int("Z").or_else(|| c.get_int("z")).unwrap_or(0),
                        )
                    })
            });
        Self {
            position,
            age: Mutex::new(age),
            teleport_cooldown: Mutex::new(0),
            exact_teleport: Mutex::new(exact_teleport),
            exit_portal: Mutex::new(exit_portal),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(age) = self.age.lock() {
            nbt.put_long("Age", *age);
        }
        if let Ok(exact_teleport) = self.exact_teleport.lock()
            && *exact_teleport
        {
            nbt.put_bool("ExactTeleport", true);
        }
        if let Ok(exit_portal) = self.exit_portal.lock()
            && let Some(exit) = exit_portal.as_ref()
        {
            nbt.put(
                "exit_portal",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![exit.0.x, exit.0.y, exit.0.z]),
            );
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_long("Age", *self.age.try_lock().ok()?);
        if let Ok(exact_teleport) = self.exact_teleport.try_lock()
            && *exact_teleport
        {
            nbt.put_bool("ExactTeleport", true);
        }
        if let Ok(exit_portal) = self.exit_portal.try_lock()
            && let Some(ref exit) = *exit_portal
        {
            nbt.put(
                "exit_portal",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![exit.0.x, exit.0.y, exit.0.z]),
            );
        }
        Some(nbt)
    }

    fn tick(&self, world: &Arc<World>) {
        let cooling_down = self.is_cooling_down();
        if let Ok(mut age) = self.age.lock() {
            *age += 1;
            if cooling_down {
                if let Ok(mut cooldown) = self.teleport_cooldown.lock() {
                    *cooldown -= 1;
                }
            } else if *age % Self::ATTENTION_INTERVAL == 0 {
                self.trigger_cooldown(world, self.position);
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl EndGatewayBlockEntity {
    pub const ID: &'static str = "minecraft:end_gateway";
    pub const SPAWN_TIME: i64 = 200;
    pub const COOLDOWN_TIME: i32 = 40;
    pub const ATTENTION_INTERVAL: i64 = 2400;
    pub const EVENT_COOLDOWN: u8 = 1;
    pub const GATEWAY_HEIGHT_ABOVE_SURFACE: i32 = 10;

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            age: Mutex::new(0),
            teleport_cooldown: Mutex::new(0),
            exact_teleport: Mutex::new(false),
            exit_portal: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn is_spawning(&self) -> bool {
        self.age.lock().is_ok_and(|age| *age < Self::SPAWN_TIME)
    }

    #[must_use]
    pub fn is_cooling_down(&self) -> bool {
        self.teleport_cooldown.lock().is_ok_and(|cd| *cd > 0)
    }

    #[must_use]
    pub fn get_spawn_percent(&self, a: f32) -> f32 {
        let age = self.age.lock().map_or(0, |g| *g);
        ((age as f32 + a) / Self::SPAWN_TIME as f32).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn get_cooldown_percent(&self, a: f32) -> f32 {
        let cd = self.teleport_cooldown.lock().map_or(0, |g| *g);
        1.0 - ((cd as f32 - a) / Self::COOLDOWN_TIME as f32).clamp(0.0, 1.0)
    }

    pub fn trigger_cooldown(&self, world: &World, pos: BlockPos) {
        if let Ok(mut cd) = self.teleport_cooldown.lock() {
            *cd = Self::COOLDOWN_TIME;
        }
        world.add_synced_block_event(pos, Self::EVENT_COOLDOWN, 0);
    }

    pub fn beam_animation_tick(&self) {
        if let Ok(mut age) = self.age.lock() {
            *age += 1;
        }
        if self.is_cooling_down()
            && let Ok(mut cd) = self.teleport_cooldown.lock()
        {
            *cd -= 1;
        }
    }

    pub fn trigger_event(&self, b0: u8, _b1: u8) -> bool {
        if b0 == Self::EVENT_COOLDOWN {
            if let Ok(mut cd) = self.teleport_cooldown.lock() {
                *cd = Self::COOLDOWN_TIME;
            }
            true
        } else {
            false
        }
    }

    pub fn set_exit_position(&self, exact_position: BlockPos, exact: bool) {
        if let Ok(mut exact_tp) = self.exact_teleport.lock() {
            *exact_tp = exact;
        }
        if let Ok(mut exit) = self.exit_portal.lock() {
            *exit = Some(exact_position);
        }
    }

    #[must_use]
    pub fn get_portal_position(
        &self,
        current_level: &Arc<World>,
        portal_entry_pos: BlockPos,
    ) -> Option<Vector3<f64>> {
        let exit_portal_opt = self.exit_portal.lock().ok().and_then(|opt| *opt);
        if exit_portal_opt.is_none()
            && current_level.dimension == pumpkin_data::dimension::Dimension::THE_END
        {
            let exit_portal_pos =
                Self::find_or_create_valid_teleport_pos(current_level, portal_entry_pos);
            let exit_portal_pos = BlockPos::new(
                exit_portal_pos.0.x,
                exit_portal_pos.0.y + Self::GATEWAY_HEIGHT_ABOVE_SURFACE,
                exit_portal_pos.0.z,
            );
            tracing::debug!("Creating portal at {:?}", exit_portal_pos);
            Self::spawn_gateway_portal(current_level, exit_portal_pos, portal_entry_pos);
            let exact = self.exact_teleport.lock().is_ok_and(|g| *g);
            self.set_exit_position(exit_portal_pos, exact);
        }

        let exit_portal = self.exit_portal.lock().ok().and_then(|opt| *opt)?;
        let exact = self.exact_teleport.lock().is_ok_and(|g| *g);
        let pos = if exact {
            exit_portal
        } else {
            Self::find_exit_position(current_level, exit_portal)
        };
        Some(Vector3::new(
            pos.0.x as f64 + 0.5,
            pos.0.y as f64,
            pos.0.z as f64 + 0.5,
        ))
    }

    fn find_exit_position(level: &World, exit_portal: BlockPos) -> BlockPos {
        let search_center = BlockPos::new(exit_portal.0.x, exit_portal.0.y + 2, exit_portal.0.z);
        let tallest = Self::find_tallest_block(level, search_center, 5, false);
        tracing::debug!(
            "Best exit position for portal at {:?} is {:?}",
            exit_portal,
            tallest
        );
        BlockPos::new(tallest.0.x, tallest.0.y + 1, tallest.0.z)
    }

    fn find_or_create_valid_teleport_pos(
        level: &Arc<World>,
        end_gateway_pos: BlockPos,
    ) -> BlockPos {
        let exit_portal_xz = Self::find_exit_portal_xz_pos_tentative(level, end_gateway_pos);
        let chunk_x = (exit_portal_xz.x / 16.0).floor() as i32;
        let chunk_z = (exit_portal_xz.z / 16.0).floor() as i32;

        let exit_portal_pos = Self::find_valid_spawn_in_chunk(level, chunk_x, chunk_z);
        let exit_portal_pos = exit_portal_pos.map_or_else(
            || {
                let new_exit_portal_pos = BlockPos::new(
                    (exit_portal_xz.x + 0.5).floor() as i32,
                    75,
                    (exit_portal_xz.z + 0.5).floor() as i32,
                );
                tracing::debug!(
                    "Failed to find a suitable block to teleport to, spawning an island on {:?}",
                    new_exit_portal_pos
                );
                Self::spawn_end_island(level, new_exit_portal_pos);
                new_exit_portal_pos
            },
            |pos| {
                tracing::debug!("Found suitable block to teleport to: {:?}", pos);
                pos
            },
        );

        Self::find_tallest_block(level, exit_portal_pos, 16, true)
    }

    fn find_exit_portal_xz_pos_tentative(level: &World, end_gateway_pos: BlockPos) -> Vector3<f64> {
        let mut dir = Vector3::new(end_gateway_pos.0.x as f64, 0.0, end_gateway_pos.0.z as f64);
        let len = dir.x.hypot(dir.z);
        if len > 0.0 {
            dir.x /= len;
            dir.z /= len;
        } else {
            dir.x = 1.0;
        }

        let mut exit_portal_xz = Vector3::new(dir.x * 1024.0, 0.0, dir.z * 1024.0);

        let mut chunk_limit = 16;
        while !Self::is_chunk_empty(level, exit_portal_xz) && chunk_limit > 0 {
            chunk_limit -= 1;
            exit_portal_xz.x -= dir.x * 16.0;
            exit_portal_xz.z -= dir.z * 16.0;
        }

        let mut forward_limit = 16;
        while Self::is_chunk_empty(level, exit_portal_xz) && forward_limit > 0 {
            forward_limit -= 1;
            exit_portal_xz.x += dir.x * 16.0;
            exit_portal_xz.z += dir.z * 16.0;
        }

        exit_portal_xz
    }

    fn is_chunk_empty(level: &World, xz_pos: Vector3<f64>) -> bool {
        let chunk_x = (xz_pos.x / 16.0).floor() as i32;
        let chunk_z = (xz_pos.z / 16.0).floor() as i32;
        let chunk_pos = Vector2::new(chunk_x, chunk_z);
        level
            .level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let sections = chunk
                    .section
                    .block_sections
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                sections.iter().all(|s| match s {
                    pumpkin_world::chunk::palette::BlockPalette::Homogeneous(id) => {
                        pumpkin_data::block_properties::is_air(*id)
                    }
                    pumpkin_world::chunk::palette::BlockPalette::Heterogeneous(_) => false,
                })
            })
            .unwrap_or(true)
    }

    fn find_tallest_block(
        level: &World,
        around: BlockPos,
        dist: i32,
        allow_bedrock: bool,
    ) -> BlockPos {
        let mut tallest: Option<BlockPos> = None;
        let max_y = level.dimension.min_y + level.dimension.height - 1;
        let min_y = level.dimension.min_y;

        for xd in -dist..=dist {
            for zd in -dist..=dist {
                if xd != 0 || zd != 0 || allow_bedrock {
                    let mut y = max_y;
                    let target_min_y = tallest.map_or(min_y, |t| t.0.y);
                    while y > target_min_y {
                        let check_pos = BlockPos::new(around.0.x + xd, y, around.0.z + zd);
                        let (block, state) = level.get_block_and_state(&check_pos);
                        if !state.is_air()
                            && (allow_bedrock || block != &Block::BEDROCK)
                            && state.is_full_cube()
                        {
                            tallest = Some(check_pos);
                            break;
                        }
                        y -= 1;
                    }
                }
            }
        }
        tallest.unwrap_or(around)
    }

    fn find_valid_spawn_in_chunk(level: &World, chunk_x: i32, chunk_z: i32) -> Option<BlockPos> {
        let min_x = chunk_x * 16;
        let max_x = min_x + 15;
        let min_z = chunk_z * 16;
        let max_z = min_z + 15;
        let min_y = 30;
        let max_y = level.dimension.min_y + level.dimension.height - 1;

        let mut closest: Option<BlockPos> = None;
        let mut closest_dist = 0.0f64;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                for y in min_y..=max_y {
                    let pos = BlockPos::new(x, y, z);
                    let (block, _state) = level.get_block_and_state(&pos);
                    if block == &Block::END_STONE {
                        let above = BlockPos::new(x, y + 1, z);
                        let above2 = BlockPos::new(x, y + 2, z);
                        let above_state = level.get_block_state(&above);
                        let above2_state = level.get_block_state(&above2);
                        if !above_state.is_full_cube() && !above2_state.is_full_cube() {
                            let dist = (x * x + z * z) as f64;
                            if closest.is_none() || dist < closest_dist {
                                closest = Some(pos);
                                closest_dist = dist;
                            }
                        }
                    }
                }
            }
        }

        closest
    }

    fn spawn_end_island(world: &Arc<World>, pos: BlockPos) {
        let seed = (pos.0.x as u64) ^ ((pos.0.z as u64) << 32) ^ (pos.0.y as u64);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
        let mut size = random.next_bounded_i32(3) as f32 + 4.0;
        let mut dy = 0i32;
        while size > 0.5 {
            let start = size.copysign(-1.0).floor() as i32;
            let end = size.ceil() as i32;
            let radius_sq = (size + 1.0) * (size + 1.0);

            for dx in start..=end {
                for dz in start..=end {
                    if (dx * dx + dz * dz) as f32 <= radius_sq {
                        let target = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                        world.set_block_state(
                            &target,
                            Block::END_STONE.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        );
                    }
                }
            }

            size -= random.next_bounded_i32(2) as f32 + 0.5;
            dy -= 1;
        }
    }

    fn spawn_gateway_portal(world: &Arc<World>, portal_pos: BlockPos, exit_pos: BlockPos) {
        let origin = portal_pos;
        for dx in -1i32..=1 {
            for dy in -2i32..=2 {
                for dz in -1i32..=1 {
                    let same_x = dx == 0;
                    let same_y = dy == 0;
                    let same_z = dz == 0;
                    let end = dy.abs() == 2;
                    let target = BlockPos::new(origin.0.x + dx, origin.0.y + dy, origin.0.z + dz);

                    if same_x && same_y && same_z {
                        world.set_block_state(
                            &target,
                            Block::END_GATEWAY.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                        let return_gateway = Arc::new(Self {
                            position: target,
                            age: Mutex::new(0),
                            teleport_cooldown: Mutex::new(0),
                            exact_teleport: Mutex::new(false),
                            exit_portal: Mutex::new(Some(exit_pos)),
                        });
                        world.add_block_entity(return_gateway);
                    } else if same_y {
                        world.set_block_state(
                            &target,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                        world.set_block_state(
                            &target,
                            Block::BEDROCK.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    } else {
                        world.set_block_state(
                            &target,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    }
                }
            }
        }
    }
}
