use std::sync::atomic::{AtomicI8, AtomicI32, AtomicU8, Ordering};

use arc_swap::ArcSwap;
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::entity::decoration::display::DisplayEntity;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};

const FLAG_SHADOW: u8 = 1;
const FLAG_SEE_THROUGH: u8 = 2;
const FLAG_USE_DEFAULT_BACKGROUND: u8 = 4;
const FLAG_ALIGN_LEFT: u8 = 8;
const FLAG_ALIGN_RIGHT: u8 = 16;

pub struct TextDisplayEntity {
    display: DisplayEntity,
    text: ArcSwap<TextComponent>,
    line_width: AtomicI32,
    background_color: AtomicI32,
    text_opacity: AtomicI8,
    style_flags: AtomicU8,
}

impl TextDisplayEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            display: DisplayEntity::new(entity),
            text: ArcSwap::new(std::sync::Arc::new(TextComponent::text(""))),
            line_width: AtomicI32::new(200),
            background_color: AtomicI32::new(1_073_741_824),
            text_opacity: AtomicI8::new(-1),
            style_flags: AtomicU8::new(0),
        }
    }
}

impl NBTStorage for TextDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_nbt(nbt).await;

            if let Ok(text_json) = pumpkin_util::serde_json::to_string(&**self.text.load()) {
                nbt.put_string("text", text_json);
            }
            nbt.put_int("line_width", self.line_width.load(Ordering::Relaxed));
            nbt.put_int("background", self.background_color.load(Ordering::Relaxed));
            nbt.put_byte("text_opacity", self.text_opacity.load(Ordering::Relaxed));

            let flags = self.style_flags.load(Ordering::Relaxed);
            nbt.put_bool("shadow", flags & FLAG_SHADOW != 0);
            nbt.put_bool("see_through", flags & FLAG_SEE_THROUGH != 0);
            nbt.put_bool(
                "default_background",
                flags & FLAG_USE_DEFAULT_BACKGROUND != 0,
            );
            let alignment = if flags & FLAG_ALIGN_LEFT != 0 {
                "left"
            } else if flags & FLAG_ALIGN_RIGHT != 0 {
                "right"
            } else {
                "center"
            };
            nbt.put_string("alignment", alignment.to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_nbt_non_mut(nbt).await;

            if let Some(text_json) = nbt.get_string("text")
                && let Ok(component) = pumpkin_util::serde_json::from_str(text_json)
            {
                self.text.store(std::sync::Arc::new(component));
            }
            self.line_width
                .store(nbt.get_int("line_width").unwrap_or(200), Ordering::Relaxed);
            self.background_color.store(
                nbt.get_int("background").unwrap_or(1_073_741_824),
                Ordering::Relaxed,
            );
            self.text_opacity.store(
                nbt.get_byte("text_opacity").unwrap_or(-1),
                Ordering::Relaxed,
            );

            let mut flags = 0u8;
            if nbt.get_bool("shadow").unwrap_or(false) {
                flags |= FLAG_SHADOW;
            }
            if nbt.get_bool("see_through").unwrap_or(false) {
                flags |= FLAG_SEE_THROUGH;
            }
            if nbt.get_bool("default_background").unwrap_or(false) {
                flags |= FLAG_USE_DEFAULT_BACKGROUND;
            }
            match nbt.get_string("alignment") {
                Some("left") => flags |= FLAG_ALIGN_LEFT,
                Some("right") => flags |= FLAG_ALIGN_RIGHT,
                _ => {}
            }
            self.style_flags.store(flags, Ordering::Relaxed);
        })
    }
}

impl EntityBase for TextDisplayEntity {
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

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.send_metadata();

            self.display.entity.send_meta_data(
                &[
                    Metadata::new(
                        TrackedData::TEXT,
                        MetaDataType::TEXT_COMPONENT,
                        (**self.text.load()).clone(),
                    ),
                    Metadata::new(
                        TrackedData::TEXT_ID,
                        MetaDataType::COMPONENT,
                        (**self.text.load()).clone(),
                    ),
                ],
                None,
            );

            let ints = [
                Metadata::new(
                    TrackedData::LINE_WIDTH,
                    MetaDataType::INTEGER,
                    self.line_width.load(Ordering::Relaxed),
                ),
                Metadata::new(
                    TrackedData::LINE_WIDTH_ID,
                    MetaDataType::INTEGER,
                    self.line_width.load(Ordering::Relaxed),
                ),
                Metadata::new(
                    TrackedData::BACKGROUND,
                    MetaDataType::INTEGER,
                    self.background_color.load(Ordering::Relaxed),
                ),
                Metadata::new(
                    TrackedData::BACKGROUND_COLOR_ID,
                    MetaDataType::INTEGER,
                    self.background_color.load(Ordering::Relaxed),
                ),
            ];
            self.display.entity.send_meta_data(&ints, None);

            let bytes = [
                Metadata::new(
                    TrackedData::TEXT_OPACITY,
                    MetaDataType::BYTE,
                    self.text_opacity.load(Ordering::Relaxed) as u8,
                ),
                Metadata::new(
                    TrackedData::TEXT_OPACITY_ID,
                    MetaDataType::BYTE,
                    self.text_opacity.load(Ordering::Relaxed) as u8,
                ),
                Metadata::new(
                    TrackedData::TEXT_DISPLAY_FLAGS,
                    MetaDataType::BYTE,
                    self.style_flags.load(Ordering::Relaxed),
                ),
                Metadata::new(
                    TrackedData::STYLE_FLAGS_ID,
                    MetaDataType::BYTE,
                    self.style_flags.load(Ordering::Relaxed),
                ),
            ];
            self.display.entity.send_meta_data(&bytes, None);
        })
    }

    /// `Display.hurtServer` always returns false; display entities cannot be damaged.
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { false })
    }
}
