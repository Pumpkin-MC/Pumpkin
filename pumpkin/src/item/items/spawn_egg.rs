use std::pin::Pin;

use crate::block::entities::mob_spawner::MobSpawnerBlockEntity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::data_component_impl::{
    AxolotlVariantImpl, CatVariantImpl, ChickenVariantImpl, CowVariantImpl, FoxVariantImpl,
    FrogVariantImpl, HorseVariantImpl, LlamaVariantImpl, MooshroomVariantImpl, PigVariantImpl,
    RabbitVariantImpl, SheepColorImpl, ShulkerColorImpl, VillagerVariantImpl, WolfVariantImpl,
};
use pumpkin_data::entity::entity_from_egg;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use uuid::Uuid;

pub struct SpawnEggItem;

impl ItemMetadata for SpawnEggItem {
    fn ids() -> Box<[u16]> {
        pumpkin_data::entity::spawn_egg_ids()
    }
}

pub(crate) fn apply_entity_variant(item: &ItemStack, mob: &dyn EntityBase) {
    if let Some(comp) = item.get_data_component::<ChickenVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<FrogVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<WolfVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<CatVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<VillagerVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<FoxVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<MooshroomVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<RabbitVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<PigVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<CowVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<HorseVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<LlamaVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<AxolotlVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<SheepColorImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<ShulkerColorImpl>() {
        mob.set_variant_name(&comp.value);
    }
}

impl ItemBehaviour for SpawnEggItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(entity_type) = entity_from_egg(item.item.id) {
                let world = player.world();

                if let Some(block_entity) = player.world().get_block_entity(&location)
                    && let Some(spawner) = block_entity
                        .as_any()
                        .downcast_ref::<MobSpawnerBlockEntity>()
                {
                    spawner.set_entity_type(entity_type);
                    world.update_block_entity(&block_entity);
                    item.decrement_unless_creative(player.gamemode.load(), 1);
                    return;
                }
                let pos = BlockPos(location.0 + face.to_offset());
                let pos = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y),
                    f64::from(pos.0.z) + 0.5,
                );
                // Create rotation like Vanilla
                let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;

                let mob = from_type(entity_type, pos, &world, Uuid::new_v4());

                // Set the rotation
                mob.get_entity().set_rotation(yaw, 0.0);

                apply_entity_variant(item, mob.as_ref());

                // Broadcast the new mob to all players
                world.spawn_entity(mob).await;
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
