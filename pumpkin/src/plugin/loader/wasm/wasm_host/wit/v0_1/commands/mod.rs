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
        tree::{
            CommandTree,
            builder::{NonLeafNodeBuilder, argument, literal},
        },
    },
    plugin::loader::wasm::wasm_host::{
        state::{
            CommandNodeResource, CommandResource, CommandSenderResource, ConsumedArgsResource,
            PluginHostState, ServerResource, TextComponentResource,
        },
        wit::v0_1::{
            commands::executor::WasmCommandExecutor,
            pumpkin::{
                self,
                plugin::{
                    command::{
                        Arg, ArgumentType, Command, CommandNode, CommandSender, CommandSenderType,
                        ConsumedArgs, PermissionLevel, StringType,
                    },
                    common::{BlockPos as WitBlockPos, Locale, Position},
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

impl PluginHostState {
    fn get_command_mut(
        &mut self,
        res: &Resource<Command>,
    ) -> wasmtime::Result<&mut CommandResource> {
        self.resource_table
            .get_mut::<CommandResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
    fn get_node_mut(
        &mut self,
        res: &Resource<CommandNode>,
    ) -> wasmtime::Result<&mut CommandNodeResource> {
        self.resource_table
            .get_mut::<CommandNodeResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
    fn take_node(&mut self, res: &Resource<CommandNode>) -> wasmtime::Result<CommandNodeResource> {
        self.resource_table
            .delete::<CommandNodeResource>(Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
    fn get_sender_res(
        &self,
        res: &Resource<CommandSender>,
    ) -> wasmtime::Result<&CommandSenderResource> {
        self.resource_table
            .get::<CommandSenderResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
    fn get_sender_mut(
        &mut self,
        res: &Resource<CommandSender>,
    ) -> wasmtime::Result<&mut CommandSenderResource> {
        self.resource_table
            .get_mut::<CommandSenderResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl pumpkin::plugin::command::Host for PluginHostState {}

impl pumpkin::plugin::command::HostConsumedArgs for PluginHostState {
    #[expect(clippy::too_many_lines)]
    async fn get_value(
        &mut self,
        consumed_args: Resource<ConsumedArgs>,
        key: String,
    ) -> wasmtime::Result<Arg> {
        use crate::plugin::loader::wasm::wasm_host::args::OwnedArg;

        let resource = self
            .resource_table
            .get::<ConsumedArgsResource>(&Resource::new_own(consumed_args.rep()))
            .map_err(wasmtime::Error::from)?;

        let Some(owned_arg) = resource.provider.get(&key).cloned() else {
            return Ok(Arg::Simple(String::new()));
        };

        Ok(match owned_arg {
            OwnedArg::Simple(s) => Arg::Simple(s),
            OwnedArg::Msg(s) => Arg::Msg(s),
            OwnedArg::Bool(b) => Arg::Bool(b),
            OwnedArg::Item(s) => Arg::Item(s),
            OwnedArg::ItemPredicate(s) => Arg::ItemPredicate(s),
            OwnedArg::ResourceLocation(s) => Arg::ResourceLocation(s),
            OwnedArg::Block(s) => Arg::Block(s),
            OwnedArg::BlockPredicate(s) => Arg::BlockPredicate(s),
            OwnedArg::Time(t) => Arg::Time(t),
            OwnedArg::Num(n) => {
                use crate::command::args::bounded_num::{NotInBounds, Number};
                let convert_num = |n: Number| match n {
                    Number::F64(v) => pumpkin::plugin::command::Number::Float64(v),
                    Number::F32(v) => pumpkin::plugin::command::Number::Float32(v),
                    Number::I32(v) => pumpkin::plugin::command::Number::Int32(v),
                    Number::I64(v) => pumpkin::plugin::command::Number::Int64(v),
                };
                Arg::Num(n.map(convert_num).map_err(|e| match e {
                    NotInBounds::LowerBound(a, b) => {
                        pumpkin::plugin::command::NotInBounds::LowerBound((
                            convert_num(a),
                            convert_num(b),
                        ))
                    }
                    NotInBounds::UpperBound(a, b) => {
                        pumpkin::plugin::command::NotInBounds::UpperBound((
                            convert_num(a),
                            convert_num(b),
                        ))
                    }
                }))
            }
            OwnedArg::BlockPos(p) => Arg::BlockPos(WitBlockPos {
                x: p.0.x,
                y: p.0.y,
                z: p.0.z,
            }),
            OwnedArg::Pos3D(v) => Arg::Pos3d((v.x, v.y, v.z)),
            OwnedArg::Pos2D(v) => Arg::Pos2d((v.x, v.y)),
            OwnedArg::Rotation(a, b, c, d) => Arg::Rotation((a, b, c, d)),
            OwnedArg::GameMode(g) => Arg::Gamemode(match g {
                pumpkin_util::GameMode::Survival => pumpkin::plugin::common::GameMode::Survival,
                pumpkin_util::GameMode::Creative => pumpkin::plugin::common::GameMode::Creative,
                pumpkin_util::GameMode::Adventure => pumpkin::plugin::common::GameMode::Adventure,
                pumpkin_util::GameMode::Spectator => pumpkin::plugin::common::GameMode::Spectator,
            }),
            OwnedArg::Difficulty(d) => Arg::Difficulty(match d {
                pumpkin_util::Difficulty::Peaceful => pumpkin::plugin::server::Difficulty::Peaceful,
                pumpkin_util::Difficulty::Easy => pumpkin::plugin::server::Difficulty::Easy,
                pumpkin_util::Difficulty::Normal => pumpkin::plugin::server::Difficulty::Normal,
                pumpkin_util::Difficulty::Hard => pumpkin::plugin::server::Difficulty::Hard,
            }),
            OwnedArg::Players(players) => {
                let mut resources = Vec::new();
                for p in players {
                    if let Ok(r) = self.add_player(p) {
                        resources.push(r);
                    }
                }
                Arg::Players(resources)
            }
            OwnedArg::Particle(p) => Arg::Particle(format!("{p:?}")),
            OwnedArg::TextComponent(t) => {
                let r = self
                    .resource_table
                    .push(TextComponentResource { provider: t })
                    .map_err(wasmtime::Error::from)?;
                Arg::TextComponent(wasmtime::component::Resource::new_own(r.rep()))
            }
            OwnedArg::BossbarColor(c) => Arg::BossbarColor(match c {
                crate::world::bossbar::BossbarColor::Pink => {
                    pumpkin::plugin::command::BossbarColor::Pink
                }
                crate::world::bossbar::BossbarColor::Blue => {
                    pumpkin::plugin::command::BossbarColor::Blue
                }
                crate::world::bossbar::BossbarColor::Red => {
                    pumpkin::plugin::command::BossbarColor::Red
                }
                crate::world::bossbar::BossbarColor::Green => {
                    pumpkin::plugin::command::BossbarColor::Green
                }
                crate::world::bossbar::BossbarColor::Yellow => {
                    pumpkin::plugin::command::BossbarColor::Yellow
                }
                crate::world::bossbar::BossbarColor::Purple => {
                    pumpkin::plugin::command::BossbarColor::Purple
                }
                crate::world::bossbar::BossbarColor::White => {
                    pumpkin::plugin::command::BossbarColor::White
                }
            }),
            OwnedArg::BossbarStyle(s) => Arg::BossbarStyle(match s {
                crate::world::bossbar::BossbarDivisions::NoDivision => {
                    pumpkin::plugin::command::BossbarStyle::NoDivision
                }
                crate::world::bossbar::BossbarDivisions::Notches6 => {
                    pumpkin::plugin::command::BossbarStyle::Notches6
                }
                crate::world::bossbar::BossbarDivisions::Notches10 => {
                    pumpkin::plugin::command::BossbarStyle::Notches10
                }
                crate::world::bossbar::BossbarDivisions::Notches12 => {
                    pumpkin::plugin::command::BossbarStyle::Notches12
                }
                crate::world::bossbar::BossbarDivisions::Notches20 => {
                    pumpkin::plugin::command::BossbarStyle::Notches20
                }
            }),
            OwnedArg::SoundCategory(s) => Arg::SoundCategory(match s {
                pumpkin_data::sound::SoundCategory::Master
                | pumpkin_data::sound::SoundCategory::Ui => {
                    pumpkin::plugin::command::SoundCategory::Master
                }
                pumpkin_data::sound::SoundCategory::Music => {
                    pumpkin::plugin::command::SoundCategory::Music
                }
                pumpkin_data::sound::SoundCategory::Records => {
                    pumpkin::plugin::command::SoundCategory::Records
                }
                pumpkin_data::sound::SoundCategory::Weather => {
                    pumpkin::plugin::command::SoundCategory::Weather
                }
                pumpkin_data::sound::SoundCategory::Blocks => {
                    pumpkin::plugin::command::SoundCategory::Blocks
                }
                pumpkin_data::sound::SoundCategory::Hostile => {
                    pumpkin::plugin::command::SoundCategory::Hostile
                }
                pumpkin_data::sound::SoundCategory::Neutral => {
                    pumpkin::plugin::command::SoundCategory::Neutral
                }
                pumpkin_data::sound::SoundCategory::Players => {
                    pumpkin::plugin::command::SoundCategory::Players
                }
                pumpkin_data::sound::SoundCategory::Ambient => {
                    pumpkin::plugin::command::SoundCategory::Ambient
                }
                pumpkin_data::sound::SoundCategory::Voice => {
                    pumpkin::plugin::command::SoundCategory::Voice
                }
            }),
            OwnedArg::DamageType(d) => Arg::DamageType(format!("{d:?}")),
            OwnedArg::Effect(e) => Arg::Effect(e.minecraft_name.to_string()),
            OwnedArg::Enchantment(e) => Arg::Enchantment(e.name.to_string()),
            OwnedArg::EntityAnchor(a) => Arg::EntityAnchor(match a {
                crate::command::args::EntityAnchor::Eyes => {
                    pumpkin::plugin::command::EntityAnchor::Eyes
                }
                crate::command::args::EntityAnchor::Feet => {
                    pumpkin::plugin::command::EntityAnchor::Feet
                }
            }),
            // These types don't have direct WIT resource mappings yet
            OwnedArg::Entities(_)
            | OwnedArg::Entity(_)
            | OwnedArg::GameProfiles(_)
            | OwnedArg::CommandTree(_) => Arg::Simple(String::new()),
        })
    }

    async fn drop(&mut self, rep: Resource<ConsumedArgs>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ConsumedArgsResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::command::HostCommand for PluginHostState {
    async fn new(
        &mut self,
        names: Vec<String>,
        description: String,
    ) -> wasmtime::Result<Resource<Command>> {
        self.add_command(CommandTree::new(names, description))
            .map_err(|_| wasmtime::Error::msg("Failed to add command resource"))
    }

    async fn then(
        &mut self,
        command: Resource<Command>,
        node: Resource<CommandNode>,
    ) -> wasmtime::Result<()> {
        let node_data = self.take_node(&node)?;
        let command_res = self.get_command_mut(&command)?;
        command_res.provider = command_res.provider.clone().then(node_data.provider);
        Ok(())
    }

    async fn execute_with_handler_id(
        &mut self,
        command: Resource<Command>,
        handler_id: u32,
    ) -> wasmtime::Result<()> {
        let plugin = self
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin dropped"))?;
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not initialized"))?;

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };
        let command_res = self.get_command_mut(&command)?;
        command_res.provider = command_res.provider.clone().execute(executor);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Command>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<CommandResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::command::HostCommandSender for PluginHostState {
    async fn get_command_sender_type(
        &mut self,
        _res: Resource<CommandSender>,
    ) -> wasmtime::Result<CommandSenderType> {
        Err(wasmtime::Error::msg(
            "get_command_sender_type not implemented",
        ))
    }

    async fn get_name(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<String> {
        Ok(self.get_sender_res(&sender)?.provider.to_string())
    }

    async fn send_message(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))?
            .provider
            .clone();
        self.get_sender_res(&sender)?
            .provider
            .send_message(component)
            .await;
        Ok(())
    }

    async fn send_system_message(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))?
            .provider
            .clone();
        self.get_sender_res(&sender)?
            .provider
            .send_message(component)
            .await;
        Ok(())
    }

    async fn send_error(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))?
            .provider
            .clone();
        self.get_sender_res(&sender)?
            .provider
            .send_message(component.color(pumpkin_util::text::color::Color::Named(
                pumpkin_util::text::color::NamedColor::Red,
            )))
            .await;
        Ok(())
    }

    async fn set_success_count(
        &mut self,
        sender: Resource<CommandSender>,
        count: i32,
    ) -> wasmtime::Result<()> {
        self.get_sender_mut(&sender)?
            .provider
            .set_success_count(count as u32);
        Ok(())
    }

    async fn is_player(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<bool> {
        Ok(matches!(
            self.get_sender_res(&sender)?.provider,
            crate::command::CommandSender::Player(_)
        ))
    }

    async fn is_console(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<bool> {
        Ok(matches!(
            self.get_sender_res(&sender)?.provider,
            crate::command::CommandSender::Console | crate::command::CommandSender::Rcon(_)
        ))
    }

    async fn as_player(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        if let crate::command::CommandSender::Player(player) =
            &self.get_sender_res(&sender)?.provider
        {
            Ok(Some(self.add_player(player.clone()).map_err(|_| {
                wasmtime::Error::msg("Failed to add player resource")
            })?))
        } else {
            Ok(None)
        }
    }

    async fn permission_level(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<PermissionLevel> {
        Ok(
            match self.get_sender_res(&sender)?.provider.permission_lvl() {
                pumpkin_util::PermissionLvl::Zero => PermissionLevel::Zero,
                pumpkin_util::PermissionLvl::One => PermissionLevel::One,
                pumpkin_util::PermissionLvl::Two => PermissionLevel::Two,
                pumpkin_util::PermissionLvl::Three => PermissionLevel::Three,
                pumpkin_util::PermissionLvl::Four => PermissionLevel::Four,
            },
        )
    }

    async fn has_permission_level(
        &mut self,
        sender: Resource<CommandSender>,
        level: PermissionLevel,
    ) -> wasmtime::Result<bool> {
        let required = match level {
            PermissionLevel::Zero => pumpkin_util::PermissionLvl::Zero,
            PermissionLevel::One => pumpkin_util::PermissionLvl::One,
            PermissionLevel::Two => pumpkin_util::PermissionLvl::Two,
            PermissionLevel::Three => pumpkin_util::PermissionLvl::Three,
            PermissionLevel::Four => pumpkin_util::PermissionLvl::Four,
        };
        Ok(self.get_sender_res(&sender)?.provider.permission_lvl() >= required)
    }

    async fn has_permission(
        &mut self,
        sender: Resource<CommandSender>,
        server: Resource<Server>,
        node: String,
    ) -> wasmtime::Result<bool> {
        let sender_provider = &self.get_sender_res(&sender)?.provider;
        let server_provider = &self
            .resource_table
            .get::<ServerResource>(&Resource::new_own(server.rep()))?
            .provider;
        Ok(sender_provider.has_permission(server_provider, &node).await)
    }

    async fn position(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<Option<Position>> {
        Ok(self
            .get_sender_res(&sender)?
            .provider
            .position()
            .map(|p| (p.x, p.y, p.z)))
    }

    async fn world(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<Option<Resource<World>>> {
        if let Some(world) = self.get_sender_res(&sender)?.provider.world() {
            Ok(Some(self.add_world(world).map_err(|_| {
                wasmtime::Error::msg("Failed to add world resource")
            })?))
        } else {
            Ok(None)
        }
    }

    async fn get_locale(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<Locale> {
        Ok(map_util_locale_to_wit(
            self.get_sender_res(&sender)?.provider.get_locale(),
        ))
    }

    async fn should_receive_feedback(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<bool> {
        Ok(self
            .get_sender_res(&sender)?
            .provider
            .should_receive_feedback())
    }

    async fn should_broadcast_console_to_ops(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<bool> {
        Ok(self
            .get_sender_res(&sender)?
            .provider
            .should_broadcast_console_to_ops())
    }

    async fn should_track_output(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<bool> {
        Ok(self.get_sender_res(&sender)?.provider.should_track_output())
    }

    async fn drop(&mut self, rep: Resource<CommandSender>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<CommandSenderResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::command::HostCommandNode for PluginHostState {
    async fn literal(&mut self, name: String) -> wasmtime::Result<Resource<CommandNode>> {
        self.add_command_node(literal(name))
            .map_err(|_| wasmtime::Error::msg("Failed to add literal node"))
    }

    async fn argument(
        &mut self,
        name: String,
        arg_type: ArgumentType,
    ) -> wasmtime::Result<Resource<CommandNode>> {
        let node = match arg_type {
            ArgumentType::Bool => argument(name, BoolArgConsumer),
            ArgumentType::Float((min, max)) => build_bounded_node::<f32>(name, min, max),
            ArgumentType::Double((min, max)) => build_bounded_node::<f64>(name, min, max),
            ArgumentType::Integer((min, max)) => build_bounded_node::<i32>(name, min, max),
            ArgumentType::Long((min, max)) => build_bounded_node::<i64>(name, min, max),
            ArgumentType::String(st) => match st {
                StringType::SingleWord | StringType::Quotable => argument(name, SimpleArgConsumer),
                StringType::Greedy => argument(name, MsgArgConsumer),
            },
            ArgumentType::Entities => argument(name, EntitiesArgumentConsumer),
            ArgumentType::Entity => argument(name, EntityArgumentConsumer),
            ArgumentType::Players | ArgumentType::GameProfile => {
                argument(name, PlayersArgumentConsumer)
            }
            ArgumentType::BlockPos => argument(name, BlockPosArgumentConsumer),
            ArgumentType::Position3d => argument(name, Position3DArgumentConsumer),
            ArgumentType::Position2d => argument(name, Position2DArgumentConsumer),
            ArgumentType::BlockState => argument(name, BlockArgumentConsumer),
            ArgumentType::BlockPredicate => argument(name, BlockPredicateArgumentConsumer),
            ArgumentType::Item => argument(name, ItemArgumentConsumer),
            ArgumentType::ItemPredicate => argument(name, ItemPredicateArgumentConsumer),
            ArgumentType::Component => argument(name, TextComponentArgConsumer),
            ArgumentType::Rotation => argument(name, RotationArgumentConsumer),
            ArgumentType::ResourceLocation | ArgumentType::Resource(_) => {
                argument(name, ResourceLocationArgumentConsumer)
            }
            ArgumentType::EntityAnchor => argument(name, EntityAnchorArgumentConsumer),
            ArgumentType::Gamemode => argument(name, GamemodeArgumentConsumer),
            ArgumentType::Difficulty => argument(name, DifficultyArgumentConsumer),
            ArgumentType::Time(_) => argument(name, TimeArgumentConsumer),
            _ => {
                return Err(wasmtime::Error::msg(format!(
                    "Unimplemented argument type: {arg_type:?}"
                )));
            }
        };
        self.add_command_node(node)
            .map_err(|_| wasmtime::Error::msg("Failed to add argument node"))
    }

    async fn then(
        &mut self,
        self_node: Resource<CommandNode>,
        node: Resource<CommandNode>,
    ) -> wasmtime::Result<()> {
        let child = self.take_node(&node)?;
        let parent = self.get_node_mut(&self_node)?;
        let builder = std::mem::replace(&mut parent.provider, literal(""));
        parent.provider = builder.then(child.provider);
        Ok(())
    }

    async fn execute_with_handler_id(
        &mut self,
        node: Resource<CommandNode>,
        handler_id: u32,
    ) -> wasmtime::Result<()> {
        let plugin = self
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin dropped"))?;
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not initialized"))?;

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };
        let resource = self.get_node_mut(&node)?;
        let builder = std::mem::replace(&mut resource.provider, literal(""));
        resource.provider = builder.execute(executor);
        Ok(())
    }

    async fn require_with_handler_id(
        &mut self,
        _node: Resource<CommandNode>,
        _handler_id: u32,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "require_with_handler_id not implemented",
        ))
    }

    async fn drop(&mut self, rep: Resource<CommandNode>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<CommandNodeResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

fn build_bounded_node<T: ToFromNumber + 'static>(
    name: String,
    min: Option<T>,
    max: Option<T>,
) -> NonLeafNodeBuilder
where
    BoundedNumArgumentConsumer<T>: GetClientSideArgParser,
{
    let mut consumer = BoundedNumArgumentConsumer::<T>::new();
    if let Some(m) = min {
        consumer = consumer.min(m);
    }
    if let Some(m) = max {
        consumer = consumer.max(m);
    }

    argument(name, consumer)
}

#[expect(clippy::too_many_lines)]
const fn map_util_locale_to_wit(locale: pumpkin_i18n::Locale) -> Locale {
    match locale {
        pumpkin_i18n::Locale::AfZa => Locale::AfZa,
        pumpkin_i18n::Locale::ArSa => Locale::ArSa,
        pumpkin_i18n::Locale::AstEs => Locale::AstEs,
        pumpkin_i18n::Locale::AzAz => Locale::AzAz,
        pumpkin_i18n::Locale::BaRu => Locale::BaRu,
        pumpkin_i18n::Locale::Bar => Locale::Bar,
        pumpkin_i18n::Locale::BeBy => Locale::BeBy,
        pumpkin_i18n::Locale::BgBg => Locale::BgBg,
        pumpkin_i18n::Locale::BrFr => Locale::BrFr,
        pumpkin_i18n::Locale::Brb => Locale::Brb,
        pumpkin_i18n::Locale::BsBa => Locale::BsBa,
        pumpkin_i18n::Locale::CaEs => Locale::CaEs,
        pumpkin_i18n::Locale::CsCz => Locale::CsCz,
        pumpkin_i18n::Locale::CyGb => Locale::CyGb,
        pumpkin_i18n::Locale::DaDk => Locale::DaDk,
        pumpkin_i18n::Locale::DeAt => Locale::DeAt,
        pumpkin_i18n::Locale::DeCh => Locale::DeCh,
        pumpkin_i18n::Locale::DeDe => Locale::DeDe,
        pumpkin_i18n::Locale::ElGr => Locale::ElGr,
        pumpkin_i18n::Locale::EnAu => Locale::EnAu,
        pumpkin_i18n::Locale::EnCa => Locale::EnCa,
        pumpkin_i18n::Locale::EnGb => Locale::EnGb,
        pumpkin_i18n::Locale::EnNz => Locale::EnNz,
        pumpkin_i18n::Locale::EnPt => Locale::EnPt,
        pumpkin_i18n::Locale::EnUd => Locale::EnUd,
        pumpkin_i18n::Locale::EnUs => Locale::EnUs,
        pumpkin_i18n::Locale::Enp => Locale::Enp,
        pumpkin_i18n::Locale::Enws => Locale::Enws,
        pumpkin_i18n::Locale::EoUy => Locale::EoUy,
        pumpkin_i18n::Locale::EsAr => Locale::EsAr,
        pumpkin_i18n::Locale::EsCl => Locale::EsCl,
        pumpkin_i18n::Locale::EsEc => Locale::EsEc,
        pumpkin_i18n::Locale::EsEs => Locale::EsEs,
        pumpkin_i18n::Locale::EsMx => Locale::EsMx,
        pumpkin_i18n::Locale::EsUy => Locale::EsUy,
        pumpkin_i18n::Locale::EsVe => Locale::EsVe,
        pumpkin_i18n::Locale::Esan => Locale::Esan,
        pumpkin_i18n::Locale::EtEe => Locale::EtEe,
        pumpkin_i18n::Locale::EuEs => Locale::EuEs,
        pumpkin_i18n::Locale::FaIr => Locale::FaIr,
        pumpkin_i18n::Locale::FiFi => Locale::FiFi,
        pumpkin_i18n::Locale::FilPh => Locale::FilPh,
        pumpkin_i18n::Locale::FoFo => Locale::FoFo,
        pumpkin_i18n::Locale::FrCa => Locale::FrCa,
        pumpkin_i18n::Locale::FrFr => Locale::FrFr,
        pumpkin_i18n::Locale::FraDe => Locale::FraDe,
        pumpkin_i18n::Locale::FurIt => Locale::FurIt,
        pumpkin_i18n::Locale::FyNl => Locale::FyNl,
        pumpkin_i18n::Locale::GaIe => Locale::GaIe,
        pumpkin_i18n::Locale::GdGb => Locale::GdGb,
        pumpkin_i18n::Locale::GlEs => Locale::GlEs,
        pumpkin_i18n::Locale::HawUs => Locale::HawUs,
        pumpkin_i18n::Locale::HeIl => Locale::HeIl,
        pumpkin_i18n::Locale::HiIn => Locale::HiIn,
        pumpkin_i18n::Locale::HrHr => Locale::HrHr,
        pumpkin_i18n::Locale::HuHu => Locale::HuHu,
        pumpkin_i18n::Locale::HyAm => Locale::HyAm,
        pumpkin_i18n::Locale::IdId => Locale::IdId,
        pumpkin_i18n::Locale::IgNg => Locale::IgNg,
        pumpkin_i18n::Locale::IoEn => Locale::IoEn,
        pumpkin_i18n::Locale::IsIs => Locale::IsIs,
        pumpkin_i18n::Locale::Isv => Locale::Isv,
        pumpkin_i18n::Locale::ItIt => Locale::ItIt,
        pumpkin_i18n::Locale::JaJp => Locale::JaJp,
        pumpkin_i18n::Locale::JboEn => Locale::JboEn,
        pumpkin_i18n::Locale::KaGe => Locale::KaGe,
        pumpkin_i18n::Locale::KkKz => Locale::KkKz,
        pumpkin_i18n::Locale::KnIn => Locale::KnIn,
        pumpkin_i18n::Locale::KoKr => Locale::KoKr,
        pumpkin_i18n::Locale::Ksh => Locale::Ksh,
        pumpkin_i18n::Locale::KwGb => Locale::KwGb,
        pumpkin_i18n::Locale::LaLa => Locale::LaLa,
        pumpkin_i18n::Locale::LbLu => Locale::LbLu,
        pumpkin_i18n::Locale::LiLi => Locale::LiLi,
        pumpkin_i18n::Locale::Lmo => Locale::Lmo,
        pumpkin_i18n::Locale::LoLa => Locale::LoLa,
        pumpkin_i18n::Locale::LolUs => Locale::LolUs,
        pumpkin_i18n::Locale::LtLt => Locale::LtLt,
        pumpkin_i18n::Locale::LvLv => Locale::LvLv,
        pumpkin_i18n::Locale::Lzh => Locale::Lzh,
        pumpkin_i18n::Locale::MkMk => Locale::MkMk,
        pumpkin_i18n::Locale::MnMn => Locale::MnMn,
        pumpkin_i18n::Locale::MsMy => Locale::MsMy,
        pumpkin_i18n::Locale::MtMt => Locale::MtMt,
        pumpkin_i18n::Locale::Nah => Locale::Nah,
        pumpkin_i18n::Locale::NdsDe => Locale::NdsDe,
        pumpkin_i18n::Locale::NlBe => Locale::NlBe,
        pumpkin_i18n::Locale::NlNl => Locale::NlNl,
        pumpkin_i18n::Locale::NnNo => Locale::NnNo,
        pumpkin_i18n::Locale::NoNo => Locale::NoNo,
        pumpkin_i18n::Locale::OcFr => Locale::OcFr,
        pumpkin_i18n::Locale::Ovd => Locale::Ovd,
        pumpkin_i18n::Locale::PlPl => Locale::PlPl,
        pumpkin_i18n::Locale::PtBr => Locale::PtBr,
        pumpkin_i18n::Locale::PtPt => Locale::PtPt,
        pumpkin_i18n::Locale::QyaAa => Locale::QyaAa,
        pumpkin_i18n::Locale::RoRo => Locale::RoRo,
        pumpkin_i18n::Locale::Rpr => Locale::Rpr,
        pumpkin_i18n::Locale::RuRu => Locale::RuRu,
        pumpkin_i18n::Locale::RyUa => Locale::RyUa,
        pumpkin_i18n::Locale::SahSah => Locale::SahSah,
        pumpkin_i18n::Locale::SeNo => Locale::SeNo,
        pumpkin_i18n::Locale::SkSk => Locale::SkSk,
        pumpkin_i18n::Locale::SlSi => Locale::SlSi,
        pumpkin_i18n::Locale::SoSo => Locale::SoSo,
        pumpkin_i18n::Locale::SqAl => Locale::SqAl,
        pumpkin_i18n::Locale::SrCs => Locale::SrCs,
        pumpkin_i18n::Locale::SrSp => Locale::SrSp,
        pumpkin_i18n::Locale::SvSe => Locale::SvSe,
        pumpkin_i18n::Locale::Sxu => Locale::Sxu,
        pumpkin_i18n::Locale::Szl => Locale::Szl,
        pumpkin_i18n::Locale::TaIn => Locale::TaIn,
        pumpkin_i18n::Locale::ThTh => Locale::ThTh,
        pumpkin_i18n::Locale::TlPh => Locale::TlPh,
        pumpkin_i18n::Locale::TlhAa => Locale::TlhAa,
        pumpkin_i18n::Locale::Tok => Locale::Tok,
        pumpkin_i18n::Locale::TrTr => Locale::TrTr,
        pumpkin_i18n::Locale::TtRu => Locale::TtRu,
        pumpkin_i18n::Locale::UkUa => Locale::UkUa,
        pumpkin_i18n::Locale::ValEs => Locale::ValEs,
        pumpkin_i18n::Locale::VecIt => Locale::VecIt,
        pumpkin_i18n::Locale::ViVn => Locale::ViVn,
        pumpkin_i18n::Locale::YiDe => Locale::YiDe,
        pumpkin_i18n::Locale::YoNg => Locale::YoNg,
        pumpkin_i18n::Locale::ZhCn => Locale::ZhCn,
        pumpkin_i18n::Locale::ZhHk => Locale::ZhHk,
        pumpkin_i18n::Locale::ZhTw => Locale::ZhTw,
        pumpkin_i18n::Locale::ZlmArab => Locale::ZlmArab,
    }
}
