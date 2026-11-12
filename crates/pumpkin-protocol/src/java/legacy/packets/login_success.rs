//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/login/login_success.rs`.

use crate::java::client::login::CLoginSuccess;
use crate::java::legacy::LegacyWrite;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CLoginSuccess<'_> {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if version < &JavaMinecraftVersion::V_1_16 {
            write.write_string(&self.uuid.to_string())?;
        } else {
            write.write_uuid(self.uuid)?;
        }
        write.write_string(self.username)?;
        if version >= &JavaMinecraftVersion::V_1_19 {
            write.write_list(self.properties, |write, property| property.write(write))?;
        }
        if version >= &JavaMinecraftVersion::V_26_2 {
            write.write_uuid(&self.session_id)?;
        }
        if version >= &JavaMinecraftVersion::V_1_20_5 && version < &JavaMinecraftVersion::V_1_21_2 {
            write.write_bool(self.strict_error_handling)?;
        }
        Ok(())
    }
}
