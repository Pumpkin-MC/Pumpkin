//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_info_update.rs`.

use crate::java::client::play::CPlayerInfoUpdate;
use crate::java::client::play::PlayerInfoFlags;
use crate::java::legacy::{LegacyWrite, LegacyWriteExt};
use crate::ser::WritingError;
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CPlayerInfoUpdate<'_> {
    fn write_legacy(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // List order was added in 1.21.2 and the hat in 1.21.4
        let mut flags = PlayerInfoFlags::from_bits_truncate(self.actions);
        if *version < JavaMinecraftVersion::V_1_21_2 {
            flags.remove(PlayerInfoFlags::UPDATE_LIST_PRIORITY);
        }
        if *version < JavaMinecraftVersion::V_1_21_4 {
            flags.remove(PlayerInfoFlags::UPDATE_HAT);
        }
        self.write_flagged(write, flags, |p, name| {
            p.write_component_legacy(name, version)
        })
    }
}
