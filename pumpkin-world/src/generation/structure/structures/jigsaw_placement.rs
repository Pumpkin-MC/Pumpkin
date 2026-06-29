use pumpkin_data::{Mirror, Rotation};

use crate::generation::structure::{
    structures::{StructureGeneratorContext, StructurePosition},
    template::{StructureTemplate, get_template},
};
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;
use std::collections::BinaryHeap;
use std::sync::Arc;

use super::jigsaw::{
    JigsawBlock, JigsawJointType, JigsawJunction, JigsawProjection, PoolElementStructurePiece,
    TemplatePool,
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

/// Simple lookup for Pool Aliases introduced in 1.20+
pub struct PoolAliasLookup;

impl PoolAliasLookup {
    #[must_use]
    pub const fn lookup<'a>(&self, id: &'a str) -> &'a str {
        // In a complete implementation, this would look up the alias in the context/registry.
        // Returning the ID directly acts as a fallback/default behavior.
        id
    }
}

impl JigsawPlacement {
    #[expect(clippy::too_many_arguments)]
    pub fn add_pieces(
        context: &mut StructureGeneratorContext,
        start_pool_id: &str,
        start_jigsaw: Option<&str>,
        max_depth: i32,
        position: BlockPos,
        do_expansion_hack: bool,
        project_start_to_heightmap: bool,
        max_distance_from_center: MaxDistance,
        dimension_padding: DimensionPadding,
        liquid_settings: LiquidSettings,
        pool_alias_lookup: &PoolAliasLookup,
    ) -> Option<StructurePosition> {
        if max_distance_from_center.horizontal > MAX_TOTAL_STRUCTURE_RANGE {
            return None;
        }

        let max_depth = max_depth.clamp(MIN_DEPTH, MAX_DEPTH);

        let actual_start_pool_id = pool_alias_lookup.lookup(start_pool_id);
        let pool = TemplatePool::discover(actual_start_pool_id)?;
        let element = pool.get_random_element(&mut context.random);
        let template = get_template(element.template)?;

        let rotation = Rotation::from_index(context.random.next_bounded_i32(4) as u8);

        let mut anchored_position = position;
        if let Some(target_jigsaw_id) = start_jigsaw {
            let mut found_anchor = None;
            let jigsaws = get_jigsaw_blocks(&template);
            for jigsaw in jigsaws {
                if jigsaw.name == target_jigsaw_id {
                    let rotated_pos = rotation.transform_pos(jigsaw.pos.0, template.size);
                    found_anchor = Some(position.add(rotated_pos.x, rotated_pos.y, rotated_pos.z));
                    break;
                }
            }

            if let Some(anchor) = found_anchor {
                anchored_position = anchor;
            } else {
                return None;
            }
        }

        let local_anchor_position = anchored_position.0.sub(&position.0);
        let adjusted_position = BlockPos(position.0.sub(&local_anchor_position));

        let rotated_size = rotation.transform_size(template.size);
        let mut box_ = BlockBox::new(
            adjusted_position.0.x,
            adjusted_position.0.y,
            adjusted_position.0.z,
            adjusted_position.0.x + rotated_size.x - 1,
            adjusted_position.0.y + rotated_size.y - 1,
            adjusted_position.0.z + rotated_size.z - 1,
        );

        let center_x = i32::midpoint(box_.max.x, box_.min.x);
        let center_z = i32::midpoint(box_.max.z, box_.min.z);

        let bottom_y = if project_start_to_heightmap {
            if let Some(sampler) = &mut context.height_sampler {
                sampler
                    .estimate_height(center_x, center_z)
                    .max(context.sea_level)
            } else {
                adjusted_position.0.y
            }
        } else {
            adjusted_position.0.y
        };

        let old_min_y = box_.min.y;
        box_.move_pos(0, bottom_y - old_min_y, 0);

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
                let rotated_pos = rotation.transform_pos(jigsaw.pos.0, template.size);
                jigsaw.pos = BlockPos(rotated_pos).add(
                    adjusted_position.0.x,
                    bottom_y,
                    adjusted_position.0.z,
                );
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
            template: Arc::clone(&template),
            pos: BlockPos::new(adjusted_position.0.x, bottom_y, adjusted_position.0.z),
            rotation,
            mirror: Mirror::None,
            jigsaw_blocks,
            junctions: Vec::new(),
            ground_level_delta: 0,
            liquid_settings,
            projection: element.projection,
        });

        let mut collector = super::StructurePiecesCollector::new();
        let mut pieces = Vec::new();
        let mut piece_projections = Vec::new();

        pieces.push(first_piece);
        piece_projections.push(element.projection);

        if max_depth > 0 {
            let mut placing = BinaryHeap::new();
            let mut sequence = 0;

            placing.push(PieceState {
                piece_idx: 0,
                depth: 0,
                priority: 0,
                sequence,
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
                let source_projection = piece_projections[source_piece_idx];
                let source_rigid = source_projection == JigsawProjection::Rigid;

                'jigsaw_loop: for source_jigsaw in &source_jigsaws {
                    let raw_pool_id = &source_jigsaw.pool;
                    if raw_pool_id == "minecraft:empty" || raw_pool_id.is_empty() {
                        continue;
                    }

                    let target_pool_id = pool_alias_lookup.lookup(raw_pool_id);
                    let target_pool = match TemplatePool::discover(target_pool_id) {
                        Some(p) => p,
                        None => continue,
                    };

                    let mut target_elements = Vec::new();
                    if depth < max_depth {
                        let mut main_elements = target_pool.elements.clone();
                        for i in (1..main_elements.len()).rev() {
                            let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                            main_elements.swap(i, j);
                        }
                        target_elements.extend(main_elements);
                    }

                    let fallback_pool_id = pool_alias_lookup.lookup(&target_pool.fallback);
                    if let Some(fallback_pool) = TemplatePool::discover(fallback_pool_id) {
                        let mut fallback_elements = fallback_pool.elements.clone();
                        for i in (1..fallback_elements.len()).rev() {
                            let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                            fallback_elements.swap(i, j);
                        }
                        target_elements.extend(fallback_elements);
                    }

                    for element in target_elements {
                        if element.template == "minecraft:empty" {
                            break;
                        }

                        let target_template_arc = match get_template(element.template) {
                            Some(t) => t,
                            None => continue,
                        };
                        let target_template = target_template_arc.as_ref();
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
                            let target_jigsaws = get_jigsaw_blocks(target_template);

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
                                let target_jigsaw_local_pos = target_rotation
                                    .transform_pos(target_jigsaw.pos.0, target_template.size);
                                let target_jigsaw_local_y = target_jigsaw_local_pos.y;

                                let delta_y = source_jigsaw_local_y - target_jigsaw_local_y
                                    + source_facing.to_vector().y;

                                let target_box_y;
                                let mut source_jigsaw_base_height = i32::MIN;

                                if source_rigid && target_rigid {
                                    target_box_y = source_box.min.y + delta_y;
                                } else {
                                    if source_jigsaw_base_height == i32::MIN {
                                        source_jigsaw_base_height =
                                            if let Some(sampler) = &mut context.height_sampler {
                                                let height = sampler.estimate_height(
                                                    source_jigsaw_pos.0.x,
                                                    source_jigsaw_pos.0.z,
                                                );
                                                if project_start_to_heightmap {
                                                    height.max(context.sea_level)
                                                } else {
                                                    height
                                                }
                                            } else {
                                                source_jigsaw_pos.0.y
                                            };
                                    }
                                    target_box_y =
                                        source_jigsaw_base_height - target_jigsaw_local_y;
                                }

                                let raw_target_pos = BlockPos::new(
                                    target_jigsaw_pos.0.x - target_jigsaw_local_pos.x,
                                    target_jigsaw_pos.0.y - target_jigsaw_local_pos.y,
                                    target_jigsaw_pos.0.z - target_jigsaw_local_pos.z,
                                );
                                let y_offset = target_box_y - raw_target_pos.0.y;

                                let mut target_pos = raw_target_pos;
                                target_pos.0.y += y_offset;

                                let rotated_target_size =
                                    target_rotation.transform_size(target_template.size);
                                let mut target_box = BlockBox::new(
                                    target_pos.0.x,
                                    target_pos.0.y,
                                    target_pos.0.z,
                                    target_pos.0.x + rotated_target_size.x - 1,
                                    target_pos.0.y + rotated_target_size.y - 1,
                                    target_pos.0.z + rotated_target_size.z - 1,
                                );

                                let mut expand_to = 0;
                                if do_expansion_hack
                                    && (target_box.max.y - target_box.min.y + 1) <= 16
                                {
                                    for tj in &target_jigsaws {
                                        let tj_facing =
                                            rotate_direction(tj.facing, target_rotation);
                                        let rotated_tj_pos = target_rotation
                                            .transform_pos(tj.pos.0, target_template.size);
                                        let rotated_tj_target_pos =
                                            rotated_tj_pos.add(&tj_facing.to_vector());

                                        let rotated_size =
                                            target_rotation.transform_size(target_template.size);
                                        let hack_box = BlockBox::new(
                                            0,
                                            0,
                                            0,
                                            rotated_size.x - 1,
                                            rotated_size.y - 1,
                                            rotated_size.z - 1,
                                        );

                                        if hack_box.contains(
                                            rotated_tj_target_pos.x,
                                            rotated_tj_target_pos.y,
                                            rotated_tj_target_pos.z,
                                        ) {
                                            let child_pool_id = pool_alias_lookup.lookup(&tj.pool);
                                            let child_pool_max_y =
                                                get_pool_max_y_size(child_pool_id);

                                            let mut child_fallback_max_y = 0;
                                            if let Some(cp) = TemplatePool::discover(child_pool_id)
                                            {
                                                child_fallback_max_y = get_pool_max_y_size(
                                                    pool_alias_lookup.lookup(&cp.fallback),
                                                );
                                            }

                                            expand_to = expand_to
                                                .max(child_pool_max_y)
                                                .max(child_fallback_max_y);
                                        }
                                    }
                                }

                                if expand_to > 0 {
                                    let new_size = (expand_to + 1)
                                        .max(target_box.max.y - target_box.min.y + 1);
                                    target_box.max.y = target_box.min.y + new_size - 1;
                                }

                                if !is_box_inside(&global_bounding_box, &target_box) {
                                    continue;
                                }

                                if !intersects_any(&pieces, &target_box) {
                                    let mut child_jigsaw_blocks = Vec::new();
                                    for block in &target_template.blocks {
                                        if let Some(mut cj) = JigsawBlock::from_template_block(
                                            block,
                                            &target_template.palette[block.state as usize],
                                        ) {
                                            let rotated_pos = target_rotation
                                                .transform_pos(cj.pos.0, target_template.size);
                                            cj.pos = BlockPos(rotated_pos).add(
                                                target_pos.0.x,
                                                target_pos.0.y,
                                                target_pos.0.z,
                                            );
                                            cj.facing =
                                                rotate_direction(cj.facing, target_rotation);
                                            cj.up = rotate_direction(cj.up, target_rotation);
                                            child_jigsaw_blocks.push(cj);
                                        }
                                    }

                                    let source_ground_level_delta =
                                        pieces[source_piece_idx].ground_level_delta;
                                    let target_ground_level_delta = if target_rigid {
                                        source_ground_level_delta - delta_y
                                    } else {
                                        0
                                    };

                                    let target_piece = Box::new(PoolElementStructurePiece {
                                        piece: crate::generation::structure::structures::StructurePiece::new(
                                            crate::generation::structure::piece::StructurePieceType::Jigsaw,
                                            target_box,
                                            depth as u32 + 1,
                                        ),
                                        template: target_template_arc.clone(),
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
                                    piece_projections.push(target_projection);

                                    let junction_y = if source_rigid {
                                        source_box.min.y + source_jigsaw_local_y
                                    } else if target_rigid {
                                        target_box_y + target_jigsaw_local_y
                                    } else {
                                        if source_jigsaw_base_height == i32::MIN {
                                            source_jigsaw_base_height = if let Some(sampler) =
                                                &mut context.height_sampler
                                            {
                                                let height = sampler.estimate_height(
                                                    source_jigsaw_pos.0.x,
                                                    source_jigsaw_pos.0.z,
                                                );
                                                if project_start_to_heightmap {
                                                    height.max(context.sea_level)
                                                } else {
                                                    height
                                                }
                                            } else {
                                                source_jigsaw_pos.0.y
                                            };
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
        if element.template == "minecraft:empty" {
            continue;
        }
        if let Some(template) = get_template(element.template) {
            max_y = max_y.max(template.size.y);
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

const fn intersects_exclusive(a: &BlockBox, b: &BlockBox) -> bool {
    // Strictly greater/less than checks perfectly emulate Vanilla's AABB deflate(0.25)
    // by completely ignoring touching boundaries where coords are equal.
    a.max.x > b.min.x
        && a.min.x < b.max.x
        && a.max.y > b.min.y
        && a.min.y < b.max.y
        && a.max.z > b.min.z
        && a.min.z < b.max.z
}

fn intersects_any(pieces: &[Box<PoolElementStructurePiece>], box_: &BlockBox) -> bool {
    pieces
        .iter()
        .any(|piece| intersects_exclusive(&piece.piece.bounding_box, box_))
}

struct PieceState {
    piece_idx: usize,
    depth: i32,
    priority: i32,
    sequence: usize,
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

fn can_attach(source: &JigsawBlock, target: &JigsawBlock, target_rotation: Rotation) -> bool {
    if source.target != target.name || target.target != source.name {
        return false;
    }
    let rotated_target_facing = rotate_direction(target.facing, target_rotation);
    if source.facing.opposite() != rotated_target_facing {
        return false;
    }

    // Joint alignment for vertical connections
    if (source.joint == JigsawJointType::Aligned || target.joint == JigsawJointType::Aligned)
        && (source.facing == pumpkin_util::BlockDirection::Up
            || source.facing == pumpkin_util::BlockDirection::Down)
    {
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
        Rotation::Rotate180 => dir.opposite(),
        Rotation::CounterClockwise90 => match dir {
            BlockDirection::North => BlockDirection::West,
            BlockDirection::West => BlockDirection::South,
            BlockDirection::South => BlockDirection::East,
            BlockDirection::East => BlockDirection::North,
            _ => dir,
        },
    }
}
