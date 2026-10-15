use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;
use std::sync::Mutex;
use uuid::Uuid;

pub struct VaultBlockEntity {
    pub position: BlockPos,
    pub config: Mutex<Option<NbtCompound>>,
    pub server_data: Mutex<Option<NbtCompound>>,
    pub shared_data: Mutex<Option<NbtCompound>>,
    pub rewarded_players: Mutex<FxHashSet<Uuid>>,
}

impl BlockEntity for VaultBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut rewarded_players = FxHashSet::default();
        if let Some(server_data) = nbt.get_compound("server_data") {
            if let Some(list) = server_data.get_list("rewarded_players") {
                for tag in list {
                    match tag {
                        pumpkin_nbt::tag::NbtTag::IntArray(arr) if arr.len() == 4 => {
                            let value = ((arr[0] as u128) << 96)
                                | (((arr[1] as u32) as u128) << 64)
                                | (((arr[2] as u32) as u128) << 32)
                                | ((arr[3] as u32) as u128);
                            rewarded_players.insert(Uuid::from_u128(value));
                        }
                        pumpkin_nbt::tag::NbtTag::String(s) => {
                            if let Ok(uuid) = Uuid::parse_str(s) {
                                rewarded_players.insert(uuid);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Self {
            position,
            config: Mutex::new(nbt.get_compound("config").cloned()),
            server_data: Mutex::new(nbt.get_compound("server_data").cloned()),
            shared_data: Mutex::new(nbt.get_compound("shared_data").cloned()),
            rewarded_players: Mutex::new(rewarded_players),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(cfg) = self.config.lock()
            && let Some(cfg) = cfg.as_ref()
        {
            nbt.put_compound("config", cfg.clone());
        }
        let mut data = self
            .server_data
            .lock()
            .ok()
            .and_then(|d| d.clone())
            .unwrap_or_default();
        if let Ok(rewarded) = self.rewarded_players.lock() {
            let list: Vec<pumpkin_nbt::tag::NbtTag> = rewarded
                .iter()
                .map(|u| {
                    let value = u.as_u128();
                    pumpkin_nbt::tag::NbtTag::IntArray(vec![
                        (value >> 96) as i32,
                        ((value >> 64) & 0xFFFF_FFFF) as i32,
                        ((value >> 32) & 0xFFFF_FFFF) as i32,
                        (value & 0xFFFF_FFFF) as i32,
                    ])
                })
                .collect();
            data.put("rewarded_players", pumpkin_nbt::tag::NbtTag::List(list));
        }
        nbt.put_compound("server_data", data);

        if let Ok(shared) = self.shared_data.lock()
            && let Some(shared) = shared.as_ref()
        {
            nbt.put_compound("shared_data", shared.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(shared) = self.shared_data.try_lock()
            && let Some(ref shared) = *shared
        {
            nbt.put_compound("shared_data", shared.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VaultBlockEntity {
    pub const ID: &'static str = "minecraft:vault";
    pub const MAX_REWARDED_PLAYERS: usize = 128;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            config: Mutex::new(None),
            server_data: Mutex::new(None),
            shared_data: Mutex::new(None),
            rewarded_players: Mutex::new(FxHashSet::default()),
        }
    }

    pub fn has_rewarded(&self, player_id: &Uuid) -> bool {
        self.rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(player_id)
    }

    pub fn mark_rewarded(&self, player_id: Uuid) {
        let mut rewarded = self
            .rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        rewarded.insert(player_id);
        if rewarded.len() > Self::MAX_REWARDED_PLAYERS {
            if let Some(first) = rewarded.iter().copied().next() {
                rewarded.remove(&first);
            }
        }
    }

    pub fn get_loot_table_key(&self, ominous: bool) -> String {
        if let Ok(config_guard) = self.config.lock()
            && let Some(cfg) = config_guard.as_ref()
            && let Some(loot_table) = cfg.get_string("loot_table")
        {
            return loot_table.to_string();
        }
        if ominous {
            "minecraft:chests/trial_chambers/reward_ominous".to_string()
        } else {
            "minecraft:chests/trial_chambers/reward".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_rewarded_players_and_limit() {
        let vault = VaultBlockEntity::new(BlockPos::new(0, 60, 0));
        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        assert!(!vault.has_rewarded(&player1));
        vault.mark_rewarded(player1);
        assert!(vault.has_rewarded(&player1));
        assert!(!vault.has_rewarded(&player2));

        // Test limit of 128
        for _ in 0..130 {
            vault.mark_rewarded(Uuid::new_v4());
        }
        assert!(
            vault
                .rewarded_players
                .lock()
                .unwrap()
                .len()
                <= VaultBlockEntity::MAX_REWARDED_PLAYERS
        );
    }

    #[test]
    fn test_vault_loot_table_keys() {
        let vault = VaultBlockEntity::new(BlockPos::new(0, 60, 0));
        assert_eq!(
            vault.get_loot_table_key(false),
            "minecraft:chests/trial_chambers/reward"
        );
        assert_eq!(
            vault.get_loot_table_key(true),
            "minecraft:chests/trial_chambers/reward_ominous"
        );
    }

    #[test]
    fn test_vault_nbt_roundtrip() {
        let vault = VaultBlockEntity::new(BlockPos::new(10, 64, 10));
        let player = Uuid::new_v4();
        vault.mark_rewarded(player);

        let mut nbt = NbtCompound::new();
        vault.write_nbt(&mut nbt);

        let loaded = VaultBlockEntity::from_nbt(&nbt, BlockPos::new(10, 64, 10));
        assert!(loaded.has_rewarded(&player));
    }
}
