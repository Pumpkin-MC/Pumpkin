/// Creates a [`Vec<String>`] of examples from
/// the given string literals.
#[macro_export]
macro_rules! examples {
    ( $( $example:literal ),* ) => {
        vec! [
            $( $example.to_string(), )*
        ]
    };
}

pub use pumpkin_command::argument_types::*;

pub mod entity;
pub mod entity_anchor;
pub mod entity_selector;
pub mod game_profile;
pub mod objective;
pub mod pool;
pub mod resource_key;
pub mod team;
pub mod template;
