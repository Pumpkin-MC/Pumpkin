use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock},
};

pub mod error;
mod key;
mod lookup;
mod mapping;
mod registry;

pub use crate::key::DataKey;
pub use crate::lookup::RegistryLookup;
pub use crate::registry::Registry;

pub trait RegistryAccess {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
    fn type_id(&self) -> TypeId;
    fn type_name(&self) -> &'static str;
}

pub static ROOT: LazyLock<Arc<Registry<dyn RegistryAccess + Send + Sync>>> =
    LazyLock::new(|| Arc::new(Registry::new()));

pub mod keys {
    use crate::{DataKey, RegistryAccess};
    use pumpkin_util::identifier::Identifier;
    use std::sync::LazyLock;

    pub static ROOT: LazyLock<DataKey<dyn RegistryAccess + Send + Sync>> =
        LazyLock::new(|| DataKey::builder(Identifier::from_static("minecraft", "root")).build());
}
