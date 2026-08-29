use pumpkin_data::packet::clientbound::config::UPDATE_ENABLED_FEATURES;
use pumpkin_macros::java_packet;
use pumpkin_util::{resource_location::ResourceLocation, version::JavaMinecraftVersion};

use crate::{ClientPacket, ser::NetworkWriteExt};

#[java_packet(UPDATE_ENABLED_FEATURES)]
pub struct CFeatureFlags<'a> {
    pub features: &'a [ResourceLocation],
}

impl<'a> CFeatureFlags<'a> {
    #[must_use]
    pub const fn new(features: &'a [ResourceLocation]) -> Self {
        Self { features }
    }
}

impl ClientPacket for CFeatureFlags<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_list(self.features, |write, feature| write.write_string(feature))?;
        Ok(())
    }
}
