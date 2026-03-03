/// Represents a specific version of the Minecraft Java Edition protocol.
///
/// Each variant corresponds to a released client version and its associated
/// network protocol number. Ordering reflects chronological release order,
/// allowing version comparisons using standard comparison operators.
///
/// `Unknown` is used when a protocol number is not recognized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum MinecraftVersion {
    /// 1.7.2: The Update That Changed The World.
    V_1_7_2,
    V_1_7_6,
    /// 1.8: The Bountiful Update.
    V_1_8,
    /// 1.9: The Combat Update.
    V_1_9,
    V_1_9_1,
    V_1_9_2,
    V_1_9_3,
    /// 1.10: The Frostburn Update.
    V_1_10,
    /// 1.11: The Exploration Update.
    V_1_11,
    V_1_11_1,
    /// 1.12: The World of Color Update.
    V_1_12,
    V_1_12_1,
    V_1_12_2,
    /// 1.13: Update Aquatic.
    V_1_13,
    V_1_13_1,
    V_1_13_2,
    /// 1.14: Village & Pillage.
    V_1_14,
    V_1_14_1,
    V_1_14_2,
    V_1_14_3,
    V_1_14_4,
    /// 1.15: Buzzy Bees.
    V_1_15,
    V_1_15_1,
    V_1_15_2,
    /// 1.16: Nether Update.
    V_1_16,
    V_1_16_1,
    V_1_16_2,
    V_1_16_3,
    V_1_16_4,
    /// 1.17: Caves & Cliffs: Part I.
    V_1_17,
    V_1_17_1,
    /// 1.18: Caves & Cliffs: Part II.
    V_1_18,
    V_1_18_2,
    /// 1.19: The Wild Update.
    V_1_19,
    V_1_19_1,
    V_1_19_3,
    V_1_19_4,
    /// 1.20: Trails & Tales.
    V_1_20,
    V_1_20_2,
    V_1_20_3,
    /// 1.20.5: Armored Paws.
    V_1_20_5,
    /// 1.21: Tricky Trials.
    V_1_21,
    V_1_21_2,
    V_1_21_4,
    /// 1.21.5: Bundles of Bravery.
    V_1_21_5,
    V_1_21_6,
    V_1_21_7,
    V_1_21_9,
    V_1_21_11,
    /// Fallback for unrecognized protocol versions.
    Unknown,
}

impl MinecraftVersion {
    /// Returns the network protocol number for this version.
    ///
    /// Returns `-1` for [`MinecraftVersion::Unknown`].
    #[must_use]
    pub const fn protocol_version(&self) -> i32 {
        match self {
            Self::V_1_7_2 => 4,
            Self::V_1_7_6 => 5,
            Self::V_1_8 => 47,
            Self::V_1_9 => 107,
            Self::V_1_9_1 => 108,
            Self::V_1_9_2 => 109,
            Self::V_1_9_3 => 110,
            Self::V_1_10 => 210,
            Self::V_1_11 => 315,
            Self::V_1_11_1 => 316,
            Self::V_1_12 => 335,
            Self::V_1_12_1 => 338,
            Self::V_1_12_2 => 340,
            Self::V_1_13 => 393,
            Self::V_1_13_1 => 401,
            Self::V_1_13_2 => 404,
            Self::V_1_14 => 477,
            Self::V_1_14_1 => 480,
            Self::V_1_14_2 => 485,
            Self::V_1_14_3 => 490,
            Self::V_1_14_4 => 498,
            Self::V_1_15 => 573,
            Self::V_1_15_1 => 575,
            Self::V_1_15_2 => 578,
            Self::V_1_16 => 735,
            Self::V_1_16_1 => 736,
            Self::V_1_16_2 => 751,
            Self::V_1_16_3 => 753,
            Self::V_1_16_4 => 754,
            Self::V_1_17 => 755,
            Self::V_1_17_1 => 756,
            Self::V_1_18 => 757,
            Self::V_1_18_2 => 758,
            Self::V_1_19 => 759,
            Self::V_1_19_1 => 760,
            Self::V_1_19_3 => 761,
            Self::V_1_19_4 => 762,
            Self::V_1_20 => 763,
            Self::V_1_20_2 => 764,
            Self::V_1_20_3 => 765,
            Self::V_1_20_5 => 766,
            Self::V_1_21 => 767,
            Self::V_1_21_2 => 768,
            Self::V_1_21_4 => 769,
            Self::V_1_21_5 => 770,
            Self::V_1_21_6 => 771,
            Self::V_1_21_7 => 772,
            Self::V_1_21_9 => 773,
            Self::V_1_21_11 => 774,
            Self::Unknown => -1,
        }
    }

    /// Resolves a version from a network protocol number.
    ///
    /// Returns [`MinecraftVersion::Unknown`] if the protocol is not supported.
    #[must_use]
    pub const fn from_protocol(protocol: u32) -> Self {
        match protocol {
            4 => Self::V_1_7_2,
            5 => Self::V_1_7_6,
            47 => Self::V_1_8,
            107 => Self::V_1_9,
            108 => Self::V_1_9_1,
            109 => Self::V_1_9_2,
            110 => Self::V_1_9_3,
            210 => Self::V_1_10,
            315 => Self::V_1_11,
            316 => Self::V_1_11_1,
            335 => Self::V_1_12,
            338 => Self::V_1_12_1,
            340 => Self::V_1_12_2,
            393 => Self::V_1_13,
            401 => Self::V_1_13_1,
            404 => Self::V_1_13_2,
            477 => Self::V_1_14,
            480 => Self::V_1_14_1,
            485 => Self::V_1_14_2,
            490 => Self::V_1_14_3,
            498 => Self::V_1_14_4,
            573 => Self::V_1_15,
            575 => Self::V_1_15_1,
            578 => Self::V_1_15_2,
            735 => Self::V_1_16,
            736 => Self::V_1_16_1,
            751 => Self::V_1_16_2,
            753 => Self::V_1_16_3,
            754 => Self::V_1_16_4,
            755 => Self::V_1_17,
            756 => Self::V_1_17_1,
            757 => Self::V_1_18,
            758 => Self::V_1_18_2,
            759 => Self::V_1_19,
            760 => Self::V_1_19_1,
            761 => Self::V_1_19_3,
            762 => Self::V_1_19_4,
            763 => Self::V_1_20,
            764 => Self::V_1_20_2,
            765 => Self::V_1_20_3,
            766 => Self::V_1_20_5,
            767 => Self::V_1_21,
            768 => Self::V_1_21_2,
            769 => Self::V_1_21_4,
            770 => Self::V_1_21_5,
            771 => Self::V_1_21_6,
            772 => Self::V_1_21_7,
            773 => Self::V_1_21_9,
            774 => Self::V_1_21_11,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V_1_7_2 => write!(f, "1.7.2"),
            Self::V_1_7_6 => write!(f, "1.7.6"),
            Self::V_1_8 => write!(f, "1.8"),
            Self::V_1_9 => write!(f, "1.9"),
            Self::V_1_9_1 => write!(f, "1.9.1"),
            Self::V_1_9_2 => write!(f, "1.9.2"),
            Self::V_1_9_3 => write!(f, "1.9.3"),
            Self::V_1_10 => write!(f, "1.10"),
            Self::V_1_11 => write!(f, "1.11"),
            Self::V_1_11_1 => write!(f, "1.11.1"),
            Self::V_1_12 => write!(f, "1.12"),
            Self::V_1_12_1 => write!(f, "1.12.1"),
            Self::V_1_12_2 => write!(f, "1.12.2"),
            Self::V_1_13 => write!(f, "1.13"),
            Self::V_1_13_1 => write!(f, "1.13.1"),
            Self::V_1_13_2 => write!(f, "1.13.2"),
            Self::V_1_14 => write!(f, "1.14"),
            Self::V_1_14_1 => write!(f, "1.14.1"),
            Self::V_1_14_2 => write!(f, "1.14.2"),
            Self::V_1_14_3 => write!(f, "1.14.3"),
            Self::V_1_14_4 => write!(f, "1.14.4"),
            Self::V_1_15 => write!(f, "1.15"),
            Self::V_1_15_1 => write!(f, "1.15.1"),
            Self::V_1_15_2 => write!(f, "1.15.2"),
            Self::V_1_16 => write!(f, "1.16"),
            Self::V_1_16_1 => write!(f, "1.16.1"),
            Self::V_1_16_2 => write!(f, "1.16.2"),
            Self::V_1_16_3 => write!(f, "1.16.3"),
            Self::V_1_16_4 => write!(f, "1.16.4"),
            Self::V_1_17 => write!(f, "1.17"),
            Self::V_1_17_1 => write!(f, "1.17.1"),
            Self::V_1_18 => write!(f, "1.18"),
            Self::V_1_18_2 => write!(f, "1.18.2"),
            Self::V_1_19 => write!(f, "1.19"),
            Self::V_1_19_1 => write!(f, "1.19.1"),
            Self::V_1_19_3 => write!(f, "1.19.3"),
            Self::V_1_19_4 => write!(f, "1.19.4"),
            Self::V_1_20 => write!(f, "1.20"),
            Self::V_1_20_2 => write!(f, "1.20.2"),
            Self::V_1_20_3 => write!(f, "1.20.3"),
            Self::V_1_20_5 => write!(f, "1.20.5"),
            Self::V_1_21 => write!(f, "1.21"),
            Self::V_1_21_2 => write!(f, "1.21.2"),
            Self::V_1_21_4 => write!(f, "1.21.4"),
            Self::V_1_21_5 => write!(f, "1.21.5"),
            Self::V_1_21_6 => write!(f, "1.21.6"),
            Self::V_1_21_7 => write!(f, "1.21.7"),
            Self::V_1_21_9 => write!(f, "1.21.9"),
            Self::V_1_21_11 => write!(f, "1.21.11"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
