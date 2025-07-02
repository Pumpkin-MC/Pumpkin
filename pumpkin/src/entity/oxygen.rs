use super::{player::Player, EntityBase, NBTStorage};
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use rand::Rng;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EffectType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{MetaDataType, Metadata};
use pumpkin_util::GameMode;
use pumpkin_world::entity::entity_data_flags::DATA_AIR_SUPPLY_ID;

pub struct OxygenManager {
    /// Current oxygen level in ticks (0 = depleted)
    pub oxygen_level: AtomicCell<u16>,
    /// Timer for drowning damage ticks
    pub damage_timer: AtomicCell<u16>,
}

impl Default for OxygenManager {
    fn default() -> Self {
        Self {
            oxygen_level: AtomicCell::new(300), // Default full oxygen (15 seconds)
            damage_timer: AtomicCell::new(0),
        }
    }
}

impl OxygenManager {
    /// Maximum oxygen capacity (constant in vanilla)
    const MAX_OXYGEN: u16 = 300; // 15 seconds * 20 ticks/sec

    pub async fn tick(&self, player: &Player) {
        let current_oxygen = self.oxygen_level.load();

        // Check if we are in a mode that consumes oxygen and in water
        if matches!(player.gamemode.load(), GameMode::Survival | GameMode::Adventure)
            && player.living_entity.is_in_water().await
        {
            let mut damage_timer = self.damage_timer.load();

            // Water breathing effect grants immunity
            if player.living_entity.has_effect(EffectType::WaterBreathing).await {
                if current_oxygen < Self::MAX_OXYGEN {
                    self.update_oxygen(player, Self::MAX_OXYGEN).await;
                }
                self.damage_timer.store(0);
                return;
            }

            // Consume oxygen with respiration chance
            if current_oxygen > 0 {
                let respiration_level = player.get_respiration_level();
                let should_consume = if respiration_level > 0 {
                    // 1/(level+1) chance to preserve oxygen each tick
                    !rand::rng().random_bool(1.0 / (respiration_level as f64 + 1.0))
                } else {
                    true
                };

                if should_consume {
                    self.update_oxygen(player, current_oxygen - 1).await;
                }
                // Reset damage timer because we are not drowning (we have oxygen left)
                self.damage_timer.store(0);
            }
            // Handle oxygen depletion (drowning)
            else {
                damage_timer += 1;
                self.damage_timer.store(damage_timer);

                // Apply damage every second (20 ticks)
                if damage_timer >= 20 {
                    player.damage(2.0, DamageType::DROWN).await;
                    self.damage_timer.store(0);
                }
            }
        }
        // Replenish oxygen when not submerged (vanilla: 15x faster replenish)
        else if current_oxygen < Self::MAX_OXYGEN {
            // Vanilla replenishes 15 per tick when out of water
            self.update_oxygen(player, current_oxygen + 15).await;
            self.damage_timer.store(0);
        }
    }

    pub fn reset(&self) {
        self.oxygen_level.store(Self::MAX_OXYGEN);
        self.damage_timer.store(0);
    }

    async fn update_oxygen(&self, player: &Player, new_oxygen: u16) {
        // Clamp to valid range [0, MAX_OXYGEN]
        let clamped = new_oxygen.min(Self::MAX_OXYGEN);
        self.oxygen_level.store(clamped);

        player
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                DATA_AIR_SUPPLY_ID,
                MetaDataType::Integer,
                VarInt(i32::from(clamped)),
            )])
            .await;
    }
}

#[async_trait]
impl NBTStorage for OxygenManager {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_short("Air", self.oxygen_level.load() as i16);
        nbt.put_short("DrownTimer", self.damage_timer.load() as i16);
    }

    async fn read_nbt(&mut self, nbt: &mut NbtCompound) {
        self.oxygen_level
            .store(nbt.get_short("Air").unwrap_or(Self::MAX_OXYGEN as i16) as u16);
        self.damage_timer
            .store(nbt.get_short("DrownTimer").unwrap_or(0) as u16);
    }
}