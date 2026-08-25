use std::sync::LazyLock;

use pumpkin_protocol::bedrock::client::{
    CResourcePackChunkData, CResourcePackDataInfo,
    resource_packs_info::{PackIdVersion, PackInfoData},
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const ID: Uuid = Uuid::from_u128(0xb762d721269944a1a166b6eeafbc1943);
pub const VERSION: &str = "1.0.7";
const ID_STRING: &str = "b762d721-2699-44a1-a166-b6eeafbc1943";
pub(super) const RESOURCE_NAME: &str = "b762d721-2699-44a1-a166-b6eeafbc1943_1.0.7";
const CHUNK_SIZE: usize = 1_048_576;
const CHUNK_SIZE_U32: u32 = 1_048_576;
const DATA: &[u8] = include_bytes!("../../../../../assets/bedrock/advancement_translations.mcpack");
const RESOURCE_TYPE: u8 = 6;

static HASH: LazyLock<[u8; 32]> = LazyLock::new(|| Sha256::digest(DATA).into());

pub fn info_entry() -> PackInfoData {
    PackInfoData {
        pack_id_version: PackIdVersion::new(ID, VERSION.to_owned()),
        pack_size: u64::try_from(DATA.len()).unwrap_or(u64::MAX),
        content_key: String::new(),
        subpack_name: String::new(),
        content_identity: String::new(),
        has_scripts: false,
        is_addon_pack: false,
        is_ray_tracing_capable: false,
        cdn_url: String::new(),
    }
}

pub fn matches(resource_name: &str) -> bool {
    resource_name == ID_STRING || resource_name == RESOURCE_NAME
}

pub fn data_info() -> CResourcePackDataInfo<'static> {
    CResourcePackDataInfo {
        resource_name: RESOURCE_NAME,
        chunk_size: CHUNK_SIZE_U32,
        number_of_chunks: u32::try_from(DATA.len().div_ceil(CHUNK_SIZE)).unwrap_or(u32::MAX),
        file_size: u64::try_from(DATA.len()).unwrap_or(u64::MAX),
        file_hash: HASH.as_slice(),
        is_premium_pack: false,
        pack_type: RESOURCE_TYPE,
    }
}

pub fn chunk(index: i32) -> Option<CResourcePackChunkData<'static>> {
    let index = usize::try_from(index).ok()?;
    let offset = index.checked_mul(CHUNK_SIZE)?;
    let data = DATA.get(offset..offset.saturating_add(CHUNK_SIZE).min(DATA.len()))?;
    Some(CResourcePackChunkData {
        resource_name: RESOURCE_NAME,
        chunk_id: u32::try_from(index).ok()?,
        byte_offset: u64::try_from(offset).ok()?,
        chunk_data: data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_pack_is_served_as_complete_chunks() {
        let info = data_info();
        let chunk_count = i32::try_from(info.number_of_chunks).unwrap();
        let chunks = (0..chunk_count)
            .map(|index| chunk(index).unwrap().chunk_data.len())
            .sum::<usize>();

        assert_eq!(chunks, DATA.len());
        assert!(chunk(chunk_count).is_none());
        assert!(matches(ID_STRING));
        assert!(matches(RESOURCE_NAME));
    }
}
