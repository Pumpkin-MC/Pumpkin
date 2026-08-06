use pumpkin_data::{Block, BlockId};

/// Vanilla `InfestedBlock.BLOCK_BY_HOST_BLOCK`, populated by every `new InfestedBlock(host, ...)`
/// registration in `Blocks.java`.
#[must_use]
pub const fn infested_for_host(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::STONE => Some(BlockId::INFESTED_STONE),
        BlockId::COBBLESTONE => Some(BlockId::INFESTED_COBBLESTONE),
        BlockId::STONE_BRICKS => Some(BlockId::INFESTED_STONE_BRICKS),
        BlockId::MOSSY_STONE_BRICKS => Some(BlockId::INFESTED_MOSSY_STONE_BRICKS),
        BlockId::CRACKED_STONE_BRICKS => Some(BlockId::INFESTED_CRACKED_STONE_BRICKS),
        BlockId::CHISELED_STONE_BRICKS => Some(BlockId::INFESTED_CHISELED_STONE_BRICKS),
        BlockId::DEEPSLATE => Some(BlockId::INFESTED_DEEPSLATE),
        _ => None,
    }
}

/// Reverse of [`infested_for_host`], used by `InfestedBlock#hostStateByInfested`.
#[must_use]
pub const fn host_for_infested(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::INFESTED_STONE => Some(BlockId::STONE),
        BlockId::INFESTED_COBBLESTONE => Some(BlockId::COBBLESTONE),
        BlockId::INFESTED_STONE_BRICKS => Some(BlockId::STONE_BRICKS),
        BlockId::INFESTED_MOSSY_STONE_BRICKS => Some(BlockId::MOSSY_STONE_BRICKS),
        BlockId::INFESTED_CRACKED_STONE_BRICKS => Some(BlockId::CRACKED_STONE_BRICKS),
        BlockId::INFESTED_CHISELED_STONE_BRICKS => Some(BlockId::CHISELED_STONE_BRICKS),
        BlockId::INFESTED_DEEPSLATE => Some(BlockId::DEEPSLATE),
        _ => None,
    }
}

/// Vanilla `InfestedBlock.isCompatibleHostBlock`.
#[must_use]
pub const fn is_compatible_host_block(block: &Block) -> bool {
    infested_for_host(block.id).is_some()
}

/// Reproduces the outward-search order of vanilla's `SilverfishWakeUpFriendsGoal.tick`.
///
/// The triple loop `for (int off = 0; off <= n && off >= -n; off = (off <= 0 ? 1 : 0) - off)`
/// yields `0, 1, -1, 2, -2, ..., n, -n`.
#[must_use]
pub fn zigzag_range(n: i32) -> Vec<i32> {
    let mut out = Vec::with_capacity((2 * n + 1) as usize);
    let mut off = 0;
    while off <= n && off >= -n {
        out.push(off);
        off = i32::from(off <= 0) - off;
    }
    out
}

#[cfg(test)]
mod zigzag_tests {
    use super::zigzag_range;

    #[test]
    fn matches_vanilla_offset_order() {
        assert_eq!(zigzag_range(2), vec![0, 1, -1, 2, -2]);
        assert_eq!(zigzag_range(5), vec![0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5]);
    }
}

#[cfg(test)]
mod tests {
    use super::{host_for_infested, infested_for_host};
    use pumpkin_data::BlockId;

    #[test]
    fn every_host_maps_back_to_itself() {
        let hosts = [
            BlockId::STONE,
            BlockId::COBBLESTONE,
            BlockId::STONE_BRICKS,
            BlockId::MOSSY_STONE_BRICKS,
            BlockId::CRACKED_STONE_BRICKS,
            BlockId::CHISELED_STONE_BRICKS,
            BlockId::DEEPSLATE,
        ];

        for host in hosts {
            let infested = infested_for_host(host).expect("host should have an infested variant");
            assert_eq!(host_for_infested(infested), Some(host));
        }
    }
}
