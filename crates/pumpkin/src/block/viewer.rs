use std::{
    pin::Pin,
    sync::{Arc, atomic::Ordering},
};

use pumpkin_util::math::position::BlockPos;

use crate::{block::entities::BlockEntity, world::World};

pub use pumpkin_world::block::viewer::ViewerCountTracker;

pub trait ViewerCountTrackerExt {
    fn update_viewer_count<'a, T>(
        &'a self,
        entity: &'a T,
        world: &'a Arc<World>,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    where
        T: BlockEntity + ViewerCountListener + 'static;
}

impl ViewerCountTrackerExt for ViewerCountTracker {
    fn update_viewer_count<'a, T>(
        &'a self,
        entity: &'a T,
        world: &'a Arc<World>,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    where
        T: BlockEntity + ViewerCountListener + 'static,
    {
        Box::pin(async move {
            let current = self.current.load(Ordering::Relaxed);
            let old = self.old.swap(current, Ordering::Relaxed);
            if old != current {
                match (old, current) {
                    (n, 0) if n > 0 => {
                        entity.on_container_close(world, position).await;
                    }
                    (0, n) if n > 0 => {
                        entity.on_container_open(world, position).await;
                    }
                    _ => {} // Ignore
                }

                entity
                    .on_viewer_count_update(world, position, old, current)
                    .await;
            }
        })
    }
}

pub type ViewerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ViewerCountListener: Send + Sync {
    fn on_container_open<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_container_close<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_viewer_count_update<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _position: &'a BlockPos,
        _old: u16,
        _new: u16,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async {})
    }
}
