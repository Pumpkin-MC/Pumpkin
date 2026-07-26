use super::{BlockPlacingError, BlockRegistry};
use crate::block::fluid::FluidBehaviour;
use crate::block::{
    BlockBehaviour, BlockIsReplacing, CanPlaceAtArgs, CanUpdateAtArgs, OnPlaceArgs, PlacedArgs,
    PlayerPlacedArgs,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;

impl BlockRegistry {
    fn entity_blocks_block_placement(entity: &dyn EntityBase) -> bool {
        let base_entity = entity.get_entity();
        if base_entity.is_removed()
            || base_entity
                .no_clip
                .load(std::sync::atomic::Ordering::Relaxed)
            || entity.is_spectator()
        {
            return false;
        }

        if entity.get_living_entity().is_some() {
            return true;
        }

        let entity_type = base_entity.entity_type;
        let resource_name = entity_type.resource_name;
        entity_type == &pumpkin_data::entity::EntityType::END_CRYSTAL
            || entity_type == &pumpkin_data::entity::EntityType::FALLING_BLOCK
            || entity_type == &pumpkin_data::entity::EntityType::TNT
            || resource_name.ends_with("_minecart")
            || resource_name.ends_with("_boat")
            || resource_name.ends_with("_raft")
    }

    fn has_blocking_entity_in_box(world: &World, placed_box: &BoundingBox) -> bool {
        let players = world.players.load();
        if players.iter().any(|player| {
            Self::entity_blocks_block_placement(player.as_ref())
                && player
                    .get_entity()
                    .bounding_box
                    .load()
                    .intersects(placed_box)
        }) {
            return true;
        }

        world.entities.load().iter().any(|entity| {
            Self::entity_blocks_block_placement(entity.as_ref())
                && entity
                    .get_entity()
                    .bounding_box
                    .load()
                    .intersects(placed_box)
        })
    }

    #[expect(clippy::too_many_lines)]
    pub async fn place_block(
        &self,
        player: &Arc<Player>,
        placed_block: &'static Block,
        server: &Server,
        use_item_on: &SUseItemOn,
        location: BlockPos,
        face: BlockDirection,
    ) -> Result<Option<(BlockPos, BlockStateId)>, BlockPlacingError> {
        let entity = &player.get_entity();

        match player.gamemode.load() {
            pumpkin_util::GameMode::Spectator | pumpkin_util::GameMode::Adventure => {
                return Err(BlockPlacingError::InvalidGamemode);
            }
            _ => {}
        }

        let clicked_block_pos = BlockPos(location.0);
        let world = entity.world.load_full();

        if location.0.y + face.to_offset().y < world.get_bottom_y() {
            return Err(BlockPlacingError::BlockOutOfWorld);
        }

        if location.0.y + face.to_offset().y > world.get_top_y() {
            player
                .send_system_message_raw(
                    &pumpkin_util::text::TextComponent::translate_cross(
                        pumpkin_data::translation::java::BUILD_TOOHIGH,
                        pumpkin_data::translation::bedrock::BUILD_TOOHIGH,
                        vec![pumpkin_util::text::TextComponent::text(
                            (world.get_top_y()).to_string(),
                        )],
                    )
                    .color_named(pumpkin_util::text::color::NamedColor::Red),
                    true,
                )
                .await;
            return Err(BlockPlacingError::BlockOutOfWorld);
        }

        let (clicked_block, clicked_block_state) = world.get_block_and_state(&clicked_block_pos);

        let replace_clicked_block = if clicked_block == placed_block {
            self.can_update_at(
                &world,
                clicked_block,
                clicked_block_state.id,
                &clicked_block_pos,
                face,
                use_item_on,
                player,
            )
            .then_some(BlockIsReplacing::Itself(clicked_block_state.id))
        } else if clicked_block_state.replaceable() {
            if clicked_block == &Block::WATER {
                use pumpkin_data::block_properties::{BlockProperties, WaterLikeProperties};
                let water_props =
                    WaterLikeProperties::from_state_id(clicked_block_state.id, clicked_block);
                Some(BlockIsReplacing::Water(water_props.level))
            } else {
                Some(BlockIsReplacing::Other)
            }
        } else {
            None
        };

        let (final_block_pos, final_face, replacing) =
            if let Some(replacing) = replace_clicked_block {
                (clicked_block_pos, face.opposite(), replacing)
            } else {
                let block_pos = BlockPos(location.0 + face.to_offset());
                let (previous_block, previous_block_state) = world.get_block_and_state(&block_pos);

                let replace_previous_block = if previous_block == placed_block {
                    self.can_update_at(
                        &world,
                        previous_block,
                        previous_block_state.id,
                        &block_pos,
                        face.opposite(),
                        use_item_on,
                        player,
                    )
                    .then_some(BlockIsReplacing::Itself(previous_block_state.id))
                } else {
                    previous_block_state.replaceable().then(|| {
                        if previous_block == &Block::WATER {
                            use pumpkin_data::block_properties::{
                                BlockProperties, WaterLikeProperties,
                            };
                            let water_props = WaterLikeProperties::from_state_id(
                                previous_block_state.id,
                                previous_block,
                            );
                            BlockIsReplacing::Water(water_props.level)
                        } else {
                            BlockIsReplacing::None
                        }
                    })
                };

                match replace_previous_block {
                    Some(replacing) => (block_pos, face.opposite(), replacing),
                    None => {
                        return Ok(None);
                    }
                }
            };

        if !self.can_place_at(
            Some(server),
            Some(&*world),
            &*world,
            Some(player),
            placed_block,
            placed_block.default_state,
            &final_block_pos,
            Some(final_face),
            Some(use_item_on),
        ) {
            return Ok(None);
        }

        let new_state = self
            .on_place(
                server,
                &world,
                player,
                placed_block,
                &final_block_pos,
                final_face,
                replacing,
                use_item_on,
            )
            .await;

        // Mirror vanilla obstruction checks: only entities that block building should prevent
        // placement. (e.g. arrows/xp orbs/displays/markers should not)
        let state = BlockState::from_id(new_state);
        for shape in state.get_block_collision_shapes() {
            let placed_box = shape.at_pos(final_block_pos);

            if Self::has_blocking_entity_in_box(world.as_ref(), &placed_box) {
                return Ok(None);
            }
        }

        let event = crate::plugin::block::block_place::BlockPlaceEvent::new(
            player.clone(),
            placed_block,
            clicked_block,
            final_block_pos,
            true,
        );
        let event = server
            .plugin_manager
            .fire::<crate::plugin::block::block_place::BlockPlaceEvent>(event)
            .await;
        if event.cancelled {
            return Ok(None);
        }

        let _replaced_id = world
            .set_block_state(&final_block_pos, new_state, BlockFlags::NOTIFY_ALL)
            .await;

        world
            .emit_vibration(
                crate::world::vibrations::Vibration::BlockPlace,
                final_block_pos.to_centered_f64(),
            )
            .await;

        self.player_placed(
            &world,
            placed_block,
            new_state,
            &final_block_pos,
            face,
            player,
        )
        .await;

        player
            .trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::PlacedBlock {
                    block_id: format!("minecraft:{}", placed_block.name),
                },
            )
            .await;

        // `placed` callbacks can immediately alter the state (for example, a
        // powered redstone wire). Return the committed state, not the state
        // originally selected by `on_place`.
        Ok(Some((
            final_block_pos,
            world.get_block_state_id(&final_block_pos),
        )))
    }

    #[expect(clippy::too_many_arguments)]
    pub fn can_place_at(
        &self,
        server: Option<&Server>,
        world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        player: Option<&Player>,
        block: &Block,
        state: &BlockState,
        position: &BlockPos,
        direction: Option<BlockDirection>,
        use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_place_at(CanPlaceAtArgs {
                server,
                world,
                block_accessor,
                block,
                state,
                position,
                direction,
                player,
                use_item_on,
            });
        }
        true
    }

    #[expect(clippy::too_many_arguments)]
    pub fn can_update_at(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_update_at(CanUpdateAtArgs {
                world,
                block,
                state_id,
                position,
                direction,
                player,
                use_item_on,
            });
        }
        false
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn on_place(
        &self,
        server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        position: &BlockPos,
        direction: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_place(OnPlaceArgs {
                    server,
                    world,
                    block,
                    position,
                    direction,
                    player,
                    replacing,
                    use_item_on,
                })
                .await;
        }
        block.default_state.id
    }

    pub async fn player_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        player: &Player,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .player_placed(PlayerPlacedArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    player,
                })
                .await;
        }
    }

    pub async fn on_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let state = world.get_block_state(position);
        if state.block_entity_type != u16::MAX
            && world.get_block_entity(position).is_none()
            && let Some(entity) =
                crate::block::entities::create_block_entity(state.block_entity_type, *position)
        {
            world.add_block_entity(entity);
        }

        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .placed(PlacedArgs {
                    world,
                    block,
                    state_id,
                    old_state_id,
                    position,
                    notify,
                })
                .await;
        }
    }

    pub async fn on_placed_fluid(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid.id);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid
                .placed(world, fluid, state_id, position, old_state_id, notify)
                .await;
        }
    }
}
