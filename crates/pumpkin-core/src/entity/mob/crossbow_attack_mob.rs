pub trait CrossbowAttackMob: Send + Sync {
    fn set_charging_crossbow(&self, is_charging: bool);
    fn is_charging_crossbow(&self) -> bool;
    fn on_crossbow_attack_performed(&self) {}
}
