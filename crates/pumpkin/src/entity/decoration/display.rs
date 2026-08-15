use std::sync::{
    Arc,
    atomic::{AtomicI8, AtomicI32, AtomicU8, Ordering},
};
use tokio::sync::Mutex;

use pumpkin_data::{
    damage::DamageType, item_stack::ItemStack, meta_data_type::MetaDataType,
    tracked_data::TrackedData,
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    codec::{item_stack_seralizer::ItemStackSerializer, var_int::VarInt},
    java::client::play::{Metadata, MetadataSerializer},
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::{math::vector3::Vector3, text::TextComponent};

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity},
    server::Server,
};

#[derive(Clone, Copy, Debug)]
pub struct Vector3fSerializer(pub f32, pub f32, pub f32);

impl MetadataSerializer for Vector3fSerializer {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_f32(self.0)?;
        writer.write_f32(self.1)?;
        writer.write_f32(self.2)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuaternionfSerializer(pub f32, pub f32, pub f32, pub f32);

impl MetadataSerializer for QuaternionfSerializer {
    fn write_metadata(&self, writer: &mut impl std::io::Write) -> Result<(), WritingError> {
        writer.write_f32(self.0)?;
        writer.write_f32(self.1)?;
        writer.write_f32(self.2)?;
        writer.write_f32(self.3)
    }
}

pub struct DisplayEntity {
    pub entity: Entity,
    pub interpolation_start_delta_ticks: AtomicI32,
    pub interpolation_duration: AtomicI32,
    pub view_range: Mutex<f32>,
    pub shadow_radius: Mutex<f32>,
    pub shadow_strength: Mutex<f32>,
    pub width: Mutex<f32>,
    pub height: Mutex<f32>,
    pub glow_color_override: AtomicI32,
    pub billboard: AtomicU8,
    pub brightness: AtomicI32,
    pub translation: Mutex<Vector3<f32>>,
    pub scale: Mutex<Vector3<f32>>,
    pub left_rotation: Mutex<[f32; 4]>,
    pub right_rotation: Mutex<[f32; 4]>,
}

impl DisplayEntity {
    pub fn new(entity: Entity) -> Self {
        entity.no_clip.store(true, Ordering::Relaxed);
        Self {
            entity,
            interpolation_start_delta_ticks: AtomicI32::new(0),
            interpolation_duration: AtomicI32::new(0),
            view_range: Mutex::new(1.0),
            shadow_radius: Mutex::new(0.0),
            shadow_strength: Mutex::new(1.0),
            width: Mutex::new(0.0),
            height: Mutex::new(0.0),
            glow_color_override: AtomicI32::new(-1),
            billboard: AtomicU8::new(0),
            brightness: AtomicI32::new(-1),
            translation: Mutex::new(Vector3::new(0.0, 0.0, 0.0)),
            scale: Mutex::new(Vector3::new(1.0, 1.0, 1.0)),
            left_rotation: Mutex::new([0.0, 0.0, 0.0, 1.0]),
            right_rotation: Mutex::new([0.0, 0.0, 0.0, 1.0]),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn init_display_data_tracker(&self) {
        let view_range = *self.view_range.lock().await;
        let shadow_radius = *self.shadow_radius.lock().await;
        let shadow_strength = *self.shadow_strength.lock().await;
        let width = *self.width.lock().await;
        let height = *self.height.lock().await;
        let translation = *self.translation.lock().await;
        let scale = *self.scale.lock().await;
        let left_rotation = *self.left_rotation.lock().await;
        let right_rotation = *self.right_rotation.lock().await;

        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::START_INTERPOLATION,
                MetaDataType::INT,
                VarInt(self.interpolation_start_delta_ticks.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::INTERPOLATION_DURATION,
                MetaDataType::INT,
                VarInt(self.interpolation_duration.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TRANSLATION,
                MetaDataType::VECTOR_3F,
                Vector3fSerializer(translation.x, translation.y, translation.z),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::SCALE,
                MetaDataType::VECTOR_3F,
                Vector3fSerializer(scale.x, scale.y, scale.z),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::LEFT_ROTATION,
                MetaDataType::QUATERNION_F,
                QuaternionfSerializer(
                    left_rotation[0],
                    left_rotation[1],
                    left_rotation[2],
                    left_rotation[3],
                ),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::RIGHT_ROTATION,
                MetaDataType::QUATERNION_F,
                QuaternionfSerializer(
                    right_rotation[0],
                    right_rotation[1],
                    right_rotation[2],
                    right_rotation[3],
                ),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BILLBOARD,
                MetaDataType::BYTE,
                self.billboard.load(Ordering::Relaxed),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BRIGHTNESS,
                MetaDataType::INT,
                VarInt(self.brightness.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::VIEW_RANGE,
                MetaDataType::FLOAT,
                view_range,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::SHADOW_RADIUS,
                MetaDataType::FLOAT,
                shadow_radius,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::SHADOW_STRENGTH,
                MetaDataType::FLOAT,
                shadow_strength,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::WIDTH,
                MetaDataType::FLOAT,
                width,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::HEIGHT,
                MetaDataType::FLOAT,
                height,
            )],
            None,
        );
    }

    pub async fn write_display_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.write_nbt(nbt).await;
        nbt.put_int(
            "interpolation_duration",
            self.interpolation_duration.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "start_interpolation",
            self.interpolation_start_delta_ticks.load(Ordering::Relaxed),
        );
        nbt.put_float("view_range", *self.view_range.lock().await);
        nbt.put_float("shadow_radius", *self.shadow_radius.lock().await);
        nbt.put_float("shadow_strength", *self.shadow_strength.lock().await);
        nbt.put_float("width", *self.width.lock().await);
        nbt.put_float("height", *self.height.lock().await);
        nbt.put_int(
            "glow_color_override",
            self.glow_color_override.load(Ordering::Relaxed),
        );

        let billboard_str = match self.billboard.load(Ordering::Relaxed) {
            1 => "vertical",
            2 => "horizontal",
            3 => "center",
            _ => "fixed",
        };
        nbt.put_string("billboard", billboard_str.to_string());

        let mut transform = NbtCompound::new();
        let translation = *self.translation.lock().await;
        transform.put(
            "translation",
            NbtTag::List(vec![
                translation.x.into(),
                translation.y.into(),
                translation.z.into(),
            ]),
        );
        let scale = *self.scale.lock().await;
        transform.put(
            "scale",
            NbtTag::List(vec![scale.x.into(), scale.y.into(), scale.z.into()]),
        );
        let left_rot = *self.left_rotation.lock().await;
        transform.put(
            "left_rotation",
            NbtTag::List(vec![
                left_rot[0].into(),
                left_rot[1].into(),
                left_rot[2].into(),
                left_rot[3].into(),
            ]),
        );
        let right_rot = *self.right_rotation.lock().await;
        transform.put(
            "right_rotation",
            NbtTag::List(vec![
                right_rot[0].into(),
                right_rot[1].into(),
                right_rot[2].into(),
                right_rot[3].into(),
            ]),
        );
        nbt.put("transformation", NbtTag::Compound(transform));
    }

    pub async fn read_display_nbt(&self, nbt: &NbtCompound) {
        self.entity.read_nbt_non_mut(nbt).await;

        if let Some(dur) = nbt.get_int("interpolation_duration") {
            self.interpolation_duration.store(dur, Ordering::Relaxed);
        }
        if let Some(start) = nbt.get_int("start_interpolation") {
            self.interpolation_start_delta_ticks
                .store(start, Ordering::Relaxed);
        }
        if let Some(vr) = nbt.get_float("view_range") {
            *self.view_range.lock().await = vr;
        }
        if let Some(sr) = nbt.get_float("shadow_radius") {
            *self.shadow_radius.lock().await = sr;
        }
        if let Some(ss) = nbt.get_float("shadow_strength") {
            *self.shadow_strength.lock().await = ss;
        }
        if let Some(w) = nbt.get_float("width") {
            *self.width.lock().await = w;
        }
        if let Some(h) = nbt.get_float("height") {
            *self.height.lock().await = h;
        }
        if let Some(glow) = nbt.get_int("glow_color_override") {
            self.glow_color_override.store(glow, Ordering::Relaxed);
        }
        if let Some(bb) = nbt.get_string("billboard") {
            let mode = match bb {
                "vertical" => 1,
                "horizontal" => 2,
                "center" => 3,
                _ => 0,
            };
            self.billboard.store(mode, Ordering::Relaxed);
        }

        if let Some(transform) = nbt.get_compound("transformation") {
            if let Some(t_list) = transform.get_list("translation")
                && t_list.len() >= 3
            {
                let x = t_list[0].extract_float().unwrap_or(0.0);
                let y = t_list[1].extract_float().unwrap_or(0.0);
                let z = t_list[2].extract_float().unwrap_or(0.0);
                *self.translation.lock().await = Vector3::new(x, y, z);
            }
            if let Some(s_list) = transform.get_list("scale")
                && s_list.len() >= 3
            {
                let x = s_list[0].extract_float().unwrap_or(1.0);
                let y = s_list[1].extract_float().unwrap_or(1.0);
                let z = s_list[2].extract_float().unwrap_or(1.0);
                *self.scale.lock().await = Vector3::new(x, y, z);
            }
            if let Some(lr_list) = transform.get_list("left_rotation")
                && lr_list.len() >= 4
            {
                let x = lr_list[0].extract_float().unwrap_or(0.0);
                let y = lr_list[1].extract_float().unwrap_or(0.0);
                let z = lr_list[2].extract_float().unwrap_or(0.0);
                let w = lr_list[3].extract_float().unwrap_or(1.0);
                *self.left_rotation.lock().await = [x, y, z, w];
            }
            if let Some(rr_list) = transform.get_list("right_rotation")
                && rr_list.len() >= 4
            {
                let x = rr_list[0].extract_float().unwrap_or(0.0);
                let y = rr_list[1].extract_float().unwrap_or(0.0);
                let z = rr_list[2].extract_float().unwrap_or(0.0);
                let w = rr_list[3].extract_float().unwrap_or(1.0);
                *self.right_rotation.lock().await = [x, y, z, w];
            }
        }
    }
}

pub struct BlockDisplayEntity {
    pub display: DisplayEntity,
    pub block_state: AtomicI32,
}

impl BlockDisplayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            display: DisplayEntity::new(entity),
            block_state: AtomicI32::new(0),
        })
    }
}

impl NBTStorage for BlockDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_display_nbt(nbt).await;
            nbt.put_int("block_state", self.block_state.load(Ordering::Relaxed));
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.read_nbt_non_mut(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_display_nbt(nbt).await;
            if let Some(state) = nbt.get_int("block_state") {
                self.block_state.store(state, Ordering::Relaxed);
            }
        })
    }
}

impl EntityBase for BlockDisplayEntity {
    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.init_display_data_tracker().await;
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::BLOCK_STATE,
                    MetaDataType::BLOCK_STATE,
                    VarInt(self.block_state.load(Ordering::Relaxed)),
                )],
                None,
            );
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.display.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { false })
    }
}

pub struct ItemDisplayEntity {
    pub display: DisplayEntity,
    pub item_stack: Mutex<ItemStack>,
    pub item_display: AtomicU8,
}

impl ItemDisplayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            display: DisplayEntity::new(entity),
            item_stack: Mutex::new(ItemStack::new(0, &pumpkin_data::item::Item::AIR)),
            item_display: AtomicU8::new(0),
        })
    }
}

impl NBTStorage for ItemDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_display_nbt(nbt).await;
            let display_mode_str = match self.item_display.load(Ordering::Relaxed) {
                1 => "thirdperson_lefthand",
                2 => "thirdperson_righthand",
                3 => "firstperson_lefthand",
                4 => "firstperson_righthand",
                5 => "head",
                6 => "gui",
                7 => "ground",
                8 => "fixed",
                _ => "none",
            };
            nbt.put_string("item_display", display_mode_str.to_string());
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.read_nbt_non_mut(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_display_nbt(nbt).await;
            if let Some(mode_str) = nbt.get_string("item_display") {
                let mode = match mode_str {
                    "thirdperson_lefthand" => 1,
                    "thirdperson_righthand" => 2,
                    "firstperson_lefthand" => 3,
                    "firstperson_righthand" => 4,
                    "head" => 5,
                    "gui" => 6,
                    "ground" => 7,
                    "fixed" => 8,
                    _ => 0,
                };
                self.item_display.store(mode, Ordering::Relaxed);
            }
        })
    }
}

impl EntityBase for ItemDisplayEntity {
    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.init_display_data_tracker().await;
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::ITEM,
                    MetaDataType::ITEM_STACK,
                    ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::ITEM_DISPLAY,
                    MetaDataType::BYTE,
                    self.item_display.load(Ordering::Relaxed),
                )],
                None,
            );
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.display.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { false })
    }
}

pub struct TextDisplayEntity {
    pub display: DisplayEntity,
    pub text: Mutex<TextComponent>,
    pub line_width: AtomicI32,
    pub background: AtomicI32,
    pub text_opacity: AtomicI8,
    pub flags: AtomicU8,
}

impl TextDisplayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            display: DisplayEntity::new(entity),
            text: Mutex::new(TextComponent::text("")),
            line_width: AtomicI32::new(200),
            background: AtomicI32::new(1_073_741_824),
            text_opacity: AtomicI8::new(-1),
            flags: AtomicU8::new(0),
        })
    }
}

impl NBTStorage for TextDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_display_nbt(nbt).await;
            let text_json_res = pumpkin_util::serde_json::to_string(&*self.text.lock().await);
            if let Ok(text_json) = text_json_res {
                nbt.put_string("text", text_json);
            }
            nbt.put_int("line_width", self.line_width.load(Ordering::Relaxed));
            nbt.put_int("background", self.background.load(Ordering::Relaxed));
            nbt.put_byte("text_opacity", self.text_opacity.load(Ordering::Relaxed));

            let flags = self.flags.load(Ordering::Relaxed);
            nbt.put_bool("shadow", flags & 1 != 0);
            nbt.put_bool("see_through", flags & 2 != 0);
            nbt.put_bool("default_background", flags & 4 != 0);
            let align_str = if flags & 8 != 0 {
                "left"
            } else if flags & 16 != 0 {
                "right"
            } else {
                "center"
            };
            nbt.put_string("alignment", align_str.to_string());
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.read_nbt_non_mut(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_display_nbt(nbt).await;
            if let Some(text_json) = nbt.get_string("text")
                && let Ok(component) = pumpkin_util::serde_json::from_str(text_json)
            {
                *self.text.lock().await = component;
            }
            if let Some(lw) = nbt.get_int("line_width") {
                self.line_width.store(lw, Ordering::Relaxed);
            }
            if let Some(bg) = nbt.get_int("background") {
                self.background.store(bg, Ordering::Relaxed);
            }
            if let Some(opacity) = nbt.get_byte("text_opacity") {
                self.text_opacity.store(opacity, Ordering::Relaxed);
            }

            let mut flags = 0u8;
            if nbt.get_bool("shadow").unwrap_or(false) {
                flags |= 1;
            }
            if nbt.get_bool("see_through").unwrap_or(false) {
                flags |= 2;
            }
            if nbt.get_bool("default_background").unwrap_or(false) {
                flags |= 4;
            }
            if let Some(align) = nbt.get_string("alignment") {
                match align {
                    "left" => flags |= 8,
                    "right" => flags |= 16,
                    _ => {}
                }
            }
            self.flags.store(flags, Ordering::Relaxed);
        })
    }
}

impl EntityBase for TextDisplayEntity {
    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.init_display_data_tracker().await;
            let text = self.text.lock().await.clone();
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::TEXT,
                    MetaDataType::COMPONENT,
                    text,
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::LINE_WIDTH,
                    MetaDataType::INT,
                    VarInt(self.line_width.load(Ordering::Relaxed)),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::BACKGROUND,
                    MetaDataType::INT,
                    VarInt(self.background.load(Ordering::Relaxed)),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::TEXT_OPACITY,
                    MetaDataType::BYTE,
                    self.text_opacity.load(Ordering::Relaxed) as u8,
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::TEXT_DISPLAY_FLAGS,
                    MetaDataType::BYTE,
                    self.flags.load(Ordering::Relaxed),
                )],
                None,
            );
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.display.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { false })
    }
}
