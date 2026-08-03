use std::{pin::Pin, sync::Arc};

use crate::{
    entity::EntityBase,
    entity::player::Player,
    entity::r#type::from_type,
    item::{ItemBehaviour, ItemMetadata},
};
use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    dimension::Dimension,
    entity::EntityType,
    fluid::Fluid,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{inventory::Inventory, tick::TickPriority, world::BlockFlags};
use uuid::Uuid;

use crate::world::World;

pub struct EmptyBucketItem;
pub struct FilledBucketItem;
pub struct MilkBucketItem;

impl ItemMetadata for EmptyBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::BUCKET.id].into()
    }
}

impl ItemMetadata for FilledBucketItem {
    fn ids() -> Box<[u16]> {
        [
            Item::WATER_BUCKET.id,
            Item::LAVA_BUCKET.id,
            Item::POWDER_SNOW_BUCKET.id,
            Item::AXOLOTL_BUCKET.id,
            Item::COD_BUCKET.id,
            Item::SALMON_BUCKET.id,
            Item::TROPICAL_FISH_BUCKET.id,
            Item::PUFFERFISH_BUCKET.id,
            Item::TADPOLE_BUCKET.id,
        ]
        .into()
    }
}

impl ItemMetadata for MilkBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::MILK_BUCKET.id].into()
    }
}

fn get_start_and_end_pos(player: &Player) -> (Vector3<f64>, Vector3<f64>) {
    let start_pos = player.eye_position();
    let (yaw, pitch) = player.rotation();
    let (yaw_rad, pitch_rad) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
    let block_interaction_range = 4.5; // This is not the same as the block_interaction_range in the
    // player entity.
    let direction = Vector3::new(
        -yaw_rad.sin() * pitch_rad.cos() * block_interaction_range,
        -pitch_rad.sin() * block_interaction_range,
        pitch_rad.cos() * yaw_rad.cos() * block_interaction_range,
    );

    let end_pos = start_pos.add(&direction);
    (start_pos, end_pos)
}

fn waterlogged_check(block: &Block, state: BlockStateId) -> Option<bool> {
    block.properties(state).and_then(|properties| {
        properties
            .to_props()
            .into_iter()
            .find(|p| p.0 == "waterlogged")
            .map(|(_, value)| value == "true")
    })
}

fn is_waterlogged(block: &Block, state: BlockStateId) -> bool {
    waterlogged_check(block, state).unwrap_or(false)
}

fn set_waterlogged(block: &Block, state: BlockStateId, waterlogged: bool) -> BlockStateId {
    let original_props = &block.properties(state).unwrap().to_props();
    let waterlogged = waterlogged.to_string();
    let props: Vec<(&str, &str)> = original_props
        .iter()
        .map(|(key, value)| {
            if *key == "waterlogged" {
                ("waterlogged", waterlogged.as_str())
            } else {
                (*key, *value)
            }
        })
        .collect();
    block.from_properties(&props).to_state_id(block)
}

async fn give_player_bucket_item(player: &Player, item: &'static Item) {
    if player.gamemode.load() == GameMode::Creative {
        for i in 0..player.inventory.main_inventory.len() {
            if player.inventory.main_inventory[i].lock().await.item.id == item.id {
                return;
            }
        }
        let mut item_stack = ItemStack::new(1, item);
        player
            .inventory
            .insert_stack_anywhere(&mut item_stack)
            .await;
    } else {
        let item_stack = ItemStack::new(1, item);
        let held_item = player.inventory.held_item();
        let mut held_stack = held_item.lock().await;

        if held_stack.item_count == 1 {
            *held_stack = item_stack;
        } else {
            held_stack.decrement(1);
            drop(held_stack);
            player
                .inventory
                .offer_or_drop_stack(item_stack, player)
                .await;
        }
    }
}

/// Returns the bucket item obtained and the position of the block actually acted on
/// (needed for `GameEvent::FluidPickup`, which vanilla emits at that exact position --
/// `BucketItem.java:77` -- not always `block_pos` itself, since the waterlogged-neighbor
/// branch below acts on `target_pos` instead).
async fn try_pickup_bucket_item(
    world: &Arc<World>,
    block_pos: BlockPos,
    direction: BlockDirection,
) -> Option<(&'static Item, BlockPos)> {
    let (block, state) = world.get_block_and_state_id(&block_pos);

    if block == &Block::POWDER_SNOW {
        world
            .break_block(
                &block_pos,
                None,
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
            )
            .await;
        return Some((&Item::POWDER_SNOW_BUCKET, block_pos));
    }

    if is_waterlogged(block, state) {
        let state_id = set_waterlogged(block, state, false);
        world
            .set_block_state(&block_pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
            .await;
        world.schedule_fluid_tick(&Fluid::WATER, block_pos, 5, TickPriority::Normal);
        return Some((&Item::WATER_BUCKET, block_pos));
    }

    if state == Block::LAVA.default_state.id || state == Block::WATER.default_state.id {
        world
            .break_block(&block_pos, None, BlockFlags::NOTIFY_NEIGHBORS)
            .await;
        world
            .set_block_state(
                &block_pos,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_NEIGHBORS,
            )
            .await;
        return Some((
            if state == Block::LAVA.default_state.id {
                &Item::LAVA_BUCKET
            } else {
                &Item::WATER_BUCKET
            },
            block_pos,
        ));
    }

    let target_pos = block_pos.offset(direction.to_offset());
    let (block, state) = world.get_block_and_state_id(&target_pos);
    if waterlogged_check(block, state).is_some() {
        let state_id = set_waterlogged(block, state, false);
        world
            .set_block_state(&target_pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
            .await;
        world.schedule_fluid_tick(&Fluid::WATER, target_pos, 5, TickPriority::Normal);
        return Some((&Item::WATER_BUCKET, target_pos));
    }

    None
}

fn should_evaporate_in_nether(item: &Item, world: &World) -> bool {
    item.id != Item::LAVA_BUCKET.id
        && item.id != Item::POWDER_SNOW_BUCKET.id
        && world.dimension == Dimension::THE_NETHER
}

/// Returns the aquatic entity carried by a vanilla mob bucket.
///
/// The bucket's `bucket_entity_data` component is currently an opaque marker in
/// Pumpkin, so entity-specific NBT (such as a custom name or tropical-fish
/// variant) cannot yet be restored here.  Spawning the correct base entity is
/// still required after the water has been successfully placed.
const fn mob_bucket_entity_type(item: &Item) -> Option<&'static EntityType> {
    match item.id {
        id if id == Item::AXOLOTL_BUCKET.id => Some(&EntityType::AXOLOTL),
        id if id == Item::COD_BUCKET.id => Some(&EntityType::COD),
        id if id == Item::SALMON_BUCKET.id => Some(&EntityType::SALMON),
        id if id == Item::TROPICAL_FISH_BUCKET.id => Some(&EntityType::TROPICAL_FISH),
        id if id == Item::PUFFERFISH_BUCKET.id => Some(&EntityType::PUFFERFISH),
        id if id == Item::TADPOLE_BUCKET.id => Some(&EntityType::TADPOLE),
        _ => None,
    }
}

const fn mob_bucket_empty_sound(item: &Item) -> Option<Sound> {
    match item.id {
        id if id == Item::AXOLOTL_BUCKET.id => Some(Sound::ItemBucketEmptyAxolotl),
        id if id == Item::TADPOLE_BUCKET.id => Some(Sound::ItemBucketEmptyTadpole),
        id if id == Item::COD_BUCKET.id
            || id == Item::SALMON_BUCKET.id
            || id == Item::TROPICAL_FISH_BUCKET.id
            || id == Item::PUFFERFISH_BUCKET.id =>
        {
            Some(Sound::ItemBucketEmptyFish)
        }
        _ => None,
    }
}

fn play_bucket_evaporation(world: &Arc<World>, player: &Player) {
    world.play_sound_raw(
        Sound::BlockFireExtinguish as u16,
        SoundCategory::Blocks,
        &player.position(),
        0.5,
        (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.8, 2.6),
    );
}

async fn try_place_powder_snow(
    world: &Arc<World>,
    pos: BlockPos,
    direction: BlockDirection,
) -> bool {
    let state = world.get_block_state(&pos);
    let target_pos = if state.replaceable() {
        pos
    } else {
        pos.offset(direction.to_offset())
    };
    let target_state = world.get_block_state(&target_pos);
    if !target_state.is_air() && !target_state.is_liquid() && !target_state.replaceable() {
        return false;
    }
    world
        .set_block_state(
            &target_pos,
            Block::POWDER_SNOW.default_state.id,
            BlockFlags::NOTIFY_NEIGHBORS,
        )
        .await;
    true
}

async fn try_place_filled_bucket(
    world: &Arc<World>,
    item: &Item,
    pos: BlockPos,
    direction: BlockDirection,
) -> Option<BlockPos> {
    let (block, state) = world.get_block_and_state(&pos);
    if item.id == Item::POWDER_SNOW_BUCKET.id {
        return try_place_powder_snow(world, pos, direction)
            .await
            .then_some(pos.offset(direction.to_offset()));
    }

    if is_waterlogged(block, state.id) && item.id == Item::WATER_BUCKET.id {
        let state_id = set_waterlogged(block, state.id, true);
        world
            .set_block_state(&pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
            .await;
        world.schedule_fluid_tick(&Fluid::WATER, pos, 5, TickPriority::Normal);
        return Some(pos);
    }

    let target_pos = pos.offset(direction.to_offset());
    let (block, state) = world.get_block_and_state(&target_pos);

    if waterlogged_check(block, state.id).is_some() {
        if item.id == Item::LAVA_BUCKET.id {
            return None;
        }
        let state_id = set_waterlogged(block, state.id, true);
        world
            .set_block_state(&target_pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
            .await;
        world.schedule_fluid_tick(&Fluid::WATER, target_pos, 5, TickPriority::Normal);
        return Some(target_pos);
    }

    if state.id == Block::AIR.default_state.id || state.is_liquid() {
        world
            .set_block_state(
                &target_pos,
                if item.id == Item::LAVA_BUCKET.id {
                    Block::LAVA.default_state.id
                } else {
                    Block::WATER.default_state.id
                },
                BlockFlags::NOTIFY_NEIGHBORS,
            )
            .await;
        return Some(target_pos);
    }

    None
}

async fn spawn_mob_bucket_entity(
    world: &Arc<World>,
    item: &Item,
    pos: BlockPos,
    player: Option<Arc<Player>>,
) {
    let Some(entity_type) = mob_bucket_entity_type(item) else {
        return;
    };

    // MobBucketItem.checkExtraContent adds its entity at the successfully
    // filled fluid block, rather than at the clicked block.
    let spawn_pos = Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y) + 0.5,
        f64::from(pos.0.z) + 0.5,
    );
    world
        .spawn_entity(from_type(entity_type, spawn_pos, world, Uuid::new_v4()))
        .await;
    if let Some(sound) = mob_bucket_empty_sound(item) {
        world.play_sound(sound, SoundCategory::Neutral, &spawn_pos);
    }

    // MobBucketItem.checkExtraContent, line 37: level.gameEvent(user, GameEvent.ENTITY_PLACE, pos)
    if let Some(player) = player {
        crate::world::game_event::emit_game_event(
            world,
            pumpkin_data::game_event::GameEvent::EntityPlace,
            Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 0.5,
                f64::from(pos.0.z) + 0.5,
            ),
            crate::world::game_event::GameEventContext::of_entity(player),
        )
        .await;
    }
}

impl ItemBehaviour for EmptyBucketItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let (start_pos, end_pos) = get_start_and_end_pos(player);

            let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos);

                let block = Block::from_state_id(state_id);

                if state_id == Block::AIR.default_state.id {
                    return false;
                }

                (block.id != Block::WATER.id && block.id != Block::LAVA.id)
                    || ((block.id == Block::WATER.id && state_id == Block::WATER.default_state.id)
                        || (block.id == Block::LAVA.id && state_id == Block::LAVA.default_state.id))
            };

            let Some((block_pos, direction)) = world.raycast(start_pos, end_pos, checker).await
            else {
                return;
            };

            let Some((item, acted_pos)) =
                try_pickup_bucket_item(&world, block_pos, direction).await
            else {
                return;
            };

            // BucketItem.java:77: level.gameEvent(player, GameEvent.FLUID_PICKUP, pos)
            if let Some(player_arc) = world.get_player_by_id(player.get_entity().entity_id) {
                crate::world::game_event::emit_game_event(
                    &world,
                    pumpkin_data::game_event::GameEvent::FluidPickup,
                    Vector3::new(
                        f64::from(acted_pos.0.x) + 0.5,
                        f64::from(acted_pos.0.y) + 0.5,
                        f64::from(acted_pos.0.z) + 0.5,
                    ),
                    crate::world::game_event::GameEventContext::of_entity(player_arc),
                )
                .await;
            }

            give_player_bucket_item(player, item).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for FilledBucketItem {
    fn normal_use<'a>(
        &'a self,
        item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let (start_pos, end_pos) = get_start_and_end_pos(player);
            let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos);
                if Fluid::from_state_id(state_id).is_some() {
                    return false;
                }
                state_id != Block::AIR.default_state.id
            };

            let Some((pos, direction)) = world.raycast(start_pos, end_pos, checker).await else {
                return;
            };

            let player_arc = world.get_player_by_id(player.get_entity().entity_id);

            // BucketItem.emptyContents: the water-evaporates branch still returns true (a
            // successful use), it just skips placing the fluid -- checkExtraContent (which
            // spawns the bucketed mob) still runs afterward. Pumpkin previously returned
            // early here, silently losing both the water AND the mob for a mob bucket used
            // in the Nether.
            let evaporated = should_evaporate_in_nether(item, &world);
            let placed_pos = if evaporated {
                play_bucket_evaporation(&world, player);
                pos
            } else {
                let Some(placed_pos) = try_place_filled_bucket(&world, item, pos, direction).await
                else {
                    return;
                };
                placed_pos
            };

            // BucketItem.playEmptySound / SolidBucketItem.emptyContents: both call
            // level.gameEvent(user, GameEvent.FLUID_PLACE, pos) (BucketItem.java:157,
            // SolidBucketItem.java, emptyContents) -- but the evaporation branch returns
            // before either is reached (BucketItem.java:122-130), so no fluid was actually
            // placed and no event fires. MobBucketItem also overrides playEmptySound
            // (MobBucketItem.java:42-44) without that call, so mob buckets (axolotl/fish/
            // tadpole/etc.) must not emit FLUID_PLACE here either.
            if !evaporated
                && mob_bucket_entity_type(item).is_none()
                && let Some(player_arc) = player_arc.clone()
            {
                crate::world::game_event::emit_game_event(
                    &world,
                    pumpkin_data::game_event::GameEvent::FluidPlace,
                    Vector3::new(
                        f64::from(placed_pos.0.x) + 0.5,
                        f64::from(placed_pos.0.y) + 0.5,
                        f64::from(placed_pos.0.z) + 0.5,
                    ),
                    crate::world::game_event::GameEventContext::of_entity(player_arc),
                )
                .await;
            }

            spawn_mob_bucket_entity(&world, item, placed_pos, player_arc).await;
            if player.gamemode.load() != GameMode::Creative {
                let item_stack = ItemStack::new(1, &Item::BUCKET);
                player
                    .inventory
                    .set_stack(player.inventory.get_selected_slot().into(), item_stack)
                    .await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for MilkBucketItem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mob_buckets_map_to_their_vanilla_entity_types() {
        let cases = [
            (&Item::AXOLOTL_BUCKET, &EntityType::AXOLOTL),
            (&Item::COD_BUCKET, &EntityType::COD),
            (&Item::SALMON_BUCKET, &EntityType::SALMON),
            (&Item::TROPICAL_FISH_BUCKET, &EntityType::TROPICAL_FISH),
            (&Item::PUFFERFISH_BUCKET, &EntityType::PUFFERFISH),
            (&Item::TADPOLE_BUCKET, &EntityType::TADPOLE),
        ];

        for (bucket, entity_type) in cases {
            assert_eq!(mob_bucket_entity_type(bucket), Some(entity_type));
        }
        assert_eq!(mob_bucket_entity_type(&Item::WATER_BUCKET), None);
        assert_eq!(mob_bucket_entity_type(&Item::LAVA_BUCKET), None);
    }

    #[test]
    fn mob_buckets_use_vanilla_empty_sounds() {
        assert_eq!(
            mob_bucket_empty_sound(&Item::AXOLOTL_BUCKET),
            Some(Sound::ItemBucketEmptyAxolotl)
        );
        assert_eq!(
            mob_bucket_empty_sound(&Item::TADPOLE_BUCKET),
            Some(Sound::ItemBucketEmptyTadpole)
        );
        assert_eq!(
            mob_bucket_empty_sound(&Item::COD_BUCKET),
            Some(Sound::ItemBucketEmptyFish)
        );
        assert_eq!(mob_bucket_empty_sound(&Item::WATER_BUCKET), None);
    }
}
