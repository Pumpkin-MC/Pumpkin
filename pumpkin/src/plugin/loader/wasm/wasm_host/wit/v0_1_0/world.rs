use pumpkin_data::block_state::PistonBehavior;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::wit::v0_1_0::pumpkin::plugin::world::{
    BlockFlags as WitBlockFlags, BlockPos as WitBlockPos, BlockState as WitBlockState,
    PistonBehavior as WitPistonBehavior,
};
use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{PluginHostState, WorldResource},
    wit::v0_1_0::pumpkin::{self, plugin::world::World},
};

impl DowncastResourceExt<WorldResource> for Resource<World> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a WorldResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid world resource handle")
            .downcast_ref::<WorldResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut WorldResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid world resource handle")
            .downcast_mut::<WorldResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> WorldResource {
        state
            .resource_table
            .delete::<WorldResource>(Resource::new_own(self.rep()))
            .expect("invalid world resource handle")
    }
}

impl pumpkin::plugin::world::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> String {
        world
            .downcast_ref(self)
            .provider
            .get_world_name()
            .to_string()
    }

    async fn get_block_state_id(&mut self, world: Resource<World>, pos: WitBlockPos) -> u16 {
        let world_ref = world.downcast_ref(self);
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        world_ref.provider.get_block_state_id(&internal_pos).await
    }

    async fn get_block_state(&mut self, world: Resource<World>, pos: WitBlockPos) -> WitBlockState {
        let world_ref = world.downcast_ref(self);
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        // Fetch the actual BlockState struct from the world
        // get_block_state typically returns &'static BlockState in Pumpkin
        let state = world_ref.provider.get_block_state(&internal_pos).await;

        WitBlockState {
            id: state.id,
            luminance: state.luminance,
            opacity: state.opacity,
            hardness: state.hardness,
            is_air: state.is_air(),
            is_liquid: state.is_liquid(),
            is_solid: state.is_solid(),
            is_full_cube: state.is_full_cube(),
            has_random_ticks: state.has_random_ticks(),
            piston_behavior: match state.piston_behavior {
                PistonBehavior::Normal => WitPistonBehavior::Normal,
                PistonBehavior::Destroy => WitPistonBehavior::Destroy,
                PistonBehavior::Block => WitPistonBehavior::Block,
                PistonBehavior::Ignore => WitPistonBehavior::Ignore,
                PistonBehavior::PushOnly => WitPistonBehavior::PushOnly,
            },
        }
    }

    async fn set_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        state: u16,
        update_flags: WitBlockFlags,
    ) {
        let world_ref = world.downcast_ref(self);
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        // Map WIT flags to your internal bitflags
        let mut internal_flags = BlockFlags::empty();
        if update_flags.contains(WitBlockFlags::NOTIFY_NEIGHBORS) {
            internal_flags |= BlockFlags::NOTIFY_NEIGHBORS;
        }
        if update_flags.contains(WitBlockFlags::NOTIFY_LISTENERS) {
            internal_flags |= BlockFlags::NOTIFY_LISTENERS;
        }
        if update_flags.contains(WitBlockFlags::FORCE_STATE) {
            internal_flags |= BlockFlags::FORCE_STATE;
        }
        if update_flags.contains(WitBlockFlags::SKIP_DROPS) {
            internal_flags |= BlockFlags::SKIP_DROPS;
        }
        if update_flags.contains(WitBlockFlags::MOVED) {
            internal_flags |= BlockFlags::MOVED;
        }
        if update_flags.contains(WitBlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT) {
            internal_flags |= BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ADDED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ADDED_CALLBACK;
        }

        // Update the world
        world_ref
            .provider
            .set_block_state(&internal_pos, state, internal_flags)
            .await;
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<WorldResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
