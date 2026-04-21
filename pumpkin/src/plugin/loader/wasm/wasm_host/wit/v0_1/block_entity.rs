use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::{
        self,
        plugin::{
            block_entity::{BlockEntity, CommandBlockEntity},
            common::BlockPosition,
        },
    },
};

impl pumpkin::plugin::block_entity::Host for PluginHostState {}

impl pumpkin::plugin::block_entity::HostBlockEntity for PluginHostState {
    async fn resource_location(&mut self, _res: Resource<BlockEntity>) -> wasmtime::Result<String> {
        todo!("Implement resource_location lookup")
    }

    async fn get_position(
        &mut self,
        _res: Resource<BlockEntity>,
    ) -> wasmtime::Result<BlockPosition> {
        todo!("Implement get_position lookup")
    }

    async fn get_id(&mut self, _res: Resource<BlockEntity>) -> wasmtime::Result<u32> {
        todo!("Implement get_id lookup")
    }

    async fn is_dirty(&mut self, _res: Resource<BlockEntity>) -> wasmtime::Result<bool> {
        todo!("Implement is_dirty lookup")
    }

    async fn clear_dirty(&mut self, _res: Resource<BlockEntity>) -> wasmtime::Result<()> {
        todo!("Implement clear_dirty logic")
    }

    async fn drop(&mut self, _rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        todo!();
    }
}

impl pumpkin::plugin::block_entity::HostCommandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        _res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        todo!("Implement upcasting/mapping to BlockEntity")
    }

    async fn last_output(
        &mut self,
        _res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<String> {
        todo!("Implement last_output lookup")
    }

    async fn track_output(&mut self, _res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        todo!("Implement track_output lookup")
    }

    async fn success_count(&mut self, _res: Resource<CommandBlockEntity>) -> wasmtime::Result<u32> {
        todo!("Implement success_count lookup")
    }

    async fn command(&mut self, _res: Resource<CommandBlockEntity>) -> wasmtime::Result<String> {
        todo!("Implement command lookup")
    }

    async fn auto(&mut self, _res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        todo!("Implement auto lookup")
    }

    async fn condition_met(
        &mut self,
        _res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<bool> {
        todo!("Implement condition_met lookup")
    }

    async fn powered(&mut self, _res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        todo!("Implement powered lookup")
    }

    async fn drop(&mut self, _rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        todo!();
    }
}
