use crate::damage_type::DamageTypeFile;
use pumpkin_registry::{RegistryBuilder, ReloadableRegistry, error::BootstrapError};
use pumpkin_util::identifier::Identifier;
use std::sync::Arc;

pub struct DatapackRegistries {
    pub damage_types: Arc<ReloadableRegistry<DamageTypeFile>>,
}

impl DatapackRegistries {
    #[must_use]
    pub fn new() -> Self {
        Self {
            damage_types: Arc::new(RegistryBuilder::empty_reloadable(
                &Identifier::vanilla_static("damage_type"),
            )),
        }
    }
    pub async fn reload_damage_types<I>(&self, entries: I) -> Result<(), BootstrapError>
    where
        I: IntoIterator<Item = (Identifier, DamageTypeFile)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();

        entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));

        self.damage_types.replace_entries(entries).await
    }
}

#[cfg(test)]
mod tests {
    use super::DatapackRegistries;
    use crate::damage_type::DamageTypeFile;
    use pumpkin_registry::{AsyncTypedRegistry, Registry, error::BootstrapError};
    use pumpkin_util::identifier::Identifier;

    fn damage_type(id: Identifier, message_id: &str) -> DamageTypeFile {
        DamageTypeFile {
            id,
            data: serde_json::json!({ "message_id": message_id }),
        }
    }

    #[tokio::test]
    async fn reload_damage_types_sorts_and_populates_registry() -> Result<(), BootstrapError> {
        let registries = DatapackRegistries::new();
        let alpha = Identifier::parse_static("test:alpha");
        let zeta = Identifier::parse_static("test:zeta");

        registries
            .reload_damage_types([
                (zeta.clone(), damage_type(zeta.clone(), "zeta")),
                (alpha.clone(), damage_type(alpha.clone(), "alpha")),
            ])
            .await?;

        assert_eq!(
            Registry::get_id_async(registries.damage_types.as_ref(), &alpha).await,
            Some(0),
        );
        assert_eq!(
            Registry::get_id_async(registries.damage_types.as_ref(), &zeta).await,
            Some(1),
        );

        let alpha_entry = AsyncTypedRegistry::get(registries.damage_types.as_ref(), &alpha).await;
        assert_eq!(
            alpha_entry.as_deref().map(DamageTypeFile::message_id),
            Some("alpha"),
        );

        Ok(())
    }
}
