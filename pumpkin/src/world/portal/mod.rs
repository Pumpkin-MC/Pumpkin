use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use super::World;
use border::BorderSnapshot;

pub mod border;
pub mod chunk_view;
pub mod end;
pub mod nether;
pub mod poi;
pub mod rectangle;
pub mod spiral;

pub use nether::{NetherPortal, PortalSearchResult};
pub use poi::PortalPoiStorage;

/// Vanilla `DimensionType.getTeleportationScale` (`DimensionType.java:77-81`).
///
/// The ratio of the source dimension's `coordinate_scale` to the destination's,
/// which is 8 going overworld -> nether and 1/8 coming back.
#[must_use]
pub fn teleportation_scale(from: &Dimension, to: &Dimension) -> f64 {
    from.coordinate_scale / to.coordinate_scale
}

#[derive(Clone)]
pub struct SourcePortalInfo {
    pub lower_corner: BlockPos,
    pub axis: HorizontalAxis,
    pub width: u32,
    pub height: u32,
}

impl From<&PortalSearchResult> for SourcePortalInfo {
    fn from(result: &PortalSearchResult) -> Self {
        Self {
            lower_corner: result.lower_corner,
            axis: result.axis,
            width: result.width,
            height: result.height,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PortalType {
    Nether,
    End,
}

impl PortalType {
    pub fn get_portal_transition_time(
        &self,
        current_world: &World,
        entity: &dyn crate::entity::EntityBase,
    ) -> u32 {
        match self {
            Self::End => 0,
            Self::Nether => {
                let entity_type = entity.get_entity().entity_type;
                let level_info = current_world.level_info.load();
                match entity_type.id {
                    id if id == pumpkin_data::entity::EntityType::PLAYER.id => (current_world
                        .get_player_by_id(entity.get_entity().entity_id))
                    .map_or(80, |player| match player.gamemode.load() {
                        pumpkin_util::GameMode::Creative => {
                            level_info.game_rules.players_nether_portal_creative_delay as u32
                        }
                        _ => level_info.game_rules.players_nether_portal_default_delay as u32,
                    }),
                    _ => 0,
                }
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn get_portal_destination(
        &self,
        current_level: &World,
        dest_world: Arc<World>,
        caller: &Arc<dyn crate::entity::EntityBase>,
        _portal_entry_pos: BlockPos,
        source_portal: Option<SourcePortalInfo>,
    ) -> Option<TeleportTransition> {
        match self {
            Self::End => {
                let is_end_portal = dest_world.dimension == Dimension::THE_END
                    || current_level.dimension == Dimension::THE_END;

                if is_end_portal {
                    if dest_world.dimension == Dimension::THE_END {
                        // Entering the End: spawn on the obsidian platform at (100, 49, 0) for players, or (100, 50, 0) for other entities
                        let is_player = caller
                            .get_living_entity()
                            .is_some_and(crate::entity::living::LivingEntity::is_player);
                        let y = if is_player { 49.0 } else { 50.0 };

                        // Ensure chunks covering the platform are loaded/generated
                        dest_world
                            .get_block_state_async(&BlockPos::new(98, 49, -2))
                            .await;
                        dest_world
                            .get_block_state_async(&BlockPos::new(102, 49, 2))
                            .await;

                        // Generate/regenerate the obsidian platform (5x5 obsidian at Y=48, and 5x5x3 air above it)
                        let platform_pos = BlockPos::new(100, 49, 0);
                        for dx in -2..=2 {
                            for dz in -2..=2 {
                                for dy in -1..3 {
                                    let block = if dy == -1 {
                                        Block::OBSIDIAN
                                    } else {
                                        Block::AIR
                                    };
                                    let target_pos = BlockPos::new(
                                        platform_pos.0.x + dx,
                                        platform_pos.0.y + dy,
                                        platform_pos.0.z + dz,
                                    );
                                    dest_world
                                        .set_block_state(
                                            &target_pos,
                                            block.default_state.id,
                                            BlockFlags::NOTIFY_ALL,
                                        )
                                        .await;
                                }
                            }
                        }

                        Some(TeleportTransition {
                            new_world: dest_world,
                            position: Vector3::new(100.5f64, y, 0.5f64),
                            yaw: Some(90.0f32),
                            pitch: None,
                        })
                    } else {
                        // Leaving the End through the exit portal: return to overworld spawn
                        if let Some(player) =
                            current_level.get_player_by_id(caller.get_entity().entity_id)
                        {
                            match player.client.as_ref() {
                                crate::net::ClientPlatform::Java(client) => {
                                    client
                                        .enqueue_packet(&pumpkin_protocol::java::client::play::CGameEvent::new(
                                            pumpkin_protocol::java::client::play::GameEvent::WinGame,
                                            1.0,
                                        ))
                                        .await;
                                }
                                crate::net::ClientPlatform::Bedrock(client) => {
                                    client
                                        .send_game_packet(
                                            &pumpkin_protocol::bedrock::client::CShowCredits::new(
                                                pumpkin_protocol::codec::var_ulong::VarULong(
                                                    caller.get_entity().entity_id as u64,
                                                ),
                                                pumpkin_protocol::codec::var_int::VarInt(0),
                                            ),
                                        )
                                        .await;
                                }
                            }
                        }

                        let info = dest_world.level_info.load();
                        Some(TeleportTransition {
                            new_world: dest_world,
                            position: Vector3::new(
                                f64::from(info.spawn_x) + 0.5,
                                f64::from(info.spawn_y),
                                f64::from(info.spawn_z) + 0.5,
                            ),
                            yaw: None,
                            pitch: None,
                        })
                    }
                } else {
                    None
                }
            }
            Self::Nether => {
                let pos = caller.get_entity().pos.load();
                let current_yaw = caller.get_entity().yaw.load();
                let dimensions = caller.get_entity().entity_dimension.load();
                let scale_factor =
                    teleportation_scale(&current_level.dimension, &dest_world.dimension);
                // NetherPortalBlock.java:138: the scaled position is clamped to
                // the destination world border *before* any portal search, so a
                // transit near the border cannot aim outside it.
                let target_pos = BorderSnapshot::capture(&dest_world).await.clamp_coords(
                    pos.x * scale_factor,
                    pos.y,
                    pos.z * scale_factor,
                );

                let source_axis = source_portal.as_ref().map(|p| p.axis);

                let (final_pos, yaw) = if let Some(dest_result) =
                    NetherPortal::search_for_portal(&dest_world, target_pos).await
                {
                    let base_pos = source_portal.as_ref().map_or_else(
                        || dest_result.get_teleport_position(),
                        |source| {
                            let source_result = PortalSearchResult {
                                lower_corner: source.lower_corner,
                                axis: source.axis,
                                width: source.width,
                                height: source.height,
                            };
                            let relative_pos = source_result.entity_pos_in_portal(pos, &dimensions);
                            dest_result.calculate_exit_position(relative_pos, &dimensions)
                        },
                    );
                    let final_pos =
                        dest_result.find_open_position(&dest_world, base_pos, &dimensions);
                    let yaw = dest_result.calculate_teleport_yaw(current_yaw, source_axis);
                    (final_pos, Some(yaw))
                } else if let Some((build_pos, axis, is_fallback)) =
                    // NetherPortalBlock.java:152: the new portal keeps the axis of
                    // the portal that was entered, defaulting to X when the entry
                    // block has no axis (e.g. it was already broken).
                    NetherPortal::find_safe_location(
                            &dest_world,
                            target_pos,
                            source_axis.unwrap_or(HorizontalAxis::X),
                        )
                        .await
                {
                    NetherPortal::build_portal_frame(&dest_world, build_pos, axis, is_fallback)
                        .await;
                    let new_portal = PortalSearchResult {
                        lower_corner: build_pos,
                        axis,
                        width: 2,
                        height: 3,
                    };
                    let center_pos = new_portal.get_teleport_position();
                    let final_pos =
                        new_portal.find_open_position(&dest_world, center_pos, &dimensions);
                    let yaw = new_portal.calculate_teleport_yaw(current_yaw, source_axis);
                    (final_pos, Some(yaw))
                } else {
                    (target_pos.0.to_f64(), None)
                };

                Some(TeleportTransition {
                    new_world: dest_world,
                    position: final_pos,
                    yaw,
                    pitch: None,
                })
            }
        }
    }
}

pub struct TeleportTransition {
    pub new_world: Arc<World>,
    pub position: Vector3<f64>,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
}

pub struct PortalProcessor {
    pub portal_type: PortalType,
    pub entry_position: BlockPos,
    pub portal_time: u32,
    pub inside_portal_this_tick: bool,
    pub destination_world: Arc<World>,
    pub source_portal: Option<SourcePortalInfo>,
}

impl PortalProcessor {
    pub const fn new(
        portal_type: PortalType,
        entry_position: BlockPos,
        destination_world: Arc<World>,
    ) -> Self {
        Self {
            portal_type,
            entry_position,
            portal_time: 0,
            inside_portal_this_tick: true,
            destination_world,
            source_portal: None,
        }
    }

    pub const fn set_source_portal(&mut self, info: SourcePortalInfo) {
        self.source_portal = Some(info);
    }

    pub fn process_portal_teleportation(
        &mut self,
        current_world: &World,
        entity: &dyn crate::entity::EntityBase,
        allowed_to_teleport: bool,
    ) -> bool {
        if self.inside_portal_this_tick {
            self.inside_portal_this_tick = false;
            if allowed_to_teleport {
                self.portal_time += 1;
                let transition_time = self
                    .portal_type
                    .get_portal_transition_time(current_world, entity);
                self.portal_time >= transition_time
            } else {
                false
            }
        } else {
            self.decay_tick();
            false
        }
    }

    pub const fn decay_tick(&mut self) {
        self.portal_time = self.portal_time.saturating_sub(4);
    }

    #[must_use]
    pub const fn has_expired(&self) -> bool {
        self.portal_time == 0
    }
}

#[cfg(test)]
mod tests {
    use super::teleportation_scale;
    use pumpkin_data::dimension::Dimension;

    #[test]
    fn overworld_to_nether_divides_by_eight() {
        let scale = teleportation_scale(&Dimension::OVERWORLD, &Dimension::THE_NETHER);
        assert!((scale - 0.125).abs() < f64::EPSILON);
        // A player at x=1000 in the overworld aims at x=125 in the nether.
        assert!((1000.0 * scale - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn nether_to_overworld_multiplies_by_eight() {
        let scale = teleportation_scale(&Dimension::THE_NETHER, &Dimension::OVERWORLD);
        assert!((scale - 8.0).abs() < f64::EPSILON);
        assert!((125.0 * scale - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn round_trip_scale_is_identity() {
        let out = teleportation_scale(&Dimension::OVERWORLD, &Dimension::THE_NETHER);
        let back = teleportation_scale(&Dimension::THE_NETHER, &Dimension::OVERWORLD);
        assert!((out * back - 1.0).abs() < f64::EPSILON);
        // This is the property the bug report was about: a round trip must land
        // on the same column it started from.
        for x in [-4096.0, -1.0, 0.0, 7.0, 30_000_000.0] {
            let there = x * out;
            let home = there * back;
            assert!((home - x).abs() < 1e-9, "round trip moved {x} to {home}");
        }
    }

    #[test]
    fn end_keeps_coordinates() {
        let scale = teleportation_scale(&Dimension::OVERWORLD, &Dimension::THE_END);
        assert!((scale - 1.0).abs() < f64::EPSILON);
    }
}
