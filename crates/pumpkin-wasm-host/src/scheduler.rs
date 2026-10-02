use std::sync::{Arc, atomic::Ordering};

use pumpkin_core::server::Server;
use pumpkin_wasm_host_common::{
    plugin::PluginApiVersion,
    scheduler::{ScheduledTask, TaskScheduler},
};

use crate::runtime::guest_exports;

/// Runs the plugin tasks that are due on this tick.
pub fn tick(scheduler: &TaskScheduler, server: &Arc<Server>) {
    let current_tick = server.tick_count.load(Ordering::Relaxed) as u64;
    for task in scheduler.take_due_tasks(current_tick) {
        run_task(server, &task);
        scheduler.reschedule(task, current_tick);
    }
}

fn run_task(server: &Arc<Server>, task: &ScheduledTask) {
    let plugin = task.plugin.clone();
    let handler_id = task.handler_id;
    let server_clone = server.clone();

    server.spawn_task(async move {
        let plugin_instance = plugin.plugin_instance.clone();
        let api_version = plugin.api_version;
        if let Err(error) = plugin
            .store
            .call_guest(move |mut guest| {
                Box::pin(async move {
                    match api_version {
                        PluginApiVersion::V0_1 => {
                            let instance =
                                guest_exports::<pumpkin_wasm_host_v0_1::Plugin>(&plugin_instance)?;
                            let (server_resource, server_rep) = guest.with(|mut store| {
                                let resource = store.data_mut().add(server_clone)?;
                                let rep = resource.rep();
                                Ok::<_, wasmtime::Error>((resource, rep))
                            })?;
                            let result = guest
                                .call(instance.func_handle_task(), (handler_id, server_resource))
                                .await;
                            guest.with(|mut store| {
                                let _ = store.data_mut().resource_table.delete::<Arc<Server>>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                            });
                            result
                        }
                        PluginApiVersion::V0_2 => {
                            let instance =
                                guest_exports::<pumpkin_wasm_host_v0_2::Plugin>(&plugin_instance)?;
                            let (server_resource, server_rep) = guest.with(|mut store| {
                                let resource = store.data_mut().add(server_clone)?;
                                let rep = resource.rep();
                                Ok::<_, wasmtime::Error>((resource, rep))
                            })?;
                            let result = guest
                                .call(instance.func_handle_task(), (handler_id, server_resource))
                                .await;
                            guest.with(|mut store| {
                                let _ = store.data_mut().resource_table.delete::<Arc<Server>>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                            });
                            result
                        }
                    }
                })
            })
            .await
        {
            tracing::error!(handler_id, %error, "Wasm scheduled task failed");
        }
    });
}
