use crate::entity::player::advancement::PlayerAdvancement;
use pumpkin_data::advancement_data::AdvancementNode;
use pumpkin_data::{ADVANCEMENT_TREE, Advancement};
use std::cmp::PartialEq;

static VISIBILITY_DEPTH: usize = 2;

fn evaluate_visibility_rule(advancement: &'static Advancement, is_done: bool) -> VisibilityRule {
    let display = advancement.display;
    display.map_or(VisibilityRule::Hide, |display| {
        if is_done {
            VisibilityRule::Show
        } else if display.hidden {
            VisibilityRule::Hide
        } else {
            VisibilityRule::NoChange
        }
    })
}

fn evaluate_visibility_for_unfinished_node(ascendants: &[VisibilityRule]) -> bool {
    for visibility in ascendants.iter().take(VISIBILITY_DEPTH) {
        if *visibility == VisibilityRule::Show {
            return true;
        }
        if *visibility == VisibilityRule::Hide {
            return false;
        }
    }
    false
}

pub fn evaluate_visibility_with_rules(
    node: &AdvancementNode,
    player_advancement: &mut PlayerAdvancement,
    ascendants: &mut Vec<VisibilityRule>,
    is_done_test: &mut impl FnMut(&mut PlayerAdvancement, &AdvancementNode) -> bool,
    output: &mut impl FnMut(&mut PlayerAdvancement, &AdvancementNode, bool),
) -> bool {
    let is_self_done = is_done_test(player_advancement, node);
    let descendant_visibility = evaluate_visibility_rule(node.value, is_self_done);
    let mut is_self_or_descendant_done = is_self_done;
    ascendants.push(descendant_visibility);

    for child in &node.children {
        is_self_or_descendant_done |= evaluate_visibility_with_rules(
            &ADVANCEMENT_TREE.nodes_vector[*child],
            player_advancement,
            ascendants,
            is_done_test,
            output,
        );
    }

    let visibility =
        is_self_or_descendant_done || evaluate_visibility_for_unfinished_node(ascendants);
    ascendants.pop();
    output(player_advancement, node, visibility);
    is_self_or_descendant_done
}

pub fn evaluate_visibility(
    node: &AdvancementNode,
    player_advancement: &mut PlayerAdvancement,
    is_done: &mut impl FnMut(&mut PlayerAdvancement, &AdvancementNode) -> bool,
    output: &mut impl FnMut(&mut PlayerAdvancement, &AdvancementNode, bool),
) -> bool {
    let root = node.root();
    let mut visibility_stack: Vec<VisibilityRule> =
        vec![VisibilityRule::NoChange; VISIBILITY_DEPTH];
    evaluate_visibility_with_rules(
        root,
        player_advancement,
        &mut visibility_stack,
        is_done,
        output,
    )
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum VisibilityRule {
    Show,
    Hide,
    NoChange,
}
