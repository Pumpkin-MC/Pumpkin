//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/config/add_resource_pack.rs`.

use crate::java::client::config::CConfigAddResourcePack;
use crate::java::legacy::{LegacyWrite, LegacyWriteExt};
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CConfigAddResourcePack<'_> {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_20_3 {
            write.write_uuid(self.uuid)?;
        }
        write.write_string(self.url)?;
        write.write_string(self.hash)?;
        write.write_bool(self.forced)?;
        if let Some(prompt) = &self.prompt_message {
            write.write_bool(true)?;
            write.write_component_legacy(prompt, version)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
