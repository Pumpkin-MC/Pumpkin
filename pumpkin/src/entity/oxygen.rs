use super::{EntityBase, NBTStorage, player::Player};
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EffectType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{MetaDataType, Metadata};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::entity::entity_data_flags::DATA_AIR_SUPPLY_ID;
use rand::Rng;

pub struct OxygenManager {
    /// Current oxygen level in ticks (0 = depleted)
    pub oxygen_level: AtomicCell<u16>,
    /// Timer for drowning damage ticks
    pub damage_timer: AtomicCell<u16>,
}

impl Default for OxygenManager {
    fn default() -> Self {
        Self {
            oxygen_level: AtomicCell::new(300), // 15 seconds * 20 ticks/sec
            damage_timer: AtomicCell::new(0),
        }
    }
}

impl OxygenManager {
    /// Maximum oxygen capacity (constant in vanilla)
    const MAX_OXYGEN: u16 = 300;

    pub async fn tick(&self, player: &Player) {
        if !matches!(
            player.gamemode.load(),
            GameMode::Survival | GameMode::Adventure
        ) {
            return;
        }

        let current_oxygen = self.oxygen_level.load();
        let is_eyes_in_water = self.are_eyes_in_water(player).await;
        let has_water_breathing = player
            .living_entity
            .has_effect(EffectType::WaterBreathing)
            .await;

        // Water breathing effect grants immunity
        if has_water_breathing {
            if current_oxygen < Self::MAX_OXYGEN {
                self.update_oxygen(player, Self::MAX_OXYGEN).await;
            }
            self.damage_timer.store(0);
            return;
        }

        // Handle oxygen mechanics when eyes are in water
        if is_eyes_in_water {
            let mut damage_timer = self.damage_timer.load();

            // Consume oxygen if available (with respiration chance)
            if current_oxygen > 0 {
                let respiration_level = self.get_respiration_level(player).await;
                let should_consume = if respiration_level > 0 {
                    // Vanilla: 1/(level+1) chance to preserve oxygen
                    !rand::rng().random_ratio(1, respiration_level as u32 + 1)
                } else {
                    true
                };

                if should_consume {
                    self.update_oxygen(player, current_oxygen - 1).await;
                }
                self.damage_timer.store(0);
            }
            // Handle oxygen depletion
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
        // Replenish oxygen when eyes are not in water
        else if current_oxygen < Self::MAX_OXYGEN {
            // Vanilla: Oxygen replenishes at 4 per tick
            let new_oxygen = (current_oxygen + 4).min(Self::MAX_OXYGEN);
            self.update_oxygen(player, new_oxygen).await;
            self.damage_timer.store(0);
        }
    }

    pub fn reset(&self) {
        self.oxygen_level.store(Self::MAX_OXYGEN);
        self.damage_timer.store(0);
    }

    async fn update_oxygen(&self, player: &Player, new_oxygen: u16) {
        self.oxygen_level.store(new_oxygen);

        player
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                DATA_AIR_SUPPLY_ID,
                MetaDataType::Integer,
                VarInt(i32::from(new_oxygen)),
            )])
            .await;
    }

    async fn are_eyes_in_water(&self, player: &Player) -> bool {
        let world = player.world().await;
        let eye_pos = player.eye_position();

        let block_pos = BlockPos(Vector3 {
            x: eye_pos.x.floor() as i32,
            y: eye_pos.y.floor() as i32,
            z: eye_pos.z.floor() as i32,
        });

        let block = world.get_block(&block_pos).await;

        if block == &Block::WATER || block == &Block::BUBBLE_COLUMN {
            let block_y_min = block_pos.0.y as f64;
            let eye_rel_y = eye_pos.y - block_y_min;
            let water_height = self.get_fluid_height(&block_pos).await;
            eye_rel_y < water_height
        } else {
            false
        }
    }

    async fn get_fluid_height(&self, _block_pos: &BlockPos) -> f64 {
        // todo: calculate for flowing water
        1f64
    }

    pub async fn get_respiration_level(&self, _player: &Player) -> u8 {
        // todo: implement when we will have support of enchanted items

        0
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
