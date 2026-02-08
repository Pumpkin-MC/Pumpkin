//! Player data NBT helpers.
//!
//! Provides utility functions for encoding and decoding Minecraft player data
//! between Rust types and NBT format. These helpers match the vanilla Minecraft
//! player data file format (`.dat` files in the `playerdata/` directory).
//!
//! # UUID Encoding
//!
//! Minecraft stores UUIDs as `IntArray` with 4 elements (most-significant
//! first), not as strings.
//!
//! ```
//! use pumpkin_nbt::player_data::{uuid_to_int_array, uuid_from_int_array};
//!
//! let uuid = 0x12345678_9abcdef0_12345678_9abcdef0u128;
//! let arr = uuid_to_int_array(uuid);
//! assert_eq!(uuid_from_int_array(&arr), Some(uuid));
//! ```
//!
//! # Position and Rotation
//!
//! Position is stored as a `List` of 3 `Double` values `[x, y, z]`.
//! Rotation is stored as a `List` of 2 `Float` values `[yaw, pitch]`.
//! Motion/velocity uses the same 3-double format as position.

use crate::compound::NbtCompound;
use crate::tag::NbtTag;

/// Encode a UUID as an `IntArray` of 4 i32 values (most-significant first).
///
/// This matches Minecraft's NBT UUID encoding: the 128-bit UUID is split
/// into four 32-bit signed integers in big-endian order.
#[must_use]
pub const fn uuid_to_int_array(uuid: u128) -> [i32; 4] {
    [
        ((uuid >> 96) & 0xFFFF_FFFF) as i32,
        ((uuid >> 64) & 0xFFFF_FFFF) as i32,
        ((uuid >> 32) & 0xFFFF_FFFF) as i32,
        (uuid & 0xFFFF_FFFF) as i32,
    ]
}

/// Decode a UUID from an `IntArray` of 4 i32 values.
///
/// Returns `None` if the slice does not contain exactly 4 elements.
#[must_use]
pub fn uuid_from_int_array(arr: &[i32]) -> Option<u128> {
    if arr.len() != 4 {
        return None;
    }
    let a = (arr[0] as u32) as u128;
    let b = (arr[1] as u32) as u128;
    let c = (arr[2] as u32) as u128;
    let d = (arr[3] as u32) as u128;
    Some((a << 96) | (b << 64) | (c << 32) | d)
}

/// Encode a UUID into an [`NbtTag::IntArray`].
#[must_use]
pub fn uuid_to_nbt(uuid: u128) -> NbtTag {
    let arr = uuid_to_int_array(uuid);
    NbtTag::IntArray(arr.to_vec())
}

/// Decode a UUID from an [`NbtTag::IntArray`].
///
/// Returns `None` if the tag is not an `IntArray` or does not contain
/// exactly 4 elements.
#[must_use]
pub fn uuid_from_nbt(tag: &NbtTag) -> Option<u128> {
    tag.extract_int_array().and_then(uuid_from_int_array)
}

/// Encode a 3D position as an [`NbtTag::List`] of 3 `Double` values.
///
/// Matches Minecraft's `Pos` tag format: `[x, y, z]`.
#[must_use]
pub fn position_to_nbt(x: f64, y: f64, z: f64) -> NbtTag {
    NbtTag::List(vec![NbtTag::Double(x), NbtTag::Double(y), NbtTag::Double(z)])
}

/// Decode a 3D position from an [`NbtTag::List`] of 3 `Double` values.
///
/// Returns `None` if the tag is not a `List` of exactly 3 `Double` elements.
#[must_use]
pub fn position_from_nbt(tag: &NbtTag) -> Option<(f64, f64, f64)> {
    let list = tag.extract_list()?;
    if list.len() != 3 {
        return None;
    }
    let x = list[0].extract_double()?;
    let y = list[1].extract_double()?;
    let z = list[2].extract_double()?;
    Some((x, y, z))
}

/// Encode a rotation as an [`NbtTag::List`] of 2 `Float` values.
///
/// Matches Minecraft's `Rotation` tag format: `[yaw, pitch]`.
#[must_use]
pub fn rotation_to_nbt(yaw: f32, pitch: f32) -> NbtTag {
    NbtTag::List(vec![NbtTag::Float(yaw), NbtTag::Float(pitch)])
}

/// Decode a rotation from an [`NbtTag::List`] of 2 `Float` values.
///
/// Returns `None` if the tag is not a `List` of exactly 2 `Float` elements.
#[must_use]
pub fn rotation_from_nbt(tag: &NbtTag) -> Option<(f32, f32)> {
    let list = tag.extract_list()?;
    if list.len() != 2 {
        return None;
    }
    let yaw = list[0].extract_float()?;
    let pitch = list[1].extract_float()?;
    Some((yaw, pitch))
}

/// Encode a 3D velocity/motion as an [`NbtTag::List`] of 3 `Double` values.
///
/// Matches Minecraft's `Motion` tag format: `[dx, dy, dz]`.
#[must_use]
pub fn motion_to_nbt(dx: f64, dy: f64, dz: f64) -> NbtTag {
    position_to_nbt(dx, dy, dz)
}

/// Decode a 3D velocity/motion from an [`NbtTag::List`] of 3 `Double` values.
///
/// Returns `None` if the tag is not a `List` of exactly 3 `Double` elements.
#[must_use]
pub fn motion_from_nbt(tag: &NbtTag) -> Option<(f64, f64, f64)> {
    position_from_nbt(tag)
}

/// Standard entity fields matching vanilla Minecraft's NBT format.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBase {
    pub uuid: u128,
    pub pos: (f64, f64, f64),
    pub motion: (f64, f64, f64),
    pub rotation: (f32, f32),
    pub on_ground: bool,
    pub fire_ticks: i16,
}

impl EntityBase {
    /// Write this entity's fields into an [`NbtCompound`].
    pub fn write_to(&self, compound: &mut NbtCompound) {
        compound.put("UUID", uuid_to_nbt(self.uuid));
        compound.put("Pos", position_to_nbt(self.pos.0, self.pos.1, self.pos.2));
        compound.put(
            "Motion",
            motion_to_nbt(self.motion.0, self.motion.1, self.motion.2),
        );
        compound.put(
            "Rotation",
            rotation_to_nbt(self.rotation.0, self.rotation.1),
        );
        compound.put_bool("OnGround", self.on_ground);
        compound.put_short("Fire", self.fire_ticks);
    }

    /// Read entity fields from an [`NbtCompound`].
    ///
    /// Returns `None` if any required field is missing or has the wrong type.
    #[must_use]
    pub fn read_from(compound: &NbtCompound) -> Option<Self> {
        Some(Self {
            uuid: compound.get("UUID").and_then(uuid_from_nbt)?,
            pos: compound.get("Pos").and_then(position_from_nbt)?,
            motion: compound.get("Motion").and_then(motion_from_nbt)?,
            rotation: compound.get("Rotation").and_then(rotation_from_nbt)?,
            on_ground: compound.get_bool("OnGround")?,
            fire_ticks: compound.get_short("Fire")?,
        })
    }
}

/// Player abilities matching vanilla Minecraft's NBT format.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub may_fly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl PlayerAbilities {
    /// Write abilities as an `abilities` sub-compound into the given compound.
    pub fn write_to(&self, compound: &mut NbtCompound) {
        let mut abilities = NbtCompound::new();
        abilities.put_bool("invulnerable", self.invulnerable);
        abilities.put_bool("flying", self.flying);
        abilities.put_bool("mayfly", self.may_fly);
        abilities.put_bool("instabuild", self.instabuild);
        abilities.put_bool("mayBuild", self.may_build);
        abilities.put_float("flySpeed", self.fly_speed);
        abilities.put_float("walkSpeed", self.walk_speed);
        compound.put_component("abilities", abilities);
    }

    /// Read abilities from the `abilities` sub-compound.
    ///
    /// Returns `None` if the compound is missing or incomplete.
    #[must_use]
    pub fn read_from(compound: &NbtCompound) -> Option<Self> {
        let abilities = compound.get_compound("abilities")?;
        Some(Self {
            invulnerable: abilities.get_bool("invulnerable")?,
            flying: abilities.get_bool("flying")?,
            may_fly: abilities.get_bool("mayfly")?,
            instabuild: abilities.get_bool("instabuild")?,
            may_build: abilities.get_bool("mayBuild")?,
            fly_speed: abilities.get_float("flySpeed")?,
            walk_speed: abilities.get_float("walkSpeed")?,
        })
    }
}

/// Encode a game mode as a byte value for NBT storage.
///
/// Matches Minecraft's `playerGameType` encoding:
/// - 0 = Survival
/// - 1 = Creative
/// - 2 = Adventure
/// - 3 = Spectator
#[must_use]
pub const fn game_mode_to_byte(mode: u8) -> i8 {
    mode as i8
}

/// Decode a game mode from a byte value.
///
/// Returns `None` if the value is not a valid game mode (0-3).
#[must_use]
pub const fn game_mode_from_byte(byte: i8) -> Option<u8> {
    match byte {
        0..=3 => Some(byte as u8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_roundtrip() {
        let uuid = 0x12345678_9ABCDEF0_12345678_9ABCDEF0u128;
        let arr = uuid_to_int_array(uuid);
        assert_eq!(uuid_from_int_array(&arr), Some(uuid));
    }

    #[test]
    fn uuid_zero() {
        let arr = uuid_to_int_array(0);
        assert_eq!(arr, [0, 0, 0, 0]);
        assert_eq!(uuid_from_int_array(&arr), Some(0));
    }

    #[test]
    fn uuid_max() {
        let uuid = u128::MAX;
        let arr = uuid_to_int_array(uuid);
        assert_eq!(arr, [-1, -1, -1, -1]); // all 0xFFFFFFFF as i32
        assert_eq!(uuid_from_int_array(&arr), Some(uuid));
    }

    #[test]
    fn uuid_wrong_length() {
        assert_eq!(uuid_from_int_array(&[1, 2, 3]), None);
        assert_eq!(uuid_from_int_array(&[1, 2, 3, 4, 5]), None);
        assert_eq!(uuid_from_int_array(&[]), None);
    }

    #[test]
    fn uuid_nbt_roundtrip() {
        let uuid = 0xDEADBEEF_CAFEBABE_12345678_87654321u128;
        let tag = uuid_to_nbt(uuid);
        assert_eq!(uuid_from_nbt(&tag), Some(uuid));
    }

    #[test]
    fn uuid_nbt_wrong_type() {
        assert_eq!(uuid_from_nbt(&NbtTag::Int(42)), None);
        assert_eq!(uuid_from_nbt(&NbtTag::String("test".into())), None);
    }

    #[test]
    fn position_roundtrip() {
        let tag = position_to_nbt(1.5, 64.0, -3.25);
        assert_eq!(position_from_nbt(&tag), Some((1.5, 64.0, -3.25)));
    }

    #[test]
    fn position_wrong_length() {
        let tag = NbtTag::List(vec![NbtTag::Double(1.0), NbtTag::Double(2.0)]);
        assert_eq!(position_from_nbt(&tag), None);
    }

    #[test]
    fn position_wrong_type() {
        let tag = NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)]);
        assert_eq!(position_from_nbt(&tag), None);
    }

    #[test]
    fn rotation_roundtrip() {
        let tag = rotation_to_nbt(90.0, -45.5);
        assert_eq!(rotation_from_nbt(&tag), Some((90.0, -45.5)));
    }

    #[test]
    fn rotation_wrong_length() {
        let tag = NbtTag::List(vec![NbtTag::Float(1.0)]);
        assert_eq!(rotation_from_nbt(&tag), None);
    }

    #[test]
    fn motion_roundtrip() {
        let tag = motion_to_nbt(0.1, -0.5, 0.3);
        assert_eq!(motion_from_nbt(&tag), Some((0.1, -0.5, 0.3)));
    }

    #[test]
    fn entity_base_roundtrip() {
        let mut compound = NbtCompound::new();
        let entity = EntityBase {
            uuid: 0xABCDEF01_23456789_ABCDEF01_23456789u128,
            pos: (100.5, 64.0, -200.25),
            motion: (0.1, -0.5, 0.3),
            rotation: (90.0f32, -45.0f32),
            on_ground: true,
            fire_ticks: 100,
        };
        entity.write_to(&mut compound);

        let parsed = EntityBase::read_from(&compound).unwrap();
        assert_eq!(parsed, entity);
    }

    #[test]
    fn entity_base_missing_field() {
        let compound = NbtCompound::new();
        assert!(EntityBase::read_from(&compound).is_none());
    }

    #[test]
    fn abilities_roundtrip() {
        let mut compound = NbtCompound::new();
        let abilities = PlayerAbilities {
            invulnerable: false,
            flying: true,
            may_fly: true,
            instabuild: false,
            may_build: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        };
        abilities.write_to(&mut compound);

        let parsed = PlayerAbilities::read_from(&compound).unwrap();
        assert_eq!(parsed, abilities);
    }

    #[test]
    fn abilities_missing() {
        let compound = NbtCompound::new();
        assert!(PlayerAbilities::read_from(&compound).is_none());
    }

    #[test]
    fn game_mode_valid() {
        assert_eq!(game_mode_from_byte(0), Some(0)); // Survival
        assert_eq!(game_mode_from_byte(1), Some(1)); // Creative
        assert_eq!(game_mode_from_byte(2), Some(2)); // Adventure
        assert_eq!(game_mode_from_byte(3), Some(3)); // Spectator
    }

    #[test]
    fn game_mode_invalid() {
        assert_eq!(game_mode_from_byte(-1), None);
        assert_eq!(game_mode_from_byte(4), None);
        assert_eq!(game_mode_from_byte(127), None);
    }

    #[test]
    fn full_player_compound() {
        let mut compound = NbtCompound::new();

        // Entity base
        let entity = EntityBase {
            uuid: 0x01234567_89ABCDEF_01234567_89ABCDEFu128,
            pos: (100.0, 65.0, -200.0),
            motion: (0.0, 0.0, 0.0),
            rotation: (180.0, 0.0),
            on_ground: true,
            fire_ticks: 0,
        };
        entity.write_to(&mut compound);

        // Player-specific
        compound.put_int("DataVersion", 4671);
        compound.put_byte("playerGameType", game_mode_to_byte(1));
        compound.put_bool("HasPlayedBefore", true);
        compound.put_string("Dimension", "minecraft:overworld".to_string());
        compound.put_int("XpTotal", 1000);

        // Hunger
        compound.put_int("foodLevel", 20);
        compound.put_float("foodSaturationLevel", 5.0);
        compound.put_float("foodExhaustionLevel", 0.0);
        compound.put_int("foodTickTimer", 0);

        // Health
        compound.put_float("Health", 20.0);

        // Abilities
        let abilities = PlayerAbilities {
            invulnerable: false,
            flying: false,
            may_fly: true,
            instabuild: true,
            may_build: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        };
        abilities.write_to(&mut compound);

        // Spawn point
        compound.put_int("SpawnX", 0);
        compound.put_int("SpawnY", 64);
        compound.put_int("SpawnZ", 0);

        // Verify all fields
        assert_eq!(compound.get_int("DataVersion"), Some(4671));
        assert_eq!(compound.get_byte("playerGameType"), Some(1));
        assert_eq!(
            game_mode_from_byte(compound.get_byte("playerGameType").unwrap()),
            Some(1)
        );
        assert_eq!(compound.get_bool("HasPlayedBefore"), Some(true));
        assert_eq!(compound.get_string("Dimension"), Some("minecraft:overworld"));
        assert_eq!(compound.get_int("XpTotal"), Some(1000));
        assert_eq!(compound.get_int("foodLevel"), Some(20));
        assert_eq!(compound.get_float("Health"), Some(20.0));
        assert_eq!(compound.get_int("SpawnX"), Some(0));

        let parsed_entity = EntityBase::read_from(&compound).unwrap();
        assert_eq!(parsed_entity.pos, (100.0, 65.0, -200.0));

        let parsed_abilities = PlayerAbilities::read_from(&compound).unwrap();
        assert!(parsed_abilities.instabuild);
    }

    #[test]
    fn nbt_binary_roundtrip() {
        use crate::Nbt;

        let mut compound = NbtCompound::new();
        let uuid = 0xDEADBEEF_12345678_CAFEBABE_87654321u128;
        let entity = EntityBase {
            uuid,
            pos: (1.0, 2.0, 3.0),
            motion: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0),
            on_ground: false,
            fire_ticks: 0,
        };
        entity.write_to(&mut compound);

        let abilities = PlayerAbilities {
            invulnerable: false,
            flying: false,
            may_fly: false,
            instabuild: false,
            may_build: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        };
        abilities.write_to(&mut compound);

        // Serialize to NBT binary
        let nbt = Nbt::new(String::new(), compound);
        let bytes = nbt.write();

        // Deserialize back
        let mut cursor = std::io::Cursor::new(bytes.to_vec());
        let mut reader = crate::deserializer::NbtReadHelper::new(&mut cursor);
        let parsed = Nbt::read(&mut reader).unwrap();

        // Verify UUID survives roundtrip
        let re_uuid = parsed.root_tag.get("UUID").and_then(uuid_from_nbt);
        assert_eq!(re_uuid, Some(uuid));

        // Verify abilities survive roundtrip
        let parsed_abilities = PlayerAbilities::read_from(&parsed.root_tag);
        assert!(parsed_abilities.is_some());
        assert!(parsed_abilities.unwrap().may_build);
    }
}
