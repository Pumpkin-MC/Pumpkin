use pumpkin_registry::{
    Registry, RegistryBuilder,
    bootstrap::RegistryEntry,
    bootstrap_provider,
};
use pumpkin_util::identifier::Identifier;
use std::sync::Arc;

bootstrap_provider! {
    WORLDGEN_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        vec![RegistryEntry::new(
            Identifier::vanilla_static("worldgen"),
            RegistryBuilder::<Arc<dyn Registry>>::frozen(
                &Identifier::vanilla_static("worldgen"),
            )
            .unwrap()
            .arc_dyn(),
        )]
    }
}
