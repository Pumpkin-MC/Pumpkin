#[derive(Clone, Debug)]
pub struct BlockState {
    pub id: u16,
    pub state_flags: u8,
    pub luminance: u8,
    pub hardness: f32,
    pub collision_shapes: &'static [u16],
    //u8::MAX is used as None
    pub opacity: u8,
    //u16::MAX is used as None
    pub block_entity_type: u16,
}

// Add your methods here
impl BlockState {
    pub const fn is_air(&self) -> bool {
        self.state_flags & IS_AIR != 0
    }

    pub const fn burnable(&self) -> bool {
        self.state_flags & BURNABLE != 0
    }

    pub const fn tool_required(&self) -> bool {
        self.state_flags & TOOL_REQUIRED != 0
    }

    pub const fn sided_transparency(&self) -> bool {
        self.state_flags & SIDED_TRANSPARENCY != 0
    }

    pub const fn replaceable(&self) -> bool {
        self.state_flags & REPLACEABLE != 0
    }

    pub const fn is_liquid(&self) -> bool {
        self.state_flags & IS_LIQUID != 0
    }

    pub const fn is_solid(&self) -> bool {
        self.state_flags & IS_SOLID != 0
    }

    pub const fn is_full_cube(&self) -> bool {
        self.state_flags & IS_FULL_CUBE != 0
    }
}

#[derive(Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}

//This is the Layout of state_props in the right order
const IS_AIR: u8 = 0b00000001;
const BURNABLE: u8 = 0b00000010;
const TOOL_REQUIRED: u8 = 0b00000100;
const SIDED_TRANSPARENCY: u8 = 0b00001000;
const REPLACEABLE: u8 = 0b00010000;
const IS_LIQUID: u8 = 0b00100000;
const IS_SOLID: u8 = 0b01000000;
const IS_FULL_CUBE: u8 = 0b10000000;
