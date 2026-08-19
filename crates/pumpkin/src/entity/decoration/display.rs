use std::sync::{
    Arc,
    atomic::{AtomicI8, AtomicI32, AtomicU8, Ordering},
};
use tokio::sync::Mutex;

use pumpkin_data::{damage::DamageType, item_stack::ItemStack};
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
    pub teleport_duration: AtomicI32,
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
            teleport_duration: AtomicI32::new(0),
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

    pub fn get_interpolation_start_delta_ticks(&self) -> i32 {
        self.interpolation_start_delta_ticks.load(Ordering::Relaxed)
    }

    pub fn set_interpolation_start_delta_ticks(&self, ticks: i32) {
        self.interpolation_start_delta_ticks
            .store(ticks, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::START_INTERPOLATION,
                VarInt(ticks),
            )],
            None,
        );
    }

    pub fn get_interpolation_duration(&self) -> i32 {
        self.interpolation_duration.load(Ordering::Relaxed)
    }

    pub fn set_interpolation_duration(&self, duration: i32) {
        self.interpolation_duration
            .store(duration, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::INTERPOLATION_DURATION,
                VarInt(duration),
            )],
            None,
        );
    }

    pub fn get_teleport_duration(&self) -> i32 {
        self.teleport_duration.load(Ordering::Relaxed)
    }

    pub fn set_teleport_duration(&self, duration: i32) {
        self.teleport_duration.store(duration, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::TELEPORT_DURATION,
                VarInt(duration),
            )],
            None,
        );
    }

    pub async fn get_translation(&self) -> Vector3<f32> {
        *self.translation.lock().await
    }

    pub async fn set_translation(&self, translation: Vector3<f32>) {
        *self.translation.lock().await = translation;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::TRANSLATION,
                Vector3fSerializer(translation.x, translation.y, translation.z),
            )],
            None,
        );
    }

    pub async fn get_scale(&self) -> Vector3<f32> {
        *self.scale.lock().await
    }

    pub async fn set_scale(&self, scale: Vector3<f32>) {
        *self.scale.lock().await = scale;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SCALE,
                Vector3fSerializer(scale.x, scale.y, scale.z),
            )],
            None,
        );
    }

    pub async fn get_left_rotation(&self) -> [f32; 4] {
        *self.left_rotation.lock().await
    }

    pub async fn set_left_rotation(&self, left_rotation: [f32; 4]) {
        *self.left_rotation.lock().await = left_rotation;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::LEFT_ROTATION,
                QuaternionfSerializer(
                    left_rotation[0],
                    left_rotation[1],
                    left_rotation[2],
                    left_rotation[3],
                ),
            )],
            None,
        );
    }

    pub async fn get_right_rotation(&self) -> [f32; 4] {
        *self.right_rotation.lock().await
    }

    pub async fn set_right_rotation(&self, right_rotation: [f32; 4]) {
        *self.right_rotation.lock().await = right_rotation;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::RIGHT_ROTATION,
                QuaternionfSerializer(
                    right_rotation[0],
                    right_rotation[1],
                    right_rotation[2],
                    right_rotation[3],
                ),
            )],
            None,
        );
    }

    pub fn get_billboard(&self) -> u8 {
        self.billboard.load(Ordering::Relaxed)
    }

    pub fn set_billboard(&self, billboard: u8) {
        self.billboard.store(billboard, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::BILLBOARD,
                billboard,
            )],
            None,
        );
    }

    pub fn get_brightness(&self) -> i32 {
        self.brightness.load(Ordering::Relaxed)
    }

    pub fn set_brightness(&self, brightness: i32) {
        self.brightness.store(brightness, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::BRIGHTNESS,
                VarInt(brightness),
            )],
            None,
        );
    }

    pub async fn get_view_range(&self) -> f32 {
        *self.view_range.lock().await
    }

    pub async fn set_view_range(&self, view_range: f32) {
        *self.view_range.lock().await = view_range;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::VIEW_RANGE,
                view_range,
            )],
            None,
        );
    }

    pub async fn get_shadow_radius(&self) -> f32 {
        *self.shadow_radius.lock().await
    }

    pub async fn set_shadow_radius(&self, shadow_radius: f32) {
        *self.shadow_radius.lock().await = shadow_radius;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SHADOW_RADIUS,
                shadow_radius,
            )],
            None,
        );
    }

    pub async fn get_shadow_strength(&self) -> f32 {
        *self.shadow_strength.lock().await
    }

    pub async fn set_shadow_strength(&self, shadow_strength: f32) {
        *self.shadow_strength.lock().await = shadow_strength;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SHADOW_STRENGTH,
                shadow_strength,
            )],
            None,
        );
    }

    pub async fn get_display_width(&self) -> f32 {
        *self.width.lock().await
    }

    pub async fn set_display_width(&self, width: f32) {
        *self.width.lock().await = width;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::WIDTH,
                width,
            )],
            None,
        );
    }

    pub async fn get_display_height(&self) -> f32 {
        *self.height.lock().await
    }

    pub async fn set_display_height(&self, height: f32) {
        *self.height.lock().await = height;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::HEIGHT,
                height,
            )],
            None,
        );
    }

    pub fn get_glow_color_override(&self) -> i32 {
        self.glow_color_override.load(Ordering::Relaxed)
    }

    pub fn set_glow_color_override(&self, color: i32) {
        self.glow_color_override.store(color, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::GLOW_COLOR_OVERRIDE,
                VarInt(color),
            )],
            None,
        );
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
                pumpkin_data::tracked_data::display::START_INTERPOLATION,
                VarInt(self.interpolation_start_delta_ticks.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::INTERPOLATION_DURATION,
                VarInt(self.interpolation_duration.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::TRANSLATION,
                Vector3fSerializer(translation.x, translation.y, translation.z),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SCALE,
                Vector3fSerializer(scale.x, scale.y, scale.z),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::LEFT_ROTATION,
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
                pumpkin_data::tracked_data::display::RIGHT_ROTATION,
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
                pumpkin_data::tracked_data::display::BILLBOARD,
                self.billboard.load(Ordering::Relaxed),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::BRIGHTNESS,
                VarInt(self.brightness.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::VIEW_RANGE,
                view_range,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SHADOW_RADIUS,
                shadow_radius,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::SHADOW_STRENGTH,
                shadow_strength,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::WIDTH,
                width,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::HEIGHT,
                height,
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::TELEPORT_DURATION,
                VarInt(self.teleport_duration.load(Ordering::Relaxed)),
            )],
            None,
        );
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::display::GLOW_COLOR_OVERRIDE,
                VarInt(self.glow_color_override.load(Ordering::Relaxed)),
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

    pub fn get_block_state(&self) -> i32 {
        self.block_state.load(Ordering::Relaxed)
    }

    pub fn set_block_state(&self, block_state: i32) {
        self.block_state.store(block_state, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::block_display::BLOCK_STATE,
                VarInt(block_state),
            )],
            None,
        );
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
                    pumpkin_data::tracked_data::block_display::BLOCK_STATE,
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

    pub async fn get_item(&self) -> ItemStack {
        self.item_stack.lock().await.clone()
    }

    pub async fn set_item(&self, item: ItemStack) {
        *self.item_stack.lock().await = item.clone();
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::item_display::ITEM,
                ItemStackSerializer::from(item),
            )],
            None,
        );
    }

    pub fn get_item_display_mode(&self) -> u8 {
        self.item_display.load(Ordering::Relaxed)
    }

    pub fn set_item_display_mode(&self, mode: u8) {
        self.item_display.store(mode, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::item_display::ITEM_DISPLAY,
                mode,
            )],
            None,
        );
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
                    pumpkin_data::tracked_data::item_display::ITEM,
                    ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::item_display::ITEM_DISPLAY,
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

    pub async fn get_text(&self) -> TextComponent {
        self.text.lock().await.clone()
    }

    pub async fn set_text(&self, text: TextComponent) {
        *self.text.lock().await = text.clone();
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT,
                text,
            )],
            None,
        );
    }

    pub fn get_line_width(&self) -> i32 {
        self.line_width.load(Ordering::Relaxed)
    }

    pub fn set_line_width(&self, width: i32) {
        self.line_width.store(width, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::LINE_WIDTH,
                VarInt(width),
            )],
            None,
        );
    }

    pub fn get_background_color(&self) -> i32 {
        self.background.load(Ordering::Relaxed)
    }

    pub fn set_background_color(&self, color: i32) {
        self.background.store(color, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::BACKGROUND,
                VarInt(color),
            )],
            None,
        );
    }

    pub fn get_text_opacity(&self) -> i8 {
        self.text_opacity.load(Ordering::Relaxed)
    }

    pub fn set_text_opacity(&self, opacity: i8) {
        self.text_opacity.store(opacity, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT_OPACITY,
                opacity as u8,
            )],
            None,
        );
    }

    pub fn get_shadow(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & 1) != 0
    }

    pub fn set_shadow(&self, shadow: bool) {
        let mut flags = self.flags.load(Ordering::Relaxed);
        if shadow {
            flags |= 1;
        } else {
            flags &= !1;
        }
        self.flags.store(flags, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT_DISPLAY_FLAGS,
                flags,
            )],
            None,
        );
    }

    pub fn get_see_through(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & 2) != 0
    }

    pub fn set_see_through(&self, see_through: bool) {
        let mut flags = self.flags.load(Ordering::Relaxed);
        if see_through {
            flags |= 2;
        } else {
            flags &= !2;
        }
        self.flags.store(flags, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT_DISPLAY_FLAGS,
                flags,
            )],
            None,
        );
    }

    pub fn get_use_default_background(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & 4) != 0
    }

    pub fn set_use_default_background(&self, default_bg: bool) {
        let mut flags = self.flags.load(Ordering::Relaxed);
        if default_bg {
            flags |= 4;
        } else {
            flags &= !4;
        }
        self.flags.store(flags, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT_DISPLAY_FLAGS,
                flags,
            )],
            None,
        );
    }

    pub fn get_alignment(&self) -> u8 {
        let flags = self.flags.load(Ordering::Relaxed);
        if flags & 8 != 0 {
            1
        } else if flags & 16 != 0 {
            2
        } else {
            0
        }
    }

    pub fn set_alignment(&self, align: u8) {
        let mut flags = self.flags.load(Ordering::Relaxed) & !0b1_1000;
        if align == 1 {
            flags |= 8;
        } else if align == 2 {
            flags |= 16;
        }
        self.flags.store(flags, Ordering::Relaxed);
        self.display.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::text_display::TEXT_DISPLAY_FLAGS,
                flags,
            )],
            None,
        );
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
                    pumpkin_data::tracked_data::text_display::TEXT,
                    text,
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::text_display::LINE_WIDTH,
                    VarInt(self.line_width.load(Ordering::Relaxed)),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::text_display::BACKGROUND,
                    VarInt(self.background.load(Ordering::Relaxed)),
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::text_display::TEXT_OPACITY,
                    self.text_opacity.load(Ordering::Relaxed) as u8,
                )],
                None,
            );
            self.display.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::text_display::TEXT_DISPLAY_FLAGS,
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
