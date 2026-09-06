use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bytes::BufMut;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::{TrackedData, TrackedId};
use pumpkin_protocol::java::client::play::{Metadata, MetadataSerializer};
use pumpkin_protocol::ser::WritingError;
use pumpkin_util::version::JavaMinecraftVersion;

pub trait ErasedSerializer: Send + Sync {
    fn write(
        &self,
        index: TrackedId,
        r#type: MetaDataType,
        writer: &mut dyn std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;

    fn write_canonical(&self, index: TrackedId, r#type: MetaDataType) -> Vec<u8>;
}

struct SerializerHolder<T> {
    value: T,
}

impl<T: MetadataSerializer + Clone + Send + Sync + 'static> ErasedSerializer
    for SerializerHolder<T>
{
    fn write(
        &self,
        index: TrackedId,
        r#type: MetaDataType,
        writer: &mut dyn std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let meta = Metadata::new_raw(index, r#type, &self.value);
        meta.write(writer, version)
    }

    fn write_canonical(&self, index: TrackedId, r#type: MetaDataType) -> Vec<u8> {
        let mut buf = Vec::new();
        let meta = Metadata::new_raw(index, r#type, &self.value);
        let _ = meta.write(&mut buf, &JavaMinecraftVersion::V_26_2);
        buf
    }
}

pub struct DataItem {
    pub tracked: TrackedData,
    pub serializer: Box<dyn ErasedSerializer>,
    pub canonical_bytes: Vec<u8>,
    pub dirty: bool,
    pub is_default: bool,
}

pub struct SynchedEntityData {
    items: Mutex<HashMap<u8, DataItem>>,
    is_dirty: AtomicBool,
}

impl Default for SynchedEntityData {
    fn default() -> Self {
        Self::new()
    }
}

impl SynchedEntityData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
            is_dirty: AtomicBool::new(false),
        }
    }

    pub fn define<T: MetadataSerializer + Clone + Send + Sync + 'static>(
        &self,
        tracked: TrackedData,
        value: T,
    ) {
        let id = tracked.id.v26_2;
        let holder = SerializerHolder { value };
        let canonical_bytes = holder.write_canonical(tracked.id, tracked.r#type);
        let mut items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.insert(
            id,
            DataItem {
                tracked,
                serializer: Box::new(holder),
                canonical_bytes,
                dirty: false,
                is_default: true,
            },
        );
    }

    pub fn set<T: MetadataSerializer + Clone + Send + Sync + 'static>(
        &self,
        tracked: TrackedData,
        value: T,
    ) -> bool {
        let id = tracked.id.v26_2;
        let holder = SerializerHolder { value };
        let new_canonical = holder.write_canonical(tracked.id, tracked.r#type);

        let mut items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(item) = items.get_mut(&id) {
            if item.canonical_bytes == new_canonical {
                return false;
            }
            item.canonical_bytes = new_canonical;
            item.serializer = Box::new(holder);
            item.dirty = true;
            item.is_default = false;
        } else {
            items.insert(
                id,
                DataItem {
                    tracked,
                    serializer: Box::new(holder),
                    canonical_bytes: new_canonical,
                    dirty: true,
                    is_default: false,
                },
            );
        }
        self.is_dirty.store(true, Ordering::Release);
        true
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(Ordering::Acquire)
    }

    pub fn pack_dirty_for_version(&self, version: &JavaMinecraftVersion) -> Option<Box<[u8]>> {
        self.pack_dirty_for_version_filtered(version, |_| true)
    }

    pub fn pack_dirty_for_version_filtered(
        &self,
        version: &JavaMinecraftVersion,
        include: impl Fn(TrackedData) -> bool,
    ) -> Option<Box<[u8]>> {
        if !self.is_dirty.load(Ordering::Acquire) {
            return None;
        }

        let items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut buf = Vec::new();
        let mut has_any = false;

        for item in items.values() {
            if item.dirty && include(item.tracked) {
                let before_len = buf.len();
                if item
                    .serializer
                    .write(item.tracked.id, item.tracked.r#type, &mut buf, version)
                    .is_ok()
                    && buf.len() > before_len
                {
                    has_any = true;
                }
            }
        }

        if !has_any {
            return None;
        }

        buf.put_u8(255);
        Some(buf.into_boxed_slice())
    }

    pub fn clear_dirty(&self) {
        let mut items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for item in items.values_mut() {
            item.dirty = false;
        }
        self.is_dirty.store(false, Ordering::Release);
    }

    pub fn get_non_default_values_for_version(
        &self,
        version: &JavaMinecraftVersion,
    ) -> Option<Box<[u8]>> {
        self.get_non_default_values_for_version_filtered(version, |_| true)
    }

    pub fn get_non_default_values_for_version_filtered(
        &self,
        version: &JavaMinecraftVersion,
        include: impl Fn(TrackedData) -> bool,
    ) -> Option<Box<[u8]>> {
        let items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut buf = Vec::new();
        let mut has_any = false;

        for item in items.values() {
            if !item.is_default && include(item.tracked) {
                let before_len = buf.len();
                if item
                    .serializer
                    .write(item.tracked.id, item.tracked.r#type, &mut buf, version)
                    .is_ok()
                    && buf.len() > before_len
                {
                    has_any = true;
                }
            }
        }

        if !has_any {
            return None;
        }

        buf.put_u8(255);
        Some(buf.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::player::mannequin_shared_metadata;
    use pumpkin_data::tracked_data::player;

    #[test]
    fn mannequin_snapshot_and_updates_exclude_player_only_fields_and_gravity() {
        let data = SynchedEntityData::new();
        let version = JavaMinecraftVersion::V_26_2;
        data.set(player::PLAYER_MODE_CUSTOMISATION, 0x7fu8);
        // These indices mean profile and immovability on the Java mannequin.
        data.set(player::DATA_PLAYER_ABSORPTION_ID, 4.0f32);
        data.set(player::DATA_SCORE_ID, 10i32);
        // The proxy's no-gravity setting must not be replaced by the real player's.
        data.set(player::NO_GRAVITY, false);

        let mut expected = Vec::new();
        Metadata::new(player::PLAYER_MODE_CUSTOMISATION, 0x7fu8)
            .write(&mut expected, &version)
            .unwrap();
        expected.push(255);

        assert_eq!(
            data.get_non_default_values_for_version_filtered(&version, mannequin_shared_metadata)
                .unwrap()
                .as_ref(),
            expected,
        );
        assert_eq!(
            data.pack_dirty_for_version_filtered(&version, mannequin_shared_metadata)
                .unwrap()
                .as_ref(),
            expected,
        );
        // Filtering one representation must not discard updates for ordinary player viewers.
        assert!(data.pack_dirty_for_version(&version).unwrap().len() > expected.len());
        data.clear_dirty();
        assert!(data.pack_dirty_for_version(&version).is_none());
    }

    #[test]
    fn mannequin_ignores_updates_containing_only_incompatible_metadata() {
        let data = SynchedEntityData::new();
        let version = JavaMinecraftVersion::V_26_2;
        data.set(player::DATA_PLAYER_ABSORPTION_ID, 8.0f32);
        assert!(
            data.pack_dirty_for_version_filtered(&version, mannequin_shared_metadata)
                .is_none()
        );
        assert!(
            data.get_non_default_values_for_version_filtered(&version, mannequin_shared_metadata)
                .is_none()
        );
        assert!(data.pack_dirty_for_version(&version).is_some());
    }
}
