use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::EnumVariants;
use pumpkin_data::block_properties::Integer0To15;
use pumpkin_data::tag::Taggable;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::sign::SignBlockEntity;
use uuid::Uuid;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::NormalUseArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::block::PlayerPlacedArgs;
use crate::block::UseWithItemArgs;
use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::items::dye::DyeItem;
use crate::item::items::glowing_ink_sac::GlowingInkSacItem;
use crate::item::items::honeycomb::HoneyCombItem;
use crate::item::items::ink_sac::InkSacItem;
use crate::net::ClientPlatform;
use crate::world::World;

#[pumpkin_block_from_tag("minecraft:all_signs")]
pub struct SignBlock;

/// Helper struct to hold support detection results
struct SupportInfo {
    above_is_valid: bool,
    side_direction: Option<BlockDirection>,
}

/// Helper struct for sign placement configuration
struct SignPlacement {
    block_id: u16,
    facing: Option<String>,
    rotation: Option<Integer0To15>,
    attached: bool,
}

impl SignBlock {
    /// Checks if a block can provide support for a sign.
    async fn is_valid_support(world: &World, pos: &BlockPos, direction: BlockDirection) -> bool {
        let (block, state) = world.get_block_and_state(pos).await;
        let is_permissive = block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_LEAVES)
            || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SIGNS);

        match direction {
            BlockDirection::Up => state.is_side_solid(BlockDirection::Down) || is_permissive,
            BlockDirection::Down => state.is_center_solid(BlockDirection::Up) || is_permissive,
            _ => state.is_side_solid(direction.opposite()) || is_permissive,
        }
    }

    /// Detects available support points around a position.
    async fn detect_support(world: &World, position: &BlockPos) -> SupportInfo {
        let (block_above, state_above) = world.get_block_and_state(&position.up()).await;
        let above_is_valid = state_above.is_side_solid(BlockDirection::Down)
            || block_above.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SIGNS)
            || block_above.has_tag(&pumpkin_data::tag::Block::MINECRAFT_LEAVES);

        let mut side_direction = None;
        for direction in BlockDirection::horizontal() {
            let pos = position.offset(direction.to_offset());
            if Self::is_valid_support(world, &pos, direction.opposite()).await {
                side_direction = Some(direction);
                break;
            }
        }

        SupportInfo {
            above_is_valid,
            side_direction,
        }
    }

    /// Determines the appropriate facing direction for a wall-hanging sign.
    fn calculate_wall_hanging_facing(wall_dir: BlockDirection, player_yaw: f32) -> &'static str {
        match wall_dir {
            BlockDirection::North | BlockDirection::South => {
                // Wall runs North-South, sign faces East or West
                if (player_yaw + 360.0) % 360.0 < 180.0 {
                    "east"
                } else {
                    "west"
                }
            }
            BlockDirection::East | BlockDirection::West => {
                // Wall runs East-West, sign faces North or South
                if (-270.0..=270.0).contains(&player_yaw) {
                    "south"
                } else {
                    "north"
                }
            }
            _ => wall_dir.opposite().to_cardinal_direction().to_value(),
        }
    }

    /// Calculates rotation for wall-hanging signs.
    fn calculate_wall_hanging_rotation(
        wall_dir: BlockDirection,
        player_rot: Integer0To15,
        is_sneaking: bool,
    ) -> Integer0To15 {
        if is_sneaking {
            return player_rot;
        }

        let idx = player_rot.to_index();
        match wall_dir {
            BlockDirection::North | BlockDirection::South => {
                // Snap to North-South axis (0 or 8)
                if (4..12).contains(&idx) {
                    Integer0To15::from_index(8)
                } else {
                    Integer0To15::from_index(0)
                }
            }
            BlockDirection::East | BlockDirection::West => {
                // Snap to East-West axis (4 or 12)
                if (2..10).contains(&idx) {
                    Integer0To15::from_index(4)
                } else {
                    Integer0To15::from_index(12)
                }
            }
            _ => player_rot,
        }
    }

    /// Determines the block variant and placement properties for a sign.
    fn determine_placement(args: &OnPlaceArgs, support: &SupportInfo) -> Option<SignPlacement> {
        let is_hanging = args.block.name.contains("hanging");
        let is_sneaking = args.player.get_entity().sneaking.load(Ordering::Relaxed);

        // Select block variant
        let block_id = if is_hanging {
            Self::select_hanging_variant(args, support)?
        } else {
            Self::select_standing_variant(args, support)
        };

        let actual_block = Block::from_id(block_id);
        let is_wall_hanging = is_hanging && actual_block.name.contains("wall_hanging");

        // Calculate orientation
        let (facing, rotation, attached) = if is_wall_hanging {
            Self::calculate_wall_hanging_orientation(args, support, is_sneaking)
        } else if is_hanging {
            Self::calculate_ceiling_orientation(args, is_sneaking)
        } else if actual_block.name.contains("wall") {
            Self::calculate_wall_orientation(args)
        } else {
            Self::calculate_standing_orientation(args)
        };

        Some(SignPlacement {
            block_id,
            facing,
            rotation,
            attached,
        })
    }

    /// Selects the appropriate hanging sign variant.
    fn select_hanging_variant(args: &OnPlaceArgs, support: &SupportInfo) -> Option<u16> {
        if args.direction == BlockDirection::Down && support.above_is_valid {
            Some(args.block.id) // Ceiling hanging
        } else if (args.direction.is_horizontal() || args.direction == BlockDirection::Up)
            && support.side_direction.is_some()
        {
            Some(get_sign_variant(args.block, true)) // Wall-hanging with post
        } else if support.above_is_valid {
            Some(args.block.id)
        } else {
            None // No valid placement
        }
    }

    /// Selects the appropriate standing sign variant.
    fn select_standing_variant(args: &OnPlaceArgs, support: &SupportInfo) -> u16 {
        if args.direction.is_horizontal() && support.side_direction.is_some() {
            get_sign_variant(args.block, false) // Wall sign
        } else {
            args.block.id // Standing sign
        }
    }

    /// Calculates orientation for wall-hanging signs.
    fn calculate_wall_hanging_orientation(
        args: &OnPlaceArgs,
        support: &SupportInfo,
        is_sneaking: bool,
    ) -> (Option<String>, Option<Integer0To15>, bool) {
        let wall_dir = if args.direction.is_horizontal() {
            args.direction
        } else {
            support.side_direction.unwrap_or(args.direction)
        };

        let player_yaw = args.player.get_entity().yaw.load();
        let facing = Self::calculate_wall_hanging_facing(wall_dir, player_yaw);

        let player_rot = args.player.get_entity().get_flipped_rotation_16();
        let rotation = Self::calculate_wall_hanging_rotation(wall_dir, player_rot, is_sneaking);

        let is_angled = rotation.to_index() % 4 != 0;
        let attached = is_angled || is_sneaking;

        (Some(facing.to_string()), Some(rotation), attached)
    }

    /// Calculates orientation for ceiling-hanging signs.
    fn calculate_ceiling_orientation(
        args: &OnPlaceArgs,
        is_sneaking: bool,
    ) -> (Option<String>, Option<Integer0To15>, bool) {
        let rotation = if is_sneaking {
            args.player.get_entity().get_flipped_rotation_16()
        } else {
            // Snap to nearest cardinal
            let index = args
                .player
                .get_entity()
                .get_flipped_rotation_16()
                .to_index();
            Integer0To15::from_index(((index + 2) / 4 * 4) % 16)
        };

        let is_angled = rotation.to_index() % 4 != 0;
        let attached = is_angled || is_sneaking;

        (None, Some(rotation), attached)
    }

    /// Calculates orientation for wall signs.
    fn calculate_wall_orientation(
        args: &OnPlaceArgs,
    ) -> (Option<String>, Option<Integer0To15>, bool) {
        let facing = args.direction.opposite().to_cardinal_direction().to_value();
        (Some(facing.to_string()), None, false)
    }

    /// Calculates orientation for standing signs.
    fn calculate_standing_orientation(
        args: &OnPlaceArgs,
    ) -> (Option<String>, Option<Integer0To15>, bool) {
        let rotation = args.player.get_entity().get_flipped_rotation_16();
        (None, Some(rotation), false)
    }

    /// Applies placement properties to a block.
    fn apply_placement_properties(block: &Block, placement: &SignPlacement) -> BlockStateId {
        let mut props = block
            .properties(block.default_state.id)
            .map(|p| p.to_props())
            .unwrap_or_default();

        if let Some(facing) = &placement.facing
            && let Some(prop) = props.iter_mut().find(|(k, _)| *k == "facing")
        {
            prop.1 = facing;
        }

        if let Some(rotation) = placement.rotation
            && let Some(prop) = props.iter_mut().find(|(k, _)| *k == "rotation")
        {
            prop.1 = rotation.to_value();
        }

        if let Some(prop) = props.iter_mut().find(|(k, _)| *k == "attached") {
            prop.1 = if placement.attached { "true" } else { "false" };
        }

        block.from_properties(&props).to_state_id(block)
    }
}

//TODO: add support for click commands
impl BlockBehaviour for SignBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let support = Self::detect_support(args.world, args.position).await;

            let Some(placement) = Self::determine_placement(&args, &support) else {
                return 0; // Invalid placement
            };

            let actual_block = Block::from_id(placement.block_id);
            Self::apply_placement_properties(actual_block, &placement)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .add_block_entity(Arc::new(SignBlockEntity::empty(*args.position)))
                .await;
        })
    }

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            match &args.player.client {
                crate::net::ClientPlatform::Java(java) => {
                    java.send_sign_packet(*args.position, true).await;
                }
                crate::net::ClientPlatform::Bedrock(_bedrock) => todo!(),
            }
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let is_hanging = args.block.name.contains("hanging");
            let clicked_face = args
                .use_item_on
                .and_then(|u| pumpkin_data::BlockDirection::try_from(u.face.0).ok())
                .unwrap_or(pumpkin_data::BlockDirection::Up);

            // Detection for floor-to-wall attachment (broken rn)
            if is_hanging && clicked_face == BlockDirection::Up {
                for d in pumpkin_data::BlockDirection::horizontal() {
                    let wall_pos = args.position.offset(d.to_offset());
                    let (block, state) = args.block_accessor.get_block_and_state(&wall_pos).await;
                    if state.is_side_solid(d.opposite())
                        || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_LEAVES)
                        || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SIGNS)
                    {
                        return true;
                    }
                }
            }

            // Standard support validation with permissive tags
            let support_pos = match clicked_face {
                BlockDirection::Up => args.position.down(),
                BlockDirection::Down => args.position.up(),
                _ => args.position.offset(clicked_face.opposite().to_offset()),
            };

            let (block, state) = args.block_accessor.get_block_and_state(&support_pos).await;
            let is_permissive = block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_LEAVES)
                || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SIGNS);

            match clicked_face {
                BlockDirection::Up => {
                    !is_hanging && (state.is_center_solid(BlockDirection::Up) || is_permissive)
                }
                BlockDirection::Down => {
                    is_hanging && (state.is_side_solid(BlockDirection::Down) || is_permissive)
                }
                _ => state.is_side_solid(clicked_face.opposite()) || is_permissive,
            }
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.remove_block_entity(args.position).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        let is_hanging = args.block.name.contains("hanging");
        let is_wall_sign = args.block.name.contains("wall");

        // Determine the expected support direction
        let support_dir = if is_wall_sign {
            // Look up the 'facing' property to find the support behind the wall sign
            get_wall_support_direction(args.block, args.state_id)
        } else if is_hanging {
            // Ceiling-hanging signs always look Up
            Some(BlockDirection::Up)
        } else {
            // Standing signs always look Down
            Some(BlockDirection::Down)
        };

        Box::pin(async move {
            if let Some(dir) = support_dir {
                // Only check if the neighbor that changed is our support neighbor
                if args.direction == dir {
                    let support_pos = args.position.offset(dir.to_offset());
                    let (support_block, support_state) =
                        args.world.get_block_and_state(&support_pos).await;

                    // Permissive support check
                    let is_leaf =
                        support_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_LEAVES);
                    let is_sign = support_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SIGNS);

                    let is_valid = match dir {
                        BlockDirection::Up => {
                            support_state.is_side_solid(BlockDirection::Down) || is_leaf || is_sign
                        }
                        BlockDirection::Down => {
                            support_state.is_center_solid(BlockDirection::Up) || is_leaf || is_sign
                        }
                        _ => support_state.is_side_solid(dir.opposite()) || is_leaf || is_sign,
                    };

                    if !is_valid {
                        return 0; // Return AIR to break the block
                    }
                }
            }
            args.state_id
        })
    }

    /// Handles normal use (right-click) on the sign block.
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let Some(block_entity) = args.world.get_block_entity(args.position).await else {
                return BlockActionResult::Pass;
            };
            let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
                return BlockActionResult::Pass;
            };

            if sign_entity.is_waxed.load(Ordering::Relaxed) {
                args.world
                    .play_block_sound(
                        pumpkin_data::sound::Sound::BlockSignWaxedInteractFail,
                        pumpkin_data::sound::SoundCategory::Blocks,
                        *args.position,
                    )
                    .await;
                return BlockActionResult::SuccessServer;
            }

            let mut currently_editing = sign_entity.currently_editing_player.lock().await;
            if !try_claim_sign(
                &mut currently_editing,
                &args.player.gameprofile.id,
                args.world,
                args.position,
            ) {
                return BlockActionResult::Pass;
            }

            let is_facing_front_text =
                is_facing_front_text(args.world, args.position, args.block, args.player).await;
            match &args.player.client {
                ClientPlatform::Java(java) => {
                    java.send_sign_packet(*args.position, is_facing_front_text)
                        .await;
                }
                ClientPlatform::Bedrock(_bedrock) => todo!(),
            }

            BlockActionResult::SuccessServer
        })
    }

    /// Handles use with an item on the sign block.
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let Some(block_entity) = args.world.get_block_entity(args.position).await else {
                return BlockActionResult::Pass;
            };
            let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
                return BlockActionResult::Pass;
            };

            if sign_entity.is_waxed.load(Ordering::Relaxed) {
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let mut currently_editing = sign_entity.currently_editing_player.lock().await;
            if !try_claim_sign(
                &mut currently_editing,
                &args.player.gameprofile.id,
                args.world,
                args.position,
            ) {
                // I don't think that makes sense, since it will also just return in normal_use, but vanilla does it like this
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let text =
                if is_facing_front_text(args.world, args.position, args.block, args.player).await {
                    &sign_entity.front_text
                } else {
                    &sign_entity.back_text
                };

            let mut item = args.item_stack.lock().await;

            let Some(pumpkin_item) = args.server.item_registry.get_pumpkin_item(item.item.id)
            else {
                return BlockActionResult::PassToDefaultBlockAction;
            };

            let result = if let Some(honeycomb_item) =
                pumpkin_item.as_any().downcast_ref::<HoneyCombItem>()
            {
                honeycomb_item
                    .apply_to_sign(&args, &block_entity, sign_entity)
                    .await
            } else if let Some(g_ink_sac_item) =
                pumpkin_item.as_any().downcast_ref::<GlowingInkSacItem>()
            {
                g_ink_sac_item
                    .apply_to_sign(&args, &block_entity, text)
                    .await
            } else if let Some(ink_sac_item) = pumpkin_item.as_any().downcast_ref::<InkSacItem>() {
                ink_sac_item.apply_to_sign(&args, &block_entity, text).await
            } else if let Some(dye) = pumpkin_item.as_any().downcast_ref::<DyeItem>() {
                let color_name = item.item.registry_key.strip_suffix("_dye").unwrap();
                dye.apply_to_sign(&args, &block_entity, text, color_name)
                    .await
            } else {
                BlockActionResult::PassToDefaultBlockAction
            };

            if result == BlockActionResult::Success {
                if !args.player.has_infinite_materials() {
                    item.decrement(1);
                }
                *currently_editing = None;
            }

            result
        })
    }
}

/// Returns the direction of the block supporting the wall sign.
fn get_wall_support_direction(block: &Block, state_id: BlockStateId) -> Option<BlockDirection> {
    block.properties(state_id).and_then(|props| {
        let prop_map = props.to_props();
        prop_map
            .into_iter()
            .find(|(k, _)| k == &"facing")
            .map(|(_, v)| match v {
                "north" => BlockDirection::South,
                "south" => BlockDirection::North,
                "east" => BlockDirection::West,
                _ => BlockDirection::East, // "west" and default case
            })
    })
}

/// Helper to convert a regular sign to its wall variant.
/// Returns the block ID of the wall variant, or the base block's ID if not found.
fn get_sign_variant(base: &Block, is_hanging: bool) -> u16 {
    let base_name = base.name;
    let wood_type = base_name
        .strip_suffix("_hanging_sign")
        .or_else(|| base_name.strip_suffix("_sign"))
        .unwrap_or("oak");

    let target_name = if is_hanging {
        // This is the variant that provides the "horizontal wooden post"
        format!("{wood_type}_wall_hanging_sign")
    } else {
        format!("{wood_type}_wall_sign")
    };

    pumpkin_data::Block::from_name(&target_name).map_or(base.id, |b| b.id)
}

async fn is_facing_front_text(
    world: &World,
    location: &BlockPos,
    block: &Block,
    player: &Player,
) -> bool {
    let state_id = world.get_block_state_id(location).await;
    // Read properties dynamically: some sign types use a `rotation` property (0..15),
    // others (wall signs) use a `facing` property (north/south/west/east),
    // hanging signs may have `rotation` + `attached`.
    let mut rotation: f32 = 0.0;
    if let Some(props) = block.properties(state_id) {
        let prop_map = props.to_props();
        if let Some((_, val)) = prop_map.iter().find(|(k, _)| k == &"rotation") {
            let r = Integer0To15::from_value(val);
            rotation = get_yaw_from_rotation_16(r);
        } else if let Some((_, val)) = prop_map.iter().find(|(k, _)| k == &"facing") {
            rotation = match &val[..] {
                "north" => 180.0,
                "west" => 90.0,
                "east" => -90.0,
                _ => 0.0,
            };
        }
    }
    let bounding_box = Vector3::new(0.5, 0.5, 0.5);

    let d = player.eye_position().x - (f64::from(location.0.x) + bounding_box.x);
    let d1 = player.eye_position().z - (f64::from(location.0.z) + bounding_box.z);

    let f = (d1.atan2(d).to_degrees() as f32) - 90.0;

    let diff = (f - rotation + 180.0).rem_euclid(360.0) - 180.0;
    diff.abs() <= 90.0
}

fn get_yaw_from_rotation_16(rotation: Integer0To15) -> f32 {
    let index = rotation.to_index();

    f32::from(index) * 22.5
}

fn try_claim_sign(
    currently_editing: &mut Option<Uuid>,
    uuid: &Uuid,
    world: &World,
    position: &BlockPos,
) -> bool {
    if let Some(editing_player_id) = *currently_editing
        && editing_player_id != *uuid
        && let Some(editing_player) = world.get_player_by_uuid(editing_player_id)
        && editing_player
            .as_ref()
            .can_interact_with_block_at(position, 4.0f64)
    {
        return false;
    }

    *currently_editing = Some(*uuid);
    true
}
