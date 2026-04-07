use pumpkin_data::block_state::PistonBehavior;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockFlags, SimpleWorld};
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::wit::v0_1_0::pumpkin::plugin::world::{
    BlockFlags as WitBlockFlags, BlockPos as WitBlockPos, BlockState as WitBlockState,
    PistonBehavior as WitPistonBehavior,
};
use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, TextComponentResource, WorldResource},
    wit::v0_1_0::pumpkin::{self, plugin::world::World},
};

// --- Trapping Helpers ---
impl PluginHostState {
    fn get_world_res(&self, res: &Resource<World>) -> wasmtime::Result<&WorldResource> {
        self.resource_table
            .get::<WorldResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_text_provider(
        &self,
        res: &Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<pumpkin_util::text::TextComponent> {
        Ok(self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)?
            .provider
            .clone())
    }
}

impl pumpkin::plugin::world::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_world_name()
            .to_string())
    }

    async fn get_block_state_id(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref.provider.get_block_state_id(&internal_pos).await)
    }

    async fn get_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlockState> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world_ref.provider.get_block_state(&internal_pos).await;

        Ok(WitBlockState {
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
        })
    }

    async fn set_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        state: u16,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

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

        world_ref
            .provider
            .clone()
            .set_block_state(&internal_pos, state, internal_flags)
            .await;
        Ok(())
    }

    async fn get_time_of_day(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_time_of_day().await as u64)
    }

    async fn set_time_of_day(&mut self, world: Resource<World>, time: u64) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_time_of_day(time as i64)
            .await;
        Ok(())
    }

    async fn get_world_age(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_world_age().await as u64)
    }

    async fn get_dimension(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .dimension
            .minecraft_name
            .to_string())
    }

    async fn get_top_block_y(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_top_block(pumpkin_util::math::vector2::Vector2::new(x, z))
            .await)
    }

    async fn get_motion_blocking_height(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_motion_blocking_height(x, z)
            .await)
    }

    async fn is_raining(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_raining().await)
    }

    async fn set_raining(&mut self, world: Resource<World>, raining: bool) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_raining(raining)
            .await;
        Ok(())
    }

    async fn is_thundering(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_thundering().await)
    }

    async fn set_thundering(
        &mut self,
        world: Resource<World>,
        thundering: bool,
    ) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_thundering(thundering)
            .await;
        Ok(())
    }

    async fn broadcast_system_message(
        &mut self,
        world: Resource<World>,
        message: Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let msg = self.get_text_provider(&message)?;
        self.get_world_res(&world)?
            .provider
            .broadcast_system_message(&msg, overlay)
            .await;
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<WorldResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
