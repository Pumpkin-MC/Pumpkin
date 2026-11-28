//! Concurrent (`StoreMode::Concurrent`) runtime bridge for native WIT v0.2 plugins.
//!
//! Unlike sequential v0.1 plugins, concurrent v0.2 plugins execute synchronous native exports on
//! Tokio's blocking pool while the Wasmtime `Store` stays inside `Store::run_concurrent`. Host
//! imports from both export threads and arbitrary plugin background threads (such as JVM threads)
//! dispatch onto the `Store`'s `Accessor` event loop through [`ConcurrentStoreRunner`].

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::{AllocatorGuard, CabiRealloc, install_allocator};

/// Maximum number of simultaneously loaded concurrent native WIT plugins.
///
/// Matches the compile-time per-plugin trampoline table generated for resource-free imports.
pub const MAX_CONCURRENT_PLUGINS: usize = 64;

/// Boxed future executed against a Wasmtime concurrent [`Accessor`](wasmtime::component::Accessor).
pub type AccessorFuture<'a, R> = Pin<Box<dyn Future<Output = wasmtime::Result<R>> + Send + 'a>>;

/// Type-erased callback run on the store's concurrent [`Accessor`](wasmtime::component::Accessor).
pub type BoxedAccessorOp<T> = Box<
    dyn for<'a> FnOnce(
            &'a wasmtime::component::Accessor<T, wasmtime::component::HasSelf<T>>,
        ) -> AccessorFuture<'a, Box<dyn Any + Send>>
        + Send,
>;

/// Submits host operations from any OS thread onto a Wasmtime concurrent `Store`'s `Accessor` loop.
pub trait ConcurrentStoreRunner<T: Send + 'static>: Send + Sync + 'static {
    /// Executes `op` on the store's concurrent `Accessor` and blocks the calling thread until it
    /// completes.
    fn run_with_accessor(&self, op: BoxedAccessorOp<T>) -> wasmtime::Result<Box<dyn Any + Send>>;
}

/// Bit set on concurrent native wire handles passed to a guest as `borrow<T>`.
///
/// Matches the Canonical ABI rule that releasing a guest's borrow handle at export exit must not
/// drop the host's owned `ResourceTable` entry.
const CONCURRENT_BORROW_BIT: u32 = 1 << 23;

/// Bitmask for the 23-bit 1-indexed resource rep stored inside a tagged native handle.
const REP_MASK: u32 = 0x007F_FFFF;

/// Maximum 0-indexed `ResourceTable` rep representable in a tagged native handle.
const MAX_TAGGED_REP: u32 = REP_MASK - 1;

struct RegistryEntry {
    type_id: TypeId,
    bridge: Weak<dyn Any + Send + Sync>,
}

struct BridgeRegistry {
    slots: Vec<Option<RegistryEntry>>,
}

impl BridgeRegistry {
    const fn new() -> Self {
        Self { slots: Vec::new() }
    }

    fn allocate(
        &mut self,
        type_id: TypeId,
        bridge: Weak<dyn Any + Send + Sync>,
    ) -> wasmtime::Result<u8> {
        // Slot 0 is reserved as "no plugin tag"; valid IDs are 1..=MAX_CONCURRENT_PLUGINS.
        // Only reuse `None` slots so a newly allocated bridge cannot be cleared by an older
        // bridge's `Drop` impl after its strong count reached zero.
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(RegistryEntry { type_id, bridge });
                return Ok((idx + 1) as u8);
            }
        }
        if self.slots.len() >= MAX_CONCURRENT_PLUGINS {
            return Err(wasmtime::Error::msg(format!(
                "exceeded maximum of {MAX_CONCURRENT_PLUGINS} simultaneously loaded concurrent native WIT plugins"
            )));
        }
        self.slots.push(Some(RegistryEntry { type_id, bridge }));
        Ok(self.slots.len() as u8)
    }

    fn remove(&mut self, plugin_id: u8) {
        if let Some(idx) = usize::from(plugin_id).checked_sub(1)
            && let Some(slot) = self.slots.get_mut(idx)
        {
            *slot = None;
        }
    }

    fn get(&self, plugin_id: u8, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        let idx = usize::from(plugin_id).checked_sub(1)?;
        let entry = self.slots.get(idx)?.as_ref()?;
        (entry.type_id == type_id)
            .then(|| entry.bridge.upgrade())
            .flatten()
    }

    fn fallback(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        self.slots.iter().rev().flatten().find_map(|entry| {
            (entry.type_id == type_id)
                .then(|| entry.bridge.upgrade())
                .flatten()
        })
    }
}

static BRIDGE_REGISTRY: Mutex<BridgeRegistry> = Mutex::new(BridgeRegistry::new());

/// Per-plugin concurrent dispatch state shared between generated exports and import callbacks.
pub struct ConcurrentBridge<T: Send + 'static> {
    plugin_id: u8,
    cabi_realloc: CabiRealloc,
    runner: OnceLock<Arc<dyn ConcurrentStoreRunner<T>>>,
}

impl<T: Send + 'static> ConcurrentBridge<T> {
    /// Registers a new concurrent native WIT plugin instance and assigns it a unique handle tag.
    pub fn new(cabi_realloc: CabiRealloc) -> wasmtime::Result<Arc<Self>> {
        let mut reg = BRIDGE_REGISTRY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut alloc_result: Option<wasmtime::Result<u8>> = None;
        let bridge = Arc::new_cyclic(|weak: &Weak<Self>| {
            let weak_any: Weak<dyn Any + Send + Sync> = weak.clone();
            let res = reg.allocate(TypeId::of::<T>(), weak_any);
            let plugin_id = *res.as_ref().unwrap_or(&0);
            alloc_result = Some(res);
            Self {
                plugin_id,
                cabi_realloc,
                runner: OnceLock::new(),
            }
        });
        drop(reg);
        if let Some(Err(error)) = alloc_result {
            return Err(error);
        }
        Ok(bridge)
    }

    /// Attaches the `StoreExecutor`-backed runner once the plugin's concurrent store is started.
    pub fn attach_runner(&self, runner: Arc<dyn ConcurrentStoreRunner<T>>) {
        let _ = self.runner.set(runner);
    }

    #[must_use]
    pub const fn plugin_id(&self) -> u8 {
        self.plugin_id
    }

    #[must_use]
    pub const fn cabi_realloc(&self) -> CabiRealloc {
        self.cabi_realloc
    }

    #[expect(
        clippy::panic,
        reason = "calling a concurrent import before attaching the store runner is an unrecoverable host lifecycle bug"
    )]
    fn runner(&self, import_name: &str) -> &Arc<dyn ConcurrentStoreRunner<T>> {
        self.runner.get().unwrap_or_else(|| {
            panic!(
                "native WIT plugin called concurrent import `{import_name}` before its store runner was attached"
            )
        })
    }

    /// Executes a synchronous host trait method on the store's `Accessor`.
    #[expect(
        clippy::expect_used,
        reason = "the boxed return type is fixed to `R` by this generic method"
    )]
    pub fn run_sync<R: Send + 'static>(
        &self,
        import_name: &str,
        f: impl FnOnce(&mut T) -> R + Send + 'static,
    ) -> wasmtime::Result<R> {
        let boxed = self
            .runner(import_name)
            .run_with_accessor(Box::new(move |accessor| {
                Box::pin(async move {
                    let out: R = accessor.with(|mut host| f(host.get()));
                    Ok(Box::new(out) as Box<dyn Any + Send>)
                })
            }))?;
        Ok(*boxed
            .downcast::<R>()
            .expect("concurrent sync import return type mismatch"))
    }

    /// Executes an async host trait method on the store's `Accessor`.
    #[expect(
        clippy::expect_used,
        reason = "the boxed return type is fixed to `R` by this generic method"
    )]
    pub fn run_async<R: Send + 'static>(
        &self,
        import_name: &str,
        f: impl for<'a> FnOnce(
            &'a wasmtime::component::Accessor<T, wasmtime::component::HasSelf<T>>,
        ) -> AccessorFuture<'a, R>
        + Send
        + 'static,
    ) -> wasmtime::Result<R> {
        let boxed = self
            .runner(import_name)
            .run_with_accessor(Box::new(move |accessor| {
                Box::pin(async move {
                    let out: R = f(accessor).await?;
                    Ok(Box::new(out) as Box<dyn Any + Send>)
                })
            }))?;
        Ok(*boxed
            .downcast::<R>()
            .expect("concurrent async import return type mismatch"))
    }
}

impl<T: Send + 'static> Drop for ConcurrentBridge<T> {
    fn drop(&mut self) {
        let mut reg = BRIDGE_REGISTRY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        reg.remove(self.plugin_id);
    }
}

struct ActiveConcurrentEntry {
    type_id: TypeId,
    plugin_id: u8,
    bridge: Arc<dyn Any + Send + Sync>,
}

thread_local! {
    static ACTIVE_CONCURRENT: RefCell<Option<ActiveConcurrentEntry>> = const { RefCell::new(None) };
}

/// Restores the thread-local active concurrent bridge on drop.
#[doc(hidden)]
pub struct ConcurrentBridgeGuard {
    prev: Option<ActiveConcurrentEntry>,
    _not_send: PhantomData<*const ()>,
}

impl Drop for ConcurrentBridgeGuard {
    fn drop(&mut self) {
        ACTIVE_CONCURRENT.with(|slot| {
            *slot.borrow_mut() = self.prev.take();
        });
    }
}

/// Installs `bridge` on the current thread for the duration of a concurrent native export or
/// import call.
#[doc(hidden)]
#[must_use]
pub fn install_concurrent_bridge<T: Send + 'static>(
    bridge: Arc<ConcurrentBridge<T>>,
) -> ConcurrentBridgeGuard {
    let entry = ActiveConcurrentEntry {
        type_id: TypeId::of::<T>(),
        plugin_id: bridge.plugin_id,
        bridge,
    };
    let prev = ACTIVE_CONCURRENT.with(|slot| slot.borrow_mut().replace(entry));
    ConcurrentBridgeGuard {
        prev,
        _not_send: PhantomData,
    }
}

/// Restores both the thread-local concurrent bridge and guest allocator on drop.
#[doc(hidden)]
pub struct ConcurrentImportGuard {
    _alloc: AllocatorGuard,
    _bridge: ConcurrentBridgeGuard,
}

#[expect(
    clippy::panic,
    reason = "invoking a native WIT import with no matching loaded concurrent plugin is an unrecoverable ABI violation"
)]
fn resolve_concurrent_bridge<T: Send + 'static>(
    hint_handle: Option<u32>,
    import_name: &str,
) -> Arc<ConcurrentBridge<T>> {
    let target_type = TypeId::of::<T>();

    // Fast path: already inside a native export or enclosing import on this thread.
    let from_tls = ACTIVE_CONCURRENT.with(|slot| {
        let guard = slot.borrow();
        let entry = guard.as_ref()?;
        (entry.type_id == target_type)
            .then(|| Arc::clone(&entry.bridge))
            .and_then(|arc| arc.downcast::<ConcurrentBridge<T>>().ok())
    });
    if let Some(bridge) = from_tls {
        return bridge;
    }

    // Out-of-band thread path (e.g. JVM / background worker thread): route by tagged handle first,
    // then fall back to the active concurrent plugin for this host state type.
    let reg = BRIDGE_REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(handle) = hint_handle {
        let plugin_id = tagged_handle_plugin_id(handle);
        if plugin_id != 0
            && let Some(bridge) = reg
                .get(plugin_id, target_type)
                .and_then(|arc| arc.downcast::<ConcurrentBridge<T>>().ok())
        {
            return bridge;
        }
    }
    if let Some(bridge) = reg
        .fallback(target_type)
        .and_then(|arc| arc.downcast::<ConcurrentBridge<T>>().ok())
    {
        return bridge;
    }

    panic!(
        "native WIT plugin called concurrent import `{import_name}` with no registered concurrent plugin instance"
    );
}

/// Enters a concurrent native WIT import callback from any thread, ensuring both the owning
/// plugin's `ConcurrentBridge` and `cabi_realloc` are active on this thread for lifting, host
/// dispatch, and return lowering.
#[doc(hidden)]
#[must_use]
pub fn enter_concurrent_import<T: Send + 'static>(
    hint_handle: Option<u32>,
    import_name: &str,
) -> ConcurrentImportGuard {
    let bridge = resolve_concurrent_bridge::<T>(hint_handle, import_name);
    let alloc = install_allocator(bridge.cabi_realloc);
    let bridge_guard = install_concurrent_bridge(bridge);
    ConcurrentImportGuard {
        _alloc: alloc,
        _bridge: bridge_guard,
    }
}

/// Enters a resource-free concurrent native WIT import callback for the explicitly identified
/// plugin slot, avoiding cross-plugin fallback routing on out-of-band threads.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::panic,
    reason = "invoking a resource-free native WIT import for an unloaded plugin ID is an unrecoverable ABI violation"
)]
pub fn enter_concurrent_import_for_plugin<T: Send + 'static>(
    plugin_id: u8,
    import_name: &str,
) -> ConcurrentImportGuard {
    let target_type = TypeId::of::<T>();
    let bridge = ACTIVE_CONCURRENT
        .with(|slot| {
            let guard = slot.borrow();
            let entry = guard.as_ref()?;
            (entry.plugin_id == plugin_id && entry.type_id == target_type)
                .then(|| Arc::clone(&entry.bridge))
                .and_then(|arc| arc.downcast::<ConcurrentBridge<T>>().ok())
        })
        .or_else(|| {
            let reg = BRIDGE_REGISTRY
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            reg.get(plugin_id, target_type)
                .and_then(|arc| arc.downcast::<ConcurrentBridge<T>>().ok())
        })
        .unwrap_or_else(|| {
            panic!(
                "native WIT plugin slot {plugin_id} called concurrent import `{import_name}` \
                 after the plugin was unloaded or with a mismatched store type"
            )
        });
    let alloc = install_allocator(bridge.cabi_realloc);
    let bridge_guard = install_concurrent_bridge(bridge);
    ConcurrentImportGuard {
        _alloc: alloc,
        _bridge: bridge_guard,
    }
}

/// Runs a synchronous host trait method for the currently active concurrent plugin.
#[doc(hidden)]
pub fn with_concurrent_sync<T: Send + 'static, R: Send + 'static>(
    import_name: &str,
    f: impl FnOnce(&mut T) -> R + Send + 'static,
) -> wasmtime::Result<R> {
    let bridge = resolve_concurrent_bridge::<T>(None, import_name);
    bridge.run_sync(import_name, f)
}

/// Runs an async host trait method for the currently active concurrent plugin.
#[doc(hidden)]
pub fn with_concurrent_async<T: Send + 'static, R: Send + 'static>(
    import_name: &str,
    f: impl for<'a> FnOnce(
        &'a wasmtime::component::Accessor<T, wasmtime::component::HasSelf<T>>,
    ) -> AccessorFuture<'a, R>
    + Send
    + 'static,
) -> wasmtime::Result<R> {
    let bridge = resolve_concurrent_bridge::<T>(None, import_name);
    bridge.run_async(import_name, f)
}

/// Extracts the plugin tag embedded in the top byte of a concurrent native wire handle.
#[doc(hidden)]
#[must_use]
pub const fn tagged_handle_plugin_id(handle: u32) -> u8 {
    (handle >> 24) as u8
}

/// Returns whether a tagged concurrent wire handle was created for a `borrow<T>` parameter.
#[doc(hidden)]
#[must_use]
pub const fn concurrent_handle_is_borrow(handle: u32) -> bool {
    (handle & CONCURRENT_BORROW_BIT) != 0
}

/// Converts a tagged concurrent wire handle into its 0-indexed `ResourceTable` rep.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "handle 0 is explicitly invalid in the Component Model Canonical ABI"
)]
pub const fn tagged_handle_to_rep(handle: u32) -> u32 {
    (handle & REP_MASK)
        .checked_sub(1)
        .expect("native WIT plugin passed invalid concurrent resource handle 0")
}

/// Converts a 0-indexed `ResourceTable` rep into a 1-indexed owned wire handle tagged with the
/// currently active concurrent plugin's ID.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "exceeding 8M simultaneous live handles per plugin is unrecoverable"
)]
pub fn concurrent_rep_to_handle(rep: u32) -> u32 {
    assert!(
        rep <= MAX_TAGGED_REP,
        "resource table rep {rep} exceeds 23-bit concurrent native handle limit"
    );
    let plugin_id =
        ACTIVE_CONCURRENT.with(|slot| slot.borrow().as_ref().map_or(0, |e| e.plugin_id));
    let low = rep.checked_add(1).expect("rep within MAX_TAGGED_REP");
    (u32::from(plugin_id) << 24) | low
}

/// Converts a 0-indexed `ResourceTable` rep into a 1-indexed borrowed wire handle tagged with the
/// currently active concurrent plugin's ID.
#[doc(hidden)]
#[must_use]
pub fn concurrent_rep_to_borrow_handle(rep: u32) -> u32 {
    CONCURRENT_BORROW_BIT | concurrent_rep_to_handle(rep)
}

#[cfg(test)]
mod tests {
    use super::{
        BridgeRegistry, ConcurrentBridge, MAX_CONCURRENT_PLUGINS, concurrent_handle_is_borrow,
        concurrent_rep_to_borrow_handle, concurrent_rep_to_handle,
        enter_concurrent_import_for_plugin, install_concurrent_bridge, tagged_handle_plugin_id,
        tagged_handle_to_rep,
    };
    use std::any::{Any, TypeId};
    use std::sync::{Arc, Weak};

    unsafe extern "C" fn dummy_realloc(
        _old_ptr: *mut u8,
        _old_len: usize,
        align: usize,
        _new_len: usize,
    ) -> *mut u8 {
        std::ptr::without_provenance_mut(align)
    }

    #[test]
    fn tagged_handles_round_trip_with_plugin_id() -> wasmtime::Result<()> {
        let bridge = ConcurrentBridge::<()>::new(dummy_realloc)?;
        let id = bridge.plugin_id();
        assert_ne!(id, 0);
        let _guard = install_concurrent_bridge(bridge);
        let handle = concurrent_rep_to_handle(41);
        assert!(!concurrent_handle_is_borrow(handle));
        assert_eq!(tagged_handle_plugin_id(handle), id);
        assert_eq!(tagged_handle_to_rep(handle), 41);

        let borrowed = concurrent_rep_to_borrow_handle(41);
        assert!(concurrent_handle_is_borrow(borrowed));
        assert_eq!(tagged_handle_plugin_id(borrowed), id);
        assert_eq!(tagged_handle_to_rep(borrowed), 41);
        Ok(())
    }

    #[test]
    fn registry_skips_expired_fallback_and_waits_for_slot_removal() -> wasmtime::Result<()> {
        struct Marker;
        let mut reg = BridgeRegistry::new();
        let a: Arc<dyn Any + Send + Sync> = Arc::new(1u32);
        let b: Arc<dyn Any + Send + Sync> = Arc::new(2u32);
        let c: Arc<dyn Any + Send + Sync> = Arc::new(3u32);

        let id_a = reg.allocate(TypeId::of::<Marker>(), Arc::downgrade(&a))?;
        let id_b = reg.allocate(TypeId::of::<Marker>(), Arc::downgrade(&b))?;
        drop(b);

        // Slot `id_b` is expired (strong_count == 0) but not yet removed by `Drop`, so allocating
        // `c` must not overwrite `id_b` and fallback lookup must skip `id_b` and find `a`.
        assert!(reg.fallback(TypeId::of::<Marker>()).is_some());
        let id_c = reg.allocate(TypeId::of::<Marker>(), Arc::downgrade(&c))?;
        assert_ne!(id_c, id_b);
        assert_ne!(id_c, id_a);

        // Once `Drop` removes `id_b`, the next allocation reuses that slot.
        reg.remove(id_b);
        let d: Arc<dyn Any + Send + Sync> = Arc::new(4u32);
        let id_d = reg.allocate(TypeId::of::<Marker>(), Arc::downgrade(&d))?;
        assert_eq!(id_d, id_b);

        // Exceeding MAX_CONCURRENT_PLUGINS returns an error instead of panicking.
        let mut live: Vec<Arc<dyn Any + Send + Sync>> = vec![a, c, d];
        while live.len() < MAX_CONCURRENT_PLUGINS {
            let item: Arc<dyn Any + Send + Sync> = Arc::new(0u8);
            let _ = reg.allocate(TypeId::of::<Marker>(), Arc::downgrade(&item))?;
            live.push(item);
        }
        let extra: Weak<dyn Any + Send + Sync> = Weak::<u8>::new();
        assert!(reg.allocate(TypeId::of::<Marker>(), extra).is_err());
        Ok(())
    }

    #[test]
    fn resource_free_import_routes_to_exact_plugin_slot() -> wasmtime::Result<()> {
        struct SharedHost;
        let a = ConcurrentBridge::<SharedHost>::new(dummy_realloc)?;
        let b = ConcurrentBridge::<SharedHost>::new(dummy_realloc)?;
        assert_ne!(a.plugin_id(), b.plugin_id());

        let _guard = enter_concurrent_import_for_plugin::<SharedHost>(a.plugin_id(), "test#free");
        let handle = concurrent_rep_to_handle(7);
        assert_eq!(tagged_handle_plugin_id(handle), a.plugin_id());
        assert_eq!(tagged_handle_to_rep(handle), 7);
        Ok(())
    }
}
