use pumpkin_data::advancement_data::{AdvancementNode, AdvancementTree};
use pumpkin_data::{Advancement, ADVANCEMENT_TREE};
use std::cmp::PartialEq;

static VISIBILITY_DEPTH : i32 = 2;

fn evaluate_visibility_rule(advancement:&'static Advancement, is_done:bool) -> VisibilityRule {
    let display = advancement.display;
    if let Some(display) = display {
        if is_done {
            VisibilityRule::Show
        } else if display.is_hidden() {
            VisibilityRule::Hide
        } else {
            VisibilityRule::NoChange
        }
    } else {
        VisibilityRule::Hide
    }
}

fn evaluate_visibility_for_unfinished_node(ascendants:Vec<VisibilityRule>) -> bool{
    for i in 0..2 {
        let visibility = ascendants[i];
        if visibility == VisibilityRule::Show {
            return true;
        }
        if visibility == VisibilityRule::Hide {
            return false;
        }
    }
    false
}

pub fn evaluate_visibility_with_rules(
    node: &AdvancementNode,
    ascendants:&mut Vec<VisibilityRule>,
    is_done_test: &impl Fn(&AdvancementNode) -> bool,
    output : &OutputFn,
) -> bool {
    let is_self_done = is_done_test(node);
    let descendant_visibility = evaluate_visibility_rule(node.advancement(), is_self_done);
    let mut is_self_or_descendant_done = is_self_done;
    ascendants.push(descendant_visibility);

    for child in node.children {
        is_self_or_descendant_done |= evaluate_visibility(ADVANCEMENT_TREE.nodes_vector[child], ascendants, is_done_test, output);
    }

    let visibility = is_self_or_descendant_done || evaluate_visibility_for_unfinished_node(ascendants);
    ascendants.pop();
    output(node, visibility);
    is_self_or_descendant_done
}

pub fn evaluate_visibility(
    node:&AdvancementNode, is_done: &impl Fn(&AdvancementNode) -> bool, output : &OutputFn
    ) {
    let root = node.root();
    let mut visibility_stack: Vec<VisibilityRule> = vec![VisibilityRule::NoChange; 2];
    evaluate_visibility_with_rules(root, &mut visibility_stack, is_done, output);
}

pub type OutputFn = impl Fn(AdvancementNode,bool);

#[derive(PartialEq,Eq,Copy)]
enum VisibilityRule {
    Show,
    Hide,
    NoChange,
}