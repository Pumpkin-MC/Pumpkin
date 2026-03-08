use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{
        self,
        plugin::{
            common::BlockPosition,
            entity::{BlockEntity, CommandBlockEntity},
        },
    },
};

impl pumpkin::plugin::entity::Host for PluginHostState {}

impl pumpkin::plugin::entity::HostBlockEntity for PluginHostState {
    async fn resource_location(&mut self, _block_entity: Resource<BlockEntity>) -> String {
        todo!()
    }

    async fn get_position(&mut self, _block_entity: Resource<BlockEntity>) -> BlockPosition {
        todo!()
    }

    async fn get_id(&mut self, _block_entity: Resource<BlockEntity>) -> u32 {
        todo!()
    }

    async fn is_dirty(&mut self, _block_entity: Resource<BlockEntity>) -> bool {
        todo!()
    }

    async fn clear_dirty(&mut self, _block_entity: Resource<BlockEntity>) {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostCommandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        _command_block_entity: Resource<CommandBlockEntity>,
    ) -> Resource<BlockEntity> {
        todo!()
    }

    async fn last_output(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> String {
        todo!()
    }

    async fn track_output(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn success_count(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> u32 {
        todo!()
    }

    async fn command(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> String {
        todo!()
    }

    async fn auto(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn condition_met(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn powered(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}
