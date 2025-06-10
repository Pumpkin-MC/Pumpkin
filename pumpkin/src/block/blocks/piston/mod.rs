use piston::PistonBlock;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

#[expect(clippy::module_inception)]
pub mod piston;
pub mod piston_extension;
pub mod piston_head;

const MAX_MOVABLE_BLOCKS: usize = 12;

pub struct PistonHandler<'a> {
    world: &'a World,
    pos_from: BlockPos,
    retracted: bool,
    pos_to: BlockPos,
    motion_direction: BlockDirection,
    pub moved_blocks: Vec<BlockPos>,
    pub broken_blocks: Vec<BlockPos>,
    piston_direction: BlockDirection,
}

impl<'a> PistonHandler<'a> {
    pub fn new(world: &'a World, pos: BlockPos, dir: BlockDirection, retracted: bool) -> Self {
        let motion_direction;
        let pos_to = if retracted {
            motion_direction = dir;
            pos.offset(dir.to_offset())
        } else {
            motion_direction = dir.opposite();
            pos.offset_dir(dir.to_offset(), 2)
        };
        PistonHandler {
            world,
            pos_from: pos,
            piston_direction: dir,
            retracted,
            motion_direction,
            pos_to,
            moved_blocks: Vec::new(),
            broken_blocks: Vec::new(),
        }
    }

    pub async fn calculate_push(&mut self) -> bool {
        self.moved_blocks.clear();
        self.broken_blocks.clear();
        let (block, block_state) = self.world.get_block_and_block_state(&self.pos_to).await;
        dbg!(PistonBlock::is_movable(
            &block,
            &block_state,
            self.motion_direction,
            false,
            self.piston_direction,
        ));
        if !PistonBlock::is_movable(
            &block,
            &block_state,
            self.motion_direction,
            false,
            self.piston_direction,
        ) {
            if self.retracted && block_state.piston_behavior == PistonBehavior::Destroy {
                self.broken_blocks.push(self.pos_to);
                return true;
            }
            return false;
        }
        if !self.try_move(self.pos_to, self.motion_direction).await {
            return false;
        }
        for block_pos in self.moved_blocks.clone() {
            let block = self.world.get_block(&block_pos).await;
            if Self::is_block_sticky(&block)
                && !self.try_move_adjacent_block(&block, block_pos).await
            {
                return false;
            }
        }
        true
    }

    fn is_block_sticky(block: &Block) -> bool {
        block == &Block::SLIME_BLOCK || block == &Block::HONEY_BLOCK
    }

    fn is_adjacent_block_stuck(state: &Block, adjacent_state: &Block) -> bool {
        if state == &Block::HONEY_BLOCK && adjacent_state == &Block::SLIME_BLOCK {
            return false;
        }
        if state == &Block::SLIME_BLOCK && adjacent_state == &Block::HONEY_BLOCK {
            return false;
        }
        Self::is_block_sticky(state) || Self::is_block_sticky(adjacent_state)
    }

    async fn try_move(&mut self, pos: BlockPos, dir: BlockDirection) -> bool {
        let (mut block, mut block_state) = self.world.get_block_and_block_state(&pos).await;
        if block_state.is_air() {
            return true;
        }
        if !PistonBlock::is_movable(&block, &block_state, self.motion_direction, false, dir) {
            return true;
        }
        if pos == self.pos_from {
            return true;
        }
        if self.moved_blocks.contains(&pos) {
            return true;
        }
        let mut i = 1;
        if i + self.moved_blocks.len() > MAX_MOVABLE_BLOCKS {
            return false;
        }
        while Self::is_block_sticky(&block) {
            let block_pos = pos.offset_dir(self.motion_direction.opposite().to_offset(), i as i32);
            let block2 = block;
            (block, block_state) = self.world.get_block_and_block_state(&block_pos).await;
            if block_state.is_air()
                || !Self::is_adjacent_block_stuck(&block2, &block)
                || !PistonBlock::is_movable(
                    &block,
                    &block_state,
                    self.motion_direction,
                    false,
                    self.motion_direction.opposite(),
                )
                || block_pos == self.pos_from
            {
                break;
            }
            i += 1;
            if i + self.moved_blocks.len() > MAX_MOVABLE_BLOCKS {
                return false;
            }
        }
        let mut j = 0;
        for k in (0..i).rev() {
            self.moved_blocks
                .push(pos.offset_dir(self.motion_direction.opposite().to_offset(), k as i32));
            j += 1;
        }
        let mut k = 1;
        loop {
            let block_pos2 = pos.offset_dir(self.motion_direction.to_offset(), k);
            if let Some(l) = self.moved_blocks.iter().position(|&p| p == block_pos2) {
                self.set_moved_blocks(j, l);
                for m in 0..=(l + j) {
                    let block_pos3 = self.moved_blocks[m];
                    let block = self.world.get_block(&block_pos3).await;
                    if Self::is_block_sticky(&block)
                        && !Box::pin(self.try_move_adjacent_block(&block, block_pos3)).await
                    {
                        return false;
                    }
                }
                return true;
            }
            (block, block_state) = self.world.get_block_and_block_state(&block_pos2).await;
            if block_state.is_air() {
                return true;
            }
            if !PistonBlock::is_movable(
                &block,
                &block_state,
                self.motion_direction,
                true,
                self.motion_direction,
            ) || block_pos2 == self.pos_from
            {
                return false;
            }
            if block_state.piston_behavior == PistonBehavior::Destroy {
                self.broken_blocks.push(block_pos2);
                return true;
            }
            if self.moved_blocks.len() >= MAX_MOVABLE_BLOCKS {
                return false;
            }
            self.moved_blocks.push(block_pos2);
            j += 1;
            k += 1;
        }
    }

    fn set_moved_blocks(&mut self, from: usize, to: usize) {
        let mut list = Vec::new();
        let mut list2 = Vec::new();
        let mut list3 = Vec::new();
        list.extend_from_slice(&self.moved_blocks[0..to]);
        list2.extend_from_slice(&self.moved_blocks[self.moved_blocks.len() - from..]);
        list3.extend_from_slice(&self.moved_blocks[to..self.moved_blocks.len() - from]);
        self.moved_blocks.clear();
        self.moved_blocks.extend(list);
        self.moved_blocks.extend(list2);
        self.moved_blocks.extend(list3);
    }

    async fn try_move_adjacent_block(&mut self, block: &Block, pos: BlockPos) -> bool {
        for direction in BlockDirection::all() {
            if direction.to_axis() == self.motion_direction.to_axis() {
                continue;
            }
            let block_pos = pos.offset(direction.to_offset());
            let block_state2 = self.world.get_block(&block_pos).await;
            if Self::is_adjacent_block_stuck(&block_state2, block)
                && !self.try_move(block_pos, direction).await
            {
                return false;
            }
        }
        true
    }
}
