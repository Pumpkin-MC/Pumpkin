use wasmtime::component::Resource;

use crate::{
    command::{
        args::bool::BoolArgConsumer,
        tree::builder::{argument, literal},
    },
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt,
        state::{CommandNodeResource, PluginHostState},
        wit::v0_1_0::{
            commands::executor::WasmCommandExecutor,
            pumpkin::{
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
        },
    },
};

pub mod executor;

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

    async fn execute_with_handler_id(&mut self, command: Resource<Command>, handler_id: u32) -> () {
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

impl DowncastResourceExt<CommandNodeResource> for Resource<CommandNode> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a CommandNodeResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid command-node resource handle")
            .downcast_ref()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut CommandNodeResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid command-node resource handle")
            .downcast_mut()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> CommandNodeResource {
        state
            .resource_table
            .delete(Resource::new_own(self.rep()))
            .expect("invalid command-node resource handle")
    }
}

impl pumpkin::plugin::command::HostCommandNode for PluginHostState {
    async fn literal(&mut self, name: String) -> Resource<CommandNode> {
        self.add_command_node(literal(name)).unwrap()
    }

    async fn argument(&mut self, name: String, arg_type: ArgumentType) -> Resource<CommandNode> {
        match arg_type {
            ArgumentType::Bool => self
                .add_command_node(argument(name, BoolArgConsumer))
                .unwrap(),
            ArgumentType::Float((min, max)) => todo!(),
            ArgumentType::Double(_) => todo!(),
            ArgumentType::Integer(_) => todo!(),
            ArgumentType::Long(_) => todo!(),
            ArgumentType::String(string_type) => todo!(),
            ArgumentType::Entities => todo!(),
            ArgumentType::Entity => todo!(),
            ArgumentType::Players => todo!(),
            ArgumentType::GameProfile => todo!(),
            ArgumentType::BlockPos => todo!(),
            ArgumentType::Position3d => todo!(),
            ArgumentType::Position2d => todo!(),
            ArgumentType::BlockState => todo!(),
            ArgumentType::BlockPredicate => todo!(),
            ArgumentType::Item => todo!(),
            ArgumentType::ItemPredicate => todo!(),
            ArgumentType::Component => todo!(),
            ArgumentType::Rotation => todo!(),
            ArgumentType::ResourceLocation => todo!(),
            ArgumentType::EntityAnchor => todo!(),
            ArgumentType::Gamemode => todo!(),
            ArgumentType::Difficulty => todo!(),
            ArgumentType::Time(_) => todo!(),
            ArgumentType::Resource(_) => todo!(),
        }
    }

    async fn then(
        &mut self,
        self_command_node: Resource<CommandNode>,
        node: Resource<CommandNode>,
    ) {
        todo!()
    }

    async fn execute_with_handler_id(
        &mut self,
        command_node: Resource<CommandNode>,
        handler_id: u32,
    ) {
        let plugin = self
            .plugin
            .as_ref()
            .expect("plugin should always be initialized here")
            .upgrade()
            .expect("plugin has been dropped");

        let server = self
            .server
            .clone()
            .expect("server should be set before command registration");

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };

        let resource = command_node.downcast_mut(self);
        // Unless we make the native command registration code less convinent to use, this is our best option
        let builder = std::mem::replace(&mut resource.provider, literal(""));
        resource.provider = builder.execute(executor);
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
