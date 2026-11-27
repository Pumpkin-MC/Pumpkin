//! `GameTest` callback registration for the async v0.2 world
//!
//! The `GameTest` callbacks are the one place where the WIT mixes sync and async exports on
//! purpose: the void, test and predicate callbacks are plain exports and only the async test body
//! is async, so their closures here are plain closures that cannot await
//!
//! Callbacks are handed to the host as ids, through the `GameTest` registration imports, which is
//! how a test run finds its way back into the tables behind these functions

use std::sync::Arc;

pub use crate::host::gametest::{
    AsyncTestCallbackId, BlockPermutation, BlockPredicateCallbackId, EntityPredicateCallbackId,
    Test, TestCallbackId, VoidCallbackId,
};
use crate::{
    host::world::Entity,
    registry::{LocalBoxFuture, Registry},
};

type VoidCallback = dyn Fn() + Send + Sync;
type TestCallback = dyn Fn(Test) + Send + Sync;
type AsyncTestCallback = dyn Fn(Test) -> LocalBoxFuture<'static, ()> + Send + Sync;
type BlockPredicate = dyn Fn(BlockPermutation) -> bool + Send + Sync;
type EntityPredicate = dyn Fn(&Entity) -> bool + Send + Sync;

static VOID_CALLBACKS: Registry<VoidCallback> = Registry::new();
static TEST_CALLBACKS: Registry<TestCallback> = Registry::new();
static ASYNC_TEST_CALLBACKS: Registry<AsyncTestCallback> = Registry::new();
static BLOCK_PREDICATES: Registry<BlockPredicate> = Registry::new();
static ENTITY_PREDICATES: Registry<EntityPredicate> = Registry::new();

/// Registers a synchronous callback that takes no arguments
///
/// The closure cannot await, because the export it backs is synchronous
///
/// # Examples
///
/// ```rust,ignore
/// let done = gametest::register_void_callback(|| { /* check the world */ });
/// ```
pub fn register_void_callback(callback: impl Fn() + Send + Sync + 'static) -> VoidCallbackId {
    VoidCallbackId {
        id: VOID_CALLBACKS.register(Arc::new(callback)),
    }
}

/// Registers a synchronous test body
///
/// If the test needs to wait for something, use [`register_async_test`] instead
pub fn register_test(callback: impl Fn(Test) + Send + Sync + 'static) -> TestCallbackId {
    TestCallbackId {
        id: TEST_CALLBACKS.register(Arc::new(callback)),
    }
}

/// Registers an async test body that can await host imports
///
/// # Examples
///
/// ```rust,ignore
/// let test_id = gametest::register_async_test(move |test| async move {
///     let _ = test.succeed_when(done);
/// });
/// ```
pub fn register_async_test<F, Fut>(callback: F) -> AsyncTestCallbackId
where
    F: Fn(Test) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + 'static,
{
    AsyncTestCallbackId {
        id: ASYNC_TEST_CALLBACKS.register(Arc::new(move |test| Box::pin(callback(test)))),
    }
}

/// Registers a synchronous block predicate
pub fn register_block_predicate(
    callback: impl Fn(BlockPermutation) -> bool + Send + Sync + 'static,
) -> BlockPredicateCallbackId {
    BlockPredicateCallbackId {
        id: BLOCK_PREDICATES.register(Arc::new(callback)),
    }
}

/// Registers a synchronous entity predicate
///
/// The predicate gets a borrowed entity because the export declares `borrow<entity>`; the borrow
/// ends when the predicate returns, so there is nothing to hold across an await
///
/// # Examples
///
/// ```rust,ignore
/// let grounded = gametest::register_entity_predicate(Entity::is_on_ground);
/// ```
pub fn register_entity_predicate(
    callback: impl Fn(&Entity) -> bool + Send + Sync + 'static,
) -> EntityPredicateCallbackId {
    EntityPredicateCallbackId {
        id: ENTITY_PREDICATES.register(Arc::new(callback)),
    }
}

// The `invoke_*` functions below are what the `register_plugin!` expansion forwards to

#[doc(hidden)]
pub fn invoke_void(callback: &VoidCallbackId) {
    if let Some(callback) = VOID_CALLBACKS.get(callback.id) {
        callback();
    }
}

#[doc(hidden)]
pub fn invoke_test(callback: &TestCallbackId, test: Test) {
    if let Some(callback) = TEST_CALLBACKS.get(callback.id) {
        callback(test);
    }
}

#[doc(hidden)]
pub async fn invoke_async_test(callback: &AsyncTestCallbackId, test: Test) {
    // The registry lock is released before the test body runs
    if let Some(callback) = ASYNC_TEST_CALLBACKS.get(callback.id) {
        callback(test).await;
    }
}

#[doc(hidden)]
pub fn invoke_block_predicate(
    callback: &BlockPredicateCallbackId,
    permutation: BlockPermutation,
) -> bool {
    BLOCK_PREDICATES
        .get(callback.id)
        .is_some_and(|predicate| predicate(permutation))
}

#[doc(hidden)]
pub fn invoke_entity_predicate(callback: &EntityPredicateCallbackId, entity: &Entity) -> bool {
    ENTITY_PREDICATES
        .get(callback.id)
        .is_some_and(|predicate| predicate(entity))
}
