use crate::ClientPacket;
use crate::VarInt;
use crate::packet::MultiVersionJavaPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::play::{ANIMATE, SWING_ANIMATION};
use pumpkin_util::version::JavaMinecraftVersion;

/// Vanilla `SwingAnimationType.WHACK`.
const SWING_TYPE_WHACK: i32 = 1;
/// Vanilla `SwingAnimation.DEFAULT` duration in ticks.
const SWING_DEFAULT_DURATION: i32 = 6;

/// Swings an entity's arm.
///
/// 26.3 moved swings out of `animate` into their own `swing_animation` packet, which also carries
/// the animation type and duration. Older versions still get the `animate` swing actions.
pub struct CSwingAnimation {
    /// The Entity ID of the entity swinging its arm.
    pub entity_id: VarInt,
    /// Whether the off hand swings instead of the main hand.
    pub off_hand: bool,
}

impl CSwingAnimation {
    #[must_use]
    pub const fn new(entity_id: VarInt, off_hand: bool) -> Self {
        Self {
            entity_id,
            off_hand,
        }
    }
}

impl MultiVersionJavaPacket for CSwingAnimation {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        if version >= JavaMinecraftVersion::V_26_3 {
            SWING_ANIMATION.to_id(version)
        } else {
            ANIMATE.to_id(version)
        }
    }
}

impl ClientPacket for CSwingAnimation {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_var_int(&VarInt(i32::from(self.off_hand)))?;
            write.write_var_int(&VarInt(SWING_TYPE_WHACK))?;
            write.write_var_int(&VarInt(SWING_DEFAULT_DURATION))?;
        } else {
            // `animate` actions: 0 swings the main hand, 3 the off hand
            write.write_u8(if self.off_hand { 3 } else { 0 })?;
        }
        Ok(())
    }
}
