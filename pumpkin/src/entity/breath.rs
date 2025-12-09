use crate::entity::{EntityBase, NBTStorage, NBTStorageInit, NbtFuture, player::Player};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::Arc;

/// 管理玩家的呼吸值系统
///
/// 这个结构体负责处理玩家在水下的呼吸机制，包括：
/// - 呼吸值的减少和恢复
/// - 溺水伤害的计算
/// - 与水下呼吸效果的交互
pub struct BreathManager {
    /// 当前呼吸值 (0-300，即15秒 * 20 ticks)
    /// 300 = 15秒的呼吸时间，每秒消耗20点
    pub breath: AtomicCell<i32>,
    /// 呼吸值耗尽后的伤害计时器，用于控制伤害间隔
    pub damage_timer: AtomicCell<u32>,
}

impl Default for BreathManager {
    fn default() -> Self {
        Self::new(300)
    }
}

impl BreathManager {
    /// 创建一个新的呼吸管理器，带有指定的初始呼吸值
    #[must_use]
    pub fn new(initial_breath: i32) -> Self {
        Self {
            breath: AtomicCell::new(initial_breath.clamp(0, 300)),
            damage_timer: AtomicCell::new(0),
        }
    }

    /// 每个游戏tick调用的更新逻辑
    ///
    /// # 参数
    /// * `player` - 需要更新呼吸状态的玩家
    pub async fn tick(&self, player: &Arc<Player>) {
        log::debug!(
            "用户 {} 呼吸值{} 伤害计数 {}",
            player.get_name().get_text(),
            self.breath.load(),
            self.damage_timer.load(),
        );
        // 检查是否在水下
        if !player.living_entity.is_in_water().await {
            // 不在水下时恢复呼吸值
            let current_breath = self.breath.load();
            if current_breath < 300 {
                self.breath.store((current_breath + 1).min(300));
                // 可选：发送呼吸值更新给客户端
                player.send_breath().await;
            }
            self.damage_timer.store(0);
            return;
        }

        // 检查是否有水下呼吸效果（如药水效果）
        if player
            .living_entity
            .has_effect(&pumpkin_data::effect::StatusEffect::WATER_BREATHING)
            .await
        {
            // 有水下呼吸效果时不减少呼吸值
            return;
        }

        // 检查是否有潮涌能量效果（也提供水下呼吸）
        if player
            .living_entity
            .has_effect(&pumpkin_data::effect::StatusEffect::CONDUIT_POWER)
            .await
        {
            // 潮涌能量效果提供水下呼吸
            return;
        }

        // 在水下且没有水下呼吸效果
        let current_breath = self.breath.load();

        if current_breath > -20 {
            // 减少呼吸值
            self.breath.store(current_breath - 1);

            // 当呼吸值较低时发送更新给客户端（可选优化）
            if current_breath % 10 == 0 {
                // 每秒更新一次
                player.send_breath().await;
            }
        } else {
            // 呼吸值为0，开始造成伤害
            self.damage_timer.fetch_add(1);

            // 每20 ticks（1秒）造成一次溺水伤害
            if self.damage_timer.load() >= 20 {
                // 造成溺水伤害
                let damage_dealt = player.damage(player.clone(), 1.0, DamageType::DROWN).await;

                self.damage_timer.store(0);
            }
        }
    }

    /// 重置呼吸值到最大值
    /// 通常在玩家死亡或使用特定物品时调用
    pub fn reset_breath(&self) {
        self.breath.store(300);
        self.damage_timer.store(0);
    }

    /// 设置呼吸值到指定值
    ///
    /// # 参数
    /// * `breath` - 新的呼吸值，会被限制在0-300范围内
    pub fn set_breath(&self, breath: i32) {
        self.breath.store(breath.clamp(0, 300));
        self.damage_timer.store(0);
    }

    /// 获取当前呼吸值
    pub fn get_breath(&self) -> i32 {
        self.breath.load()
    }

    /// 增加呼吸值
    ///
    /// # 参数
    /// * `amount` - 要增加的呼吸值
    pub fn add_breath(&self, amount: i32) {
        let current = self.breath.load();
        self.breath.store((current + amount).clamp(0, 300));
    }

    /// 检查玩家是否正在溺水（呼吸值为0且在水下）
    pub async fn is_drowning(&self, player: &Player) -> bool {
        self.breath.load() <= 0 && player.living_entity.is_in_water().await
    }
}

impl NBTStorage for BreathManager {
    /// 将呼吸管理器的状态写入NBT数据
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            let breath = self.breath.load();
            log::debug!(
                "用户 {} 的呼吸值更新为 {}",
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
            self.breath.store(breath.clamp(0, 300));
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

        let manager = BreathManager::new(-10); // 负值
        assert_eq!(manager.get_breath(), 0);
    }

    #[test]
    fn test_set_breath() {
        let manager = BreathManager::default();
        manager.set_breath(100);
        assert_eq!(manager.get_breath(), 100);

        manager.set_breath(500); // 超过最大值
        assert_eq!(manager.get_breath(), 300);

        manager.set_breath(-10); // 负值
        assert_eq!(manager.get_breath(), 0);
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
    }

    #[test]
    fn test_reset_breath() {
        let manager = BreathManager::new(100);
        manager.reset_breath();
        assert_eq!(manager.get_breath(), 300);
    }
}
