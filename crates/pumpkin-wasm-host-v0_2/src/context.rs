use std::{collections::HashMap, sync::Arc};
use wasmtime::component::Resource;

use crate::{
    events::{WasmPluginEventHandler, register_generated_event},
    pumpkin::{
        self,
        plugin::{
            command::Command,
            context::{Context as WitContext, MarketplaceMetadata},
            event::{EventPriority, EventType},
            permission::{Permission, PermissionDefault, PermissionLevel},
            server::Server,
        },
    },
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl pumpkin::plugin::context::Host for PluginHostState {}

impl pumpkin::plugin::context::HostContext for PluginHostState {
    fn register_event(
        &mut self,
        context: Resource<WitContext>,
        handler_id: u32,
        event_type: EventType,
        event_priority: EventPriority,
        blocking: bool,
    ) -> wasmtime::Result<()> {
        let priority = match event_priority {
            EventPriority::Highest => pumpkin_core::plugin::EventPriority::Highest,
            EventPriority::High => pumpkin_core::plugin::EventPriority::High,
            EventPriority::Normal => pumpkin_core::plugin::EventPriority::Normal,
            EventPriority::Low => pumpkin_core::plugin::EventPriority::Low,
            EventPriority::Lowest => pumpkin_core::plugin::EventPriority::Lowest,
        };

        let plugin = self
            .plugin
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Plugin state uninitialized"))?
            .upgrade()
            .ok_or_else(|| wasmtime::Error::msg("Plugin has been dropped"))?;

        let ctx = self.get(&context)?.as_ref();
        let handler = Arc::new(WasmPluginEventHandler { handler_id, plugin });

        register_generated_event(ctx, &handler, priority, blocking, event_type);

        Ok(())
    }

    fn register_command(
        &mut self,
        context: Resource<WitContext>,
        command: Resource<Command>,
        permission: String,
    ) -> wasmtime::Result<()> {
        use pumpkin_core::command::argument_builder::ArgumentBuilder;

        let command = self.take(command)?;
        let context = self.get(&context)?.clone();
        let aliases = if command.names.len() > 1 {
            command.names[1..].to_vec()
        } else {
            Vec::new()
        };
        let node = command.builder.build();
        context.register_command_with_aliases(node, &aliases, permission);

        Ok(())
    }

    fn register_permission(
        &mut self,
        context: Resource<WitContext>,
        permission: Permission,
    ) -> wasmtime::Result<Result<(), String>> {
        let mut children = HashMap::with_capacity(permission.children.len());
        for child in permission.children {
            children.insert(child.node, child.value);
        }

        let util_permission = pumpkin_util::permission::Permission {
            node: permission.node,
            description: permission.description,
            default: match permission.default {
                PermissionDefault::Deny => pumpkin_util::permission::PermissionDefault::Deny,
                PermissionDefault::Allow => pumpkin_util::permission::PermissionDefault::Allow,
                PermissionDefault::Op(lvl) => {
                    pumpkin_util::permission::PermissionDefault::Op(match lvl {
                        PermissionLevel::Zero => pumpkin_util::permission::PermissionLvl::Zero,
                        PermissionLevel::One => pumpkin_util::permission::PermissionLvl::One,
                        PermissionLevel::Two => pumpkin_util::permission::PermissionLvl::Two,
                        PermissionLevel::Three => pumpkin_util::permission::PermissionLvl::Three,
                        PermissionLevel::Four => pumpkin_util::permission::PermissionLvl::Four,
                    })
                }
            },
            children,
        };

        let context_res = self.get(&context)?;
        Ok(context_res.register_permission(util_permission))
    }

    fn register_java_versions(
        &mut self,
        context: Resource<WitContext>,
        versions: Vec<pumpkin::plugin::player::JavaMinecraftVersion>,
    ) -> wasmtime::Result<()> {
        let context_res = self.get(&context)?;
        let native_versions: Vec<pumpkin_util::version::JavaMinecraftVersion> = versions
            .into_iter()
            .map(crate::player::from_wasm_java_version)
            .collect();
        context_res
            .plugin_manager
            .java_versions
            .register(&context_res.get_metadata().name, native_versions);
        Ok(())
    }

    fn register_bedrock_versions(
        &mut self,
        context: Resource<WitContext>,
        protocols: Vec<u32>,
    ) -> wasmtime::Result<()> {
        let context_res = self.get(&context)?;
        context_res
            .plugin_manager
            .bedrock_versions
            .register(&context_res.get_metadata().name, protocols);
        Ok(())
    }

    fn get_data_folder(&mut self, _context: Resource<WitContext>) -> wasmtime::Result<String> {
        Ok("data".to_string())
    }

    fn get_server(&mut self, context: Resource<WitContext>) -> wasmtime::Result<Resource<Server>> {
        self.add(self.get(&context)?.server.clone())
    }

    fn get_marketplace_metadata(
        &mut self,
        _context: Resource<WitContext>,
    ) -> wasmtime::Result<Option<MarketplaceMetadata>> {
        Ok(self
            .marketplace_metadata
            .clone()
            .map(|metadata| MarketplaceMetadata {
                marketplace_url: metadata.marketplace_url,
                plugin_id: metadata.plugin_id,
                plugin_name: metadata.plugin_name,
                version: metadata.version,
                dev_id: metadata.dev_id,
                dev_name: metadata.dev_name,
                is_paid: metadata.is_paid,
                user_id: metadata.user_id,
                license_key: metadata.license_key,
                issued_at: metadata.issued_at,
            }))
    }

    fn drop(&mut self, rep: Resource<WitContext>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}
