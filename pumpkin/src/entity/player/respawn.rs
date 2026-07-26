use super::Player;
use super::statistics;
use crate::block::blocks::bed::BedBlock;
use crate::entity::EntityBase;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Animation;
use pumpkin_protocol::java::client::play::CEntityAnimation;
use pumpkin_protocol::java::client::play::CPlayerSpawnPosition;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use tracing::debug;
use tracing::warn;

impl Player {
    pub async fn set_respawn_point(
        &self,
        dimension: Dimension,
        block_pos: BlockPos,
        yaw: f32,
        pitch: f32,
        forced: bool,
    ) -> bool {
        if !forced
            && let Some(respawn_point) = self.respawn_point.lock().await.as_ref()
            && dimension == respawn_point.dimension
            && block_pos == respawn_point.position
        {
            return false;
        }

        let bedrock_dimension = match dimension.minecraft_name {
            "minecraft:the_nether" => 1,
            "minecraft:the_end" => 2,
            _ => 0,
        };
        self.client
            .send_packet_now_editioned(
                &CPlayerSpawnPosition::new(
                    block_pos,
                    yaw,
                    pitch,
                    dimension.minecraft_name.to_owned(),
                ),
                &pumpkin_protocol::bedrock::client::CSetSpawnPosition::new(
                    0, // Player spawn
                    block_pos,
                    bedrock_dimension,
                    block_pos,
                ),
            )
            .await;

        *self.respawn_point.lock().await = Some(RespawnPoint {
            dimension,
            position: block_pos,
            yaw,
            force: forced,
        });
        true
    }

    /// Calculates the player's respawn point based on stored spawn data.
    ///
    /// Returns `Some(CalculatedRespawnPoint)` if a valid respawn point exists, `None` otherwise.
    ///
    /// # Behavior
    /// - If `force` flag is set (via `/spawnpoint` command), validates the spawn position is safe
    ///   (both the block and block above allow mob spawn).
    /// - For beds: validates the bed block still exists and finds a valid spawn position around it.
    /// - For respawn anchors (Nether): validates the anchor has charges and finds a valid spawn position.
    /// - Returns `None` if the spawn block is invalid/missing (caller should send
    ///   `NoRespawnBlockAvailable` game event and use world spawn).
    ///
    /// # Note
    /// This function does NOT send any packets. The caller is responsible for
    /// sending `NoRespawnBlockAvailable` if this returns `None`.
    pub async fn calculate_respawn_point(&self) -> Option<CalculatedRespawnPoint> {
        type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;
        type AnchorProperties = pumpkin_data::block_properties::RespawnAnchorLikeProperties;

        let respawn_guard = self.respawn_point.lock().await;
        let respawn_point = respawn_guard.as_ref()?;
        let world = self.world();
        let pos = &respawn_point.position;
        let (block, state_id) = world.get_block_and_state_id(pos);

        // If force is set (from /spawnpoint command), validate position is safe
        if respawn_point.force {
            // For forced spawn, check if both the block and block above allow mob spawn
            let block_state = world.get_block_state(pos);
            let above_state = world.get_block_state(&pos.up());

            // Check if blocks are passable (non-solid or air)
            let block_safe = block_state.is_air() || !block_state.is_solid();
            let above_safe = above_state.is_air() || !above_state.is_solid();

            if block_safe && above_safe {
                let position = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 0.1,
                    f64::from(pos.0.z) + 0.5,
                );
                debug!(
                    "Returning forced spawn point at {:?}, dimension: {:?}",
                    position, respawn_point.dimension
                );
                return Some(CalculatedRespawnPoint {
                    position,
                    yaw: respawn_point.yaw,
                    pitch: 0.0,
                    dimension: respawn_point.dimension.clone(),
                });
            }
            return None;
        }

        // Handle bed respawn
        if block.has_tag(&tag::Block::MINECRAFT_BEDS) {
            let bed_props = BedProperties::from_state_id(state_id, block);
            let facing = bed_props.facing;

            // Try positions around the bed based on facing direction
            // Vanilla tries multiple offset patterns; we use a simplified version
            if let Some(spawn_pos) =
                Self::find_bed_spawn_position(&world, pos, facing, respawn_point.yaw)
            {
                return Some(CalculatedRespawnPoint {
                    position: spawn_pos,
                    yaw: respawn_point.yaw,
                    pitch: 0.0,
                    dimension: respawn_point.dimension.clone(),
                });
            }
            return None;
        }

        // Handle respawn anchor (Nether)
        if block == &Block::RESPAWN_ANCHOR {
            let anchor_props = AnchorProperties::from_state_id(state_id, block);
            let charges = anchor_props.charges;

            // Anchor needs at least 1 charge to work
            if charges == 0 {
                return None;
            }

            // Try positions around the anchor
            if let Some(spawn_pos) = Self::find_anchor_spawn_position(&world, pos) {
                // Decrement charges after successful respawn position found
                let new_charges = charges - 1;
                let mut new_props = anchor_props;
                new_props.charges = new_charges;
                world
                    .set_block_state(
                        pos,
                        new_props.to_state_id(block),
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                return Some(CalculatedRespawnPoint {
                    position: spawn_pos,
                    yaw: respawn_point.yaw,
                    pitch: 0.0,
                    dimension: respawn_point.dimension.clone(),
                });
            }
            return None;
        }

        None
    }

    /// Find a valid spawn position around a bed.
    /// Vanilla uses a complex algorithm based on bed facing direction.
    /// We use a simplified version that tries cardinal directions first.
    fn find_bed_spawn_position(
        world: &Arc<crate::world::World>,
        bed_pos: &BlockPos,
        facing: HorizontalFacing,
        _spawn_angle: f32,
    ) -> Option<Vector3<f64>> {
        // Get offsets based on bed facing direction (vanilla-like order)
        let offsets = Self::get_bed_spawn_offsets(facing);

        for (dx, dz) in offsets {
            let check_pos = BlockPos(Vector3::new(
                bed_pos.0.x + dx,
                bed_pos.0.y,
                bed_pos.0.z + dz,
            ));

            if let Some(pos) = Self::find_respawn_pos(world, &check_pos) {
                return Some(pos);
            }

            // Also try one block down (for beds on elevated platforms)
            let check_pos_down = BlockPos(Vector3::new(
                bed_pos.0.x + dx,
                bed_pos.0.y - 1,
                bed_pos.0.z + dz,
            ));
            if let Some(pos) = Self::find_respawn_pos(world, &check_pos_down) {
                return Some(pos);
            }
        }

        // Try on the bed itself as last resort
        if let Some(pos) = Self::find_respawn_pos(world, bed_pos) {
            return Some(pos);
        }

        None
    }

    /// Get spawn position offsets around a bed based on facing direction.
    /// This is a simplified version of vanilla's getAroundBedOffsets.
    fn get_bed_spawn_offsets(facing: HorizontalFacing) -> Vec<(i32, i32)> {
        let (fx, fz) = match facing {
            HorizontalFacing::North => (0, -1),
            HorizontalFacing::South => (0, 1),
            HorizontalFacing::West => (-1, 0),
            HorizontalFacing::East => (1, 0),
        };

        // Clockwise rotation
        let (rx, rz) = (-fz, fx);

        vec![
            (rx, rz),                   // Right of bed
            (-rx, -rz),                 // Left of bed
            (rx - fx, rz - fz),         // Right-back
            (-rx - fx, -rz - fz),       // Left-back
            (-fx, -fz),                 // Behind foot
            (-fx * 2, -fz * 2),         // Further behind
            (rx + fx, rz + fz),         // Right-front
            (-rx + fx, -rz + fz),       // Left-front
            (fx, fz),                   // In front
            (rx - fx * 2, rz - fz * 2), // Far right-back
        ]
    }

    /// Find a valid spawn position around a respawn anchor.
    fn find_anchor_spawn_position(
        world: &Arc<crate::world::World>,
        anchor_pos: &BlockPos,
    ) -> Option<Vector3<f64>> {
        // Vanilla VALID_HORIZONTAL_SPAWN_OFFSETS
        let horizontal_offsets: [(i32, i32); 8] = [
            (0, -1),
            (-1, 0),
            (0, 1),
            (1, 0),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ];

        // Try at same level, then one down, then one up
        for dy in [0, -1, 1] {
            for (dx, dz) in horizontal_offsets {
                let check_pos = BlockPos(Vector3::new(
                    anchor_pos.0.x + dx,
                    anchor_pos.0.y + dy,
                    anchor_pos.0.z + dz,
                ));

                if let Some(pos) = Self::find_respawn_pos(world, &check_pos) {
                    return Some(pos);
                }
            }
        }

        // Also try directly above the anchor
        let above_pos = anchor_pos.up();
        Self::find_respawn_pos(world, &above_pos)
    }

    /// Check if a position is valid for respawning (vanilla Dismounting.findRespawnPos logic).
    /// Returns the spawn position if valid, None otherwise.
    fn find_respawn_pos(world: &Arc<crate::world::World>, pos: &BlockPos) -> Option<Vector3<f64>> {
        let state = world.get_block_state(pos);
        let below_state = world.get_block_state(&pos.down());

        // Check if block at position is invalid for spawn (e.g., inside solid block)
        let block = world.get_block(pos);
        if block.has_tag(&tag::Block::MINECRAFT_INVALID_SPAWN_INSIDE) {
            return None;
        }

        // Check if block above is also invalid
        let above_block = world.get_block(&pos.up());
        if above_block.has_tag(&tag::Block::MINECRAFT_INVALID_SPAWN_INSIDE) {
            return None;
        }

        // Need solid floor below or at position
        let has_floor = below_state.is_solid() || state.is_solid();
        if !has_floor {
            return None;
        }

        // Position must not be inside a solid block
        if state.is_solid() && !state.is_air() {
            return None;
        }

        // Create player-sized bounding box at this position
        let x = f64::from(pos.0.x) + 0.5;
        let y = f64::from(pos.0.y) + 0.1;
        let z = f64::from(pos.0.z) + 0.5;
        let spawn_pos = Vector3::new(x, y, z);

        // Player dimensions: 0.6 wide, 1.8 tall
        let half_width = 0.3;
        let height = 1.8;
        let player_box = BoundingBox::new(
            Vector3::new(x - half_width, y, z - half_width),
            Vector3::new(x + half_width, y + height, z + half_width),
        );

        // Check if the space is empty (no block collisions)
        if !world.is_space_empty(player_box) {
            return None;
        }

        Some(spawn_pos)
    }

    pub fn sleep(&self, bed_head_pos: BlockPos) {
        // TODO: Stop riding

        self.get_entity().set_pose(EntityPose::Sleeping);
        self.living_entity
            .entity
            .set_pos(bed_head_pos.to_f64().add_raw(0.5, 0.6875, 0.5));
        self.get_entity().send_meta_data(
            &[Metadata::new(
                TrackedData::SLEEPING_POS_ID,
                MetaDataType::OPTIONAL_BLOCK_POS,
                Some(bed_head_pos),
            )],
            None,
        );
        self.get_entity().set_velocity(Vector3::default());

        self.sleeping_since.store(Some(0));
    }

    pub async fn wake_up(&self) {
        let world = self.world();
        let respawn_point = self.respawn_point.lock().await;
        let Some(respawn_point) = respawn_point.as_ref() else {
            warn!("Player waking up should have it's respawn point set on the bed");
            return;
        };

        let (bed, bed_state) = world.get_block_and_state_id(&respawn_point.position);
        BedBlock::set_occupied(false, &world, bed, &respawn_point.position, bed_state).await;

        self.living_entity.entity.set_pose(EntityPose::Standing);
        self.living_entity.entity.set_pos(self.position());
        self.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::SLEEPING_POS_ID,
                MetaDataType::OPTIONAL_BLOCK_POS,
                None::<BlockPos>,
            )],
            None,
        );

        self.set_stat(
            statistics::StatisticCategory::Custom,
            statistics::CustomStatistic::TimeSinceRest as i32,
            0,
        )
        .await;

        let chunk_pos = self.living_entity.entity.chunk_pos.load();
        world.broadcast_to_chunk(
            chunk_pos,
            &CEntityAnimation::new(self.entity_id().into(), Animation::LeaveBed),
        );

        self.sleeping_since.store(None);
    }

    pub async fn respawn(self: &Arc<Self>) {
        self.world().respawn_player(self, false).await;
    }
}

/// Represents the player's stored respawn point (bed/anchor/forced).
#[derive(Debug, Clone, PartialEq)]
pub struct RespawnPoint {
    pub dimension: Dimension,
    pub position: BlockPos,
    pub yaw: f32,
    pub force: bool,
}

pub struct CalculatedRespawnPoint {
    /// The exact position to spawn at (centered in block).
    pub position: Vector3<f64>,
    /// The yaw rotation.
    pub yaw: f32,
    /// The pitch rotation.
    pub pitch: f32,
    /// The dimension to spawn in.
    pub dimension: Dimension,
}
