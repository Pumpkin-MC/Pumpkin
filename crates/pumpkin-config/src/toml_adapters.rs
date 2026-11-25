use serde::de::DeserializeOwned;
use toml_spanner::{Context, Failed, Item};

// Reuse the wire format of external scalar types without adding TOML dependencies to their crates.
pub mod serde_value {
    use super::{Context, DeserializeOwned, Failed, Item};

    pub fn from_toml<'de, T: DeserializeOwned>(
        ctx: &mut Context<'de>,
        item: &Item<'de>,
    ) -> Result<T, Failed> {
        let value = match item.value() {
            toml_spanner::Value::Integer(integer) => i64::try_from(integer.as_i128())
                .map(toml::Value::Integer)
                .map_err(|error| ctx.report_custom_error(error, item))?,
            _ => {
                toml::Value::try_from(item).map_err(|error| ctx.report_custom_error(error, item))?
            }
        };
        value
            .try_into()
            .map_err(|error| ctx.report_custom_error(error, item))
    }
}

pub mod serde_vec {
    use super::{Context, DeserializeOwned, Failed, Item, serde_value};

    pub fn from_toml<'de, T: DeserializeOwned>(
        ctx: &mut Context<'de>,
        item: &Item<'de>,
    ) -> Result<Vec<T>, Failed> {
        let items = item.require_array(ctx)?;
        let mut values = Vec::with_capacity(items.len());
        let mut failed = false;
        for item in items {
            match serde_value::from_toml(ctx, item) {
                Ok(value) => values.push(value),
                Err(_) => failed = true,
            }
        }
        if failed { Err(Failed) } else { Ok(values) }
    }
}
