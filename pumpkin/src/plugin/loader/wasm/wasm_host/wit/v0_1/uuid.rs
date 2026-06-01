use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::uuid::{Host, Uuid};

impl Host for PluginHostState {
    async fn generate(&mut self) -> wasmtime::Result<Uuid> {
        let u = uuid::Uuid::new_v4();
        let (high, low) = u.as_u64_pair();
        Ok(Uuid { high, low })
    }

    async fn parse(&mut self, s: String) -> wasmtime::Result<Option<Uuid>> {
        uuid::Uuid::parse_str(&s).map_or_else(
            |_| Ok(None),
            |u| {
                let (high, low) = u.as_u64_pair();
                Ok(Some(Uuid { high, low }))
            },
        )
    }

    async fn to_string(&mut self, id: Uuid) -> wasmtime::Result<String> {
        let u = uuid::Uuid::from_u64_pair(id.high, id.low);
        Ok(u.to_string())
    }
}

pub trait UuidExt {
    fn from_wit(uuid: &Uuid) -> uuid::Uuid;
    fn to_wit(uuid: &uuid::Uuid) -> Uuid;
}

impl UuidExt for Uuid {
    fn from_wit(uuid: &Uuid) -> uuid::Uuid {
        uuid::Uuid::from_u64_pair(uuid.high, uuid.low)
    }

    fn to_wit(uuid: &uuid::Uuid) -> Uuid {
        let (high, low) = uuid.as_u64_pair();
        Self { high, low }
    }
}
