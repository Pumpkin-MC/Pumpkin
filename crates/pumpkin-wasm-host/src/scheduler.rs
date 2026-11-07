use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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

/// How long one tick waits for all its scheduled tasks together. One tick's worth of time.
const TASK_TICK_BUDGET: Duration = Duration::from_millis(50);

/// Runs the plugin tasks that are due on this tick.
///
/// All runs share one [`TASK_TICK_BUDGET`]. Tasks still due once it is used up are deferred to
/// the next tick rather than started without waiting, so they never overlap the world tick. They
/// keep their due tick, so they run first next time.
pub fn tick(scheduler: &TaskScheduler, server: &Arc<Server>) {
    let current_tick = server.tick_count.load(Ordering::Relaxed) as u64;
    let deadline = Instant::now() + TASK_TICK_BUDGET;
    for task in scheduler.take_due_tasks(current_tick) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            scheduler.defer(task);
            continue;
        }
        if task.running.load(Ordering::Acquire) {
            tracing::debug!(
                handler_id = task.handler_id,
                "Skipping Wasm scheduled task run, the previous run is still in flight"
            );
        } else {
            run_task(server, &task, remaining);
        }
        scheduler.reschedule(task, current_tick);
    }
}

/// Clears a task's `running` flag when its run ends, also if the run panics or is dropped.
struct RunningGuard(Arc<AtomicBool>);

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Runs one scheduled task and waits for it inside the server tick, before the worlds tick.
///
/// Spawned without waiting, the task ran concurrently with the world tick: its host calls waited
/// on world locks while the tick waited in a blocking event for the same plugin instance, which
/// froze the server. Waiting here keeps a task and the plugin's blocking events apart.
///
/// The wait is bounded by `budget`, what is left of [`TASK_TICK_BUDGET`] on this tick, so a task stuck in a host call (for example WASI
/// I/O) cannot stall the tick or shutdown. Wasmtime offers no way to interrupt a guest that is
/// parked in a host future, so a run that overshoots its budget keeps going on its own Tokio task,
/// as before this change, and `running` stays set until it ends.
fn run_task(server: &Arc<Server>, task: &ScheduledTask, budget: Duration) {
    let plugin = task.plugin.clone();
    let handler_id = task.handler_id;
    let server_clone = server.clone();
    task.running.store(true, Ordering::Release);
    let running = RunningGuard(task.running.clone());

    let fut = async move {
        let _running = running;
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
    let wait = async { tokio::time::timeout(budget, &mut handle).await };
    let waited = if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| server.runtime.block_on(wait))
    } else {
        server.runtime.block_on(wait)
    };
    if waited.is_err() {
        tracing::warn!(
            handler_id,
            budget_ms = budget.as_millis(),
            "Wasm scheduled task overran its tick budget, letting it finish in the background"
        );
    }
}
