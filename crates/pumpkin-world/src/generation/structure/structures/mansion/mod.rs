mod grid;
mod placer;

use std::sync::{Arc, Mutex};

use pumpkin_data::{Block, Mirror, Rotation};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
            StructurePiecesCollector, StructurePosition,
        },
        template::{
            BlockStateResolver, PaletteEntry, StructureTemplate, get_block_entity_id, get_template,
        },
    },
    world::WorldPortalExt,
};

use self::{
    grid::MansionGrid,
    placer::{MansionPiecePlacer, PieceDescriptor},
};

const LOOT_TABLE: &str = "minecraft:chests/woodland_mansion";

pub struct MansionGenerator;

impl StructureGenerator for MansionGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let rotation = Rotation::from_index(context.random.next_bounded_i32(4) as u8);
        let x = context.chunk_x * 16 + 7;
        let z = context.chunk_z * 16 + 7;
        let (offset_x, offset_z) = match rotation {
            Rotation::None => (5, 5),
            Rotation::Clockwise90 => (-5, 5),
            Rotation::Rotate180 => (-5, -5),
            Rotation::CounterClockwise90 => (5, -5),
        };
        let y = {
            let sampler = context.height_sampler.as_deref_mut()?;
            [
                sampler.estimate_height(x, z),
                sampler.estimate_height(x, z + offset_z),
                sampler.estimate_height(x + offset_x, z),
                sampler.estimate_height(x + offset_x, z + offset_z),
            ]
            .into_iter()
            .min()?
        };
        if y < 60 {
            return None;
        }

        let origin = Vector3::new(x, y, z);
        let grid = MansionGrid::new(&mut context.random);
        let descriptors = MansionPiecePlacer::create(&mut context.random, origin, rotation, &grid);
        let mut pieces = Vec::with_capacity(descriptors.len());
        for descriptor in descriptors {
            pieces.push(MansionTemplatePiece::new(&descriptor)?);
        }
        let piece_boxes = pieces
            .iter()
            .map(|piece| piece.piece.bounding_box)
            .collect::<Vec<_>>();
        let mut collector = StructurePiecesCollector::default();
        for piece in pieces {
            collector.add_piece(Box::new(piece));
        }
        collector.add_piece(Box::new(MansionFoundationPiece::new(piece_boxes)?));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, y, z),
            collector: Arc::new(Mutex::new(collector)),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TemplateMirror {
    #[default]
    None,
    LeftRight,
    FrontBack,
}

impl TemplateMirror {
    const fn state_mirror(self) -> Mirror {
        match self {
            Self::None => Mirror::None,
            // Mojang names a Z reflection LEFT_RIGHT and an X reflection FRONT_BACK.
            // Pumpkin names the axis being reflected, so the variants are exchanged.
            Self::LeftRight => Mirror::FrontBack,
            Self::FrontBack => Mirror::LeftRight,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    North,
    South,
    West,
    East,
    Up,
}

impl Direction {
    const HORIZONTAL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    const fn from_2d_index(index: i32) -> Self {
        [Self::South, Self::West, Self::North, Self::East][index as usize]
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "east" => Some(Self::East),
            "up" => Some(Self::Up),
            _ => None,
        }
    }

    const fn step_x(self) -> i32 {
        match self {
            Self::West => -1,
            Self::East => 1,
            _ => 0,
        }
    }

    const fn step_z(self) -> i32 {
        match self {
            Self::North => -1,
            Self::South => 1,
            _ => 0,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
            Self::Up => "up",
        }
    }

    const fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::Up => Self::Up,
        }
    }

    const fn counterclockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            Self::Up => Self::Up,
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::Up => Self::Up,
        }
    }

    const fn rotate(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => self,
            Rotation::Clockwise90 => self.clockwise(),
            Rotation::Rotate180 => self.opposite(),
            Rotation::CounterClockwise90 => self.counterclockwise(),
        }
    }
}

trait RotateDirection {
    fn rotate(self, direction: Direction) -> Direction;
}

impl RotateDirection for Rotation {
    fn rotate(self, direction: Direction) -> Direction {
        direction.rotate(self)
    }
}

const fn offset(position: Vector3<i32>, x: i32, y: i32, z: i32) -> Vector3<i32> {
    Vector3::new(position.x + x, position.y + y, position.z + z)
}

const fn relative(position: Vector3<i32>, direction: Direction, distance: i32) -> Vector3<i32> {
    offset(
        position,
        direction.step_x() * distance,
        0,
        direction.step_z() * distance,
    )
}

struct MansionTemplatePiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    template_position: Vector3<i32>,
    rotation: Rotation,
    mirror: TemplateMirror,
}

impl MansionTemplatePiece {
    fn new(descriptor: &PieceDescriptor) -> Option<Self> {
        let template = get_template(&format!("woodland_mansion/{}", descriptor.template))?;
        let max = Vector3::new(
            template.size.x - 1,
            template.size.y - 1,
            template.size.z - 1,
        );
        let first = transform(
            Vector3::new(0, 0, 0),
            descriptor.mirror,
            descriptor.rotation,
        );
        let second = transform(max, descriptor.mirror, descriptor.rotation);
        let bounding_box = BlockBox::new(
            descriptor.position.x + first.x.min(second.x),
            descriptor.position.y,
            descriptor.position.z + first.z.min(second.z),
            descriptor.position.x + first.x.max(second.x),
            descriptor.position.y + max.y,
            descriptor.position.z + first.z.max(second.z),
        );
        Some(Self {
            piece: StructurePiece::new(StructurePieceType::WoodlandMansion, bounding_box, 0),
            template,
            template_position: descriptor.position,
            rotation: descriptor.rotation,
            mirror: descriptor.mirror,
        })
    }

    const fn world_position(&self, local: Vector3<i32>) -> Vector3<i32> {
        let transformed = transform(local, self.mirror, self.rotation);
        offset(
            self.template_position,
            transformed.x,
            transformed.y,
            transformed.z,
        )
    }

    fn place_blocks(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        for template_block in &self.template.blocks {
            let palette = &self.template.palette[template_block.state as usize];
            let position = self.world_position(template_block.pos);
            if !chunk_box.contains_pos(&position) {
                continue;
            }
            if palette.name == "minecraft:structure_void" {
                continue;
            }
            if palette.name == "minecraft:structure_block" {
                if let Some(marker) = template_block
                    .nbt
                    .as_ref()
                    .and_then(|nbt| nbt.get_string("metadata"))
                {
                    self.handle_marker(chunk, random, marker, position);
                }
                continue;
            }

            let mut transformed = transformed_palette(palette, self.mirror, self.rotation);
            if chunk.get_block_state(&position).to_block_id() == Block::WATER.id
                && property(&transformed, "waterlogged").is_some()
            {
                set_property(&mut transformed, "waterlogged", "true");
            }
            let Some(state) = BlockStateResolver::resolve(
                &transformed,
                self.rotation,
                self.mirror.state_mirror(),
            ) else {
                continue;
            };
            chunk.set_block_state(position.x, position.y, position.z, state);
            Self::place_block_entity(
                chunk,
                palette,
                template_block.nbt.as_ref(),
                position,
                random,
            );
        }
    }

    fn place_block_entity(
        chunk: &mut ProtoChunk,
        palette: &PaletteEntry,
        template_nbt: Option<&NbtCompound>,
        position: Vector3<i32>,
        random: &mut RandomGenerator,
    ) {
        let block_entity_id = get_block_entity_id(&palette.name);
        if template_nbt.is_none() && block_entity_id.is_none() {
            return;
        }
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", block_entity_id.unwrap_or(&palette.name).to_string());
        nbt.put_int("x", position.x);
        nbt.put_int("y", position.y);
        nbt.put_int("z", position.z);
        if let Some(template_nbt) = template_nbt {
            for (key, value) in &template_nbt.child_tags {
                if !matches!(key.as_ref(), "x" | "y" | "z" | "id") {
                    nbt.child_tags.insert(key.clone(), value.clone());
                }
            }
        }
        if nbt.get_string("LootTable").is_some() && nbt.get_long("LootTableSeed").is_none() {
            nbt.put_long("LootTableSeed", random.next_i64());
        }
        chunk.add_block_entity(nbt);
    }

    fn handle_marker(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        marker: &str,
        position: Vector3<i32>,
    ) {
        if let Some(facing) = marker.strip_prefix("Chest") {
            let direction = match facing {
                "West" => Direction::West,
                "East" => Direction::East,
                "South" => Direction::South,
                _ => Direction::North,
            };
            let target = self.rotation.rotate(direction);
            let chest = PaletteEntry::with_properties(
                "minecraft:chest".to_string(),
                vec![("facing".to_string(), target.name().to_string())],
            );
            let state = BlockStateResolver::resolve_simple(&chest)
                .expect("the vanilla chest state must resolve");
            chunk.set_block_state(position.x, position.y, position.z, state);
            let mut nbt = NbtCompound::new();
            nbt.put_string("id", "minecraft:chest".to_string());
            nbt.put_int("x", position.x);
            nbt.put_int("y", position.y);
            nbt.put_int("z", position.z);
            nbt.put_string("LootTable", LOOT_TABLE.to_string());
            nbt.put_long("LootTableSeed", random.next_i64());
            chunk.add_block_entity(nbt);
            return;
        }

        let (entity, count) = match marker {
            "Mage" => ("minecraft:evoker", 1),
            "Warrior" => ("minecraft:vindicator", 1),
            "Group of Allays" => ("minecraft:allay", random.next_bounded_i32(3) + 1),
            _ => return,
        };
        chunk.set_block_state(position.x, position.y, position.z, Block::AIR.default_state);
        for _ in 0..count {
            let mut nbt = NbtCompound::new();
            nbt.put_string("id", entity.to_string());
            nbt.put_list(
                "Pos",
                vec![
                    (f64::from(position.x) + 0.5).into(),
                    f64::from(position.y).into(),
                    (f64::from(position.z) + 0.5).into(),
                ],
            );
            nbt.put(
                "Motion",
                NbtTag::List(vec![0.0.into(), 0.0.into(), 0.0.into()]),
            );
            nbt.put("Rotation", NbtTag::List(vec![0.0f32.into(), 0.0f32.into()]));
            nbt.put_bool("PersistenceRequired", true);
            chunk.add_structure_entity(nbt);
        }
    }
}

impl StructurePieceBase for MansionTemplatePiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        self.place_blocks(chunk, random, chunk_box);
    }
}

struct MansionFoundationPiece {
    piece: StructurePiece,
    mansion_pieces: Vec<BlockBox>,
    y: i32,
}

impl MansionFoundationPiece {
    fn new(mansion_pieces: Vec<BlockBox>) -> Option<Self> {
        let bounding_box = BlockBox::encompass_all(mansion_pieces.iter().copied())?;
        let y = bounding_box.min.y;
        Some(Self {
            piece: StructurePiece::new(StructurePieceType::WoodlandMansion, bounding_box, 0),
            mansion_pieces,
            y,
        })
    }
}

impl StructurePieceBase for MansionFoundationPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let min_x = self.piece.bounding_box.min.x.max(chunk_box.min.x);
        let max_x = self.piece.bounding_box.max.x.min(chunk_box.max.x);
        let min_z = self.piece.bounding_box.min.z.max(chunk_box.min.z);
        let max_z = self.piece.bounding_box.max.z.min(chunk_box.max.z);
        let bottom = i32::from(chunk.bottom_y()) + 1;
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let position = Vector3::new(x, self.y, z);
                if chunk.get_block_state(&position).to_state().is_air()
                    || !self
                        .mansion_pieces
                        .iter()
                        .any(|piece| piece.contains_pos(&position))
                {
                    continue;
                }
                for y in (bottom..self.y).rev() {
                    let state = chunk.get_block_state(&Vector3::new(x, y, z)).to_state();
                    if !state.is_air() && !state.is_liquid() {
                        break;
                    }
                    chunk.set_block_state(x, y, z, Block::COBBLESTONE.default_state);
                }
            }
        }
    }
}

fn transformed_palette(
    palette: &PaletteEntry,
    mirror: TemplateMirror,
    rotation: Rotation,
) -> PaletteEntry {
    let mut palette = palette.clone();

    if palette.name.ends_with("_stairs") {
        let facing = property(&palette, "facing");
        let shape = property(&palette, "shape");
        let mirrored_shape = match (mirror, facing, shape) {
            (TemplateMirror::LeftRight, Some("north" | "south"), Some("inner_left")) => {
                Some("inner_right")
            }
            (TemplateMirror::LeftRight, Some("north" | "south"), Some("inner_right")) => {
                Some("inner_left")
            }
            (TemplateMirror::LeftRight, Some("north" | "south"), Some("outer_left"))
            | (TemplateMirror::FrontBack, Some("east" | "west"), Some("outer_left")) => {
                Some("outer_right")
            }
            (TemplateMirror::LeftRight, Some("north" | "south"), Some("outer_right"))
            | (TemplateMirror::FrontBack, Some("east" | "west"), Some("outer_right")) => {
                Some("outer_left")
            }
            _ => None,
        };
        if let Some(shape) = mirrored_shape {
            set_property(&mut palette, "shape", shape);
        }
    } else if palette.name.ends_with("_door") && mirror != TemplateMirror::None {
        let hinge = match property(&palette, "hinge") {
            Some("left") => Some("right"),
            Some("right") => Some("left"),
            _ => None,
        };
        if let Some(hinge) = hinge {
            set_property(&mut palette, "hinge", hinge);
        }
    } else if palette.name == "minecraft:rail"
        && let Some(shape) = property(&palette, "shape")
        && let Some(shape) = transform_rail_shape(shape, mirror, rotation)
    {
        set_property(&mut palette, "shape", shape);
    }

    palette
}

fn property<'a>(palette: &'a PaletteEntry, name: &str) -> Option<&'a str> {
    palette
        .properties
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value.as_str()))
}

fn set_property(palette: &mut PaletteEntry, name: &str, value: &str) {
    if let Some((_, current)) = palette.properties.iter_mut().find(|(key, _)| key == name) {
        value.clone_into(current);
    }
}

fn transform_rail_shape(
    shape: &str,
    mirror: TemplateMirror,
    rotation: Rotation,
) -> Option<&'static str> {
    let ascending = shape.strip_prefix("ascending_");
    if let Some(direction) = ascending.and_then(Direction::from_name) {
        return Some(match transform_direction(direction, mirror, rotation) {
            Direction::North => "ascending_north",
            Direction::South => "ascending_south",
            Direction::West => "ascending_west",
            Direction::East => "ascending_east",
            Direction::Up => return None,
        });
    }
    let (first, second) = match shape {
        "north_south" => (Direction::North, Direction::South),
        "east_west" => (Direction::East, Direction::West),
        "north_east" => (Direction::North, Direction::East),
        "north_west" => (Direction::North, Direction::West),
        "south_east" => (Direction::South, Direction::East),
        "south_west" => (Direction::South, Direction::West),
        _ => return None,
    };
    let first = transform_direction(first, mirror, rotation);
    let second = transform_direction(second, mirror, rotation);
    match (first, second) {
        (Direction::North, Direction::South) | (Direction::South, Direction::North) => {
            Some("north_south")
        }
        (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
            Some("east_west")
        }
        (Direction::North, Direction::East) | (Direction::East, Direction::North) => {
            Some("north_east")
        }
        (Direction::North, Direction::West) | (Direction::West, Direction::North) => {
            Some("north_west")
        }
        (Direction::South, Direction::East) | (Direction::East, Direction::South) => {
            Some("south_east")
        }
        (Direction::South, Direction::West) | (Direction::West, Direction::South) => {
            Some("south_west")
        }
        _ => None,
    }
}

fn transform_direction(
    direction: Direction,
    mirror: TemplateMirror,
    rotation: Rotation,
) -> Direction {
    let mirrored = match (mirror, direction) {
        (TemplateMirror::LeftRight, Direction::North) => Direction::South,
        (TemplateMirror::LeftRight, Direction::South) => Direction::North,
        (TemplateMirror::FrontBack, Direction::East) => Direction::West,
        (TemplateMirror::FrontBack, Direction::West) => Direction::East,
        _ => direction,
    };
    rotation.rotate(mirrored)
}

const fn transform(
    position: Vector3<i32>,
    mirror: TemplateMirror,
    rotation: Rotation,
) -> Vector3<i32> {
    let x = if matches!(mirror, TemplateMirror::FrontBack) {
        -position.x
    } else {
        position.x
    };
    let z = if matches!(mirror, TemplateMirror::LeftRight) {
        -position.z
    } else {
        position.z
    };
    let (x, z) = rotation.rotate_offset(x, z);
    Vector3::new(x, position.y, z)
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::legacy_rand::LegacyRand;

    use crate::generation::structure::structures::HeightSampler;

    use super::*;

    fn palette(name: &str, properties: &[(&str, &str)]) -> PaletteEntry {
        PaletteEntry::with_properties(
            name.to_string(),
            properties
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
        )
    }

    #[test]
    fn mirrors_directional_shapes_like_java() {
        let stairs = transformed_palette(
            &palette(
                "minecraft:dark_oak_stairs",
                &[("facing", "north"), ("shape", "inner_left")],
            ),
            TemplateMirror::LeftRight,
            Rotation::Clockwise90,
        );
        assert_eq!(property(&stairs, "shape"), Some("inner_right"));

        let door = transformed_palette(
            &palette(
                "minecraft:dark_oak_door",
                &[("facing", "east"), ("hinge", "left")],
            ),
            TemplateMirror::FrontBack,
            Rotation::None,
        );
        assert_eq!(property(&door, "hinge"), Some("right"));

        let rail = transformed_palette(
            &palette("minecraft:rail", &[("shape", "north_east")]),
            TemplateMirror::LeftRight,
            Rotation::Clockwise90,
        );
        assert_eq!(property(&rail, "shape"), Some("south_west"));
    }

    struct RecordingHeightSampler {
        calls: Vec<(i32, i32)>,
        heights: std::vec::IntoIter<i32>,
    }

    impl HeightSampler for RecordingHeightSampler {
        fn estimate_height(&mut self, block_x: i32, block_z: i32) -> i32 {
            self.calls.push((block_x, block_z));
            self.heights.next().unwrap()
        }
    }

    #[test]
    fn start_position_uses_java_five_by_five_corner_sampling() {
        let seed = 42;
        let mut expected_random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
        let rotation = Rotation::from_index(expected_random.next_bounded_i32(4) as u8);
        let (offset_x, offset_z) = match rotation {
            Rotation::None => (5, 5),
            Rotation::Clockwise90 => (-5, 5),
            Rotation::Rotate180 => (-5, -5),
            Rotation::CounterClockwise90 => (5, -5),
        };
        let mut sampler = RecordingHeightSampler {
            calls: Vec::new(),
            heights: vec![74, 70, 72, 73].into_iter(),
        };
        let position = MansionGenerator
            .get_structure_position(StructureGeneratorContext {
                seed: seed as i64,
                chunk_x: 3,
                chunk_z: -2,
                random: RandomGenerator::Legacy(LegacyRand::from_seed(seed)),
                sea_level: 63,
                min_y: -64,
                height_sampler: Some(&mut sampler),
                structure_key: None,
            })
            .unwrap();
        let x = 3 * 16 + 7;
        let z = -2 * 16 + 7;
        assert_eq!(position.start_pos, BlockPos::new(x, 70, z));
        assert_eq!(
            sampler.calls,
            [
                (x, z),
                (x, z + offset_z),
                (x + offset_x, z),
                (x + offset_x, z + offset_z),
            ]
        );
    }

    #[test]
    fn vanilla_zero_pivot_transform_is_not_bounds_normalized() {
        assert_eq!(
            transform(
                Vector3::new(3, 2, 5),
                TemplateMirror::LeftRight,
                Rotation::Clockwise90,
            ),
            Vector3::new(5, 2, 3)
        );
        assert_eq!(
            transform(
                Vector3::new(3, 2, 5),
                TemplateMirror::FrontBack,
                Rotation::None,
            ),
            Vector3::new(-3, 2, 5)
        );
    }
}
