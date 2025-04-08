#[derive(Clone, Debug)]
pub struct BlockState {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub tool_required: bool,
    pub hardness: f32,
    pub sided_transparency: bool,
    pub replaceable: bool,
    pub collision_shapes: &'static [u16],
    pub opacity: Option<u32>,
    pub block_entity_type: Option<u32>,
    pub is_liquid: bool,
    pub is_solid: bool,
}

// Add your methods here
impl BlockState {
    // As you guys can see this is actually a const fn
    pub const fn is_full_cube(&self) -> bool {
        !self.collision_shapes.is_empty() && self.collision_shapes[0] == 0
    }
}

#[derive(Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}
