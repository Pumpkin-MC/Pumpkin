use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::{
    entity::{EntityBase, player::Player, r#type::from_type},
    item::{ItemBehaviour, ItemMetadata},
};
use pumpkin_data::{
    Block, BlockDirection,
    data_component_impl::{
        BucketEntityDataImpl, CustomNameImpl, EquipmentSlot, SulfurCubeContentImpl,
    },
    entity::EntityType,
    fluid::Fluid,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
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
            Item::SULFUR_CUBE_BUCKET.id,
        ]
        .into()
    }
}

impl ItemMetadata for MilkBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::MILK_BUCKET.id].into()
    }
}

fn get_start_and_end_pos(player: &Player, yaw: f32, pitch: f32) -> (Vector3<f64>, Vector3<f64>) {
    let start_pos = player.eye_position();
    let (yaw_rad, pitch_rad) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
    let block_interaction_range = 4.5;
    let direction = Vector3::new(
        -yaw_rad.sin() * pitch_rad.cos() * block_interaction_range,
        -pitch_rad.sin() * block_interaction_range,
        pitch_rad.cos() * yaw_rad.cos() * block_interaction_range,
    );

    let end_pos = start_pos.add(&direction);
    (start_pos, end_pos)
}

const fn get_mob_for_bucket(item: &Item) -> Option<(&'static EntityType, Sound)> {
    if item.id == Item::AXOLOTL_BUCKET.id {
        Some((&EntityType::AXOLOTL, Sound::ItemBucketEmptyAxolotl))
    } else if item.id == Item::COD_BUCKET.id {
        Some((&EntityType::COD, Sound::ItemBucketEmptyFish))
    } else if item.id == Item::SALMON_BUCKET.id {
        Some((&EntityType::SALMON, Sound::ItemBucketEmptyFish))
    } else if item.id == Item::TROPICAL_FISH_BUCKET.id {
        Some((&EntityType::TROPICAL_FISH, Sound::ItemBucketEmptyFish))
    } else if item.id == Item::PUFFERFISH_BUCKET.id {
        Some((&EntityType::PUFFERFISH, Sound::ItemBucketEmptyFish))
    } else if item.id == Item::TADPOLE_BUCKET.id {
        Some((&EntityType::TADPOLE, Sound::ItemBucketEmptyTadpole))
    } else if item.id == Item::SULFUR_CUBE_BUCKET.id {
        Some((&EntityType::SULFUR_CUBE, Sound::ItemBucketEmptySulfurCube))
    } else {
        None
    }
}

const fn get_empty_sound(item: &Item) -> Sound {
    if let Some((_, sound)) = get_mob_for_bucket(item) {
        sound
    } else if item.id == Item::LAVA_BUCKET.id {
        Sound::ItemBucketEmptyLava
    } else if item.id == Item::POWDER_SNOW_BUCKET.id {
        Sound::ItemBucketEmptyPowderSnow
    } else {
        Sound::ItemBucketEmpty
    }
}

const fn get_fill_sound(item: &Item) -> Sound {
    if item.id == Item::LAVA_BUCKET.id {
        Sound::ItemBucketFillLava
    } else if item.id == Item::POWDER_SNOW_BUCKET.id {
        Sound::ItemBucketFillPowderSnow
    } else {
        Sound::ItemBucketFill
    }
}

fn give_filled_bucket_stack(
    stack: &mut ItemStack,
    player: &Player,
    mut filled: ItemStack,
    limit_creative_stack_size: bool,
) {
    if limit_creative_stack_size && player.gamemode.load() == GameMode::Creative {
        let has_item = {
            let inv = player
                .inventory
                .main_inventory
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv.iter().any(|s| s.are_equal(&filled))
        };
        if has_item {
            return;
        }
        player.inventory.insert_stack_anywhere(&mut filled);
    } else if stack.item_count == 1 && player.gamemode.load() != GameMode::Creative {
        *stack = filled;
    } else {
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        let was_added = player.inventory.insert_stack_anywhere(&mut filled);
        if !was_added && !filled.is_empty() {
            player
                .world()
                .drop_stack(&player.position().to_block_pos(), filled);
        }
    }
}

pub(crate) fn give_filled_mob_bucket(stack: &mut ItemStack, player: &Player, bucket: ItemStack) {
    give_filled_bucket_stack(stack, player, bucket, false);
}

pub(crate) fn fill_bucket_stack(stack: &mut ItemStack, player: &Player, item: &'static Item) {
    give_filled_bucket_stack(stack, player, ItemStack::new(1, item), true);
}

fn fill_mob_bucket_stack(
    stack: &mut ItemStack,
    player: &Player,
    item: &'static Item,
    entity: &dyn EntityBase,
) {
    let mut bucket_stack = ItemStack::new(1, item);
    if let Some(custom_name) = entity.get_entity().custom_name.load().as_ref() {
        bucket_stack.set_data_component(CustomNameImpl {
            name: custom_name.clone(),
        });
    }
    if entity.get_entity().entity_type == &EntityType::SULFUR_CUBE
        && let Some(living) = entity.get_living_entity()
    {
        let body_stack = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::BODY);
        if !body_stack.is_empty() {
            bucket_stack.set_data_component(SulfurCubeContentImpl { item: body_stack });
        }
    }
    let mut nbt = NbtCompound::new();
    entity.write_nbt(&mut nbt);
    for key in [
        "id",
        "UUID",
        "Pos",
        "Motion",
        "Rotation",
        "FallDistance",
        "Dimension",
        "PortalCooldown",
    ] {
        nbt.child_tags.remove(key);
    }
    bucket_stack.set_data_component(BucketEntityDataImpl { nbt: Some(nbt) });
    give_filled_bucket_stack(stack, player, bucket_stack, true);
}

fn apply_mob_bucket_components(bucket_stack: &ItemStack, mob: &dyn EntityBase, player: &Player) {
    crate::item::items::spawn_egg::apply_entity_variant(bucket_stack, mob, Some(player));
    if let Some(bucketable) = mob.get_mob().and_then(|mob| mob.as_bucketable()) {
        if let Some(data) = bucket_stack.get_data_component::<BucketEntityDataImpl>()
            && let Some(nbt) = &data.nbt
        {
            bucketable.load_from_bucket_tag(nbt);
        }
        bucketable.set_from_bucket(true);
        return;
    }
    if let Some(bucket_data) = bucket_stack.get_data_component::<BucketEntityDataImpl>()
        && let Some(ref raw_nbt) = bucket_data.nbt
        && !raw_nbt.is_empty()
    {
        let mut nbt = raw_nbt.clone();
        for key in ["id", "UUID", "Pos"] {
            nbt.child_tags.remove(key);
        }
        if !nbt.is_empty() {
            let mut merged = NbtCompound::new();
            mob.write_nbt(&mut merged);
            merged.merge(&nbt);
            mob.read_nbt_non_mut(&merged);
        }
    }
    if let Some(custom_name) = bucket_stack.get_data_component::<CustomNameImpl>() {
        mob.get_entity().set_custom_name(custom_name.name.clone());
    }
    if let Some(sulfur_content) = bucket_stack.get_data_component::<SulfurCubeContentImpl>()
        && !sulfur_content.item.is_empty()
        && let Some(living) = mob.get_living_entity()
    {
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(&EquipmentSlot::BODY, sulfur_content.item.clone());
    }
    if let Some(mob_trait) = mob.get_mob() {
        mob_trait
            .get_mob_entity()
            .persistence_required
            .store(true, Ordering::Relaxed);
    }
}

fn give_player_bucket_item(player: &Player, item: &'static Item) {
    let mut held_stack = player.inventory.held_item();
    fill_bucket_stack(&mut held_stack, player, item);
    player.inventory.set_held_item(held_stack);
}

pub(crate) fn try_pickup_fluid_at(
    world: &Arc<World>,
    block_pos: BlockPos,
) -> Option<&'static Item> {
    let (block, state) = world.get_block_and_state_id(&block_pos);

    if block == &Block::POWDER_SNOW {
        world.break_block(
            &block_pos,
            None,
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
        );
        return Some(&Item::POWDER_SNOW_BUCKET);
    }

    if block.is_waterlogged(state) {
        let state_id = block.set_waterlogged(state, false).unwrap_or(state);
        world.set_block_state(&block_pos, state_id, BlockFlags::NOTIFY_ALL);
        world.schedule_fluid_tick(&Fluid::WATER, block_pos, 5, TickPriority::Normal);
        return Some(&Item::WATER_BUCKET);
    }

    if state == Block::LAVA.default_state.id || state == Block::WATER.default_state.id {
        world.break_block(&block_pos, None, BlockFlags::NOTIFY_ALL);
        world.set_block_state(
            &block_pos,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        return Some(if state == Block::LAVA.default_state.id {
            &Item::LAVA_BUCKET
        } else {
            &Item::WATER_BUCKET
        });
    }

    None
}

fn try_pickup_bucket_item(
    world: &Arc<World>,
    block_pos: BlockPos,
    direction: BlockDirection,
) -> Option<&'static Item> {
    if let Some(item) = try_pickup_fluid_at(world, block_pos) {
        return Some(item);
    }

    let target_pos = block_pos.offset(direction.to_offset());
    let (block, state) = world.get_block_and_state_id(&target_pos);

    let unwaterlogged = block.set_waterlogged(state, false)?;

    world.set_block_state(&target_pos, unwaterlogged, BlockFlags::NOTIFY_ALL);
    world.schedule_fluid_tick(&Fluid::WATER, target_pos, 5, TickPriority::Normal);
    Some(&Item::WATER_BUCKET)
}

pub(crate) const fn should_evaporate_in_nether(item: &Item, world: &World) -> bool {
    item.id != Item::LAVA_BUCKET.id
        && item.id != Item::POWDER_SNOW_BUCKET.id
        && item.id != Item::SULFUR_CUBE_BUCKET.id
        && world.dimension.water_evaporates
}

pub(crate) fn play_bucket_evaporation(world: &Arc<World>, position: &Vector3<f64>) {
    world.play_sound_raw(
        Sound::BlockFireExtinguish as u16,
        SoundCategory::Blocks,
        position,
        0.5,
        (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.8, 2.6),
    );
}

fn try_place_powder_snow(world: &Arc<World>, pos: BlockPos, direction: BlockDirection) -> bool {
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
    world.set_block_state(
        &target_pos,
        Block::POWDER_SNOW.default_state.id,
        BlockFlags::NOTIFY_NEIGHBORS,
    );
    true
}

pub(crate) fn try_place_filled_bucket(
    world: &Arc<World>,
    item: &Item,
    pos: BlockPos,
    direction: BlockDirection,
) -> bool {
    if item.id == Item::SULFUR_CUBE_BUCKET.id {
        return true;
    }
    let (block, state) = world.get_block_and_state(&pos);
    if item.id == Item::POWDER_SNOW_BUCKET.id {
        return try_place_powder_snow(world, pos, direction);
    }

    if item.id == Item::WATER_BUCKET.id && block.is_waterlogged(state.id) {
        let state_id = block.set_waterlogged(state.id, true).unwrap_or(state.id);
        world.set_block_state(&pos, state_id, BlockFlags::NOTIFY_ALL);
        world.schedule_fluid_tick(&Fluid::WATER, pos, 5, TickPriority::Normal);
        return true;
    }

    let target_pos = pos.offset(direction.to_offset());
    let (block, state) = world.get_block_and_state(&target_pos);

    if block.is_waterloggable() {
        if item.id == Item::LAVA_BUCKET.id {
            return false;
        }
        let state_id = block.set_waterlogged(state.id, true).unwrap_or(state.id);
        world.set_block_state(&target_pos, state_id, BlockFlags::NOTIFY_ALL);
        world.schedule_fluid_tick(&Fluid::WATER, target_pos, 5, TickPriority::Normal);
        return true;
    }

    if state.id == Block::AIR.default_state.id || state.is_liquid() {
        world.set_block_state(
            &target_pos,
            if item.id == Item::LAVA_BUCKET.id {
                Block::LAVA.default_state.id
            } else {
                Block::WATER.default_state.id
            },
            BlockFlags::NOTIFY_ALL,
        );
        return true;
    }

    false
}

impl ItemBehaviour for EmptyBucketItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_rotation(item, player, yaw, pitch);
    }

    fn normal_use_with_rotation(&self, _block: &Item, player: &Player, yaw: f32, pitch: f32) {
        let world = player.world();
        let (start_pos, end_pos) = get_start_and_end_pos(player, yaw, pitch);

        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::SourceOnly,
        ));
        if hit.is_miss {
            return;
        }

        let block_pos = hit.block_pos;
        let direction = hit.direction;

        let Some(item) = try_pickup_bucket_item(&world, block_pos, direction) else {
            return;
        };

        if let Some(server) = world.server.upgrade()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let mut event =
                crate::plugin::api::events::player::player_bucket::PlayerBucketFillEvent::new(
                    player_arc,
                    block_pos,
                    item.registry_key.to_string(),
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }

        world.play_sound(
            get_fill_sound(item),
            SoundCategory::Blocks,
            &block_pos.to_f64(),
        );

        give_player_bucket_item(player, item);
    }

    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        let ent = entity.get_entity();
        let entity_type = ent.entity_type;
        if entity_type == &EntityType::SULFUR_CUBE && ent.age.load(Ordering::Relaxed) >= 0 {
            let world = ent.world.load();
            world.play_sound(
                Sound::ItemBucketFillSulfurCube,
                SoundCategory::Neutral,
                &ent.pos.load(),
            );
            fill_mob_bucket_stack(item, player, &Item::SULFUR_CUBE_BUCKET, entity.as_ref());
            ent.remove();
        } else if (entity_type == &EntityType::MOOSHROOM || entity_type == &EntityType::GOAT)
            && ent.age.load(Ordering::Relaxed) >= 0
        {
            let world = ent.world.load();
            let sound = if entity_type == &EntityType::GOAT {
                if let Some(goat) = entity
                    .cast_any()
                    .downcast_ref::<crate::entity::passive::goat::GoatEntity>()
                    && goat.is_screaming()
                {
                    Sound::EntityGoatScreamingMilk
                } else {
                    Sound::EntityGoatMilk
                }
            } else {
                Sound::EntityCowMilk
            };
            world.play_sound(sound, SoundCategory::Neutral, &ent.pos.load());
            fill_bucket_stack(item, player, &Item::MILK_BUCKET);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for FilledBucketItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_rotation(item, player, yaw, pitch);
    }

    fn normal_use_with_rotation(&self, item: &Item, player: &Player, yaw: f32, pitch: f32) {
        let world = player.world();
        let (start_pos, end_pos) = get_start_and_end_pos(player, yaw, pitch);
        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::None,
        ));
        if hit.is_miss {
            return;
        }

        let pos = hit.block_pos;
        let direction = hit.direction;

        if should_evaporate_in_nether(item, &world) {
            play_bucket_evaporation(&world, &player.position());
            return;
        }
        if !try_place_filled_bucket(&world, item, pos, direction) {
            return;
        }

        if let Some(server) = world.server.upgrade()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let mut event =
                crate::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent::new(
                    player_arc,
                    pos,
                    item.registry_key.to_string(),
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
        }

        let place_pos = if item.id != Item::SULFUR_CUBE_BUCKET.id
            && world
                .get_block_and_state(&pos)
                .0
                .is_waterlogged(world.get_block_state_id(&pos))
        {
            pos
        } else {
            pos.offset(direction.to_offset())
        };

        let sound_category = if get_mob_for_bucket(item).is_some() {
            SoundCategory::Neutral
        } else {
            SoundCategory::Blocks
        };

        world.play_sound(get_empty_sound(item), sound_category, &place_pos.to_f64());

        if let Some((entity_type, _)) = get_mob_for_bucket(item) {
            let held = player.inventory.held_item();
            let bucket_stack = if !held.is_empty() && held.item.id == item.id {
                held
            } else {
                player.inventory.off_hand_item()
            };
            let spawn_coord = Vector3::new(
                f64::from(place_pos.0.x) + 0.5,
                f64::from(place_pos.0.y),
                f64::from(place_pos.0.z) + 0.5,
            );
            let mob = from_type(entity_type, spawn_coord, &world, Uuid::new_v4());
            if mob.get_mob().and_then(|mob| mob.as_bucketable()).is_some() {
                crate::entity::mob::spawn::finalize_spawn(&mob, &world, None);
                apply_mob_bucket_components(&bucket_stack, mob.as_ref(), player);
                if world.spawn_creature(
                    mob.clone(),
                    crate::plugin::api::events::entity::creature_spawn::CreatureSpawnReason::Bucket,
                    world.get_player_by_uuid(player.gameprofile.id),
                ) && let Some(fish) = mob.get_mob().and_then(|mob| mob.as_fish())
                {
                    fish.play_ambient_sound();
                }
            } else {
                apply_mob_bucket_components(&bucket_stack, mob.as_ref(), player);
                world.spawn_entity(mob);
            }
        }

        if player.gamemode.load() != GameMode::Creative {
            let item_stack = ItemStack::new(1, &Item::BUCKET);
            player
                .inventory
                .set_slot(player.inventory.get_selected_slot() as usize, item_stack);
        }
    }

    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if entity
            .get_mob()
            .and_then(|mob| mob.as_bucketable())
            .is_some()
        {
            return;
        }
        if item.item.id == Item::WATER_BUCKET.id {
            let entity_type = entity.get_entity().entity_type;
            let result_item = if entity_type == &EntityType::AXOLOTL {
                Some((&Item::AXOLOTL_BUCKET, Sound::ItemBucketFillAxolotl))
            } else if entity_type == &EntityType::COD {
                Some((&Item::COD_BUCKET, Sound::ItemBucketFillFish))
            } else if entity_type == &EntityType::SALMON {
                Some((&Item::SALMON_BUCKET, Sound::ItemBucketFillFish))
            } else if entity_type == &EntityType::TROPICAL_FISH {
                Some((&Item::TROPICAL_FISH_BUCKET, Sound::ItemBucketFillFish))
            } else if entity_type == &EntityType::PUFFERFISH {
                Some((&Item::PUFFERFISH_BUCKET, Sound::ItemBucketFillFish))
            } else if entity_type == &EntityType::TADPOLE {
                Some((&Item::TADPOLE_BUCKET, Sound::ItemBucketFillTadpole))
            } else {
                None
            };

            if let Some((mob_bucket, sound)) = result_item {
                let ent = entity.get_entity();
                let world = ent.world.load();
                world.play_sound(sound, SoundCategory::Neutral, &ent.pos.load());
                fill_mob_bucket_stack(item, player, mob_bucket, entity.as_ref());
                ent.remove();
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for MilkBucketItem {
    fn normal_use(&self, _item: &Item, _player: &Player) {
        // Drinking is handled by the consumable flow in the server (active hand + consumption tick).
    }

    fn get_use_duration(&self) -> i32 {
        32
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
