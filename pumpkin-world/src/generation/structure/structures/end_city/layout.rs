use std::sync::Arc;

use pumpkin_data::Rotation;
use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::structure::template::{StructureTemplate, get_template};

const MAX_GENERATION_DEPTH: i32 = 8;

const TOWER_BRIDGES: [(Rotation, Vector3<i32>); 4] = [
    (Rotation::None, Vector3::new(1, -1, 0)),
    (Rotation::Clockwise90, Vector3::new(6, -1, 1)),
    (Rotation::CounterClockwise90, Vector3::new(0, -1, 5)),
    (Rotation::Rotate180, Vector3::new(5, -1, 6)),
];

const FAT_TOWER_BRIDGES: [(Rotation, Vector3<i32>); 4] = [
    (Rotation::None, Vector3::new(4, -1, 0)),
    (Rotation::Clockwise90, Vector3::new(12, -1, 4)),
    (Rotation::CounterClockwise90, Vector3::new(0, -1, 8)),
    (Rotation::Rotate180, Vector3::new(8, -1, 12)),
];

#[derive(Clone)]
pub(super) struct PieceDescriptor {
    pub template: Arc<StructureTemplate>,
    pub template_position: Vector3<i32>,
    pub rotation: Rotation,
    pub overwrite: bool,
    pub bounding_box: BlockBox,
    generation_group: i32,
    #[cfg(test)]
    pub template_name: &'static str,
}

impl PieceDescriptor {
    pub(super) fn new(
        template_name: &'static str,
        template_position: Vector3<i32>,
        rotation: Rotation,
        overwrite: bool,
    ) -> Option<Self> {
        let template = get_template(&format!("end_city/{template_name}"))?;
        let max = Vector3::new(
            template.size.x - 1,
            template.size.y - 1,
            template.size.z - 1,
        );
        let (max_x, max_z) = rotation.rotate_offset(max.x, max.z);
        let bounding_box = BlockBox::new(
            template_position.x.min(template_position.x + max_x),
            template_position.y,
            template_position.z.min(template_position.z + max_z),
            template_position.x.max(template_position.x + max_x),
            template_position.y + max.y,
            template_position.z.max(template_position.z + max_z),
        );
        Some(Self {
            template,
            template_position,
            rotation,
            overwrite,
            bounding_box,
            generation_group: 0,
            #[cfg(test)]
            template_name,
        })
    }

    fn connected(
        parent: &Self,
        offset: Vector3<i32>,
        template_name: &'static str,
        rotation: Rotation,
        overwrite: bool,
    ) -> Option<Self> {
        // Vanilla's calculateConnectedPosition rotates both connector positions
        // around the template origin. End City children always use ZERO for the
        // child connector, leaving only the parent's rotated offset.
        let (x, z) = parent.rotation.rotate_offset(offset.x, offset.z);
        Self::new(
            template_name,
            Vector3::new(
                parent.template_position.x + x,
                parent.template_position.y + offset.y,
                parent.template_position.z + z,
            ),
            rotation,
            overwrite,
        )
    }
}

#[derive(Clone, Copy)]
enum Section {
    HouseTower,
    Tower,
    TowerBridge,
    FatTower,
}

pub(super) struct EndCityLayout {
    ship_created: bool,
}

impl EndCityLayout {
    pub fn create(
        origin: Vector3<i32>,
        rotation: Rotation,
        random: &mut RandomGenerator,
    ) -> Option<Vec<PieceDescriptor>> {
        let mut layout = Self {
            ship_created: false,
        };
        let mut pieces = Vec::new();
        let mut last = PieceDescriptor::new("base_floor", origin, rotation, true)?;
        pieces.push(last.clone());
        last = Self::push_connected(
            &mut pieces,
            &last,
            Vector3::new(-1, 0, -1),
            "second_floor_1",
            rotation,
            false,
        )?;
        last = Self::push_connected(
            &mut pieces,
            &last,
            Vector3::new(-1, 4, -1),
            "third_floor_1",
            rotation,
            false,
        )?;
        last = Self::push_connected(
            &mut pieces,
            &last,
            Vector3::new(-1, 8, -1),
            "third_roof",
            rotation,
            true,
        )?;
        let _ = layout.recursive_children(Section::Tower, 1, &last, None, &mut pieces, random)?;
        Some(pieces)
    }

    fn push_connected(
        pieces: &mut Vec<PieceDescriptor>,
        parent: &PieceDescriptor,
        offset: Vector3<i32>,
        template_name: &'static str,
        rotation: Rotation,
        overwrite: bool,
    ) -> Option<PieceDescriptor> {
        let piece = PieceDescriptor::connected(parent, offset, template_name, rotation, overwrite)?;
        pieces.push(piece.clone());
        Some(piece)
    }

    fn recursive_children(
        &mut self,
        section: Section,
        depth: i32,
        parent: &PieceDescriptor,
        offset: Option<Vector3<i32>>,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        if depth > MAX_GENERATION_DEPTH {
            return Some(false);
        }

        let mut children = Vec::new();
        if !self.generate_section(section, depth, parent, offset, &mut children, random)? {
            return Some(false);
        }

        let generation_group = random.next_i32();
        for child in &mut children {
            child.generation_group = generation_group;
            if let Some(collision) = pieces
                .iter()
                .find(|piece| piece.bounding_box.intersects(&child.bounding_box))
                && collision.generation_group != parent.generation_group
            {
                return Some(false);
            }
        }
        pieces.extend(children);
        Some(true)
    }

    fn generate_section(
        &mut self,
        section: Section,
        depth: i32,
        parent: &PieceDescriptor,
        offset: Option<Vector3<i32>>,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        match section {
            Section::HouseTower => {
                self.generate_house_tower(depth, parent, offset?, pieces, random)
            }
            Section::Tower => self.generate_tower(depth, parent, pieces, random),
            Section::TowerBridge => self.generate_tower_bridge(depth, parent, pieces, random),
            Section::FatTower => self.generate_fat_tower(depth, parent, pieces, random),
        }
    }

    fn generate_house_tower(
        &mut self,
        depth: i32,
        parent: &PieceDescriptor,
        offset: Vector3<i32>,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        if depth > MAX_GENERATION_DEPTH {
            return Some(false);
        }
        let rotation = parent.rotation;
        let mut last = Self::push_connected(pieces, parent, offset, "base_floor", rotation, true)?;
        match random.next_bounded_i32(3) {
            0 => {
                Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 4, -1),
                    "base_roof",
                    rotation,
                    true,
                )?;
            }
            1 => {
                last = Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 0, -1),
                    "second_floor_2",
                    rotation,
                    false,
                )?;
                last = Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 8, -1),
                    "second_roof",
                    rotation,
                    false,
                )?;
                let _ = self.recursive_children(
                    Section::Tower,
                    depth + 1,
                    &last,
                    None,
                    pieces,
                    random,
                )?;
            }
            _ => {
                last = Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 0, -1),
                    "second_floor_2",
                    rotation,
                    false,
                )?;
                last = Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 4, -1),
                    "third_floor_2",
                    rotation,
                    false,
                )?;
                last = Self::push_connected(
                    pieces,
                    &last,
                    Vector3::new(-1, 8, -1),
                    "third_roof",
                    rotation,
                    true,
                )?;
                let _ = self.recursive_children(
                    Section::Tower,
                    depth + 1,
                    &last,
                    None,
                    pieces,
                    random,
                )?;
            }
        }
        Some(true)
    }

    fn generate_tower(
        &mut self,
        depth: i32,
        parent: &PieceDescriptor,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        let rotation = parent.rotation;
        let mut last = Self::push_connected(
            pieces,
            parent,
            Vector3::new(
                3 + random.next_bounded_i32(2),
                -3,
                3 + random.next_bounded_i32(2),
            ),
            "tower_base",
            rotation,
            true,
        )?;
        last = Self::push_connected(
            pieces,
            &last,
            Vector3::new(0, 7, 0),
            "tower_piece",
            rotation,
            true,
        )?;
        let mut bridge_piece = (random.next_bounded_i32(3) == 0).then(|| last.clone());
        let tower_height = 1 + random.next_bounded_i32(3);
        for index in 0..tower_height {
            last = Self::push_connected(
                pieces,
                &last,
                Vector3::new(0, 4, 0),
                "tower_piece",
                rotation,
                true,
            )?;
            if index < tower_height - 1 && random.next_bool() {
                bridge_piece = Some(last.clone());
            }
        }

        if let Some(bridge_piece) = bridge_piece {
            for (bridge_rotation, offset) in TOWER_BRIDGES {
                if random.next_bool() {
                    let bridge_start = Self::push_connected(
                        pieces,
                        &bridge_piece,
                        offset,
                        "bridge_end",
                        rotation.then(bridge_rotation),
                        true,
                    )?;
                    let _ = self.recursive_children(
                        Section::TowerBridge,
                        depth + 1,
                        &bridge_start,
                        None,
                        pieces,
                        random,
                    )?;
                }
            }
            Self::push_connected(
                pieces,
                &last,
                Vector3::new(-1, 4, -1),
                "tower_top",
                rotation,
                true,
            )?;
            return Some(true);
        }

        if depth != 7 {
            return self.recursive_children(
                Section::FatTower,
                depth + 1,
                &last,
                None,
                pieces,
                random,
            );
        }
        Self::push_connected(
            pieces,
            &last,
            Vector3::new(-1, 4, -1),
            "tower_top",
            rotation,
            true,
        )?;
        Some(true)
    }

    fn generate_tower_bridge(
        &mut self,
        depth: i32,
        parent: &PieceDescriptor,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        let rotation = parent.rotation;
        let bridge_length = random.next_bounded_i32(4) + 1;
        let mut last = Self::push_connected(
            pieces,
            parent,
            Vector3::new(0, 0, -4),
            "bridge_piece",
            rotation,
            true,
        )?;
        last.generation_group = -1;
        if let Some(piece) = pieces.last_mut() {
            piece.generation_group = -1;
        }
        let mut next_y = 0;
        for _ in 0..bridge_length {
            let offset_y = next_y;
            let (template, z) = if random.next_bool() {
                next_y = 0;
                ("bridge_piece", -4)
            } else {
                let template = if random.next_bool() {
                    "bridge_steep_stairs"
                } else {
                    "bridge_gentle_stairs"
                };
                let z = if template == "bridge_steep_stairs" {
                    -4
                } else {
                    -8
                };
                next_y = 4;
                (template, z)
            };
            last = Self::push_connected(
                pieces,
                &last,
                Vector3::new(0, offset_y, z),
                template,
                rotation,
                true,
            )?;
        }

        if !self.ship_created && random.next_bounded_i32(10 - depth) == 0 {
            Self::push_connected(
                pieces,
                &last,
                Vector3::new(
                    -8 + random.next_bounded_i32(8),
                    next_y,
                    -70 + random.next_bounded_i32(10),
                ),
                "ship",
                rotation,
                true,
            )?;
            self.ship_created = true;
        } else if !self.recursive_children(
            Section::HouseTower,
            depth + 1,
            &last,
            Some(Vector3::new(-3, next_y + 1, -11)),
            pieces,
            random,
        )? {
            return Some(false);
        }

        Self::push_connected(
            pieces,
            &last,
            Vector3::new(4, next_y, 0),
            "bridge_end",
            rotation.then(Rotation::Rotate180),
            true,
        )?;
        if let Some(piece) = pieces.last_mut() {
            piece.generation_group = -1;
        }
        Some(true)
    }

    fn generate_fat_tower(
        &mut self,
        depth: i32,
        parent: &PieceDescriptor,
        pieces: &mut Vec<PieceDescriptor>,
        random: &mut RandomGenerator,
    ) -> Option<bool> {
        let rotation = parent.rotation;
        let mut last = Self::push_connected(
            pieces,
            parent,
            Vector3::new(-3, 4, -3),
            "fat_tower_base",
            rotation,
            true,
        )?;
        last = Self::push_connected(
            pieces,
            &last,
            Vector3::new(0, 4, 0),
            "fat_tower_middle",
            rotation,
            true,
        )?;

        for _ in 0..2 {
            if random.next_bounded_i32(3) == 0 {
                break;
            }
            last = Self::push_connected(
                pieces,
                &last,
                Vector3::new(0, 8, 0),
                "fat_tower_middle",
                rotation,
                true,
            )?;
            for (bridge_rotation, offset) in FAT_TOWER_BRIDGES {
                if random.next_bool() {
                    let bridge_start = Self::push_connected(
                        pieces,
                        &last,
                        offset,
                        "bridge_end",
                        rotation.then(bridge_rotation),
                        true,
                    )?;
                    let _ = self.recursive_children(
                        Section::TowerBridge,
                        depth + 1,
                        &bridge_start,
                        None,
                        pieces,
                        random,
                    )?;
                }
            }
        }
        Self::push_connected(
            pieces,
            &last,
            Vector3::new(-2, 8, -2),
            "fat_tower_top",
            rotation,
            true,
        )?;
        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Mirror;
    use pumpkin_util::random::legacy_rand::LegacyRand;

    use crate::generation::structure::{
        structures::create_chunk_random, template::BlockStateResolver,
    };

    use super::*;

    #[test]
    fn fixed_seed_matches_vanilla_ship_city() {
        let mut random = create_chunk_random(12_345, -298, -275);
        let rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let pieces =
            EndCityLayout::create(Vector3::new(-4761, 60, -4393), rotation, &mut random).unwrap();
        assert_eq!(rotation, Rotation::Rotate180);
        assert_eq!(pieces.len(), 78);
        let ship = &pieces[43];
        assert_eq!(ship.template_name, "ship");
        assert_eq!(ship.rotation, Rotation::CounterClockwise90);
        assert_eq!(ship.template_position, Vector3::new(-4888, 123, -4393));
        assert_eq!(ship.generation_group, -340_338_317);
        assert_eq!(
            (ship.bounding_box.min, ship.bounding_box.max),
            (
                Vector3::new(-4888, 123, -4405),
                Vector3::new(-4860, 146, -4393),
            )
        );
    }

    #[test]
    fn fixed_seed_matches_vanilla_fat_tower_city() {
        let mut random = create_chunk_random(12_345, 62, 24);
        let rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let pieces =
            EndCityLayout::create(Vector3::new(999, 61, 391), rotation, &mut random).unwrap();
        assert_eq!(
            pieces
                .iter()
                .map(|piece| piece.template_name)
                .collect::<Vec<_>>(),
            [
                "base_floor",
                "second_floor_1",
                "third_floor_1",
                "third_roof",
                "tower_base",
                "tower_piece",
                "tower_piece",
                "tower_piece",
                "fat_tower_base",
                "fat_tower_middle",
                "fat_tower_top",
            ]
        );
        let top = pieces.last().unwrap();
        assert_eq!(top.template_position, Vector3::new(994, 101, 386));
        assert_eq!(top.generation_group, 1_141_071_963);
    }

    #[test]
    fn connected_pieces_rotate_around_the_parent_origin() {
        let parent = PieceDescriptor::new(
            "base_floor",
            Vector3::new(100, 70, 200),
            Rotation::Clockwise90,
            true,
        )
        .unwrap();
        let child = PieceDescriptor::connected(
            &parent,
            Vector3::new(-1, 4, -3),
            "base_roof",
            Rotation::Clockwise90,
            true,
        )
        .unwrap();
        assert_eq!(child.template_position, Vector3::new(103, 74, 199));
    }

    #[test]
    fn vanilla_templates_are_available() {
        for name in [
            "base_floor",
            "base_roof",
            "bridge_end",
            "bridge_gentle_stairs",
            "bridge_piece",
            "bridge_steep_stairs",
            "fat_tower_base",
            "fat_tower_middle",
            "fat_tower_top",
            "second_floor_1",
            "second_floor_2",
            "second_roof",
            "ship",
            "third_floor_1",
            "third_floor_2",
            "third_roof",
            "tower_base",
            "tower_piece",
            "tower_top",
        ] {
            let template = get_template(&format!("end_city/{name}")).unwrap();
            for palette in &template.palette {
                for rotation in [
                    Rotation::None,
                    Rotation::Clockwise90,
                    Rotation::Rotate180,
                    Rotation::CounterClockwise90,
                ] {
                    assert!(
                        BlockStateResolver::resolve(palette, rotation, Mirror::None).is_some(),
                        "failed to resolve {} in {name} with {rotation:?}",
                        palette.name
                    );
                }
            }
        }
    }

    #[test]
    fn recursive_generation_never_creates_multiple_ships() {
        for seed in 0..256 {
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
            let pieces = EndCityLayout::create(
                Vector3::new(0, 70, 0),
                Rotation::from_index(random.next_bounded_i32(4) as u8),
                &mut random,
            )
            .unwrap();
            assert!(
                pieces
                    .iter()
                    .filter(|piece| piece.template_name == "ship")
                    .count()
                    <= 1
            );
        }
    }
}
