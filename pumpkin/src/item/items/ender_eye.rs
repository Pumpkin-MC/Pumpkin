use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::projectile::eye_of_ender::EyeOfEnder;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use crate::world::portal::end::EndPortal;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::structures::StructureSet;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::generator::structure_finder::find_nearest_structure;
use pumpkin_world::world::BlockFlags;

use crate::entity::player::Player;

pub struct EnderEyeItem;

impl ItemMetadata for EnderEyeItem {
    fn ids() -> Box<[u16]> {
        [Item::ENDER_EYE.id].into()
    }
}

impl ItemBehaviour for EnderEyeItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if block.id != Block::END_PORTAL_FRAME.id {
                return;
            }

            let world = player.world();
            let state_id = world.get_block_state_id(&location).await;

            // Skip if the frame already holds an eye.
            let props_raw = block.properties(state_id).unwrap().to_props();
            if props_raw.iter().any(|(k, v)| *k == "eye" && *v == "true") {
                return;
            }

            // Build new state with eye=true.
            let props: Vec<(&str, &str)> = props_raw
                .iter()
                .map(|(k, v)| if *k == "eye" { (*k, "true") } else { (*k, *v) })
                .collect();

            let new_state_id = block.from_properties(&props).to_state_id(block);
            world
                .set_block_state(&location, new_state_id, BlockFlags::empty())
                .await;

            world
                .sync_world_event(WorldEvent::EndPortalFrameFill, location, 0)
                .await;

            // Try to complete the portal.
            EndPortal::get_new_portal(&world, location).await;

            // Consume one item.
            item.decrement(1);
        })
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            let (start_pos, end_pos) = self.get_start_and_end_pos(player);
            let checker = async |pos: &BlockPos, w: &Arc<World>| {
                w.get_block_state_id(pos).await != Block::AIR.default_state.id
            };
            if let Some((hit_pos, _)) = world.raycast(start_pos, end_pos, checker).await
                && world.get_block(&hit_pos).await == &Block::END_PORTAL_FRAME
            {
                return;
            }

            let origin = player.living_entity.entity.block_pos.load();
            let target_block_pos = find_stronghold(&world, origin);

            let Some(target) = target_block_pos else {
                return;
            };

            let spawn_pos = Vector3::new(
                player.living_entity.entity.pos.load().x,
                player.living_entity.entity.pos.load().y
                    + f64::from(EntityType::EYE_OF_ENDER.dimension[1]) * 0.5,
                player.living_entity.entity.pos.load().z,
            );

            let entity = Entity::new(world.clone(), spawn_pos, &EntityType::EYE_OF_ENDER);
            let eye = Arc::new(EyeOfEnder::new(entity));

            let target_vec = Vector3::new(
                f64::from(target.0.x),
                f64::from(target.0.y),
                f64::from(target.0.z),
            );
            eye.signal_to(target_vec).await;

            world.spawn_entity(eye).await;

            let pitch = 0.33f32 + rand::random::<f32>() * (0.5 - 0.33);
            world
                .play_sound_fine(
                    Sound::EntityEnderEyeLaunch,
                    SoundCategory::Neutral,
                    &spawn_pos,
                    1.0,
                    pitch,
                )
                .await;

            player.inventory.held_item().lock().await.decrement(1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn find_stronghold(world: &Arc<World>, origin: BlockPos) -> Option<BlockPos> {
    let level = &world.level;
    let generator = &level.world_gen;
    let seed = level.seed.0;

    find_nearest_structure(
        origin,
        &[&StructureSet::get("strongholds").unwrap().placement],
        100, // max search radius in chunks, matches vanilla default
        seed as i64,
        &generator.global_structure_cache,
    )
}
