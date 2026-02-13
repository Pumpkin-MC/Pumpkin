/* This file is generated. Do not edit manually. */
use crate::tag::{RegistryKey, Taggable};
use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};
use std::hash::{Hash, Hasher};
#[derive(Clone)]
pub struct PartialFluidState {
    pub height: f32,
    pub level: i16,
    pub is_empty: bool,
    pub blast_resistance: f32,
    pub block_state_id: u16,
    pub is_still: bool,
    pub is_source: bool,
    pub falling: bool,
}
#[derive(Clone)]
pub struct FluidState {
    pub height: f32,
    pub level: i16,
    pub is_empty: bool,
    pub blast_resistance: f32,
    pub block_state_id: u16,
    pub is_still: bool,
    pub is_source: bool,
    pub falling: bool,
}
#[derive(Clone)]
pub struct FluidStateRef {
    pub id: u16,
    pub state_idx: u16,
}
#[derive(Clone)]
pub struct Fluid {
    pub id: u16,
    pub name: &'static str,
    pub properties: Option<&'static [(&'static str, &'static [&'static str])]>,
    pub states: &'static [FluidState],
    pub default_state_index: u16,
    pub flow_speed: u32,
    pub flow_distance: u32,
    pub can_convert_to_source: bool,
}
impl Hash for Fluid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Fluid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Fluid {}
pub const FLUID_STATES: &[PartialFluidState] = &[
    PartialFluidState {
        height: 0f32,
        level: 0i16,
        is_empty: true,
        blast_resistance: 0f32,
        block_state_id: 0u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.11111111f32,
        level: 1i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 101u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.22222222f32,
        level: 2i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 100u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.33333334f32,
        level: 3i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 99u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.44444445f32,
        level: 4i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 98u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.5555556f32,
        level: 5i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 97u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.6666667f32,
        level: 6i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 96u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.7777778f32,
        level: 7i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 95u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.8888889f32,
        level: 8i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 94u16,
        is_still: false,
        is_source: false,
        falling: false,
    },
    PartialFluidState {
        height: 0.8888889f32,
        level: 8i16,
        is_empty: false,
        blast_resistance: 100f32,
        block_state_id: 86u16,
        is_still: true,
        is_source: true,
        falling: false,
    },
];
pub trait EnumVariants {
    fn variant_count() -> u16;
    fn to_index(&self) -> u16;
    fn from_index(index: u16) -> Self;
    fn to_value(&self) -> &str;
    fn from_value(value: &str) -> Self;
}
pub trait FluidProperties
where
    Self: 'static,
{
    fn to_index(&self) -> u16;
    fn from_index(index: u16) -> Self
    where
        Self: Sized;
    fn to_state_id(&self, fluid: &Fluid) -> u16;
    fn from_state_id(state_id: u16, fluid: &Fluid) -> Self
    where
        Self: Sized;
    fn default(fluid: &Fluid) -> Self
    where
        Self: Sized;
    fn to_props(&self) -> Vec<(String, String)>;
    fn from_props(props: Vec<(String, String)>, fluid: &Fluid) -> Self
    where
        Self: Sized;
}
pub fn get_fluid(registry_id: &str) -> Option<&'static Fluid> {
    let key = registry_id
        .strip_prefix("minecraft:")
        .unwrap_or(registry_id);
    Fluid::from_registry_key(key)
}
impl Fluid {
    pub const EMPTY: Fluid = Fluid {
        id: 0,
        name: "empty",
        properties: None,
        states: &[FluidState {
            height: 0f32,
            level: 0i16,
            is_empty: true,
            blast_resistance: 0f32,
            block_state_id: 0u16,
            is_still: false,
            is_source: false,
            falling: false,
        }],
        default_state_index: 0u16,
        flow_speed: 5u32,
        flow_distance: 4u32,
        can_convert_to_source: false,
    };
    pub const FLOWING_WATER: Fluid = Fluid {
        id: 1,
        name: "flowing_water",
        properties: Some(&[
            ("falling", &["true", "false"]),
            ("level", &["1", "2", "3", "4", "5", "6", "7", "8"]),
        ]),
        states: &[
            FluidState {
                height: 0.11111111f32,
                level: 1i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 101u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.22222222f32,
                level: 2i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 100u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.33333334f32,
                level: 3i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 99u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.44444445f32,
                level: 4i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 98u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.5555556f32,
                level: 5i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 97u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.6666667f32,
                level: 6i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 96u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.7777778f32,
                level: 7i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 95u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 94u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.11111111f32,
                level: 1i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 93u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.22222222f32,
                level: 2i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 92u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.33333334f32,
                level: 3i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 91u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.44444445f32,
                level: 4i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 90u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.5555556f32,
                level: 5i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 89u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.6666667f32,
                level: 6i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 88u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.7777778f32,
                level: 7i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 87u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 86u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
        ],
        default_state_index: 0u16,
        flow_speed: 5u32,
        flow_distance: 4u32,
        can_convert_to_source: false,
    };
    pub const WATER: Fluid = Fluid {
        id: 2,
        name: "water",
        properties: Some(&[("falling", &["true", "false"])]),
        states: &[
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 86u16,
                is_still: true,
                is_source: true,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 86u16,
                is_still: true,
                is_source: true,
                falling: false,
            },
        ],
        default_state_index: 0u16,
        flow_speed: 5u32,
        flow_distance: 4u32,
        can_convert_to_source: false,
    };
    pub const FLOWING_LAVA: Fluid = Fluid {
        id: 3,
        name: "flowing_lava",
        properties: Some(&[
            ("falling", &["true", "false"]),
            ("level", &["1", "2", "3", "4", "5", "6", "7", "8"]),
        ]),
        states: &[
            FluidState {
                height: 0.11111111f32,
                level: 1i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 117u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.22222222f32,
                level: 2i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 116u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.33333334f32,
                level: 3i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 115u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.44444445f32,
                level: 4i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 114u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.5555556f32,
                level: 5i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 113u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.6666667f32,
                level: 6i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 112u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.7777778f32,
                level: 7i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 111u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 110u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.11111111f32,
                level: 1i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 109u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.22222222f32,
                level: 2i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 108u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.33333334f32,
                level: 3i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 107u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.44444445f32,
                level: 4i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 106u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.5555556f32,
                level: 5i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 105u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.6666667f32,
                level: 6i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 104u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.7777778f32,
                level: 7i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 103u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 102u16,
                is_still: false,
                is_source: false,
                falling: false,
            },
        ],
        default_state_index: 0u16,
        flow_speed: 5u32,
        flow_distance: 4u32,
        can_convert_to_source: false,
    };
    pub const LAVA: Fluid = Fluid {
        id: 4,
        name: "lava",
        properties: Some(&[("falling", &["true", "false"])]),
        states: &[
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 102u16,
                is_still: true,
                is_source: true,
                falling: false,
            },
            FluidState {
                height: 0.8888889f32,
                level: 8i16,
                is_empty: false,
                blast_resistance: 100f32,
                block_state_id: 102u16,
                is_still: true,
                is_source: true,
                falling: false,
            },
        ],
        default_state_index: 0u16,
        flow_speed: 5u32,
        flow_distance: 4u32,
        can_convert_to_source: false,
    };
    pub fn from_registry_key(name: &str) -> Option<&'static Self> {
        match name {
            "empty" => Some(&Self::EMPTY),
            "flowing_water" => Some(&Self::FLOWING_WATER),
            "water" => Some(&Self::WATER),
            "flowing_lava" => Some(&Self::FLOWING_LAVA),
            "lava" => Some(&Self::LAVA),
            _ => None,
        }
    }
    pub const fn from_id(id: u16) -> Option<&'static Self> {
        match id {
            0 => Some(&Self::EMPTY),
            1 => Some(&Self::FLOWING_WATER),
            2 => Some(&Self::WATER),
            3 => Some(&Self::FLOWING_LAVA),
            4 => Some(&Self::LAVA),
            _ => None,
        }
    }
    #[allow(unreachable_patterns)]
    pub const fn from_state_id(id: u16) -> Option<&'static Self> {
        match id {
            0u16..=0u16 => Some(&Fluid::EMPTY),
            86u16..=86u16 => Some(&Fluid::WATER),
            86u16..=101u16 => Some(&Fluid::FLOWING_WATER),
            102u16..=102u16 => Some(&Fluid::LAVA),
            102u16..=117u16 => Some(&Fluid::FLOWING_LAVA),
            _ => None,
        }
    }
    pub fn same_fluid_type(a: u16, b: u16) -> bool {
        a == b
            || (a == 1 && b == 2)
            || (a == 2 && b == 1)
            || (a == 3 && b == 4)
            || (a == 4 && b == 3)
    }
    pub fn matches_type(&self, other: &Fluid) -> bool {
        Self::same_fluid_type(self.id, other.id)
    }
    pub fn to_flowing(&self) -> &'static Fluid {
        match self.id {
            2 => &Fluid::FLOWING_WATER,
            4 => &Fluid::FLOWING_LAVA,
            _ => Fluid::from_id(self.id).unwrap_or(&Fluid::EMPTY),
        }
    }
    pub fn ident_to_fluid_id(name: &str) -> Option<u8> {
        match name {
            "empty" => Some(0),
            "flowing_water" => Some(1),
            "water" => Some(2),
            "flowing_lava" => Some(3),
            "lava" => Some(4),
            _ => None,
        }
    }
    #[track_caller]
    #[doc = r" Get the properties of the fluid."]
    pub fn properties(&self, state_id: u16) -> Box<dyn FluidProperties> {
        match self.name {
            "water" => Box::new(WaterLikeFluidProperties::from_state_id(
                state_id,
                &Fluid::WATER,
            )),
            "lava" => Box::new(WaterLikeFluidProperties::from_state_id(
                state_id,
                &Fluid::LAVA,
            )),
            "flowing_water" => Box::new(FlowingWaterLikeFluidProperties::from_state_id(
                state_id,
                &Fluid::FLOWING_WATER,
            )),
            "flowing_lava" => Box::new(FlowingWaterLikeFluidProperties::from_state_id(
                state_id,
                &Fluid::FLOWING_LAVA,
            )),
            _ => panic!("Invalid state_id"),
        }
    }
    #[track_caller]
    #[doc = r" Get the properties of the fluid."]
    pub fn from_properties(&self, props: Vec<(String, String)>) -> Box<dyn FluidProperties> {
        match self.name {
            "water" => Box::new(WaterLikeFluidProperties::from_props(props, &Fluid::WATER)),
            "lava" => Box::new(WaterLikeFluidProperties::from_props(props, &Fluid::LAVA)),
            "flowing_water" => Box::new(FlowingWaterLikeFluidProperties::from_props(
                props,
                &Fluid::FLOWING_WATER,
            )),
            "flowing_lava" => Box::new(FlowingWaterLikeFluidProperties::from_props(
                props,
                &Fluid::FLOWING_LAVA,
            )),
            _ => panic!("Invalid props"),
        }
    }
    pub fn is_source(&self, state_id: u16) -> bool {
        let idx = (state_id as usize) % self.states.len();
        self.states[idx].is_source
    }
    pub fn is_falling(&self, state_id: u16) -> bool {
        let idx = (state_id as usize) % self.states.len();
        self.states[idx].falling
    }
    pub fn get_level(&self, state_id: u16) -> i16 {
        let idx = (state_id as usize) % self.states.len();
        self.states[idx].level
    }
    pub fn get_height(&self, state_id: u16) -> f32 {
        let idx = (state_id as usize) % self.states.len();
        self.states[idx].height
    }
}
impl ToResourceLocation for &'static Fluid {
    fn to_resource_location(&self) -> ResourceLocation {
        format!("minecraft:{}", self.name)
    }
}
impl FromResourceLocation for &'static Fluid {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
        Fluid::from_registry_key(
            resource_location
                .strip_prefix("minecraft:")
                .unwrap_or(resource_location),
        )
    }
}
impl FluidStateRef {
    pub fn get_state(&self) -> FluidState {
        let partial_state = &FLUID_STATES[self.state_idx as usize];
        FluidState {
            height: partial_state.height,
            level: partial_state.level,
            is_empty: partial_state.is_empty,
            blast_resistance: partial_state.blast_resistance,
            block_state_id: partial_state.block_state_id,
            is_still: partial_state.is_still,
            is_source: partial_state.is_source,
            falling: partial_state.falling,
        }
    }
}
impl Taggable for Fluid {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Fluid
    }
    #[inline]
    fn registry_key(&self) -> &str {
        self.name
    }
    #[inline]
    fn registry_id(&self) -> u16 {
        self.id
    }
}
pub const FLUID_LEVEL_SOURCE: i32 = 0;
pub const FLUID_LEVEL_FLOWING_MAX: i32 = 8;
pub const FLUID_MIN_HEIGHT: f32 = 0.0;
pub const FLUID_MAX_HEIGHT: f32 = 1.0;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Falling {
    True,
    False,
}
impl EnumVariants for Falling {
    fn variant_count() -> u16 {
        2u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::True => 0u16,
            Self::False => 1u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::True,
            1u16 => Self::False,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &str {
        match self {
            Self::True => "true",
            Self::False => "false",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "true" => Self::True,
            "false" => Self::False,
            _ => panic!("Invalid value: {value:?}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,
}
impl EnumVariants for Level {
    fn variant_count() -> u16 {
        8u16
    }
    fn to_index(&self) -> u16 {
        match self {
            Self::L1 => 0u16,
            Self::L2 => 1u16,
            Self::L3 => 2u16,
            Self::L4 => 3u16,
            Self::L5 => 4u16,
            Self::L6 => 5u16,
            Self::L7 => 6u16,
            Self::L8 => 7u16,
        }
    }
    fn from_index(index: u16) -> Self {
        match index {
            0u16 => Self::L1,
            1u16 => Self::L2,
            2u16 => Self::L3,
            3u16 => Self::L4,
            4u16 => Self::L5,
            5u16 => Self::L6,
            6u16 => Self::L7,
            7u16 => Self::L8,
            _ => panic!("Invalid index: {index}"),
        }
    }
    fn to_value(&self) -> &str {
        match self {
            Self::L1 => "1",
            Self::L2 => "2",
            Self::L3 => "3",
            Self::L4 => "4",
            Self::L5 => "5",
            Self::L6 => "6",
            Self::L7 => "7",
            Self::L8 => "8",
        }
    }
    fn from_value(value: &str) -> Self {
        match value {
            "1" => Self::L1,
            "2" => Self::L2,
            "3" => Self::L3,
            "4" => Self::L4,
            "5" => Self::L5,
            "6" => Self::L6,
            "7" => Self::L7,
            "8" => Self::L8,
            _ => panic!("Invalid value: {value:?}"),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterLikeFluidProperties {
    pub r#falling: Falling,
}
impl FluidProperties for WaterLikeFluidProperties {
    #[allow(unused_assignments)]
    fn to_index(&self) -> u16 {
        let mut index = 0;
        let mut multiplier = 1;
        index += self.r#falling.to_index() * multiplier;
        multiplier *= Falling::variant_count();
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#falling: {
                let value = index % Falling::variant_count();
                index /= Falling::variant_count();
                Falling::from_index(value)
            },
        }
    }
    fn to_state_id(&self, fluid: &Fluid) -> u16 {
        if !["water", "lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "WaterLikeFluidProperties"
            );
        }
        let prop_index = self.to_index();
        if prop_index < fluid.states.len() as u16 {
            fluid.states[prop_index as usize].block_state_id
        } else {
            fluid.states[fluid.default_state_index as usize].block_state_id
        }
    }
    fn from_state_id(state_id: u16, fluid: &Fluid) -> Self {
        if !["water", "lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "WaterLikeFluidProperties"
            );
        }
        for (idx, state) in fluid.states.iter().enumerate() {
            if state.block_state_id == state_id {
                return Self::from_index(idx as u16);
            }
        }
        Self::from_index(fluid.default_state_index)
    }
    fn default(fluid: &Fluid) -> Self {
        if !["water", "lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "WaterLikeFluidProperties"
            );
        }
        Self::from_state_id(fluid.default_state_index, fluid)
    }
    fn to_props(&self) -> Vec<(String, String)> {
        vec![("falling".to_string(), self.r#falling.to_value().to_string())]
    }
    fn from_props(props: Vec<(String, String)>, fluid: &Fluid) -> Self {
        if !["water", "lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "WaterLikeFluidProperties"
            );
        }
        let mut fluid_props = Self::default(fluid);
        for (key, value) in props {
            match key.as_str() {
                "falling" => fluid_props.r#falling = Falling::from_value(&value),
                _ => panic!("Invalid key: {key}"),
            }
        }
        fluid_props
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlowingWaterLikeFluidProperties {
    pub r#falling: Falling,
    pub r#level: Level,
}
impl FluidProperties for FlowingWaterLikeFluidProperties {
    #[allow(unused_assignments)]
    fn to_index(&self) -> u16 {
        let mut index = 0;
        let mut multiplier = 1;
        index += self.r#level.to_index() * multiplier;
        multiplier *= Level::variant_count();
        index += self.r#falling.to_index() * multiplier;
        multiplier *= Falling::variant_count();
        index
    }
    #[allow(unused_assignments)]
    fn from_index(mut index: u16) -> Self {
        Self {
            r#level: {
                let value = index % Level::variant_count();
                index /= Level::variant_count();
                Level::from_index(value)
            },
            r#falling: {
                let value = index % Falling::variant_count();
                index /= Falling::variant_count();
                Falling::from_index(value)
            },
        }
    }
    fn to_state_id(&self, fluid: &Fluid) -> u16 {
        if !["flowing_water", "flowing_lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "FlowingWaterLikeFluidProperties"
            );
        }
        let prop_index = self.to_index();
        if prop_index < fluid.states.len() as u16 {
            fluid.states[prop_index as usize].block_state_id
        } else {
            fluid.states[fluid.default_state_index as usize].block_state_id
        }
    }
    fn from_state_id(state_id: u16, fluid: &Fluid) -> Self {
        if !["flowing_water", "flowing_lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "FlowingWaterLikeFluidProperties"
            );
        }
        for (idx, state) in fluid.states.iter().enumerate() {
            if state.block_state_id == state_id {
                return Self::from_index(idx as u16);
            }
        }
        Self::from_index(fluid.default_state_index)
    }
    fn default(fluid: &Fluid) -> Self {
        if !["flowing_water", "flowing_lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "FlowingWaterLikeFluidProperties"
            );
        }
        Self::from_state_id(fluid.default_state_index, fluid)
    }
    fn to_props(&self) -> Vec<(String, String)> {
        vec![
            ("falling".to_string(), self.r#falling.to_value().to_string()),
            ("level".to_string(), self.r#level.to_value().to_string()),
        ]
    }
    fn from_props(props: Vec<(String, String)>, fluid: &Fluid) -> Self {
        if !["flowing_water", "flowing_lava"].contains(&fluid.name) {
            panic!(
                "{} is not a valid fluid for {}",
                &fluid.name, "FlowingWaterLikeFluidProperties"
            );
        }
        let mut fluid_props = Self::default(fluid);
        for (key, value) in props {
            match key.as_str() {
                "falling" => fluid_props.r#falling = Falling::from_value(&value),
                "level" => fluid_props.r#level = Level::from_value(&value),
                _ => panic!("Invalid key: {key}"),
            }
        }
        fluid_props
    }
}
