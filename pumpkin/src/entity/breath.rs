use crate::entity::{EntityBase, NBTStorage, NBTStorageInit, NbtFuture, player::Player};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use std::sync::Arc;

pub struct BreathManager {
    pub breath: AtomicCell<i32>,
}

impl Default for BreathManager {
    fn default() -> Self {
        Self::new(300)
    }
}

impl BreathManager {
    /// 创建一个新的呼吸管理器，带有指定的初始氧气值
    #[must_use]
    pub fn new(initial_breath: i32) -> Self {
        Self {
            breath: AtomicCell::new(initial_breath.clamp(-20, 300)),
        }
    }

    /// 每个游戏tick调用的更新逻辑
    ///
    /// # 参数
    /// * `player` - 需要更新呼吸状态的玩家
    pub async fn tick(&self, player: &Arc<Player>) {
        // 在创造模式或生存模式无需更新
        let player_gamemode = player.gamemode.load();
        if player_gamemode == GameMode::Creative || player_gamemode == GameMode::Survival {
            return;
        }

        let mut breath = self.breath.load();

        // 是否在水中
        let is_in_water = player.living_entity.is_in_water().await;
        // 是否有水呼吸效果
        let has_water_breathing = player
            .living_entity
            .has_effect(&pumpkin_data::effect::StatusEffect::WATER_BREATHING)
            .await;
        // 是否有水路连接效果
        let has_conduit_power = player
            .living_entity
            .has_effect(&pumpkin_data::effect::StatusEffect::CONDUIT_POWER)
            .await;

        let can_breathe = !is_in_water || has_water_breathing || has_conduit_power;

        // --------------------------------------------------
        // 1. 可以呼吸 → 氧气先重置到0, 随后恢复到 300
        // --------------------------------------------------
        if can_breathe {
            if breath < 300 {
                breath = (breath.max(0) + 1).min(300);
                self.breath.store(breath);
                log::debug!("用户 {} 的呼吸值更新为 {}", player.gameprofile.name, breath);
                player.send_breath().await;
            }
            return;
        }

        // --------------------------------------------------
        // 2. 不能呼吸 → 氧气值下降
        // --------------------------------------------------
        breath -= 1;
        self.breath.store(breath);
        log::debug!("用户 {} 的呼吸值更新为 {}", player.gameprofile.name, breath);

        // 可选：每秒更新一次客户端
        if breath % 20 == 0 {
            player.send_breath().await;
        }

        // --------------------------------------------------
        // 3. 氧气值到达 -20 → 触发一次伤害 → 氧气值重置为 0
        // --------------------------------------------------
        if breath == -20 {
            // 伤害类型（空气呼吸 → 溺水）
            let damage_type = DamageType::DROWN;

            player.damage(player.clone(), 2.0, damage_type).await;

            // 按 Wiki 设回 0
            self.breath.store(0);

            // 通知客户端
            player.send_breath().await;
        }
    }

    /// 重置氧气值到最大值
    /// 通常在玩家死亡或使用特定物品时调用
    pub fn restart(&self) {
        self.breath.store(300);
    }

    /// 设置氧气值到指定值
    ///
    /// # 参数
    /// * `breath` - 新的氧气值，会被限制在-20到300范围内
    pub fn set_breath(&self, breath: i32) {
        self.breath.store(breath.clamp(-20, 300));
    }

    /// 获取当前氧气值
    pub fn get_breath(&self) -> i32 {
        self.breath.load()
    }

    /// 增加氧气值
    ///
    /// # 参数
    /// * `amount` - 要增加的氧气值
    pub fn add_breath(&self, amount: i32) {
        let current = self.breath.load();
        self.breath.store((current + amount).clamp(-20, 300));
    }

    /// 检查玩家是否正在溺水（氧气值为0且在水下且处于扣血计时）
    pub async fn is_drowning(&self, player: &Player) -> bool {
        self.breath.load() == 0 && player.living_entity.is_in_water().await
    }

    /// 检查玩家是否处于危险状态（氧气值<=0且在水下）
    pub async fn is_in_danger(&self, player: &Player) -> bool {
        self.breath.load() <= 0 && player.living_entity.is_in_water().await
    }

    /// 检查玩家是否处于扣血计时阶段（氧气值为0且在水下）
    pub async fn is_in_damage_phase(&self, player: &Player) -> bool {
        self.breath.load() == 0 && player.living_entity.is_in_water().await
    }
}

impl NBTStorage for BreathManager {
    /// 将呼吸管理器的状态写入NBT数据
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            let breath = self.breath.load();
            log::debug!(
                "用户 {} 的氧气值更新为 {}",
                nbt.get_string("name").unwrap_or("未知"),
                breath,
            );
            nbt.put_int("air", breath);
            // Minecraft原版也保存这些字段，为了兼容性我们也保存
            nbt.put_int("Air", breath);
        })
    }

    /// 从NBT数据读取呼吸管理器的状态
    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            // 优先读取"air"字段，如果不存在则读取"Air"字段（兼容性）
            let breath = nbt
                .get_int("air")
                .unwrap_or_else(|| nbt.get_int("Air").unwrap_or(300));
            self.breath.store(breath.clamp(-20, 300));
        })
    }
}

impl NBTStorageInit for BreathManager {}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要模拟环境，在实际项目中可能需要mock
    #[test]
    fn test_breath_manager_default() {
        let manager = BreathManager::default();
        assert_eq!(manager.get_breath(), 300);
    }

    #[test]
    fn test_breath_manager_new() {
        let manager = BreathManager::new(150);
        assert_eq!(manager.get_breath(), 150);

        let manager = BreathManager::new(500); // 超过最大值
        assert_eq!(manager.get_breath(), 300);

        let manager = BreathManager::new(-10); // 负值在范围内
        assert_eq!(manager.get_breath(), -10);

        let manager = BreathManager::new(-30); // 超过最小值
        assert_eq!(manager.get_breath(), -20);
    }

    #[test]
    fn test_set_breath() {
        let manager = BreathManager::default();
        manager.set_breath(100);
        assert_eq!(manager.get_breath(), 100);

        manager.set_breath(500); // 超过最大值
        assert_eq!(manager.get_breath(), 300);

        manager.set_breath(-10); // 负值在范围内
        assert_eq!(manager.get_breath(), -10);

        manager.set_breath(-30); // 超过最小值
        assert_eq!(manager.get_breath(), -20);
    }

    #[test]
    fn test_add_breath() {
        let manager = BreathManager::default();
        manager.add_breath(50);
        assert_eq!(manager.get_breath(), 300); // 会被限制到300

        manager.set_breath(200);
        manager.add_breath(50);
        assert_eq!(manager.get_breath(), 250);

        manager.add_breath(100);
        assert_eq!(manager.get_breath(), 300); // 达到最大值

        // 测试负值范围
        manager.set_breath(-10);
        manager.add_breath(-15);
        assert_eq!(manager.get_breath(), -20); // 会被限制到-20
    }

    #[test]
    fn test_reset_breath() {
        let manager = BreathManager::new(-10);
        manager.restart();
        assert_eq!(manager.get_breath(), 300);
    }
}
