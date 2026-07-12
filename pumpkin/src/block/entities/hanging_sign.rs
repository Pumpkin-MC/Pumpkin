use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use super::BlockEntity;
use crate::block::entities::sign::Text;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

pub struct HangingSignBlockEntity {
    pub front_text: Text,
    pub back_text: Text,
    pub is_waxed: AtomicBool,
    position: BlockPos,
    pub currently_editing_player: Arc<Mutex<Option<uuid::Uuid>>>,
}

impl BlockEntity for HangingSignBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let front_text = nbt
            .get("front_text")
            .cloned()
            .map(Text::from)
            .unwrap_or_default();
        let back_text = nbt
            .get("back_text")
            .cloned()
            .map(Text::from)
            .unwrap_or_default();
        let is_waxed = nbt.get_bool("is_waxed").unwrap_or(false);
        Self {
            position,
            front_text,
            back_text,
            is_waxed: AtomicBool::new(is_waxed),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put("front_text", self.front_text.clone());
            nbt.put("back_text", self.back_text.clone());
            nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HangingSignBlockEntity {
    pub const ID: &'static str = "minecraft:hanging_sign";
    #[must_use]
    pub fn new(position: BlockPos, is_front: bool, messages: [Box<str>; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: if is_front {
                Text::new(messages.clone())
            } else {
                Text::default()
            },
            back_text: if is_front {
                Text::default()
            } else {
                Text::new(messages)
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
    #[must_use]
    pub fn empty(position: BlockPos) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: Text::default(),
            back_text: Text::default(),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
}
