use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
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
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        if block.id != Block::END_PORTAL_FRAME.id {
            return;
        }

        let world = player.world();
        let state_id = world.get_block_state_id(&location);

        let new_state_id = {
            // Skip if the frame already holds an eye.
            let Some(props) = block.properties(state_id) else {
                return;
            };
            let props_raw = props.to_props();
            if props_raw.iter().any(|(k, v)| *k == "eye" && *v == "true") {
                return;
            }

            // Build new state with eye=true.
            let props: Vec<(&str, &str)> = props_raw
                .iter()
                .map(|(k, v)| if *k == "eye" { (*k, "true") } else { (*k, *v) })
                .collect();

            block.from_properties(&props).to_state_id(block)
        };

        world.set_block_state(&location, new_state_id, BlockFlags::NOTIFY_LISTENERS);
        // Consume one item.
        item.decrement_unless_creative(player.gamemode.load(), 1);
        world.sync_world_event(WorldEvent::EndPortalFrameFill, location, 0);

        // Try to complete the portal.
        EndPortal::get_new_portal(&world, location);
    }

    fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world();

        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let checker = |pos: &BlockPos, w: &Arc<World>| {
            w.get_block_state_id(pos) != Block::AIR.default_state.id
        };
        if let Some((hit_pos, _)) = world.raycast(start_pos, end_pos, checker)
            && world.get_block(&hit_pos) == &Block::END_PORTAL_FRAME
        {
            return;
        }

        let origin = player.get_entity().block_pos.load();
        let target_block_pos = find_stronghold(&world, origin);

        let Some(target) = target_block_pos else {
            return;
        };

        let spawn_pos = Vector3::new(
            player.get_entity().pos.load().x,
            player.get_entity().pos.load().y
                + f64::from(EntityType::EYE_OF_ENDER.dimension[1]) * 0.5,
            player.get_entity().pos.load().z,
        );

        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::EYE_OF_ENDER);
        let eye = Arc::new(EyeOfEnder::new(entity));

        let target_vec = Vector3::new(
            f64::from(target.0.x),
            f64::from(target.0.y),
            f64::from(target.0.z),
        );
        eye.signal_to(target_vec);

        world.spawn_entity(eye);

        let pitch = 0.33f32 + rand::random::<f32>() * (0.5 - 0.33);
        world.play_sound_fine(
            Sound::EntityEnderEyeLaunch,
            SoundCategory::Neutral,
            &spawn_pos,
            1.0,
            pitch,
        );

        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::LaunchedEyeOfEnder,
        );
        let mut stack = player.inventory.held_item();
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        player.inventory.set_held_item(stack);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn find_stronghold(world: &Arc<World>, origin: BlockPos) -> Option<BlockPos> {
    let level = &world.level;
    let generator = level.world_gen();
    let seed = level.seed.0;

    let global_cache = generator.global_structure_cache()?;

    let strongholds = StructureSet::get("strongholds")?;

    find_nearest_structure(
        origin,
        &[&strongholds.placement],
        100, // max search radius in chunks, matches vanilla default
        seed as i64,
        global_cache,
    )
}
