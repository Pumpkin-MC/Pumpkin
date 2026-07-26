use super::{CURRENT_ID, Entity, MAX_SCOREBOARD_TAGS, RemovalReason};
use crate::world::World;
use arc_swap::ArcSwap;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::biome::Biome;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::bedrock::client::CSetActorMotion;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    EntityMetadata, MetadataValue, entity_data_flag, entity_data_key,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{CEntityVelocity, CSpawnEntity, Metadata};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord,
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
};
use pumpkin_util::text::TextComponent;
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{
        AtomicBool, AtomicI32, AtomicU8, AtomicU32,
        Ordering::{self, Relaxed},
    },
};
use tokio::sync::Mutex;
use uuid::Uuid;

impl Entity {
    pub fn new(
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        Self::from_uuid(Uuid::new_v4(), world, position, entity_type)
    }

    pub fn reserve_ids(count: i32) -> i32 {
        CURRENT_ID.fetch_add(count, Relaxed)
    }

    pub fn from_uuid(
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        Self::from_uuid_with_id(
            CURRENT_ID.fetch_add(1, Relaxed),
            entity_uuid,
            world,
            position,
            entity_type,
        )
    }

    pub fn from_uuid_with_id(
        entity_id: i32,
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        let floor_x = position.x.floor() as i32;
        let floor_y = position.y.floor() as i32;
        let floor_z = position.z.floor() as i32;

        let bounding_box_size = EntityDimensions {
            width: entity_type.dimension[0],
            height: entity_type.dimension[1],
            eye_height: entity_type.eye_height,
        };

        Self {
            entity_id,
            entity_uuid,
            entity_type,
            on_ground: AtomicBool::new(false),
            touching_water: AtomicBool::new(false),
            water_height: AtomicCell::new(0.0),
            touching_lava: AtomicBool::new(false),
            lava_height: AtomicCell::new(0.0),
            horizontal_collision: AtomicBool::new(false),
            pos: AtomicCell::new(position),
            last_pos: AtomicCell::new(position),
            movement: AtomicCell::new(Vector3::default()),
            block_pos: AtomicCell::new(BlockPos(Vector3::new(floor_x, floor_y, floor_z))),
            supporting_block_pos: AtomicCell::new(None),
            chunk_pos: AtomicCell::new(Vector2::new(
                get_section_cord(floor_x),
                get_section_cord(floor_z),
            )),
            sneaking: AtomicBool::new(false),
            swimming: AtomicBool::new(false),
            invisible: AtomicBool::new(false),
            glowing: AtomicBool::new(false),
            world: ArcSwap::new(world),
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            body_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            pose: AtomicCell::new(EntityPose::Standing),
            bounding_box: AtomicCell::new(BoundingBox::new_from_pos(
                position.x,
                position.y,
                position.z,
                &bounding_box_size,
            )),
            entity_dimension: AtomicCell::new(bounding_box_size),
            invulnerable: AtomicBool::new(false),
            damage_immunities: Mutex::new(Vec::new()),
            data: AtomicI32::new(0),
            flags: std::sync::atomic::AtomicI8::new(0),
            bedrock_flags: std::sync::atomic::AtomicI64::new(0),
            bedrock_flags_two: std::sync::atomic::AtomicI64::new(0),
            fire_immune: AtomicBool::new(false),
            fire_ticks: AtomicI32::new(-1),
            has_visual_fire: AtomicBool::new(false),
            frozen_ticks: AtomicI32::new(0),
            is_in_powder_snow: AtomicBool::new(false),
            was_in_powder_snow: AtomicBool::new(false),
            removal_reason: AtomicCell::new(None),
            passengers: Mutex::new(Vec::new()),
            vehicle: Mutex::new(None),
            leashed_to: Mutex::new(None),

            riding_cooldown: AtomicI32::new(0),
            age: AtomicI32::new(0),
            current_biome: ArcSwap::new(Arc::new(&Biome::PLAINS)),
            last_biome_update_pos: AtomicCell::new(BlockPos::new(floor_x, floor_y, floor_z)),
            portal_cooldown: AtomicU32::new(0),
            portal_manager: Mutex::new(None),
            custom_name: ArcSwap::new(Arc::new(None)),
            custom_name_visible: AtomicBool::new(false),
            scoreboard_tags: Mutex::new(HashSet::new()),
            no_clip: AtomicBool::new(false),
            movement_multiplier: AtomicCell::new(Vector3::default()),
            velocity_dirty: AtomicBool::new(true),
            removed: AtomicBool::new(false),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
            last_sent_pos: AtomicCell::new(position),
        }
    }

    pub fn add_velocity(&self, velocity: Vector3<f64>) {
        self.set_velocity(self.velocity.load() + velocity);
    }

    pub fn set_velocity(&self, velocity: Vector3<f64>) {
        self.velocity.store(velocity);
        self.send_velocity();
    }

    /// Updates the world reference for this entity.
    /// Called when the entity changes dimensions (e.g., through a nether portal).
    pub fn set_world(&self, world: Arc<World>) {
        self.world.store(world);
    }

    pub fn bedrock_metadata(&self) -> EntityMetadata {
        if self.bedrock_flags.load(Ordering::Relaxed) == 0 {
            self.bedrock_flags.fetch_or(
                (1i64 << entity_data_flag::HAS_GRAVITY)
                    | (1i64 << entity_data_flag::CLIMB)
                    | (1i64 << entity_data_flag::HAS_COLLISION)
                    | (1i64 << entity_data_flag::BREATHING),
                Ordering::Relaxed,
            );
        }

        let mut metadata = EntityMetadata::new();
        metadata.set(
            entity_data_key::WIDTH,
            MetadataValue::Float(self.entity_type.dimension[0]),
        );
        metadata.set(
            entity_data_key::HEIGHT,
            MetadataValue::Float(self.entity_type.dimension[1]),
        );
        metadata.set(entity_data_key::SCALE, MetadataValue::Float(1.0));
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Long(self.bedrock_flags.load(Ordering::Relaxed)),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Long(self.bedrock_flags_two.load(Ordering::Relaxed)),
        );

        if let Some(name) = &**self.custom_name.load() {
            metadata.set(
                entity_data_key::NAME,
                MetadataValue::String(name.clone().get_text()),
            );
            if self.custom_name_visible.load(Ordering::Relaxed) {
                metadata.set_flag(
                    entity_data_key::FLAGS,
                    entity_data_flag::SHOW_NAME as u8,
                    true,
                );
                metadata.set_flag(
                    entity_data_key::FLAGS,
                    entity_data_flag::ALWAYS_SHOW_NAME as u8,
                    true,
                );
            }
        }

        metadata
    }

    /// Sets the entity's age in ticks.
    /// Negative values indicate that the entity is a baby.
    pub fn set_age(&self, age: i32) {
        self.age.store(age, Relaxed);
    }

    /// Adds a scoreboard tag to this entity.
    ///
    /// Returns `false` if the entity already has the tag or already carries
    /// [`MAX_SCOREBOARD_TAGS`] tags.
    pub async fn add_scoreboard_tag(&self, tag: &str) -> bool {
        let mut tags = self.scoreboard_tags.lock().await;
        tags.len() < MAX_SCOREBOARD_TAGS && tags.insert(tag.to_owned())
    }

    /// Removes a scoreboard tag from this entity.
    ///
    /// Returns `false` if the entity did not have the tag.
    pub async fn remove_scoreboard_tag(&self, tag: &str) -> bool {
        self.scoreboard_tags.lock().await.remove(tag)
    }

    /// Sets a custom name for the entity, typically used with nametags
    pub fn set_custom_name(&self, name: TextComponent) {
        self.custom_name.store(Arc::new(Some(name.clone())));
        let mut bedrock_meta = EntityMetadata::new();
        bedrock_meta.set(
            entity_data_key::NAME,
            MetadataValue::String(name.clone().get_text()),
        );
        let visible = self.custom_name_visible.load(Ordering::Relaxed);
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::SHOW_NAME as u8,
            visible,
        );
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::ALWAYS_SHOW_NAME as u8,
            visible,
        );
        self.send_meta_data(
            &[Metadata::new(
                TrackedData::CUSTOM_NAME,
                MetaDataType::OPTIONAL_TEXT_COMPONENT,
                Some(name),
            )],
            Some(&bedrock_meta),
        );
    }

    pub fn set_custom_name_visible(&self, visible: bool) {
        self.custom_name_visible.store(visible, Ordering::Relaxed);
        let mut bedrock_meta = EntityMetadata::new();
        if let Some(name) = &**self.custom_name.load() {
            bedrock_meta.set(
                entity_data_key::NAME,
                MetadataValue::String(name.clone().get_text()),
            );
        }
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::SHOW_NAME as u8,
            visible,
        );
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::ALWAYS_SHOW_NAME as u8,
            visible,
        );
        self.send_meta_data(
            &[Metadata::new(
                TrackedData::CUSTOM_NAME_VISIBLE,
                MetaDataType::BOOLEAN,
                visible,
            )],
            Some(&bedrock_meta),
        );
    }

    pub fn send_velocity(&self) {
        let velocity = self.velocity.load();
        let chunk_pos = self.chunk_pos.load();
        self.world.load().broadcast_to_chunk_editioned_sync(
            chunk_pos,
            &CEntityVelocity::new(self.entity_id.into(), velocity),
            &CSetActorMotion::new(
                VarULong(self.entity_id as u64),
                Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
                VarULong(0),
            ),
        );
    }

    #[must_use]
    pub const fn get_entity_dimensions(pose: EntityPose) -> EntityDimensions {
        match pose {
            EntityPose::Sleeping => EntityDimensions::new(0.2, 0.2, 0.2),
            EntityPose::FallFlying | EntityPose::Swimming | EntityPose::SpinAttack => {
                EntityDimensions::new(0.6, 0.6, 0.4)
            }
            EntityPose::Crouching => EntityDimensions::new(0.6, 1.5, 1.27),
            EntityPose::Dying => EntityDimensions::new(0.2, 0.2, 1.62),
            _ => EntityDimensions::new(0.6, 1.8, 1.62),
        }
    }

    pub fn get_eye_height(&self) -> f64 {
        f64::from(Self::get_entity_dimensions(self.pose.load()).eye_height)
    }

    /// Mark this entity as removed. Idempotent; first caller wins for `reason`.
    pub fn mark_removed(&self, reason: RemovalReason) {
        if !self.removed.swap(true, Ordering::Relaxed) {
            self.removal_reason.store(Some(reason));
        } else if self.removal_reason.load().is_none() {
            self.removal_reason.store(Some(reason));
        }
    }

    /// Removes the `Entity` from their current `World`.
    ///
    /// Sets `removed` / `removal_reason` so concurrent entity ticks (which hold a
    /// snapshot of the entity list) can bail out instead of racing `on_death`.
    pub async fn remove(&self) {
        self.mark_removed(RemovalReason::Discarded);
        self.world.load().remove_entity(self).await;
    }

    pub fn create_spawn_packet(&self) -> CSpawnEntity {
        let entity_loc = self.pos.load();
        let entity_vel = self.velocity.load();
        CSpawnEntity::new(
            VarInt(self.entity_id),
            self.entity_uuid,
            VarInt(i32::from(self.entity_type.id)),
            entity_loc,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(), // todo: head_yaw and yaw are swapped, find out why
            self.data.load(Relaxed).into(),
            entity_vel,
        )
    }
    pub fn width(&self) -> f32 {
        self.entity_dimension.load().width
    }

    pub fn height(&self) -> f32 {
        self.entity_dimension.load().height
    }

    pub fn get_eye_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        Vector3::new(
            pos.x,
            pos.y + f64::from(self.entity_dimension.load().eye_height),
            pos.z,
        )
    }

    pub fn get_eye_y(&self) -> f64 {
        self.pos.load().y + f64::from(self.entity_dimension.load().eye_height)
    }

    pub fn is_removed(&self) -> bool {
        self.removal_reason.load().is_some()
    }

    pub fn is_alive(&self) -> bool {
        !self.is_removed()
    }
}
