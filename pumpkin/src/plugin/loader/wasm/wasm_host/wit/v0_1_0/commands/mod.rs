use wasmtime::component::Resource;

use crate::{
    command::{
        args::{
            GetClientSideArgParser,
            block::{BlockArgumentConsumer, BlockPredicateArgumentConsumer},
            bool::BoolArgConsumer,
            bounded_num::{BoundedNumArgumentConsumer, ToFromNumber},
            difficulty::DifficultyArgumentConsumer,
            entities::EntitiesArgumentConsumer,
            entity::EntityArgumentConsumer,
            entity_anchor::EntityAnchorArgumentConsumer,
            gamemode::GamemodeArgumentConsumer,
            message::MsgArgConsumer,
            players::PlayersArgumentConsumer,
            position_2d::Position2DArgumentConsumer,
            position_3d::Position3DArgumentConsumer,
            position_block::BlockPosArgumentConsumer,
            resource::item::{ItemArgumentConsumer, ItemPredicateArgumentConsumer},
            resource_location::ResourceLocationArgumentConsumer,
            rotation::RotationArgumentConsumer,
            simple::SimpleArgConsumer,
            textcomponent::TextComponentArgConsumer,
            time::TimeArgumentConsumer,
        },
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
                        ConsumedArgs, PermissionLevel, StringType,
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

fn bounded_num_argument<T: ToFromNumber + 'static>(
    state: &mut PluginHostState,
    name: String,
    min: Option<T>,
    max: Option<T>,
) -> Resource<CommandNode>
where
    BoundedNumArgumentConsumer<T>: GetClientSideArgParser,
{
    let mut consumer = BoundedNumArgumentConsumer::<T>::new();
    if let Some(min) = min {
        consumer = consumer.min(min);
    }
    if let Some(max) = max {
        consumer = consumer.max(max);
    }
    state.add_command_node(argument(name, consumer)).unwrap()
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
            ArgumentType::Float((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Double((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Integer((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Long((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::String(string_type) => match string_type {
                StringType::SingleWord | StringType::Quotable => self
                    .add_command_node(argument(name, SimpleArgConsumer))
                    .unwrap(),
                StringType::Greedy => self
                    .add_command_node(argument(name, MsgArgConsumer))
                    .unwrap(),
            },
            ArgumentType::Entities => self
                .add_command_node(argument(name, EntitiesArgumentConsumer))
                .unwrap(),
            ArgumentType::Entity => self
                .add_command_node(argument(name, EntityArgumentConsumer))
                .unwrap(),
            ArgumentType::Players | ArgumentType::GameProfile => self
                .add_command_node(argument(name, PlayersArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockPos => self
                .add_command_node(argument(name, BlockPosArgumentConsumer))
                .unwrap(),
            ArgumentType::Position3d => self
                .add_command_node(argument(name, Position3DArgumentConsumer))
                .unwrap(),
            ArgumentType::Position2d => self
                .add_command_node(argument(name, Position2DArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockState => self
                .add_command_node(argument(name, BlockArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockPredicate => self
                .add_command_node(argument(name, BlockPredicateArgumentConsumer))
                .unwrap(),
            ArgumentType::Item => self
                .add_command_node(argument(name, ItemArgumentConsumer))
                .unwrap(),
            ArgumentType::ItemPredicate => self
                .add_command_node(argument(name, ItemPredicateArgumentConsumer))
                .unwrap(),
            ArgumentType::Component => self
                .add_command_node(argument(name, TextComponentArgConsumer))
                .unwrap(),
            ArgumentType::Rotation => self
                .add_command_node(argument(name, RotationArgumentConsumer))
                .unwrap(),
            ArgumentType::ResourceLocation | ArgumentType::Resource(_) => self
                .add_command_node(argument(name, ResourceLocationArgumentConsumer))
                .unwrap(),
            ArgumentType::EntityAnchor => self
                .add_command_node(argument(name, EntityAnchorArgumentConsumer))
                .unwrap(),
            ArgumentType::Gamemode => self
                .add_command_node(argument(name, GamemodeArgumentConsumer))
                .unwrap(),
            ArgumentType::Difficulty => self
                .add_command_node(argument(name, DifficultyArgumentConsumer))
                .unwrap(),
            ArgumentType::Time(_) => self
                .add_command_node(argument(name, TimeArgumentConsumer))
                .unwrap(),
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
        // Unless we make the native command registration code less convenient to use, this is our best option
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
