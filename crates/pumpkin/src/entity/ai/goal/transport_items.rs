use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, item_stack::ItemStack};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// Ports the item-fetching half of vanilla's `TransportItemsBetweenContainers` behavior
/// (`CopperGolemAi.initIdleActivity`): a copper golem walks to a nearby copper chest,
/// takes one item, then walks it to a nearby chest/trapped chest and drops it in.
///
/// Search radii (32 blocks horizontal, 8 vertical) match
/// `CopperGolemAi.TRANSPORT_ITEM_HORIZONTAL_SEARCH_RADIUS` / `_VERTICAL_SEARCH_RADIUS`.
///
/// Deferred vs. vanilla: the real behavior queues on contested containers via
/// `ContainerOpenersCounter`/`shouldQueueForTarget` and plays a five-state open/close
/// animation (`CopperGolemState`) timed to ticks 1/9/60 after arrival. Pumpkin has no
/// `ContainerUser` plumbing for mobs, so this goal performs the pickup/drop instantly on
/// arrival instead of over that animation window.
const HORIZONTAL_RADIUS: i32 = 32;
const VERTICAL_RADIUS: i32 = 8;
const CHUNK_RADIUS: i32 = (HORIZONTAL_RADIUS + 15) / 16;

enum Phase {
    Idle,
    ToSource(BlockPos),
    ToDest { source: BlockPos, dest: BlockPos },
}

pub struct TransportItemsGoal {
    goal_control: Controls,
    speed: f64,
    phase: Phase,
    carried: Option<ItemStack>,
    cooldown: i32,
}

impl TransportItemsGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            phase: Phase::Idle,
            carried: None,
            cooldown: 0,
        }
    }

    fn is_source_block(world: &World, pos: &BlockPos) -> bool {
        world
            .get_block(pos)
            .has_tag(&tag::Block::MINECRAFT_COPPER_CHESTS)
    }

    fn is_dest_block(world: &World, pos: &BlockPos) -> bool {
        let block = world.get_block(pos);
        block == &Block::CHEST || block == &Block::TRAPPED_CHEST
    }

    async fn find_source(world: &World, origin: BlockPos) -> Option<BlockPos> {
        Self::find_nearest(world, origin, Self::is_source_block, |be| async move {
            let Some(inventory) = be.get_inventory() else {
                return false;
            };
            for i in 0..inventory.size() {
                let stack = inventory.get_stack(i).await;
                if !stack.lock().await.is_empty() {
                    return true;
                }
            }
            false
        })
        .await
    }

    async fn find_dest(world: &World, origin: BlockPos) -> Option<BlockPos> {
        Self::find_nearest(world, origin, Self::is_dest_block, |be| async move {
            let Some(inventory) = be.get_inventory() else {
                return false;
            };
            for i in 0..inventory.size() {
                let stack = inventory.get_stack(i).await;
                if stack.lock().await.is_empty() {
                    return true;
                }
            }
            false
        })
        .await
    }

    async fn find_nearest<F, C, Fut>(
        world: &World,
        origin: BlockPos,
        block_matches: F,
        candidate_ok: C,
    ) -> Option<BlockPos>
    where
        F: Fn(&World, &BlockPos) -> bool,
        C: Fn(Arc<dyn crate::block::entities::BlockEntity>) -> Fut,
        Fut: Future<Output = bool>,
    {
        let origin_chunk = origin.chunk_position();
        let mut best: Option<(BlockPos, i32)> = None;

        for dx in -CHUNK_RADIUS..=CHUNK_RADIUS {
            for dz in -CHUNK_RADIUS..=CHUNK_RADIUS {
                let chunk = Vector2::new(origin_chunk.x + dx, origin_chunk.y + dz);
                let Some(entries) = world.block_entities.get(&chunk) else {
                    continue;
                };
                for (pos, block_entity) in entries.iter() {
                    let dist_h = (pos.0.x - origin.0.x)
                        .abs()
                        .max((pos.0.z - origin.0.z).abs());
                    let dist_v = (pos.0.y - origin.0.y).abs();
                    if dist_h > HORIZONTAL_RADIUS || dist_v > VERTICAL_RADIUS {
                        continue;
                    }
                    if !block_matches(world, pos) {
                        continue;
                    }
                    if !candidate_ok(block_entity.clone()).await {
                        continue;
                    }
                    let score = dist_h + dist_v;
                    if best.is_none_or(|(_, best_score)| score < best_score) {
                        best = Some((*pos, score));
                    }
                }
            }
        }

        best.map(|(pos, _)| pos)
    }

    async fn take_one_item(world: &World, pos: BlockPos) -> Option<ItemStack> {
        let block_entity = world.get_block_entity(&pos)?;
        let inventory = block_entity.get_inventory()?;
        for i in 0..inventory.size() {
            let slot = inventory.get_stack(i).await;
            let mut stack = slot.lock().await;
            if !stack.is_empty() {
                return Some(stack.split(1));
            }
        }
        None
    }

    async fn deposit_item(world: &World, pos: BlockPos, item: ItemStack) -> Option<ItemStack> {
        let block_entity = world.get_block_entity(&pos)?;
        let inventory = block_entity.get_inventory()?;
        let mut remaining = item;
        for i in 0..inventory.size() {
            let slot = inventory.get_stack(i).await;
            let mut stack = slot.lock().await;
            if stack.is_empty() {
                *stack = remaining;
                return None;
            }
            if stack.are_items_and_components_equal(&remaining)
                && stack.item_count < stack.get_max_stack_size()
            {
                let space = stack.get_max_stack_size() - stack.item_count;
                let moved = space.min(remaining.item_count);
                stack.item_count += moved;
                remaining.item_count -= moved;
                if remaining.is_empty() {
                    return None;
                }
            }
        }
        Some(remaining)
    }

    fn block_pos_of(mob: &dyn Mob) -> BlockPos {
        mob.get_mob_entity().living_entity.entity.block_pos.load()
    }

    fn walk_to(mob: &dyn Mob, target: BlockPos, speed: f64) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let dest = Vector3::new(
            f64::from(target.0.x) + 0.5,
            f64::from(target.0.y),
            f64::from(target.0.z) + 0.5,
        );
        let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
        navigator.set_progress(NavigatorGoal::new(pos, dest, speed));
    }

    fn navigator_idle(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().navigator.lock().unwrap().is_idle()
    }
}

impl Goal for TransportItemsGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            let origin = Self::block_pos_of(mob);

            let Some(source) = Self::find_source(&world, origin).await else {
                self.cooldown = mob.get_random().random_range(60..=100);
                return false;
            };
            if Self::find_dest(&world, origin).await.is_none() {
                self.cooldown = mob.get_random().random_range(60..=100);
                return false;
            }

            self.phase = Phase::ToSource(source);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !matches!(self.phase, Phase::Idle) })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Phase::ToSource(source) = self.phase {
                Self::walk_to(mob, source, self.speed);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if !Self::navigator_idle(mob) {
                return;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();

            match self.phase {
                Phase::ToSource(source) => {
                    let origin = Self::block_pos_of(mob);
                    if source.squared_distance(&origin) > 9 {
                        self.phase = Phase::Idle;
                        self.cooldown = mob.get_random().random_range(60..=100);
                        return;
                    }
                    let Some(item) = Self::take_one_item(&world, source).await else {
                        self.phase = Phase::Idle;
                        self.cooldown = mob.get_random().random_range(60..=100);
                        return;
                    };
                    let Some(dest) = Self::find_dest(&world, origin).await else {
                        Self::deposit_item(&world, source, item).await;
                        self.phase = Phase::Idle;
                        return;
                    };
                    self.carried = Some(item);
                    self.phase = Phase::ToDest { source, dest };
                    Self::walk_to(mob, dest, self.speed);
                }
                Phase::ToDest { source, dest } => {
                    if let Some(item) = self.carried.take()
                        && let Some(leftover) = Self::deposit_item(&world, dest, item).await
                    {
                        Self::deposit_item(&world, source, leftover).await;
                    }
                    self.phase = Phase::Idle;
                    self.cooldown = mob.get_random().random_range(60..=100);
                }
                Phase::Idle => {}
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.phase = Phase::Idle;
            self.carried = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
