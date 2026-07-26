use super::{Entity, EntityBase, Flag};
use crate::net::ClientPlatform;
use crate::world::chunker::is_within_view_distance;
use bytes::BufMut;
use pumpkin_data::block_properties::{Facing, HorizontalFacing};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    CSetActorData, EntityMetadata, MetadataValue, PropertySyncData, entity_data_key,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata, MetadataSerializer};
use pumpkin_util::math::boundingbox::BoundingBox;
use std::sync::atomic::Ordering::{self, Relaxed};

impl Entity {
    /// Sets the `Entity` yaw & pitch rotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        self.yaw.store(yaw);
        self.set_pitch(pitch);
    }

    pub fn set_pitch(&self, pitch: f32) {
        self.pitch.store(pitch.clamp(-90.0, 90.0) % 360.0);
    }

    pub async fn set_sneaking(&self, sneaking: bool) {
        //assert!(self.sneaking.load(Relaxed) != sneaking);
        self.sneaking.store(sneaking, Relaxed);
        self.set_flag(Flag::Sneaking, sneaking).await;
    }
    pub fn is_sneaking(&self) -> bool {
        self.sneaking.load(Ordering::Relaxed)
    }

    pub async fn set_swimming(&self, invisible: bool) {
        if self.swimming.load(Ordering::Relaxed) != invisible {
            self.swimming.store(invisible, Relaxed);
            self.set_flag(Flag::Swimming, invisible).await;
        }
    }

    /// Sets whether the entity is invisible and sends updated metadata.
    pub async fn set_invisible(&self, invisible: bool) {
        if self.invisible.load(Ordering::Relaxed) != invisible {
            self.invisible.store(invisible, Relaxed);
            self.set_flag(Flag::Invisible, invisible).await;
        }
    }

    /// Sets whether the entity is glowing and sends updated metadata.
    pub async fn set_glowing(&self, glowing: bool) {
        if self.glowing.load(Ordering::Relaxed) != glowing {
            self.glowing.store(glowing, Ordering::Relaxed);
            self.set_flag(Flag::Glowing, glowing).await;
        }
    }

    /// Sets whether the entity is on fire for visual and damage purposes. This is separate from `fire_ticks` which tracks the damage aspect of being on fire.
    pub async fn set_on_fire(&self, on_fire: bool) {
        if self.has_visual_fire.load(Ordering::Relaxed) != on_fire {
            self.has_visual_fire.store(on_fire, Ordering::Relaxed);
            self.set_flag(Flag::OnFire, on_fire).await;
        }
    }

    pub fn get_horizontal_facing(&self) -> HorizontalFacing {
        let yaw = self.yaw.load();
        // Use vanilla's formula: floor(angle / 90.0 + 0.5) & 3
        let quarter_turns = ((yaw / 90.0) + 0.5).floor() as i32 & 3;
        match quarter_turns {
            0 => HorizontalFacing::South,
            1 => HorizontalFacing::West,
            2 => HorizontalFacing::North,
            _ => HorizontalFacing::East,
        }
    }

    pub fn get_rotation_16(&self) -> u8 {
        let adjusted_yaw = self.yaw.load().rem_euclid(360.0);

        ((adjusted_yaw / 22.5).round() as u8) % 16
    }

    pub fn get_flipped_rotation_16(&self) -> u8 {
        (self.get_rotation_16() + 8) % 16
    }

    pub fn get_facing(&self) -> Facing {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();

        let x = sin_y * cos_p;
        let y = -sin_p;
        let z = cos_y * cos_p;

        let ax = x.abs();
        let ay = y.abs();
        let az = z.abs();

        if ax > ay && ax > az {
            if x > 0.0 { Facing::East } else { Facing::West }
        } else if ay > ax && ay > az {
            if y > 0.0 { Facing::Up } else { Facing::Down }
        } else if z > 0.0 {
            Facing::South
        } else {
            Facing::North
        }
    }

    pub fn get_entity_facing_order(&self) -> [Facing; 6] {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let sin_p = pitch.sin();
        let cos_p = pitch.cos();
        let sin_y = yaw.sin();
        let cos_y = yaw.cos();

        let east_west = if sin_y > 0.0 {
            Facing::East
        } else {
            Facing::West
        };
        let up_down = if sin_p < 0.0 {
            Facing::Up
        } else {
            Facing::Down
        };
        let south_north = if cos_y > 0.0 {
            Facing::South
        } else {
            Facing::North
        };

        let x_axis = sin_y.abs();
        let y_axis = sin_p.abs();
        let z_axis = cos_y.abs();
        let x_weight = x_axis * cos_p;
        let z_weight = z_axis * cos_p;

        let (first, second, third) = if x_axis > z_axis {
            if y_axis > x_weight {
                (up_down, east_west, south_north)
            } else if z_weight > y_axis {
                (east_west, south_north, up_down)
            } else {
                (east_west, up_down, south_north)
            }
        } else if y_axis > z_weight {
            (up_down, south_north, east_west)
        } else if x_weight > y_axis {
            (south_north, east_west, up_down)
        } else {
            (south_north, up_down, east_west)
        };

        [
            first,
            second,
            third,
            third.opposite(),
            second.opposite(),
            first.opposite(),
        ]
    }

    pub async fn set_sprinting(&self, sprinting: bool) {
        //assert!(self.sprinting.load(Relaxed) != sprinting);
        self.sprinting.store(sprinting, Relaxed);
        self.set_flag(Flag::Sprinting, sprinting).await;
    }

    pub fn is_sprinting(&self) -> bool {
        self.sprinting.load(Ordering::Relaxed)
    }
    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(Relaxed)
    }

    pub async fn set_fall_flying(&self, fall_flying: bool) {
        assert_ne!(self.fall_flying.load(Relaxed), fall_flying);
        self.fall_flying.store(fall_flying, Relaxed);
        self.set_flag(Flag::FallFlying, fall_flying).await;
    }
    pub fn is_fall_flying(&self) -> bool {
        self.fall_flying.load(Ordering::Relaxed)
    }

    async fn set_flag(&self, flag: Flag, value: bool) {
        let index = flag as u8;
        let mask = (1i8).wrapping_shl(index as u32);
        let new_je_flags = if value {
            self.flags.fetch_or(mask, Ordering::Relaxed) | mask
        } else {
            self.flags.fetch_and(!mask, Ordering::Relaxed) & !mask
        };

        self.send_meta_data(
            &[Metadata::new(
                TrackedData::SHARED_FLAGS_ID,
                MetaDataType::BYTE,
                new_je_flags,
            )],
            None,
        );

        if let Some(bedrock_flag) = flag.to_bedrock() {
            let (key, index) = if bedrock_flag >= 64 {
                (entity_data_key::FLAGS_TWO, (bedrock_flag - 64) as u8)
            } else {
                (entity_data_key::FLAGS, bedrock_flag as u8)
            };

            if value {
                let mask = 1i64 << index;
                if key == entity_data_key::FLAGS {
                    self.bedrock_flags.fetch_or(mask, Ordering::Relaxed);
                } else {
                    self.bedrock_flags_two.fetch_or(mask, Ordering::Relaxed);
                }
            } else {
                let mask = !(1i64 << index);
                if key == entity_data_key::FLAGS {
                    self.bedrock_flags.fetch_and(mask, Ordering::Relaxed);
                } else {
                    self.bedrock_flags_two.fetch_and(mask, Ordering::Relaxed);
                }
            }

            let world = self.world.load();
            let chunk_pos = self.chunk_pos.load();
            for player in world.players.load().iter() {
                if let ClientPlatform::Bedrock(client) = player.client.as_ref() {
                    let center = player.get_entity().chunk_pos.load();
                    let view_distance =
                        crate::world::chunker::get_view_distance(player).get() as i32;

                    if is_within_view_distance(chunk_pos, center, view_distance) {
                        let mut metadata = EntityMetadata(std::collections::HashMap::new());
                        metadata.set(
                            entity_data_key::FLAGS,
                            MetadataValue::Long(self.bedrock_flags.load(Ordering::Relaxed)),
                        );
                        metadata.set(
                            entity_data_key::FLAGS_TWO,
                            MetadataValue::Long(self.bedrock_flags_two.load(Ordering::Relaxed)),
                        );
                        client
                            .enqueue_packet(&CSetActorData {
                                actor_runtime_id: VarULong(self.entity_id as u64),
                                metadata,
                                synced_properties: PropertySyncData {
                                    int_properties: std::collections::HashMap::new(),
                                    float_properties: std::collections::HashMap::new(),
                                },
                                tick: VarULong(0),
                            })
                            .await;
                    }
                }
            }
        }
    }

    /// Plays sound at this entity's position with the entity's sound category
    pub fn play_sound(&self, sound: Sound) {
        self.world
            .load()
            .play_sound(sound, SoundCategory::Neutral, &self.pos.load());
    }

    pub fn send_meta_data<T: MetadataSerializer>(
        &self,
        meta: &[Metadata<T>],
        bedrock_meta: Option<&EntityMetadata>,
    ) {
        let world = self.world.load();
        let chunk_pos = self.chunk_pos.load();

        for player in world.players.load().iter() {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    // Apply Chebyshev distance check
                    let center = player.get_entity().chunk_pos.load();
                    let view_distance =
                        crate::world::chunker::get_view_distance(player).get() as i32;

                    if is_within_view_distance(chunk_pos, center, view_distance) {
                        let mut buf = Vec::new();
                        for m in meta {
                            m.write(&mut buf, &client.version.load()).unwrap();
                        }
                        buf.put_u8(255);
                        player.client.try_enqueue_packet(&CSetEntityMetadata::new(
                            self.entity_id.into(),
                            buf.into(),
                        ));
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    if let Some(bedrock_meta) = bedrock_meta {
                        let center = player.get_entity().chunk_pos.load();
                        let view_distance =
                            crate::world::chunker::get_view_distance(player).get() as i32;

                        if is_within_view_distance(chunk_pos, center, view_distance) {
                            client.try_enqueue_packet(&CSetActorData {
                                actor_runtime_id: VarULong(self.entity_id as u64),
                                metadata: EntityMetadata(bedrock_meta.0.clone()),
                                synced_properties: PropertySyncData {
                                    int_properties: std::collections::HashMap::new(),
                                    float_properties: std::collections::HashMap::new(),
                                },
                                tick: VarULong(0),
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn set_pose(&self, pose: EntityPose) {
        let dimension = Self::get_entity_dimensions(pose);
        let position = self.pos.load();
        let aabb = BoundingBox::new_from_pos(position.x, position.y, position.z, &dimension);
        if self.world.load().is_space_empty(aabb.contract_all(1.0E-7)) {
            self.pose.store(pose);
            let dimension = Self::get_entity_dimensions(pose);
            self.bounding_box.store(aabb);
            self.entity_dimension.store(dimension);
            let pose = pose as i32;
            let mut bedrock_meta = EntityMetadata::new();
            bedrock_meta.set(entity_data_key::POSE_INDEX, MetadataValue::Int(pose));
            self.send_meta_data(
                &[Metadata::new(
                    TrackedData::POSE,
                    MetaDataType::ENTITY_POSE,
                    VarInt(pose),
                )],
                Some(&bedrock_meta),
            );
        }
    }

    /// Checks if the entity is invulnerable to the given damage type, considering both general invulnerability and specific immunities.
    pub async fn is_invulnerable_to(&self, damage_type: &DamageType) -> bool {
        // Nothing is immune to void or kill
        if matches!(
            *damage_type,
            DamageType::GENERIC_KILL | DamageType::OUT_OF_WORLD
        ) {
            return false;
        }

        // General invulnerability
        if self.invulnerable.load(Ordering::Relaxed) {
            return true;
        }

        // Specific type immunities
        self.damage_immunities.lock().await.contains(damage_type)
    }

    /// Sets if the entity is invulnerable to a specific damage type
    pub async fn set_damage_immunity(&self, damage_type: DamageType, immune: bool) {
        let mut immunities = self.damage_immunities.lock().await;
        if immune {
            if !immunities.contains(&damage_type) {
                immunities.push(damage_type);
            }
        } else {
            // retain is cleaner than finding index and removing
            immunities.retain(|dt| dt != &damage_type);
        }
    }

    /// Sets if the entity is invulnerable to all damage types (except `GENERIC_KILL` and `OUT_OF_WORLD`)
    pub fn set_invulnerable(&self, invulnerable: bool) {
        self.invulnerable.store(invulnerable, Relaxed);
    }
}
