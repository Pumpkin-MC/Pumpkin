//! Native (`cdylib`) host runtime for WIT plugins.
//!
//! Pairs with [`pumpkin_native_wit_macro::bindgen`] to host native shared-library
//! guests built with `wit-bindgen` alongside WebAssembly components on the same
//! Wasmtime `Store` and `Host` trait implementations.

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::OnceLock;

pub use libloading;
pub use pumpkin_native_wit_macro::bindgen;
pub use wasmtime;

pub mod concurrent;
pub mod metadata;
pub use concurrent::{
    AccessorFuture, BoxedAccessorOp, ConcurrentBridge, ConcurrentBridgeGuard,
    ConcurrentImportGuard, ConcurrentStoreRunner, MAX_CONCURRENT_PLUGINS,
    concurrent_handle_is_borrow, concurrent_rep_to_borrow_handle, concurrent_rep_to_handle,
    enter_concurrent_import, enter_concurrent_import_for_plugin, install_concurrent_bridge,
    tagged_handle_plugin_id, tagged_handle_to_rep, with_concurrent_async, with_concurrent_sync,
};
pub use metadata::{
    detect_native_wit_world_version, has_native_wit_world_symbol, validate_world_metadata,
};

/// Exported by native WIT guest libraries to delegate to `wit_bindgen::rt::cabi_realloc`.
pub const CABI_REALLOC_SYMBOL: &[u8] = b"__pumpkin_native_wit_cabi_realloc\0";

/// Exported by native WIT guest libraries to delegate to `wit_bindgen::rt::register_import_resolver`.
pub const SET_IMPORT_RESOLVER_SYMBOL: &[u8] = b"__pumpkin_native_wit_set_import_resolver\0";

/// The guest's `cabi_realloc` entry point.
pub type CabiRealloc =
    unsafe extern "C" fn(old_ptr: *mut u8, old_len: usize, align: usize, new_len: usize) -> *mut u8;

/// Host callback supplied to the guest to resolve `(module, name)` UTF-8 import pairs.
pub type ImportResolverFn = unsafe extern "C" fn(
    ctx: *mut (),
    module_ptr: *const u8,
    module_len: usize,
    name_ptr: *const u8,
    name_len: usize,
) -> *mut ();

/// Guest entry point that installs a host [`ImportResolverFn`].
pub type SetImportResolverFn = unsafe extern "C" fn(resolver: ImportResolverFn, ctx: *mut ());

/// Guest entry point emitted by `wit-bindgen` returning `wit_component::metadata::encode` bytes.
pub type WorldMetadataFn = unsafe extern "C" fn(len: *mut usize) -> *const u8;

#[doc(hidden)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "invoked directly with the boxed panic payload returned by std::panic::catch_unwind"
)]
pub fn ffi_panic_guard(payload: Box<dyn Any + Send>) -> ! {
    let msg = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_string());
    tracing::error!(panic = %msg, "fatal: native WIT plugin host call panicked");
    std::process::abort()
}

thread_local! {
    static ALLOCATOR: Cell<Option<CabiRealloc>> = const { Cell::new(None) };
    static ACTIVE_STORE: Cell<Option<(*mut (), TypeId)>> = const { Cell::new(None) };
}

/// Restores the thread-local guest allocator on drop.
#[doc(hidden)]
pub struct AllocatorGuard {
    prev: Option<CabiRealloc>,
    _not_send: PhantomData<*const ()>,
}

impl Drop for AllocatorGuard {
    fn drop(&mut self) {
        ALLOCATOR.set(self.prev);
    }
}

/// Installs the guest plugin's `cabi_realloc` on the current thread until the guard drops.
#[doc(hidden)]
#[must_use]
pub fn install_allocator(realloc: CabiRealloc) -> AllocatorGuard {
    AllocatorGuard {
        prev: ALLOCATOR.replace(Some(realloc)),
        _not_send: PhantomData,
    }
}

/// Allocates zeroed memory in the currently active native guest's allocator.
///
/// # Safety
/// Must be called while a valid guest [`CabiRealloc`] is installed on the current thread.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "invoked only inside generated FFI lowering paths where a missing allocator or OOM is unrecoverable"
)]
pub unsafe fn abi_alloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::without_provenance_mut(align);
    }
    let realloc = ALLOCATOR.get().expect(
        "a value was lowered for a native WIT plugin with no plugin allocator installed on this \
         thread; calls into a native plugin must happen through a generated `call_*` method",
    );
    // SAFETY: `realloc` was resolved from the loaded guest library's `cabi_realloc` export and
    // is called with the Canonical ABI fresh-allocation contract `(null, 0, align, size)`.
    let ptr = unsafe { realloc(std::ptr::null_mut(), 0, align, size) };
    assert!(
        !ptr.is_null(),
        "the native WIT plugin's cabi_realloc failed to allocate {size} bytes (align {align})"
    );
    // SAFETY: `realloc` returned a valid, writable buffer of `size` bytes.
    unsafe { std::ptr::write_bytes(ptr, 0, size) };
    ptr
}

/// Allocates memory in the active native guest's allocator and copies `bytes` into it.
///
/// # Safety
/// Must be called while a valid guest [`CabiRealloc`] is installed on the current thread.
#[doc(hidden)]
#[must_use]
pub unsafe fn abi_alloc_copy(bytes: &[u8], align: usize) -> *mut u8 {
    if bytes.is_empty() {
        return std::ptr::without_provenance_mut(align);
    }
    // SAFETY: Caller guarantees an active guest allocator is installed on this thread.
    let ptr = unsafe { abi_alloc(bytes.len(), align) };
    // SAFETY: `ptr` is freshly allocated for `bytes.len()` bytes and does not overlap `bytes`.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
    ptr
}

/// Restores the thread-local active Wasmtime store pointer on drop.
#[doc(hidden)]
pub struct StoreGuard(Option<(*mut (), TypeId)>);

impl Drop for StoreGuard {
    fn drop(&mut self) {
        ACTIVE_STORE.set(self.0);
    }
}

/// Installs `store` on the current thread for the duration of a synchronous native guest export.
///
/// # Safety
/// The returned [`StoreGuard`] must drop before `store` goes out of scope.
#[doc(hidden)]
#[must_use]
pub unsafe fn install_store<T: 'static>(
    store: &mut wasmtime::StoreContextMut<'_, T>,
) -> StoreGuard {
    let entry = (std::ptr::from_mut(store).cast::<()>(), TypeId::of::<T>());
    StoreGuard(ACTIVE_STORE.replace(Some(entry)))
}

/// Temporarily takes the active Wasmtime store from thread-local storage for a host import call.
///
/// Replacing the slot with `None` while `f` executes prevents overlapping `&mut StoreContextMut`
/// borrows when a host import re-enters the guest through `StoreExecutor` (`pump_blocking` /
/// `pump_reentry`).
///
/// # Safety
/// Must be called from a native import callback invoked synchronously inside [`install_store`]
/// for the same store data type `T`.
#[doc(hidden)]
#[expect(
    clippy::panic,
    reason = "calling a native WIT import without an installed store or with a mismatched store type is an unrecoverable ABI violation"
)]
pub unsafe fn with_active_store<T: 'static, R>(
    import_name: &str,
    f: impl FnOnce(&mut wasmtime::StoreContextMut<'_, T>) -> R,
) -> R {
    let entry = ACTIVE_STORE.replace(None).unwrap_or_else(|| {
        panic!(
            "native WIT plugin called import `{import_name}` on a thread with no active store; \
             host imports may only be called during an active plugin export"
        )
    });
    let _restore = StoreGuard(Some(entry));
    assert_eq!(
        entry.1,
        TypeId::of::<T>(),
        "native WIT import `{import_name}` store data type mismatch"
    );
    // SAFETY: `entry.0` came from `install_store::<T>` on an enclosing stack frame of this
    // thread, `TypeId` matches `T`, and `ACTIVE_STORE` was cleared above so no other mutable
    // reference to this `StoreContextMut` exists during `f`.
    let store = unsafe { &mut *entry.0.cast::<wasmtime::StoreContextMut<'_, T>>() };
    f(store)
}

/// Bit set on sequential native wire handles passed to a guest as `borrow<T>`.
///
/// `wit-bindgen` wraps borrowed imported handles in a temporary `Drop` value and calls
/// `[resource-drop]` when the export returns; the host ignores `[resource-drop]` for handles
/// carrying this bit so the host's owned `ResourceTable` entry stays alive until the host drops it.
const BORROW_HANDLE_BIT: u32 = 1 << 31;

/// Bitmask for the 31-bit 1-indexed resource rep stored inside a sequential native handle.
const SEQ_REP_MASK: u32 = !BORROW_HANDLE_BIT;

/// Maximum 0-indexed `ResourceTable` rep representable in a sequential native handle.
const MAX_SEQ_REP: u32 = SEQ_REP_MASK - 1;

/// Returns whether a sequential wire handle was created for a `borrow<T>` parameter.
#[doc(hidden)]
#[must_use]
pub const fn handle_is_borrow(handle: u32) -> bool {
    (handle & BORROW_HANDLE_BIT) != 0
}

/// Converts a 1-indexed wire resource handle into a 0-indexed `ResourceTable` rep.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "handle 0 is explicitly invalid in the Component Model Canonical ABI"
)]
pub const fn handle_to_rep(handle: u32) -> u32 {
    (handle & SEQ_REP_MASK)
        .checked_sub(1)
        .expect("native WIT plugin passed invalid resource handle 0")
}

/// Converts a 0-indexed `ResourceTable` rep into a 1-indexed owned wire resource handle.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "reps exceeding MAX_SEQ_REP are unrecoverable"
)]
pub fn rep_to_handle(rep: u32) -> u32 {
    assert!(
        rep <= MAX_SEQ_REP,
        "resource table rep {rep} exceeds 31-bit sequential native handle limit"
    );
    rep.checked_add(1).expect("rep within MAX_SEQ_REP")
}

/// Converts a 0-indexed `ResourceTable` rep into a 1-indexed borrowed wire resource handle.
#[doc(hidden)]
#[must_use]
pub fn rep_to_borrow_handle(rep: u32) -> u32 {
    BORROW_HANDLE_BIT | rep_to_handle(rep)
}

/// Runs a synchronous native guest export without starving a multi-threaded Tokio worker.
#[doc(hidden)]
pub fn run_export<R>(f: impl FnOnce() -> R) -> R {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(f)
        }
        _ => f(),
    }
}

/// Drives an async Wasmtime `Host` trait future from a synchronous native import callback.
#[doc(hidden)]
#[expect(
    clippy::panic,
    reason = "blocking a current-thread Tokio reactor would deadlock any async Host future"
)]
pub fn block_on<F: Future>(fut: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(fut))
        }
        Ok(_) => panic!(
            "pumpkin-native-wit cannot run async Host callbacks on a current-thread Tokio \
             runtime; use the multi-threaded runtime flavor"
        ),
        Err(_) => fallback_runtime().block_on(fut),
    }
}

#[expect(
    clippy::expect_used,
    reason = "fallback Tokio runtime initialization failure is unrecoverable"
)]
fn fallback_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to start fallback Tokio runtime for pumpkin-native-wit")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        abi_alloc_copy, handle_is_borrow, handle_to_rep, install_allocator, rep_to_borrow_handle,
        rep_to_handle,
    };
    use std::alloc::{Layout, alloc, dealloc, realloc};

    unsafe extern "C" fn test_cabi_realloc(
        old_ptr: *mut u8,
        old_len: usize,
        align: usize,
        new_len: usize,
    ) -> *mut u8 {
        if old_len == 0 {
            if new_len == 0 {
                return std::ptr::without_provenance_mut(align);
            }
            // SAFETY: Test allocator mirrors `wit_bindgen::rt::cabi_realloc`.
            let layout = unsafe { Layout::from_size_align_unchecked(new_len, align) };
            // SAFETY: `new_len > 0` and `align` is a power of two.
            unsafe { alloc(layout) }
        } else {
            // SAFETY: Test allocator mirrors `wit_bindgen::rt::cabi_realloc`.
            let layout = unsafe { Layout::from_size_align_unchecked(old_len, align) };
            // SAFETY: `old_ptr` was allocated with `layout`.
            unsafe { realloc(old_ptr, layout, new_len) }
        }
    }

    #[test]
    fn handles_round_trip_without_zero() {
        assert_eq!(rep_to_handle(0), 1);
        assert_eq!(handle_to_rep(1), 0);
        assert!(!handle_is_borrow(rep_to_handle(41)));
        assert_eq!(handle_to_rep(rep_to_handle(41)), 41);

        let borrowed = rep_to_borrow_handle(41);
        assert!(handle_is_borrow(borrowed));
        assert_eq!(handle_to_rep(borrowed), 41);
    }

    #[test]
    #[should_panic(expected = "invalid resource handle 0")]
    fn handle_zero_is_rejected() {
        let _ = handle_to_rep(0);
    }

    #[test]
    fn abi_alloc_copy_uses_guest_allocator() {
        let _guard = install_allocator(test_cabi_realloc);
        let payload = b"hello native wit";
        // SAFETY: Test allocator is installed above.
        let ptr = unsafe { abi_alloc_copy(payload, 1) };
        // SAFETY: `ptr` points to `payload.len()` initialized bytes from `test_cabi_realloc`.
        let slice = unsafe { std::slice::from_raw_parts(ptr, payload.len()) };
        assert_eq!(slice, payload);
        // SAFETY: Allocated via `test_cabi_realloc` with `(payload.len(), 1)`.
        unsafe { dealloc(ptr, Layout::from_size_align_unchecked(payload.len(), 1)) };
    }
}
