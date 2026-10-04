use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::block::entities::mob_spawner::MobSpawnerBlockEntity;
use crate::entity::EntityBase;
use crate::entity::mob::spawn::finalize_spawn;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::plugin::api::events::entity::creature_spawn::CreatureSpawnReason;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::data_component_impl::{
    AxolotlVariantImpl, CatVariantImpl, ChickenVariantImpl, CowVariantImpl, EntityDataImpl,
    FoxVariantImpl, FrogVariantImpl, HorseVariantImpl, LlamaVariantImpl, MooshroomVariantImpl,
    PigVariantImpl, RabbitVariantImpl, SheepColorImpl, ShulkerColorImpl, VillagerVariantImpl,
    WolfVariantImpl, ZombieNautilusVariantImpl,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity::entity_from_egg;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

pub struct SpawnEggItem;

impl ItemMetadata for SpawnEggItem {
    fn ids() -> Box<[u16]> {
        pumpkin_data::entity::spawn_egg_ids()
    }
}

/// Vanilla's `EntityTypes.OP_ONLY_CUSTOM_DATA`: types whose `entity_data` only a server
/// operator may set. None of them has a spawn egg today, but a datapack-authored loot table
/// can still hand out an item with this component, so the gate is checked here too.
fn only_op_can_set_nbt(entity_type: &EntityType) -> bool {
    std::ptr::eq(entity_type, &EntityType::FALLING_BLOCK)
        || std::ptr::eq(entity_type, &EntityType::COMMAND_BLOCK_MINECART)
        || std::ptr::eq(entity_type, &EntityType::SPAWNER_MINECART)
}

/// Permission node for op-only `entity_data`, so permission plugins can grant or deny it.
pub const NBT_PLACE_PERMISSION: &str = "minecraft:nbt.place";

/// Vanilla `PlayerList.isOp`: any ops list entry, whatever its level, passes the node's
/// `Op(One)` default, so without plugins or attachments the result matches vanilla.
fn can_place_op_nbt(player: &Player) -> bool {
    let world = player.world();
    let Some(server) = world.server.upgrade() else {
        return false;
    };
    let Some(player) = world.get_player_by_uuid(player.gameprofile.id) else {
        return false;
    };
    let is_op = server
        .data
        .operator_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_entry(&player.gameprofile.id)
        .is_some();
    let level = if is_op {
        PermissionLvl::Four
    } else {
        PermissionLvl::Zero
    };
    player.has_permission_at_level(&server, NBT_PLACE_PERMISSION, level)
}

/// Loads the stack's `entity_data` NBT into the mob. Identity and placement stay as spawned.
///
/// `user` is the player who caused the spawn, matching vanilla's `updateCustomEntityTag` user
/// argument; it is `None` for non-player sources such as a dispenser.
fn apply_entity_data(item: &ItemStack, mob: &dyn EntityBase, user: Option<&Player>) {
    let Some(nbt) = item
        .get_data_component::<EntityDataImpl>()
        .and_then(|comp| comp.nbt.as_ref())
    else {
        return;
    };
    let entity_type = mob.get_entity().entity_type;
    // Vanilla EntityType.updateCustomEntityTag loads the data only into the type it names.
    if let Some(id) = nbt.get_string("id")
        && id.strip_prefix("minecraft:").unwrap_or(id) != entity_type.resource_name
    {
        return;
    }
    if only_op_can_set_nbt(entity_type) && !user.is_some_and(can_place_op_nbt) {
        return;
    }
    let mut nbt = nbt.clone();
    for key in ["id", "UUID", "Pos"] {
        nbt.child_tags.remove(key);
    }
    if !nbt.is_empty() {
        // Vanilla TypedEntityData.loadInto merges into the mob's own save so unset keys keep their state.
        let mut merged = NbtCompound::new();
        mob.write_nbt(&mut merged);
        merged.merge(&nbt);
        mob.read_nbt_non_mut(&merged);
    }
}

/// Vanilla `EntityType.appendDefaultStackConfig`: item components first, `entity_data` last.
pub(crate) fn apply_entity_variant(item: &ItemStack, mob: &dyn EntityBase, user: Option<&Player>) {
    apply_variant(item, mob);
    apply_entity_data(item, mob, user);
}

fn apply_variant(item: &ItemStack, mob: &dyn EntityBase) {
    macro_rules! apply_variant {
        ($($ty:ty),+ $(,)?) => {
            $(
                if let Some(comp) = item.get_data_component::<$ty>() {
                    mob.set_variant_name(&comp.value);
                    return;
                }
            )+
        };
    }
    apply_variant!(
        ChickenVariantImpl,
        FrogVariantImpl,
        WolfVariantImpl,
        CatVariantImpl,
        VillagerVariantImpl,
        FoxVariantImpl,
        MooshroomVariantImpl,
        RabbitVariantImpl,
        PigVariantImpl,
        CowVariantImpl,
        HorseVariantImpl,
        LlamaVariantImpl,
        AxolotlVariantImpl,
        SheepColorImpl,
        ShulkerColorImpl,
        ZombieNautilusVariantImpl,
    );
}

/// Finalizes a mob spawned by a spawn egg, then applies the egg's components (vanilla order).
pub(crate) fn prepare_egg_mob(
    item: &ItemStack,
    mob: &Arc<dyn EntityBase>,
    world: &Arc<World>,
    user: Option<&Player>,
) {
    finalize_spawn(mob, world, None);
    apply_entity_variant(item, mob.as_ref(), user);
}

impl ItemBehaviour for SpawnEggItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        if let Some(entity_type) = entity_from_egg(item.id) {
            let world = player.world();
            let (start_pos, end_pos) = self.get_start_and_end_pos(player);
            let checker = |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos);
                if state_id == Block::AIR.default_state.id {
                    return false;
                }
                Fluid::from_state_id(state_id).is_some()
            };

            let Some((hit_pos, _)) = world.raycast(start_pos, end_pos, checker) else {
                return;
            };

            let pos = Vector3::new(
                f64::from(hit_pos.0.x) + 0.5,
                f64::from(hit_pos.0.y),
                f64::from(hit_pos.0.z) + 0.5,
            );
            let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;
            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());
            mob.get_entity().set_rotation(yaw, 0.0);

            let held = player.inventory.held_item();
            let stack = if !held.is_empty() && held.item.id == item.id {
                held
            } else {
                player.inventory.off_hand_item()
            };
            prepare_egg_mob(&stack, &mob, &world, Some(player));
            if !world.spawn_creature(
                mob,
                CreatureSpawnReason::SpawnerEgg,
                world.get_player_by_uuid(player.gameprofile.id),
            ) {
                return;
            }

            let mut main_hand = player.inventory.held_item();
            let consumed = if !main_hand.is_empty() && main_hand.item.id == item.id {
                main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player.inventory.set_held_item(main_hand);
                true
            } else {
                false
            };

            if !consumed {
                let mut off_hand = player.inventory.off_hand_item();
                if !off_hand.is_empty() && off_hand.item.id == item.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    player
                        .inventory
                        .set_stack_in_hand(pumpkin_util::Hand::Left, off_hand);
                }
            }
        }
    }

    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if let Some(entity_type) = entity_from_egg(item.item.id) {
            let world = player.world();

            if let Some(block_entity) = player.world().get_block_entity(&location) {
                if let Some(spawner) = block_entity
                    .as_any()
                    .downcast_ref::<MobSpawnerBlockEntity>()
                {
                    spawner.set_entity_type(entity_type);
                    world.update_block_entity(&block_entity);
                    item.decrement_unless_creative(player.gamemode.load(), 1);
                    return BlockActionResult::Success;
                }
                if let Some(trial_spawner) = block_entity
                    .as_any()
                    .downcast_ref::<crate::block::entities::trial_spawner::TrialSpawnerBlockEntity>()
                {
                    trial_spawner.set_entity_type(entity_type, &world);
                    world.update_block_entity(&block_entity);
                    item.decrement_unless_creative(player.gamemode.load(), 1);
                    return BlockActionResult::Success;
                }
            }

            let target_state = world.get_block_state(&location);
            let target_block = world.get_block(&location);
            let spawn_block_pos = if target_state.is_air()
                || target_block.id == Block::WATER.id
                || target_block.id == Block::LAVA.id
            {
                location
            } else {
                BlockPos(location.0 + face.to_offset())
            };
            let pos = Vector3::new(
                f64::from(spawn_block_pos.0.x) + 0.5,
                f64::from(spawn_block_pos.0.y),
                f64::from(spawn_block_pos.0.z) + 0.5,
            );
            let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;

            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());

            mob.get_entity().set_rotation(yaw, 0.0);

            prepare_egg_mob(item, &mob, &world, Some(player));

            if world.spawn_creature(
                mob,
                CreatureSpawnReason::SpawnerEgg,
                world.get_player_by_uuid(player.gameprofile.id),
            ) {
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
            BlockActionResult::Success
        } else {
            BlockActionResult::Pass
        }
    }

    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if let Some(entity_type) = entity_from_egg(item.item.id)
            && entity.get_entity().entity_type.id == entity_type.id
        {
            let world = player.world();
            let pos = entity.get_entity().pos.load();
            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());
            // no baby form, no offspring.
            if !mob
                .get_mob()
                .is_some_and(crate::entity::mob::Mob::spawn_as_baby)
            {
                return;
            }
            mob.get_entity()
                .set_rotation(rand::random::<f32>() * 360.0, 0.0);
            apply_entity_variant(item, mob.as_ref(), Some(player));
            if world.spawn_creature(
                mob,
                CreatureSpawnReason::SpawnerEgg,
                world.get_player_by_uuid(player.gameprofile.id),
            ) {
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
