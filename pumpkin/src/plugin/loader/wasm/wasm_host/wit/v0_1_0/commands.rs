use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{
        self,
        plugin::{
            command::{
                Arg, ArgumentType, Command, CommandNode, CommandSender, CommandSenderType,
                ConsumedArgs, PermissionLevel,
            },
            common::Position,
            player::Player,
            server::Server,
            text::TextComponent,
            world::World,
        },
    },
};

impl pumpkin::plugin::command::Host for PluginHostState {}

impl pumpkin::plugin::command::HostConsumedArgs for PluginHostState {
    async fn get_value(&mut self, consumed_args: Resource<ConsumedArgs>, key: String) -> Arg {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<ConsumedArgs>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::command::HostCommand for PluginHostState {
    async fn new(&mut self, names: Vec<String>, description: String) -> Resource<Command> {
        todo!()
    }

    async fn then(&mut self, command: Resource<Command>, node: Resource<CommandNode>) -> () {
        todo!()
    }

    async fn execute(&mut self, command: Resource<Command>, handler_id: u32) -> () {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<Command>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::command::HostCommandSender for PluginHostState {
    async fn get_command_sender_type(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> CommandSenderType {
        todo!()
    }

    async fn send_message(
        &mut self,
        command_sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> () {
        todo!()
    }

    async fn set_success_count(&mut self, command_sender: Resource<CommandSender>, count: i32) {
        todo!()
    }

    async fn is_player(&mut self, command_sender: Resource<CommandSender>) -> bool {
        todo!()
    }

    async fn is_console(&mut self, command_sender: Resource<CommandSender>) -> bool {
        todo!()
    }

    async fn as_player(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> Option<Resource<Player>> {
        todo!()
    }

    async fn permission_level(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> PermissionLevel {
        todo!()
    }

    async fn has_permission_level(
        &mut self,
        command_sender: Resource<CommandSender>,
        level: PermissionLevel,
    ) -> bool {
        todo!()
    }

    async fn has_permission(
        &mut self,
        command_sender: Resource<CommandSender>,
        server: Resource<Server>,
        node: String,
    ) -> bool {
        todo!()
    }

    async fn position(&mut self, command_sender: Resource<CommandSender>) -> Option<Position> {
        todo!()
    }

    async fn world(&mut self, command_sender: Resource<CommandSender>) -> Option<Resource<World>> {
        todo!()
    }

    async fn get_locale(&mut self, command_sender: Resource<CommandSender>) -> String {
        todo!()
    }

    async fn should_receive_feedback(&mut self, command_sender: Resource<CommandSender>) -> bool {
        todo!()
    }

    async fn should_broadcast_console_to_ops(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> bool {
        todo!()
    }

    async fn should_track_output(&mut self, command_sender: Resource<CommandSender>) -> bool {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<CommandSender>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::command::HostCommandNode for PluginHostState {
    async fn literal(&mut self, name: String) -> Resource<CommandNode> {
        todo!()
    }

    async fn argument(&mut self, name: String, arg_type: ArgumentType) -> Resource<CommandNode> {
        todo!()
    }

    async fn then(
        &mut self,
        self_command_node: Resource<CommandNode>,
        node: Resource<CommandNode>,
    ) {
        todo!()
    }

    async fn execute(&mut self, command_node: Resource<CommandNode>, handler_id: u32) {
        todo!()
    }

    async fn require_permission(
        &mut self,
        command_node: Resource<CommandNode>,
        level: PermissionLevel,
    ) {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<CommandNode>) -> wasmtime::Result<()> {
        todo!()
    }
}
