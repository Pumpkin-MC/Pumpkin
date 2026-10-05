use crate::pumpkin::plugin::scheduler;
use pumpkin_wasm_host_common::state::PluginHostState;
use std::sync::{Arc, atomic::Ordering};
use wasmtime::component::Accessor;
use wasmtime::component::HasSelf;

impl scheduler::Host for PluginHostState {}

impl scheduler::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn schedule_delayed_task(
        accessor: &Accessor<PluginHostState, Self>,
        handler_id: u32,
        delay: u64,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin not found"))?;
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not found"))?;
            let tick_count = server.tick_count.load(Ordering::Relaxed) as u64;
            let task_scheduler = Arc::clone(&plugin.task_scheduler);
            let task_id =
                task_scheduler.schedule_delayed_task(plugin, handler_id, delay, tick_count);
            Ok(task_id)
        })
    }

    async fn schedule_repeating_task(
        accessor: &Accessor<PluginHostState, Self>,
        handler_id: u32,
        delay: u64,
        period: u64,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin not found"))?;
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not found"))?;
            let tick_count = server.tick_count.load(Ordering::Relaxed) as u64;
            let task_scheduler = Arc::clone(&plugin.task_scheduler);
            let task_id = task_scheduler
                .schedule_repeating_task(plugin, handler_id, delay, period, tick_count);
            Ok(task_id)
        })
    }

    async fn cancel_task(
        accessor: &Accessor<PluginHostState, Self>,
        task_id: u32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin not found"))?;
            plugin.task_scheduler.cancel_task(task_id);
            Ok(())
        })
    }
}
