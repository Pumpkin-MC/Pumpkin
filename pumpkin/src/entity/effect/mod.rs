use pumpkin_data::entity::EffectType;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Effect {
    pub r#type: EffectType,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}
