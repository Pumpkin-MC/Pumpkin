use pumpkin::{
    command::{
        CommandSender,
        argument_builder::literal,
        argument_types::entity_anchor::EntityAnchor,
        context::command_source::{CommandSource, ResultValueTaker},
        node::detached::DetachedNode,
    },
    plugin::loader::wasm::wasm_host::state::WasmCommandNode,
};
use pumpkin_util::{
    math::{vector2::Vector2, vector3::Vector3},
    text::TextComponent,
};

fn dummy_source() -> CommandSource {
    CommandSource {
        output: CommandSender::Dummy,
        world: None,
        entity: None,
        position: Vector3::default(),
        rotation: Vector2::default(),
        name: String::new(),
        display_name: TextComponent::empty(),
        server: None,
        silent: false,
        command_result_taker: ResultValueTaker::new(),
        entity_anchor: EntityAnchor::Feet,
    }
}

fn requirement_result(requirement: impl for<'a> Fn(&'a CommandSource) -> bool + Send + Sync + 'static) -> bool {
    let node = WasmCommandNode::Literal(literal("restricted"))
        .requires(requirement)
        .into_detached_node();

    let DetachedNode::Literal(node) = node else {
        panic!("expected literal node");
    };

    node.owned.requirements.evaluate(&dummy_source())
}

#[test]
fn wasm_command_node_rejects_failed_requirement() {
    assert!(!requirement_result(|_source: &CommandSource| false));
}

#[test]
fn wasm_command_node_allows_satisfied_requirement() {
    assert!(requirement_result(|_source: &CommandSource| true));
}
