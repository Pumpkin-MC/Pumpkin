use pumpkin_data::{Mirror, Rotation};

use crate::generation::structure::{
    structures::{StructureGeneratorContext, StructurePosition},
    template::{StructureTemplate, get_template},
};
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomImpl;
use std::collections::BinaryHeap;
use std::sync::Arc;

use super::jigsaw::{
    JigsawBlock, JigsawJointType, JigsawJunction, JigsawProjection, PoolElement, PoolElementKind,
    PoolElementStructurePiece, TemplatePool,
};

pub struct JigsawPlacement;

pub struct DimensionPadding {
    pub top: i32,
    pub bottom: i32,
}

impl DimensionPadding {
    pub const ZERO: Self = Self { top: 0, bottom: 0 };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidSettings {
    IgnoreWaterlogDone,
    ApplyWaterlog,
}

pub struct MaxDistance {
    pub horizontal: i32,
    pub vertical: i32,
}

impl MaxDistance {
    #[must_use]
    pub const fn new(horizontal: i32) -> Self {
        Self {
            horizontal,
            vertical: 384, // Default Y_SIZE (min_y to max_y)
        }
    }
}

pub const MAX_TOTAL_STRUCTURE_RANGE: i32 = 128;
pub const MIN_DEPTH: i32 = 0;
pub const MAX_DEPTH: i32 = 20;

/// Dynamic lookup for Pool Aliases introduced in 1.20+ (e.g. Trial Chambers spawner/contents aliases).
#[derive(Debug, Clone, Default)]
pub struct PoolAliasLookup {
    aliases: std::collections::HashMap<String, Vec<(String, u32)>>,
}

impl PoolAliasLookup {
    #[must_use]
    pub fn new() -> Self {
        Self {
            aliases: std::collections::HashMap::new(),
        }
    }

    pub fn add_direct(&mut self, alias: String, target: String) {
        self.aliases.insert(alias, vec![(target, 1)]);
    }

    pub fn add_random(&mut self, alias: String, targets: Vec<(String, u32)>) {
        self.aliases.insert(alias, targets);
    }

    #[must_use]
    pub fn lookup<'a>(
        &'a self,
        id: &'a str,
        random: &mut pumpkin_util::random::RandomGenerator,
    ) -> &'a str {
        if let Some(targets) = self.aliases.get(id) {
            if targets.is_empty() {
                return id;
            }
            if targets.len() == 1 {
                return &targets[0].0;
            }
            let total_weight: u32 = targets.iter().map(|(_, w)| *w).sum();
            if total_weight == 0 {
                return &targets[0].0;
            }
            let mut r = random.next_bounded_i32(total_weight as i32) as u32;
            for (target, weight) in targets {
                if r < *weight {
                    return target.as_str();
                }
                r -= *weight;
            }
            &targets[0].0
        } else {
            id
        }
    }
}

impl JigsawPlacement {
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::too_many_lines)]
    pub fn add_pieces(
        context: &mut StructureGeneratorContext,
        start_pool_id: &str,
        start_jigsaw: Option<&str>,
        max_depth: i32,
        position: BlockPos,
        do_expansion_hack: bool,
        project_start_to_heightmap: bool,
        max_distance_from_center: &MaxDistance,
        dimension_padding: &DimensionPadding,
        liquid_settings: LiquidSettings,
        pool_alias_lookup: &PoolAliasLookup,
    ) -> Option<StructurePosition> {
        if max_distance_from_center.horizontal > MAX_TOTAL_STRUCTURE_RANGE {
            return None;
        }

        let max_depth = max_depth.clamp(MIN_DEPTH, MAX_DEPTH);

        let actual_start_pool_id = pool_alias_lookup.lookup(start_pool_id, &mut context.random);
        let pool = TemplatePool::discover(actual_start_pool_id)?;
        let rotation = Rotation::from_index(context.random.next_bounded_i32(4) as u8);
        let element = pool.get_random_element(&mut context.random).clone();
        let template = element.first_template()?;

        let mut anchored_position = position;
        if let Some(target_jigsaw_id) = start_jigsaw {
            let mut found_anchor = None;
            let jigsaws = get_jigsaw_blocks(&template);
            for jigsaw in jigsaws {
                if jigsaw.name == target_jigsaw_id {
                    let rotated_pos = rotate_pos(jigsaw.pos.0, rotation);
                    found_anchor = Some(position.add(rotated_pos.x, rotated_pos.y, rotated_pos.z));
                    break;
                }
            }

            {
                let anchor = found_anchor?;
                anchored_position = anchor;
            }
        }

        let local_anchor_position = anchored_position.0.sub(&position.0);
        let adjusted_position = BlockPos(position.0.sub(&local_anchor_position));

        let mut box_ = rotated_box(adjusted_position, template.size, rotation);

        let center_x = i32::midpoint(box_.max.x, box_.min.x);
        let center_z = i32::midpoint(box_.max.z, box_.min.z);

        let bottom_y = if project_start_to_heightmap {
            context
                .height_sampler
                .as_mut()
                .map_or(adjusted_position.0.y, |sampler| {
                    sampler.estimate_height(center_x, center_z)
                })
        } else {
            adjusted_position.0.y
        };

        let ground_level_delta = 1;
        let y_offset = bottom_y - (box_.min.y + ground_level_delta);
        box_.move_pos(0, y_offset, 0);
        let piece_pos = adjusted_position.add(0, y_offset, 0);

        if box_.min.y < context.min_y + dimension_padding.bottom
            || box_.max.y > context.min_y + 320 - dimension_padding.top
        {
            return None;
        }

        let center_y = bottom_y + local_anchor_position.y;

        let global_bounding_box = BlockBox::new(
            center_x - max_distance_from_center.horizontal,
            (center_y - max_distance_from_center.vertical)
                .max(context.min_y + dimension_padding.bottom),
            center_z - max_distance_from_center.horizontal,
            center_x + max_distance_from_center.horizontal + 1,
            (center_y + max_distance_from_center.vertical + 1)
                .min(context.min_y + 320 - dimension_padding.top),
            center_z + max_distance_from_center.horizontal + 1,
        );

        let mut jigsaw_blocks = Vec::new();
        for block in &template.blocks {
            if let Some(mut jigsaw) =
                JigsawBlock::from_template_block(block, &template.palette[block.state as usize])
            {
                let rotated_pos = rotate_pos(jigsaw.pos.0, rotation);
                jigsaw.pos = BlockPos(rotated_pos).add(piece_pos.0.x, piece_pos.0.y, piece_pos.0.z);
                jigsaw.facing = rotate_direction(jigsaw.facing, rotation);
                jigsaw.up = rotate_direction(jigsaw.up, rotation);
                jigsaw_blocks.push(jigsaw);
            }
        }

        let first_piece = Box::new(PoolElementStructurePiece {
            piece: crate::generation::structure::structures::StructurePiece::new(
                crate::generation::structure::piece::StructurePieceType::Jigsaw,
                box_,
                0,
            ),
            element: element.clone(),
            pos: piece_pos,
            rotation,
            mirror: Mirror::None,
            jigsaw_blocks,
            junctions: Vec::new(),
            ground_level_delta,
            liquid_settings,
            projection: element.projection,
        });

        let mut collector = super::StructurePiecesCollector::new();
        let mut pieces = Vec::new();
        // The expansion hack enlarges free space for children without changing
        // the template bounds stored on the resulting structure piece.
        let mut piece_collision_boxes = Vec::new();
        let mut piece_projections = Vec::new();
        let mut collision_spaces = vec![CollisionSpace {
            bounds: global_bounding_box,
            occupied: vec![box_],
        }];

        pieces.push(first_piece);
        piece_collision_boxes.push(box_);
        piece_projections.push(element.projection);

        if max_depth > 0 {
            let mut placing = BinaryHeap::new();
            let mut sequence = 0;

            placing.push(PieceState {
                piece_idx: 0,
                depth: 0,
                priority: 0,
                sequence,
                collision_space: 0,
            });

            while let Some(state) = placing.pop() {
                let depth = state.depth;

                let source_piece_idx = state.piece_idx;
                let mut source_jigsaws =
                    std::mem::take(&mut pieces[source_piece_idx].jigsaw_blocks);

                for i in (1..source_jigsaws.len()).rev() {
                    let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                    source_jigsaws.swap(i, j);
                }
                source_jigsaws.sort_by_key(|j| std::cmp::Reverse(j.selection_priority));

                let source_box = pieces[source_piece_idx].piece.bounding_box;
                let source_collision_box = piece_collision_boxes[source_piece_idx];
                let source_projection = piece_projections[source_piece_idx];
                let source_rigid = source_projection == JigsawProjection::Rigid;
                let mut interior_collision_space = None;

                'jigsaw_loop: for source_jigsaw in &source_jigsaws {
                    let raw_pool_id = &source_jigsaw.pool;
                    if raw_pool_id == "minecraft:empty" || raw_pool_id.is_empty() {
                        continue;
                    }

                    let target_pool_id = pool_alias_lookup.lookup(raw_pool_id, &mut context.random);
                    let Some(target_pool) = TemplatePool::discover(target_pool_id) else {
                        continue;
                    };

                    let mut target_elements = Vec::new();
                    if depth < max_depth {
                        target_elements
                            .extend(target_pool.get_shuffled_elements(&mut context.random));
                    }

                    let fallback_pool_id =
                        pool_alias_lookup.lookup(&target_pool.fallback, &mut context.random);
                    if let Some(fallback_pool) = TemplatePool::discover(fallback_pool_id) {
                        target_elements
                            .extend(fallback_pool.get_shuffled_elements(&mut context.random));
                    }

                    for element in target_elements {
                        if element.is_empty() {
                            break;
                        }

                        let Some(target_size) = get_element_size(&element) else {
                            continue;
                        };
                        let target_projection = element.projection;
                        let target_rigid = target_projection == JigsawProjection::Rigid;

                        let mut rotations = [
                            Rotation::None,
                            Rotation::Clockwise90,
                            Rotation::Rotate180,
                            Rotation::CounterClockwise90,
                        ];

                        for i in (1..4).rev() {
                            let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                            rotations.swap(i, j);
                        }

                        for target_rotation in rotations {
                            let target_jigsaws = get_element_jigsaw_blocks(&element);

                            let mut target_jigsaws_shuffled = target_jigsaws.clone();
                            for i in (1..target_jigsaws_shuffled.len()).rev() {
                                let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                                target_jigsaws_shuffled.swap(i, j);
                            }

                            for target_jigsaw in target_jigsaws_shuffled {
                                if !can_attach(source_jigsaw, &target_jigsaw, target_rotation) {
                                    continue;
                                }

                                let source_facing = source_jigsaw.facing;
                                let source_jigsaw_pos = source_jigsaw.pos;
                                let target_jigsaw_pos = source_jigsaw_pos.add(
                                    source_facing.to_vector().x,
                                    source_facing.to_vector().y,
                                    source_facing.to_vector().z,
                                );

                                let source_jigsaw_local_y =
                                    source_jigsaw_pos.0.y - source_box.min.y;
                                let target_jigsaw_local_pos =
                                    rotate_pos(target_jigsaw.pos.0, target_rotation);
                                let target_jigsaw_local_y = target_jigsaw_local_pos.y;

                                let delta_y = source_jigsaw_local_y - target_jigsaw_local_y
                                    + source_facing.to_vector().y;

                                let mut source_jigsaw_base_height = i32::MIN;

                                let target_box_y = if source_rigid && target_rigid {
                                    source_box.min.y + delta_y
                                } else {
                                    if source_jigsaw_base_height == i32::MIN {
                                        source_jigsaw_base_height = context
                                            .height_sampler
                                            .as_mut()
                                            .map_or(source_jigsaw_pos.0.y, |sampler| {
                                                sampler.estimate_height(
                                                    source_jigsaw_pos.0.x,
                                                    source_jigsaw_pos.0.z,
                                                )
                                            });
                                    }
                                    source_jigsaw_base_height - target_jigsaw_local_y
                                };

                                let raw_target_pos = BlockPos::new(
                                    target_jigsaw_pos.0.x - target_jigsaw_local_pos.x,
                                    target_jigsaw_pos.0.y - target_jigsaw_local_pos.y,
                                    target_jigsaw_pos.0.z - target_jigsaw_local_pos.z,
                                );
                                let y_offset = target_box_y - raw_target_pos.0.y;

                                let mut target_pos = raw_target_pos;
                                target_pos.0.y += y_offset;

                                let target_box =
                                    rotated_box(target_pos, target_size, target_rotation);
                                let mut target_collision_box = target_box;

                                let mut expand_to = 0;
                                if do_expansion_hack
                                    && (target_box.max.y - target_box.min.y + 1) <= 16
                                {
                                    for tj in &target_jigsaws {
                                        let tj_facing =
                                            rotate_direction(tj.facing, target_rotation);
                                        let rotated_tj_pos = rotate_pos(tj.pos.0, target_rotation);
                                        let rotated_tj_target_pos =
                                            rotated_tj_pos.add(&tj_facing.to_vector());

                                        let hack_box = rotated_box(
                                            BlockPos::new(0, 0, 0),
                                            target_size,
                                            target_rotation,
                                        );

                                        if hack_box.contains(
                                            rotated_tj_target_pos.x,
                                            rotated_tj_target_pos.y,
                                            rotated_tj_target_pos.z,
                                        ) {
                                            let child_pool_id = pool_alias_lookup
                                                .lookup(&tj.pool, &mut context.random);
                                            let child_pool_max_y =
                                                get_pool_max_y_size(child_pool_id);

                                            let child_fallback_max_y = if let Some(cp) =
                                                TemplatePool::discover(child_pool_id)
                                            {
                                                get_pool_max_y_size(
                                                    pool_alias_lookup
                                                        .lookup(&cp.fallback, &mut context.random),
                                                )
                                            } else {
                                                0
                                            };

                                            expand_to = expand_to
                                                .max(child_pool_max_y)
                                                .max(child_fallback_max_y);
                                        }
                                    }
                                }

                                if expand_to > 0 {
                                    let max_y_offset = (expand_to + 1).max(
                                        target_collision_box.max.y - target_collision_box.min.y,
                                    );
                                    target_collision_box.max.y =
                                        target_collision_box.min.y + max_y_offset;
                                }

                                let collision_space = if source_collision_box.contains(
                                    target_jigsaw_pos.0.x,
                                    target_jigsaw_pos.0.y,
                                    target_jigsaw_pos.0.z,
                                ) {
                                    *interior_collision_space.get_or_insert_with(|| {
                                        collision_spaces.push(CollisionSpace {
                                            bounds: source_collision_box,
                                            occupied: Vec::new(),
                                        });
                                        collision_spaces.len() - 1
                                    })
                                } else {
                                    state.collision_space
                                };
                                let space = &collision_spaces[collision_space];
                                let can_place = is_box_inside(&space.bounds, &target_collision_box)
                                    && !space
                                        .occupied
                                        .iter()
                                        .any(|box_| boxes_intersect(box_, &target_collision_box));

                                if can_place {
                                    collision_spaces[collision_space]
                                        .occupied
                                        .push(target_collision_box);
                                    let mut child_jigsaw_blocks = Vec::new();
                                    for mut cj in get_element_jigsaw_blocks(&element) {
                                        let rotated_pos = rotate_pos(cj.pos.0, target_rotation);
                                        cj.pos = BlockPos(rotated_pos).add(
                                            target_pos.0.x,
                                            target_pos.0.y,
                                            target_pos.0.z,
                                        );
                                        cj.facing = rotate_direction(cj.facing, target_rotation);
                                        cj.up = rotate_direction(cj.up, target_rotation);
                                        child_jigsaw_blocks.push(cj);
                                    }

                                    let source_ground_level_delta =
                                        pieces[source_piece_idx].ground_level_delta;
                                    let target_ground_level_delta = if target_rigid {
                                        source_ground_level_delta - delta_y
                                    } else {
                                        1
                                    };

                                    let target_piece = Box::new(PoolElementStructurePiece {
                                        piece: crate::generation::structure::structures::StructurePiece::new(
                                            crate::generation::structure::piece::StructurePieceType::Jigsaw,
                                            target_box,
                                            depth as u32 + 1,
                                        ),
                                        element: element.clone(),
                                        pos: target_pos,
                                        rotation: target_rotation,
                                        mirror: Mirror::None,
                                        jigsaw_blocks: child_jigsaw_blocks,
                                        junctions: Vec::new(),
                                        ground_level_delta: target_ground_level_delta,
                                        liquid_settings,
                                        projection: target_projection,
                                    });

                                    let target_piece_idx = pieces.len();
                                    pieces.push(target_piece);
                                    piece_collision_boxes.push(target_collision_box);
                                    piece_projections.push(target_projection);

                                    let junction_y = if source_rigid {
                                        source_box.min.y + source_jigsaw_local_y
                                    } else if target_rigid {
                                        target_box_y + target_jigsaw_local_y
                                    } else {
                                        if source_jigsaw_base_height == i32::MIN {
                                            source_jigsaw_base_height = context
                                                .height_sampler
                                                .as_mut()
                                                .map_or(source_jigsaw_pos.0.y, |sampler| {
                                                    sampler.estimate_height(
                                                        source_jigsaw_pos.0.x,
                                                        source_jigsaw_pos.0.z,
                                                    )
                                                });
                                        }
                                        source_jigsaw_base_height + delta_y / 2
                                    };

                                    pieces[source_piece_idx].add_junction(JigsawJunction {
                                        source_x: target_jigsaw_pos.0.x,
                                        source_ground_y: junction_y - source_jigsaw_local_y
                                            + source_ground_level_delta,
                                        source_z: target_jigsaw_pos.0.z,
                                        delta_y,
                                        projection: target_projection,
                                    });
                                    pieces[target_piece_idx].add_junction(JigsawJunction {
                                        source_x: source_jigsaw_pos.0.x,
                                        source_ground_y: junction_y - target_jigsaw_local_y
                                            + target_ground_level_delta,
                                        source_z: source_jigsaw_pos.0.z,
                                        delta_y: -delta_y,
                                        projection: source_projection,
                                    });

                                    sequence += 1;
                                    if depth < max_depth {
                                        placing.push(PieceState {
                                            piece_idx: target_piece_idx,
                                            depth: depth + 1,
                                            priority: source_jigsaw.placement_priority,
                                            sequence,
                                            collision_space,
                                        });
                                    }

                                    continue 'jigsaw_loop;
                                }
                            }
                        }
                    }
                }
                pieces[source_piece_idx].jigsaw_blocks = source_jigsaws;
            }
        }

        for piece in pieces {
            collector.add_piece(piece);
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(center_x, center_y, center_z),
            collector: Arc::new(std::sync::Mutex::new(collector)),
        })
    }
}

// Helper to determine the max Y height of a pool for the expansion hack
fn get_pool_max_y_size(pool_id: &str) -> i32 {
    let Some(pool) = TemplatePool::discover(pool_id) else {
        return 0;
    };

    let mut max_y = 0;
    for element in &pool.elements {
        if element.is_empty() {
            continue;
        }
        if let Some(size) = get_element_size(element) {
            max_y = max_y.max(size.y);
        }
    }
    max_y
}

const fn is_box_inside(outer: &BlockBox, inner: &BlockBox) -> bool {
    inner.min.x >= outer.min.x
        && inner.max.x <= outer.max.x
        && inner.min.y >= outer.min.y
        && inner.max.y <= outer.max.y
        && inner.min.z >= outer.min.z
        && inner.max.z <= outer.max.z
}

const fn boxes_intersect(a: &BlockBox, b: &BlockBox) -> bool {
    // Vanilla turns inclusive block boxes into AABBs, then deflates the candidate by 0.25.
    // Two integer boxes therefore collide exactly when they share at least one block.
    a.max.x >= b.min.x
        && a.min.x <= b.max.x
        && a.max.y >= b.min.y
        && a.min.y <= b.max.y
        && a.max.z >= b.min.z
        && a.min.z <= b.max.z
}

struct CollisionSpace {
    bounds: BlockBox,
    occupied: Vec<BlockBox>,
}

struct PieceState {
    piece_idx: usize,
    depth: i32,
    priority: i32,
    sequence: usize,
    collision_space: usize,
}

impl PartialEq for PieceState {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PieceState {}

impl PartialOrd for PieceState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PieceState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

fn get_jigsaw_blocks(template: &StructureTemplate) -> Vec<JigsawBlock> {
    let mut jigsaws = Vec::new();
    for block in &template.blocks {
        if let Some(jigsaw) =
            JigsawBlock::from_template_block(block, &template.palette[block.state as usize])
        {
            jigsaws.push(jigsaw);
        }
    }
    jigsaws
}

const fn rotate_pos(pos: Vector3<i32>, rotation: Rotation) -> Vector3<i32> {
    let (x, z) = rotation.rotate_offset(pos.x, pos.z);
    Vector3::new(x, pos.y, z)
}

fn rotated_box(origin: BlockPos, size: Vector3<i32>, rotation: Rotation) -> BlockBox {
    let corner = rotate_pos(Vector3::new(size.x - 1, size.y - 1, size.z - 1), rotation);
    BlockBox::new(
        origin.0.x.min(origin.0.x + corner.x),
        origin.0.y,
        origin.0.z.min(origin.0.z + corner.z),
        origin.0.x.max(origin.0.x + corner.x),
        origin.0.y + corner.y,
        origin.0.z.max(origin.0.z + corner.z),
    )
}

fn get_element_size(element: &PoolElement) -> Option<pumpkin_util::math::vector3::Vector3<i32>> {
    fn size_for_kind(kind: &PoolElementKind) -> Option<pumpkin_util::math::vector3::Vector3<i32>> {
        use pumpkin_util::math::vector3::Vector3;

        match kind {
            PoolElementKind::Empty => None,
            PoolElementKind::Feature(_) => Some(Vector3::new(1, 1, 1)),
            PoolElementKind::Single { template, .. } => get_template(template).map(|t| t.size),
            PoolElementKind::List(elements) => {
                let mut result = Vector3::new(0, 0, 0);
                let mut found = false;
                for size in elements.iter().filter_map(size_for_kind) {
                    result.x = result.x.max(size.x);
                    result.y = result.y.max(size.y);
                    result.z = result.z.max(size.z);
                    found = true;
                }
                found.then_some(result)
            }
        }
    }

    size_for_kind(&element.kind)
}

fn get_element_jigsaw_blocks(element: &PoolElement) -> Vec<JigsawBlock> {
    fn jigsaws_for_kind(kind: &PoolElementKind) -> Vec<JigsawBlock> {
        match kind {
            PoolElementKind::Single { template, .. } => get_template(template)
                .map_or_else(Vec::new, |template| get_jigsaw_blocks(&template)),
            PoolElementKind::List(elements) => {
                elements.first().map_or_else(Vec::new, jigsaws_for_kind)
            }
            PoolElementKind::Feature(_) => vec![JigsawBlock {
                pos: BlockPos::new(0, 0, 0),
                name: "minecraft:bottom".to_string(),
                target: "minecraft:empty".to_string(),
                pool: "minecraft:empty".to_string(),
                final_state: "minecraft:air".to_string(),
                joint: JigsawJointType::Rollable,
                facing: pumpkin_util::BlockDirection::Down,
                up: pumpkin_util::BlockDirection::South,
                selection_priority: 0,
                placement_priority: 0,
            }],
            PoolElementKind::Empty => Vec::new(),
        }
    }

    jigsaws_for_kind(&element.kind)
}

fn can_attach(source: &JigsawBlock, target: &JigsawBlock, target_rotation: Rotation) -> bool {
    if source.target != target.name {
        return false;
    }
    let rotated_target_facing = rotate_direction(target.facing, target_rotation);
    if source.facing.opposite() != rotated_target_facing {
        return false;
    }

    if source.joint == JigsawJointType::Aligned {
        let rotated_target_up = rotate_direction(target.up, target_rotation);
        return source.up == rotated_target_up;
    }

    true
}

const fn rotate_direction(
    dir: pumpkin_util::BlockDirection,
    rotation: Rotation,
) -> pumpkin_util::BlockDirection {
    use pumpkin_util::BlockDirection;
    match rotation {
        Rotation::None => dir,
        Rotation::Clockwise90 => match dir {
            BlockDirection::North => BlockDirection::East,
            BlockDirection::East => BlockDirection::South,
            BlockDirection::South => BlockDirection::West,
            BlockDirection::West => BlockDirection::North,
            _ => dir,
        },
        Rotation::Rotate180 => match dir {
            BlockDirection::North => BlockDirection::South,
            BlockDirection::South => BlockDirection::North,
            BlockDirection::West => BlockDirection::East,
            BlockDirection::East => BlockDirection::West,
            _ => dir,
        },
        Rotation::CounterClockwise90 => match dir {
            BlockDirection::North => BlockDirection::West,
            BlockDirection::West => BlockDirection::South,
            BlockDirection::South => BlockDirection::East,
            BlockDirection::East => BlockDirection::North,
            _ => dir,
        },
    }
}
