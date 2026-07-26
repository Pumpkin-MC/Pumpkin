use super::living::LivingEntity;
use super::{
    Entity, EntityBase, EntityBaseFuture, MAX_SCOREBOARD_TAGS, NBTStorage, NbtFuture,
    TeleportFuture,
};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::vector3::Vector3;
use std::sync::{
    Arc,
    atomic::Ordering::{self, Relaxed},
};

impl NBTStorage for Entity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            let position = self.pos.load();
            nbt.put_string(
                "id",
                format!("minecraft:{}", self.entity_type.resource_name),
            );
            nbt.put_uuid("UUID", self.entity_uuid);
            nbt.put(
                "Pos",
                NbtTag::List(vec![
                    position.x.into(),
                    position.y.into(),
                    position.z.into(),
                ]),
            );
            let velocity = self.velocity.load();
            nbt.put(
                "Motion",
                NbtTag::List(vec![
                    velocity.x.into(),
                    velocity.y.into(),
                    velocity.z.into(),
                ]),
            );
            nbt.put(
                "Rotation",
                NbtTag::List(vec![self.yaw.load().into(), self.pitch.load().into()]),
            );
            nbt.put_short("Fire", self.fire_ticks.load(Relaxed) as i16);
            nbt.put_bool("OnGround", self.on_ground.load(Relaxed));
            nbt.put_bool("Invulnerable", self.invulnerable.load(Relaxed));
            nbt.put_int("PortalCooldown", self.portal_cooldown.load(Relaxed) as i32);
            if self.has_visual_fire.load(Relaxed) {
                nbt.put_bool("HasVisualFire", true);
            }
            nbt.put_int("TicksFrozen", self.frozen_ticks.load(Relaxed));
            if let Some(custom_name) = &**self.custom_name.load()
                && let Ok(name_json) = pumpkin_util::serde_json::to_string(custom_name)
            {
                nbt.put_string("CustomName", name_json);
            }
            nbt.put_bool("CustomNameVisible", self.custom_name_visible.load(Relaxed));

            let tags = self.scoreboard_tags.lock().await;
            if !tags.is_empty() {
                nbt.put(
                    "Tags",
                    NbtTag::List(
                        tags.iter()
                            .map(|tag| NbtTag::String(tag.as_str().into()))
                            .collect(),
                    ),
                );
            }

            // todo more...
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            let position = nbt.get_list("Pos").unwrap();
            let x = position[0].extract_double().unwrap_or(0.0);
            let y = position[1].extract_double().unwrap_or(0.0);
            let z = position[2].extract_double().unwrap_or(0.0);
            let pos = Vector3::new(x, y, z);
            self.set_pos(pos);
            self.last_sent_pos.store(pos);
            let velocity = nbt.get_list("Motion").unwrap();
            let x = velocity[0].extract_double().unwrap_or(0.0);
            let y = velocity[1].extract_double().unwrap_or(0.0);
            let z = velocity[2].extract_double().unwrap_or(0.0);
            self.velocity.store(Vector3::new(x, y, z));
            let rotation = nbt.get_list("Rotation").unwrap();
            let yaw = rotation[0].extract_float().unwrap_or(0.0);
            let pitch = rotation[1].extract_float().unwrap_or(0.0);
            self.set_rotation(yaw, pitch);
            let yaw_byte = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let pitch_byte = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;
            self.last_sent_yaw.store(yaw_byte, Relaxed);
            self.last_sent_pitch.store(pitch_byte, Relaxed);
            self.head_yaw.store(yaw);
            self.last_sent_head_yaw.store(yaw_byte, Relaxed);
            self.fire_ticks
                .store(i32::from(nbt.get_short("Fire").unwrap_or(0)), Relaxed);
            self.on_ground
                .store(nbt.get_bool("OnGround").unwrap_or(false), Relaxed);
            self.invulnerable
                .store(nbt.get_bool("Invulnerable").unwrap_or(false), Relaxed);
            self.portal_cooldown
                .store(nbt.get_int("PortalCooldown").unwrap_or(0) as u32, Relaxed);
            self.has_visual_fire
                .store(nbt.get_bool("HasVisualFire").unwrap_or(false), Relaxed);
            self.frozen_ticks
                .store(nbt.get_int("TicksFrozen").unwrap_or(0), Relaxed);
            if let Some(name_json) = nbt.get_string("CustomName")
                && let Ok(component) = pumpkin_util::serde_json::from_str(name_json)
            {
                self.custom_name.store(Arc::new(Some(component)));
            }
            self.custom_name_visible
                .store(nbt.get_bool("CustomNameVisible").unwrap_or(false), Relaxed);

            if let Some(tag_list) = nbt.get_list("Tags") {
                let mut tags = self.scoreboard_tags.lock().await;
                tags.clear();
                tags.extend(
                    tag_list
                        .iter()
                        .filter_map(|tag| tag.extract_string().map(str::to_owned))
                        .take(MAX_SCOREBOARD_TAGS),
                );
            }

            // todo more...
        })
    }
}

impl EntityBase for Entity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Recomputed during movement/block-collision handling in the same tick.
            let was_in_powder_snow = self.is_in_powder_snow.load(Ordering::Relaxed);
            self.was_in_powder_snow
                .store(was_in_powder_snow, Ordering::Relaxed);
            self.is_in_powder_snow.store(false, Ordering::Relaxed);

            let block_pos = self.block_pos.load();
            if self.last_biome_update_pos.load() != block_pos {
                let world = self.world.load();
                let biome = world.level.get_rough_biome(&block_pos);
                self.current_biome.store(Arc::new(biome));
                self.last_biome_update_pos.store(block_pos);
            }

            self.update_last_pos();
            self.tick_portal(caller).await;
            self.update_fluid_state(caller).await;
            self.check_out_of_world(&**caller).await;
            let fire_ticks = self.fire_ticks.load(Ordering::Relaxed);

            // Check for fire immunity (or if the specific entity is)
            let is_immune =
                self.entity_type.fire_immune || self.fire_immune.load(Ordering::Relaxed);
            if fire_ticks > 0 {
                if is_immune {
                    self.fire_ticks.store(fire_ticks - 4, Ordering::Relaxed);
                    if self.fire_ticks.load(Ordering::Relaxed) < 0 {
                        self.extinguish();
                    }
                } else {
                    if fire_ticks % 20 == 0 {
                        (**caller).damage(&**caller, 1.0, DamageType::ON_FIRE).await;
                    }

                    self.fire_ticks.store(fire_ticks - 1, Ordering::Relaxed);
                }
            }

            // Check if visual fire should be sent
            let should_render_fire = self.fire_ticks.load(Ordering::Relaxed) > 0 && !is_immune;
            self.set_on_fire(should_render_fire).await;

            let riding_cooldown = self.riding_cooldown.load(Ordering::Relaxed);
            if riding_cooldown > 0 {
                self.riding_cooldown
                    .store(riding_cooldown - 1, Ordering::Relaxed);
            }
        })
    }

    fn teleport(
        self: Arc<Self>,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) -> TeleportFuture {
        // TODO: handle world change
        Box::pin(async move {
            self.get_entity().teleport(position, yaw, pitch, world);
        })
    }

    fn get_entity(&self) -> &Entity {
        self
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
