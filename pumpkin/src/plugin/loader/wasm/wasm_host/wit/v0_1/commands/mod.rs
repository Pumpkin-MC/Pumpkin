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
                    common::{Locale, Position},
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
            OwnedArg::BlockPos(p) => Arg::BlockPos((p.0.x, p.0.y, p.0.z)),
            OwnedArg::Pos3D(v) => Arg::Pos3d((v.x, v.y, v.z)),
            OwnedArg::Pos2D(v) => Arg::Pos2d((v.x, v.y)),
            OwnedArg::Rotation(a, b, c, d) => Arg::Rotation((a, b, c, d)),
            OwnedArg::GameMode(g) => Arg::GameMode(match g {
                pumpkin_util::GameMode::Survival => pumpkin::plugin::command::Gamemode::Survival,
                pumpkin_util::GameMode::Creative => pumpkin::plugin::command::Gamemode::Creative,
                pumpkin_util::GameMode::Adventure => pumpkin::plugin::command::Gamemode::Adventure,
                pumpkin_util::GameMode::Spectator => pumpkin::plugin::command::Gamemode::Spectator,
            }),
            OwnedArg::Difficulty(d) => Arg::Difficulty(match d {
                pumpkin_util::Difficulty::Peaceful => {
                    pumpkin::plugin::command::Difficulty::Peaceful
                }
                pumpkin_util::Difficulty::Easy => pumpkin::plugin::command::Difficulty::Easy,
                pumpkin_util::Difficulty::Normal => pumpkin::plugin::command::Difficulty::Normal,
                pumpkin_util::Difficulty::Hard => pumpkin::plugin::command::Difficulty::Hard,
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
        // let sender = &self.get_sender_res(&res)?.provider;
        // Ok(match sender {
        //     crate::command::CommandSender::Player(_) => CommandSenderType::Player,
        //     crate::command::CommandSender::Console => CommandSenderType::Console,
        //     crate::command::CommandSender::Rcon(_) => CommandSenderType::Rcon,
        //     crate::command::CommandSender::CommandBlock(.._) => CommandSenderType::CommandBlock,
        //     crate::command::CommandSender::Dummy => CommandSenderType::Dummy,
        // })
        todo!()
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
        todo!("Implement require_with_handler_id")
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
const fn map_util_locale_to_wit(locale: pumpkin_util::translation::Locale) -> Locale {
    match locale {
        pumpkin_util::translation::Locale::AfZa => Locale::AfZa,
        pumpkin_util::translation::Locale::ArSa => Locale::ArSa,
        pumpkin_util::translation::Locale::AstEs => Locale::AstEs,
        pumpkin_util::translation::Locale::AzAz => Locale::AzAz,
        pumpkin_util::translation::Locale::BaRu => Locale::BaRu,
        pumpkin_util::translation::Locale::Bar => Locale::Bar,
        pumpkin_util::translation::Locale::BeBy => Locale::BeBy,
        pumpkin_util::translation::Locale::BgBg => Locale::BgBg,
        pumpkin_util::translation::Locale::BrFr => Locale::BrFr,
        pumpkin_util::translation::Locale::Brb => Locale::Brb,
        pumpkin_util::translation::Locale::BsBa => Locale::BsBa,
        pumpkin_util::translation::Locale::CaEs => Locale::CaEs,
        pumpkin_util::translation::Locale::CsCz => Locale::CsCz,
        pumpkin_util::translation::Locale::CyGb => Locale::CyGb,
        pumpkin_util::translation::Locale::DaDk => Locale::DaDk,
        pumpkin_util::translation::Locale::DeAt => Locale::DeAt,
        pumpkin_util::translation::Locale::DeCh => Locale::DeCh,
        pumpkin_util::translation::Locale::DeDe => Locale::DeDe,
        pumpkin_util::translation::Locale::ElGr => Locale::ElGr,
        pumpkin_util::translation::Locale::EnAu => Locale::EnAu,
        pumpkin_util::translation::Locale::EnCa => Locale::EnCa,
        pumpkin_util::translation::Locale::EnGb => Locale::EnGb,
        pumpkin_util::translation::Locale::EnNz => Locale::EnNz,
        pumpkin_util::translation::Locale::EnPt => Locale::EnPt,
        pumpkin_util::translation::Locale::EnUd => Locale::EnUd,
        pumpkin_util::translation::Locale::EnUs => Locale::EnUs,
        pumpkin_util::translation::Locale::Enp => Locale::Enp,
        pumpkin_util::translation::Locale::Enws => Locale::Enws,
        pumpkin_util::translation::Locale::EoUy => Locale::EoUy,
        pumpkin_util::translation::Locale::EsAr => Locale::EsAr,
        pumpkin_util::translation::Locale::EsCl => Locale::EsCl,
        pumpkin_util::translation::Locale::EsEc => Locale::EsEc,
        pumpkin_util::translation::Locale::EsEs => Locale::EsEs,
        pumpkin_util::translation::Locale::EsMx => Locale::EsMx,
        pumpkin_util::translation::Locale::EsUy => Locale::EsUy,
        pumpkin_util::translation::Locale::EsVe => Locale::EsVe,
        pumpkin_util::translation::Locale::Esan => Locale::Esan,
        pumpkin_util::translation::Locale::EtEe => Locale::EtEe,
        pumpkin_util::translation::Locale::EuEs => Locale::EuEs,
        pumpkin_util::translation::Locale::FaIr => Locale::FaIr,
        pumpkin_util::translation::Locale::FiFi => Locale::FiFi,
        pumpkin_util::translation::Locale::FilPh => Locale::FilPh,
        pumpkin_util::translation::Locale::FoFo => Locale::FoFo,
        pumpkin_util::translation::Locale::FrCa => Locale::FrCa,
        pumpkin_util::translation::Locale::FrFr => Locale::FrFr,
        pumpkin_util::translation::Locale::FraDe => Locale::FraDe,
        pumpkin_util::translation::Locale::FurIt => Locale::FurIt,
        pumpkin_util::translation::Locale::FyNl => Locale::FyNl,
        pumpkin_util::translation::Locale::GaIe => Locale::GaIe,
        pumpkin_util::translation::Locale::GdGb => Locale::GdGb,
        pumpkin_util::translation::Locale::GlEs => Locale::GlEs,
        pumpkin_util::translation::Locale::HawUs => Locale::HawUs,
        pumpkin_util::translation::Locale::HeIl => Locale::HeIl,
        pumpkin_util::translation::Locale::HiIn => Locale::HiIn,
        pumpkin_util::translation::Locale::HrHr => Locale::HrHr,
        pumpkin_util::translation::Locale::HuHu => Locale::HuHu,
        pumpkin_util::translation::Locale::HyAm => Locale::HyAm,
        pumpkin_util::translation::Locale::IdId => Locale::IdId,
        pumpkin_util::translation::Locale::IgNg => Locale::IgNg,
        pumpkin_util::translation::Locale::IoEn => Locale::IoEn,
        pumpkin_util::translation::Locale::IsIs => Locale::IsIs,
        pumpkin_util::translation::Locale::Isv => Locale::Isv,
        pumpkin_util::translation::Locale::ItIt => Locale::ItIt,
        pumpkin_util::translation::Locale::JaJp => Locale::JaJp,
        pumpkin_util::translation::Locale::JboEn => Locale::JboEn,
        pumpkin_util::translation::Locale::KaGe => Locale::KaGe,
        pumpkin_util::translation::Locale::KkKz => Locale::KkKz,
        pumpkin_util::translation::Locale::KnIn => Locale::KnIn,
        pumpkin_util::translation::Locale::KoKr => Locale::KoKr,
        pumpkin_util::translation::Locale::Ksh => Locale::Ksh,
        pumpkin_util::translation::Locale::KwGb => Locale::KwGb,
        pumpkin_util::translation::Locale::LaLa => Locale::LaLa,
        pumpkin_util::translation::Locale::LbLu => Locale::LbLu,
        pumpkin_util::translation::Locale::LiLi => Locale::LiLi,
        pumpkin_util::translation::Locale::Lmo => Locale::Lmo,
        pumpkin_util::translation::Locale::LoLa => Locale::LoLa,
        pumpkin_util::translation::Locale::LolUs => Locale::LolUs,
        pumpkin_util::translation::Locale::LtLt => Locale::LtLt,
        pumpkin_util::translation::Locale::LvLv => Locale::LvLv,
        pumpkin_util::translation::Locale::Lzh => Locale::Lzh,
        pumpkin_util::translation::Locale::MkMk => Locale::MkMk,
        pumpkin_util::translation::Locale::MnMn => Locale::MnMn,
        pumpkin_util::translation::Locale::MsMy => Locale::MsMy,
        pumpkin_util::translation::Locale::MtMt => Locale::MtMt,
        pumpkin_util::translation::Locale::Nah => Locale::Nah,
        pumpkin_util::translation::Locale::NdsDe => Locale::NdsDe,
        pumpkin_util::translation::Locale::NlBe => Locale::NlBe,
        pumpkin_util::translation::Locale::NlNl => Locale::NlNl,
        pumpkin_util::translation::Locale::NnNo => Locale::NnNo,
        pumpkin_util::translation::Locale::NoNo => Locale::NoNo,
        pumpkin_util::translation::Locale::OcFr => Locale::OcFr,
        pumpkin_util::translation::Locale::Ovd => Locale::Ovd,
        pumpkin_util::translation::Locale::PlPl => Locale::PlPl,
        pumpkin_util::translation::Locale::PtBr => Locale::PtBr,
        pumpkin_util::translation::Locale::PtPt => Locale::PtPt,
        pumpkin_util::translation::Locale::QyaAa => Locale::QyaAa,
        pumpkin_util::translation::Locale::RoRo => Locale::RoRo,
        pumpkin_util::translation::Locale::Rpr => Locale::Rpr,
        pumpkin_util::translation::Locale::RuRu => Locale::RuRu,
        pumpkin_util::translation::Locale::RyUa => Locale::RyUa,
        pumpkin_util::translation::Locale::SahSah => Locale::SahSah,
        pumpkin_util::translation::Locale::SeNo => Locale::SeNo,
        pumpkin_util::translation::Locale::SkSk => Locale::SkSk,
        pumpkin_util::translation::Locale::SlSi => Locale::SlSi,
        pumpkin_util::translation::Locale::SoSo => Locale::SoSo,
        pumpkin_util::translation::Locale::SqAl => Locale::SqAl,
        pumpkin_util::translation::Locale::SrCs => Locale::SrCs,
        pumpkin_util::translation::Locale::SrSp => Locale::SrSp,
        pumpkin_util::translation::Locale::SvSe => Locale::SvSe,
        pumpkin_util::translation::Locale::Sxu => Locale::Sxu,
        pumpkin_util::translation::Locale::Szl => Locale::Szl,
        pumpkin_util::translation::Locale::TaIn => Locale::TaIn,
        pumpkin_util::translation::Locale::ThTh => Locale::ThTh,
        pumpkin_util::translation::Locale::TlPh => Locale::TlPh,
        pumpkin_util::translation::Locale::TlhAa => Locale::TlhAa,
        pumpkin_util::translation::Locale::Tok => Locale::Tok,
        pumpkin_util::translation::Locale::TrTr => Locale::TrTr,
        pumpkin_util::translation::Locale::TtRu => Locale::TtRu,
        pumpkin_util::translation::Locale::UkUa => Locale::UkUa,
        pumpkin_util::translation::Locale::ValEs => Locale::ValEs,
        pumpkin_util::translation::Locale::VecIt => Locale::VecIt,
        pumpkin_util::translation::Locale::ViVn => Locale::ViVn,
        pumpkin_util::translation::Locale::YiDe => Locale::YiDe,
        pumpkin_util::translation::Locale::YoNg => Locale::YoNg,
        pumpkin_util::translation::Locale::ZhCn => Locale::ZhCn,
        pumpkin_util::translation::Locale::ZhHk => Locale::ZhHk,
        pumpkin_util::translation::Locale::ZhTw => Locale::ZhTw,
        pumpkin_util::translation::Locale::ZlmArab => Locale::ZlmArab,
    }
}
