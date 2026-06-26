use pumpkin_i18n::Locale as I18nLocale;
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
    localized_log, localized_log_format,
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
            OwnedArg::Advancement(a) => Arg::Advancement(a.to_string()),
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
                return Err(wasmtime::Error::msg(localized_log_format(
                    "plugin.wasm.commands.unimplemented_argument_type",
                    &[format!("{arg_type:?}")],
                )));
            }
        };
        self.add_command_node(node).map_err(|_| {
            wasmtime::Error::msg(localized_log(
                "plugin.wasm.commands.failed_add_argument_node",
            ))
        })
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
const fn map_util_locale_to_wit(locale: I18nLocale) -> Locale {
    match locale {
        I18nLocale::AfZa => Locale::AfZa,
        I18nLocale::ArSa => Locale::ArSa,
        I18nLocale::AstEs => Locale::AstEs,
        I18nLocale::AzAz => Locale::AzAz,
        I18nLocale::BaRu => Locale::BaRu,
        I18nLocale::Bar => Locale::Bar,
        I18nLocale::BeBy => Locale::BeBy,
        I18nLocale::BgBg => Locale::BgBg,
        I18nLocale::BrFr => Locale::BrFr,
        I18nLocale::Brb => Locale::Brb,
        I18nLocale::BsBa => Locale::BsBa,
        I18nLocale::CaEs => Locale::CaEs,
        I18nLocale::CsCz => Locale::CsCz,
        I18nLocale::CyGb => Locale::CyGb,
        I18nLocale::DaDk => Locale::DaDk,
        I18nLocale::DeAt => Locale::DeAt,
        I18nLocale::DeCh => Locale::DeCh,
        I18nLocale::DeDe => Locale::DeDe,
        I18nLocale::ElGr => Locale::ElGr,
        I18nLocale::EnAu => Locale::EnAu,
        I18nLocale::EnCa => Locale::EnCa,
        I18nLocale::EnGb => Locale::EnGb,
        I18nLocale::EnNz => Locale::EnNz,
        I18nLocale::EnPt => Locale::EnPt,
        I18nLocale::EnUd => Locale::EnUd,
        I18nLocale::EnUs => Locale::EnUs,
        I18nLocale::Enp => Locale::Enp,
        I18nLocale::Enws => Locale::Enws,
        I18nLocale::EoUy => Locale::EoUy,
        I18nLocale::EsAr => Locale::EsAr,
        I18nLocale::EsCl => Locale::EsCl,
        I18nLocale::EsEc => Locale::EsEc,
        I18nLocale::EsEs => Locale::EsEs,
        I18nLocale::EsMx => Locale::EsMx,
        I18nLocale::EsUy => Locale::EsUy,
        I18nLocale::EsVe => Locale::EsVe,
        I18nLocale::Esan => Locale::Esan,
        I18nLocale::EtEe => Locale::EtEe,
        I18nLocale::EuEs => Locale::EuEs,
        I18nLocale::FaIr => Locale::FaIr,
        I18nLocale::FiFi => Locale::FiFi,
        I18nLocale::FilPh => Locale::FilPh,
        I18nLocale::FoFo => Locale::FoFo,
        I18nLocale::FrCa => Locale::FrCa,
        I18nLocale::FrFr => Locale::FrFr,
        I18nLocale::FraDe => Locale::FraDe,
        I18nLocale::FurIt => Locale::FurIt,
        I18nLocale::FyNl => Locale::FyNl,
        I18nLocale::GaIe => Locale::GaIe,
        I18nLocale::GdGb => Locale::GdGb,
        I18nLocale::GlEs => Locale::GlEs,
        I18nLocale::HawUs => Locale::HawUs,
        I18nLocale::HeIl => Locale::HeIl,
        I18nLocale::HiIn => Locale::HiIn,
        I18nLocale::HrHr => Locale::HrHr,
        I18nLocale::HuHu => Locale::HuHu,
        I18nLocale::HyAm => Locale::HyAm,
        I18nLocale::IdId => Locale::IdId,
        I18nLocale::IgNg => Locale::IgNg,
        I18nLocale::IoEn => Locale::IoEn,
        I18nLocale::IsIs => Locale::IsIs,
        I18nLocale::Isv => Locale::Isv,
        I18nLocale::ItIt => Locale::ItIt,
        I18nLocale::JaJp => Locale::JaJp,
        I18nLocale::JboEn => Locale::JboEn,
        I18nLocale::KaGe => Locale::KaGe,
        I18nLocale::KkKz => Locale::KkKz,
        I18nLocale::KnIn => Locale::KnIn,
        I18nLocale::KoKr => Locale::KoKr,
        I18nLocale::Ksh => Locale::Ksh,
        I18nLocale::KwGb => Locale::KwGb,
        I18nLocale::LaLa => Locale::LaLa,
        I18nLocale::LbLu => Locale::LbLu,
        I18nLocale::LiLi => Locale::LiLi,
        I18nLocale::Lmo => Locale::Lmo,
        I18nLocale::LoLa => Locale::LoLa,
        I18nLocale::LolUs => Locale::LolUs,
        I18nLocale::LtLt => Locale::LtLt,
        I18nLocale::LvLv => Locale::LvLv,
        I18nLocale::Lzh => Locale::Lzh,
        I18nLocale::MkMk => Locale::MkMk,
        I18nLocale::MnMn => Locale::MnMn,
        I18nLocale::MsMy => Locale::MsMy,
        I18nLocale::MtMt => Locale::MtMt,
        I18nLocale::Nah => Locale::Nah,
        I18nLocale::NdsDe => Locale::NdsDe,
        I18nLocale::NlBe => Locale::NlBe,
        I18nLocale::NlNl => Locale::NlNl,
        I18nLocale::NnNo => Locale::NnNo,
        I18nLocale::NoNo => Locale::NoNo,
        I18nLocale::OcFr => Locale::OcFr,
        I18nLocale::Ovd => Locale::Ovd,
        I18nLocale::PlPl => Locale::PlPl,
        I18nLocale::PtBr => Locale::PtBr,
        I18nLocale::PtPt => Locale::PtPt,
        I18nLocale::QyaAa => Locale::QyaAa,
        I18nLocale::RoRo => Locale::RoRo,
        I18nLocale::Rpr => Locale::Rpr,
        I18nLocale::RuRu => Locale::RuRu,
        I18nLocale::RyUa => Locale::RyUa,
        I18nLocale::SahSah => Locale::SahSah,
        I18nLocale::SeNo => Locale::SeNo,
        I18nLocale::SkSk => Locale::SkSk,
        I18nLocale::SlSi => Locale::SlSi,
        I18nLocale::SoSo => Locale::SoSo,
        I18nLocale::SqAl => Locale::SqAl,
        I18nLocale::SrCs => Locale::SrCs,
        I18nLocale::SrSp => Locale::SrSp,
        I18nLocale::SvSe => Locale::SvSe,
        I18nLocale::Sxu => Locale::Sxu,
        I18nLocale::Szl => Locale::Szl,
        I18nLocale::TaIn => Locale::TaIn,
        I18nLocale::ThTh => Locale::ThTh,
        I18nLocale::TlPh => Locale::TlPh,
        I18nLocale::TlhAa => Locale::TlhAa,
        I18nLocale::Tok => Locale::Tok,
        I18nLocale::TrTr => Locale::TrTr,
        I18nLocale::TtRu => Locale::TtRu,
        I18nLocale::UkUa => Locale::UkUa,
        I18nLocale::ValEs => Locale::ValEs,
        I18nLocale::VecIt => Locale::VecIt,
        I18nLocale::ViVn => Locale::ViVn,
        I18nLocale::YiDe => Locale::YiDe,
        I18nLocale::YoNg => Locale::YoNg,
        I18nLocale::ZhCn => Locale::ZhCn,
        I18nLocale::ZhHk => Locale::ZhHk,
        I18nLocale::ZhTw => Locale::ZhTw,
        I18nLocale::ZlmArab => Locale::ZlmArab,
    }
}
