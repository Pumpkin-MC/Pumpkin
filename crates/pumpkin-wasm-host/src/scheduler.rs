use std::{
    collections::HashSet,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use pumpkin_core::server::Server;
use pumpkin_wasm_host_common::{
    plugin::PluginApiVersion,
    scheduler::{ScheduledTask, TaskScheduler},
};

use crate::runtime::guest_exports;
#[cfg(not(feature = "v0_2"))]
use crate::runtime::v0_2_disabled;

/// How long one tick spends starting scheduled tasks: one tick's worth of time. Tasks still due
/// once it is used up stay queued for the next tick.
const TASK_TICK_BUDGET: Duration = Duration::from_millis(50);

/// After how long a running task gets a warning in the log. The tick keeps waiting for it.
const SLOW_TASK_WARNING: Duration = TASK_TICK_BUDGET;

/// Runs the plugin tasks that are due on this tick.
///
/// Tasks are taken from the queue one at a time, right before each runs, so a task cancelled or a
/// plugin disabled by an earlier task of this tick is not run. Once [`TASK_TICK_BUDGET`] is used
/// up, no further task is started; the rest stay queued with their due tick, so they run first
/// next tick. Tasks scheduled during this tick, and repeating tasks that are due again right away,
/// wait for the next tick, as before.
pub fn tick(scheduler: &TaskScheduler, server: &Arc<Server>) {
    let current_tick = server.tick_count.load(Ordering::Relaxed) as u64;
    let deadline = Instant::now() + TASK_TICK_BUDGET;
    let first_new_id = scheduler.next_task_id();
    let mut ran = HashSet::new();
    let mut for_next_tick = Vec::new();
    while Instant::now() < deadline {
        let Some(task) = scheduler.pop_due_task(current_tick) else {
            break;
        };
        if task.id >= first_new_id || !ran.insert(task.id) {
            for_next_tick.push(task);
            continue;
        }
        run_task(server, &task);
        scheduler.reschedule(task, current_tick);
    }
    for task in for_next_tick {
        scheduler.defer(task);
    }
}

/// Runs one scheduled task and waits for it inside the server tick, before the worlds tick.
///
/// Spawned without waiting, the task ran concurrently with the world tick: its host calls waited
/// on world locks while the tick waited in a blocking event for the same plugin instance, which
/// froze the server. Waiting here keeps a task and the plugin's blocking events apart.
///
/// The wait is not bounded. The guest call runs on the plugin's store driver and cannot be
/// interrupted while the guest is parked in a host future, so a tick that stopped waiting would
/// tick the worlds beside the still running task and bring the deadlock back. Like a Bukkit sync
/// task, a slow task delays the tick; it is logged once it passes [`SLOW_TASK_WARNING`].
fn run_task(server: &Arc<Server>, task: &ScheduledTask) {
    let plugin = task.plugin.clone();
    let handler_id = task.handler_id;
    let server_clone = server.clone();

    let fut = async move {
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
                        #[cfg(feature = "v0_2")]
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
                        #[cfg(not(feature = "v0_2"))]
                        PluginApiVersion::V0_2 => Err(v0_2_disabled()),
                    }
                })
            })
            .await
        {
            tracing::error!(handler_id, %error, "Wasm scheduled task failed");
        }
    };

    let mut handle = server.spawn_task(fut);
    let wait = async {
        let started = Instant::now();
        if tokio::time::timeout(SLOW_TASK_WARNING, &mut handle)
            .await
            .is_err()
        {
            tracing::warn!(
                handler_id,
                "Wasm scheduled task is taking longer than {} ms, the tick waits for it",
                SLOW_TASK_WARNING.as_millis()
            );
            let _ = handle.await;
            tracing::warn!(
                handler_id,
                elapsed_ms = started.elapsed().as_millis(),
                "Slow Wasm scheduled task finished"
            );
        }
    };
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| server.runtime.block_on(wait));
    } else {
        server.runtime.block_on(wait);
    }
}
