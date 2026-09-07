use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};

use bytes::Bytes;
use flate2::Crc;
use image::{ImageEncoder, imageops::FilterType};
use pumpkin_protocol::bedrock::client::Skin;
use pumpkin_protocol::serial::PacketWrite;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

const CACHE_VERSION: u8 = 5;
const MAX_SKIN_DIMENSION: u32 = 128;
const MANIFEST_FILE: &str = "manifest.json";
const PACK_FILE: &str = "bedrock_skins.zip";
const PACK_META: &[u8] = br#"{"pack":{"pack_format":88,"min_format":[84,0],"max_format":[88,0],"description":"Pumpkin Bedrock player skins"}}"#;

#[derive(Clone)]
pub struct BedrockSkin {
    pub asset: String,
    pub cape_asset: Option<String>,
    pub slim: bool,
}

pub struct BedrockSkinPack {
    pub id: Uuid,
    pub hash: String,
    pub data: Bytes,
    skins: HashMap<Uuid, BedrockSkin>,
}

impl BedrockSkinPack {
    pub fn skin(&self, player_id: Uuid) -> Option<&BedrockSkin> {
        self.skins.get(&player_id)
    }
}

pub struct BedrockSkinPacks {
    cache_dir: Option<PathBuf>,
    registry: RwLock<SkinPackRegistry>,
    accepted: Mutex<HashMap<Uuid, AcceptedSkin>>,
}

#[derive(Default)]
struct SkinPackRegistry {
    skins: HashMap<Uuid, CachedSkin>,
    current: Option<Arc<BedrockSkinPack>>,
    revisions: HashMap<Uuid, Weak<BedrockSkinPack>>,
    dirty: bool,
}

struct AcceptedSkin {
    fingerprint: [u8; 20],
    body_fingerprint: [u8; 20],
    changed_at: Instant,
    skin: Skin,
}

struct CachedSkin {
    slim: bool,
    hash: String,
    png: Vec<u8>,
    cape_png: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize)]
struct CacheManifest {
    version: u8,
    pack_hash: String,
    skins: BTreeMap<Uuid, ManifestSkin>,
}

#[derive(Deserialize, Serialize)]
struct ManifestSkin {
    slim: bool,
    hash: String,
    #[serde(default)]
    cape: bool,
}

impl Default for BedrockSkinPacks {
    fn default() -> Self {
        Self {
            cache_dir: None,
            registry: RwLock::new(SkinPackRegistry::default()),
            accepted: Mutex::new(HashMap::new()),
        }
    }
}

impl BedrockSkinPacks {
    pub fn load(cache_dir: PathBuf) -> Self {
        let registry = match load_registry(&cache_dir) {
            Ok(registry) => registry,
            Err(error) => {
                warn!(
                    path = %cache_dir.display(),
                    "Failed to load the cached Bedrock skin pack: {error}"
                );
                SkinPackRegistry::default()
            }
        };
        Self {
            cache_dir: Some(cache_dir),
            registry: RwLock::new(registry),
            accepted: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(&self, player_id: Uuid, skin: &Skin) -> Option<BedrockSkin> {
        if is_fallback_skin(player_id, skin) {
            return self
                .registry
                .read()
                .await
                .current
                .as_ref()
                .and_then(|pack| pack.skin(player_id))
                .cloned();
        }
        let Some(cached_skin) = CachedSkin::new(skin) else {
            debug!(
                %player_id,
                skin_width = skin.image_width,
                skin_height = skin.image_height,
                cape_width = skin.cape_width,
                cape_height = skin.cape_height,
                cape_bytes = skin.cape_data.len(),
                persona = skin.is_persona,
                "Bedrock skin cannot be represented by a Java mannequin"
            );
            return None;
        };
        debug!(
            %player_id,
            skin_width = skin.image_width,
            skin_height = skin.image_height,
            cape_width = skin.cape_width,
            cape_height = skin.cape_height,
            cape_bytes = skin.cape_data.len(),
            cape_exported = cached_skin.cape_png.is_some(),
            "Prepared Bedrock skin for the Java resource pack"
        );
        let mut registry = self.registry.write().await;
        if !registry.dirty
            && registry.skins.get(&player_id).is_some_and(|cached| {
                cached.hash == cached_skin.hash && cached.slim == cached_skin.slim
            })
        {
            return registry
                .current
                .as_ref()
                .and_then(|pack| pack.skin(player_id))
                .cloned();
        }

        let previous = registry.skins.insert(player_id, cached_skin);
        let pack = match build_pack(&registry.skins) {
            Ok(pack) => pack,
            Err(error) => {
                warn!("Failed to build the aggregate Bedrock skin pack: {error}");
                if let Some(previous) = previous {
                    registry.skins.insert(player_id, previous);
                } else {
                    registry.skins.remove(&player_id);
                }
                return None;
            }
        };
        let mut dirty = false;
        if let Some(cache_dir) = &self.cache_dir {
            let result = fs::create_dir_all(cache_dir)
                .and_then(|()| {
                    if registry.dirty {
                        // An earlier registration may have failed before its
                        // images were written. Repair every asset referenced by
                        // the next manifest, not just this player's skin.
                        registry.skins.iter().try_for_each(|(&player_id, skin)| {
                            persist_skin(cache_dir, player_id, skin)
                        })
                    } else if previous
                        .as_ref()
                        .is_none_or(|previous| previous.hash != registry.skins[&player_id].hash)
                    {
                        persist_skin(cache_dir, player_id, &registry.skins[&player_id])
                    } else {
                        Ok(())
                    }
                })
                .and_then(|()| persist(cache_dir, &registry.skins, &pack));
            if let Err(error) = result {
                dirty = true;
                warn!(
                    path = %cache_dir.display(),
                    "Failed to persist the Bedrock skin pack: {error}"
                );
            }
        }
        registry.dirty = dirty;
        registry.revisions.insert(pack.id, Arc::downgrade(&pack));
        let skin = pack.skin(player_id).cloned();
        registry.current = Some(pack);
        skin
    }

    /// Applies the trust policy and rate limit to an explicit live skin update.
    pub fn accept(
        &self,
        player_id: Uuid,
        skin: Skin,
        trusted_only: bool,
        change_cooldown: Duration,
    ) -> (Skin, bool) {
        self.accept_skin(player_id, skin, trusted_only, change_cooldown, false)
    }

    /// Accepts a login skin, preserving a known cape when login omits it.
    pub fn accept_initial(
        &self,
        player_id: Uuid,
        skin: Skin,
        trusted_only: bool,
        change_cooldown: Duration,
    ) -> (Skin, bool) {
        self.accept_skin(player_id, skin, trusted_only, change_cooldown, true)
    }

    fn accept_skin(
        &self,
        player_id: Uuid,
        skin: Skin,
        trusted_only: bool,
        change_cooldown: Duration,
        is_login: bool,
    ) -> (Skin, bool) {
        let rejected_skin = (trusted_only && !skin.is_trusted) || !valid_skin(&skin);
        let skin = if rejected_skin {
            fallback_skin(player_id)
        } else {
            skin
        };
        let fingerprint = skin_fingerprint(&skin);
        let body_fingerprint = skin_body_fingerprint(&skin);
        let now = Instant::now();
        let mut accepted = self
            .accepted
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(previous) = accepted.get(&player_id) {
            if previous.fingerprint == fingerprint {
                return (previous.skin.clone(), false);
            }
            let same_body = previous.body_fingerprint == body_fingerprint;
            let previous_has_cape = has_cape(&previous.skin);
            let has_cape = has_cape(&skin);
            // Bedrock may complete the login skin with a PlayerSkin packet that
            // carries its cape. This is not a second skin change and must not be
            // rejected by the user-facing skin-change cooldown. Conversely,
            // preserve a known cape when a later login omits it.
            if is_login && same_body && previous_has_cape && !has_cape {
                return (previous.skin.clone(), false);
            }
            if !(rejected_skin || same_body && !previous_has_cape && has_cape)
                && now.duration_since(previous.changed_at) < change_cooldown
            {
                return (previous.skin.clone(), false);
            }
        }

        accepted.insert(
            player_id,
            AcceptedSkin {
                fingerprint,
                body_fingerprint,
                changed_at: now,
                skin: skin.clone(),
            },
        );
        (skin, true)
    }

    pub async fn current(&self) -> Option<Arc<BedrockSkinPack>> {
        self.registry.read().await.current.clone()
    }

    pub async fn get(&self, pack_id: Uuid) -> Option<Arc<BedrockSkinPack>> {
        let mut registry = self.registry.write().await;
        registry.revisions.retain(|_, pack| pack.strong_count() > 0);
        registry.revisions.get(&pack_id).and_then(Weak::upgrade)
    }
}

fn valid_skin(skin: &Skin) -> bool {
    fn valid_image(width: u32, height: u32, data: &[u8], allow_empty: bool) -> bool {
        if allow_empty && width == 0 && height == 0 && data.is_empty() {
            return true;
        }
        width > 0
            && height > 0
            && width <= MAX_SKIN_DIMENSION
            && height <= MAX_SKIN_DIMENSION
            && (width as usize)
                .checked_mul(height as usize)
                .and_then(|pixels| pixels.checked_mul(4))
                == Some(data.len())
    }

    valid_image(skin.image_width, skin.image_height, &skin.skin_data, false)
        && valid_image(skin.cape_width, skin.cape_height, &skin.cape_data, true)
        && skin.animations.iter().all(|animation| {
            valid_image(
                animation.image_width,
                animation.image_height,
                &animation.image_data,
                false,
            )
        })
}

impl CachedSkin {
    fn new(skin: &Skin) -> Option<Self> {
        if skin.image_width > MAX_SKIN_DIMENSION
            || skin.image_height > MAX_SKIN_DIMENSION
            || skin.cape_width > MAX_SKIN_DIMENSION
            || skin.cape_height > MAX_SKIN_DIMENSION
        {
            return None;
        }
        // Java's mannequin only supports the classic wide/slim player UVs.
        // Persona and Marketplace geometry use different atlases and render as
        // disconnected body parts when forced onto the classic model.
        if skin.is_persona {
            return None;
        }
        let resource_patch =
            serde_json::from_slice::<serde_json::Value>(&skin.resource_patch).ok()?;
        let geometry = resource_patch.get("geometry")?.get("default")?.as_str()?;
        let slim = match geometry {
            "geometry.humanoid.custom" => false,
            "geometry.humanoid.customSlim" => true,
            _ => return None,
        };
        let source = image::RgbaImage::from_raw(
            skin.image_width,
            skin.image_height,
            skin.skin_data.clone(),
        )?;
        if source.width() != source.height() {
            return None;
        }
        let body = image::imageops::resize(&source, 64, 64, FilterType::Nearest);
        let png = encode_png(&body, 64, 64)?;
        let cape_png = if skin.cape_width == 0 || skin.cape_height == 0 || skin.cape_data.is_empty()
        {
            None
        } else {
            let cape = image::RgbaImage::from_raw(
                skin.cape_width,
                skin.cape_height,
                skin.cape_data.clone(),
            )?;
            // Bedrock permits cape atlases other than the classic 2:1 layout.
            // Mannequins consume Java's fixed 64x32 UV layout, so normalize the
            // validated source rather than silently dropping those capes.
            let cape = image::imageops::resize(&cape, 64, 32, FilterType::Nearest);
            Some(encode_png(&cape, 64, 32)?)
        };
        Some(Self {
            slim,
            hash: cached_skin_hash(&png, cape_png.as_deref()),
            png,
            cape_png,
        })
    }
}

fn encode_png(image: &image::RgbaImage, width: u32, height: u32) -> Option<Vec<u8>> {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(image, width, height, image::ColorType::Rgba8.into())
        .ok()?;
    Some(png)
}

fn cached_skin_hash(body: &[u8], cape: Option<&[u8]>) -> String {
    let mut hash = Sha1::new();
    hash.update(body);
    if let Some(cape) = cape {
        hash.update([1]);
        hash.update(cape);
    } else {
        hash.update([0]);
    }
    hex::encode(hash.finalize())
}

fn load_registry(cache_dir: &Path) -> io::Result<SkinPackRegistry> {
    let manifest_path = cache_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        return Ok(SkinPackRegistry::default());
    }
    let manifest: CacheManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    // Version 4 has the same cached images and manifest fields. Only the ZIP
    // needs rebuilding for model-aware revisions and Java 26.1 compatibility.
    if manifest.version != CACHE_VERSION && manifest.version != 4 {
        return Ok(SkinPackRegistry::default());
    }

    let mut skins = HashMap::with_capacity(manifest.skins.len());
    for (player_id, entry) in manifest.skins {
        let png = fs::read(skin_path(cache_dir, player_id))?;
        let cape_png = entry
            .cape
            .then(|| fs::read(cape_path(cache_dir, player_id)))
            .transpose()?;
        if cached_skin_hash(&png, cape_png.as_deref()) != entry.hash {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("cached Bedrock skin {player_id} has an invalid hash"),
            ));
        }
        skins.insert(
            player_id,
            CachedSkin {
                slim: entry.slim,
                hash: entry.hash,
                png,
                cape_png,
            },
        );
    }

    let pack_data = fs::read(cache_dir.join(PACK_FILE));
    let mut dirty = false;
    let pack = match pack_data {
        Ok(data)
            if manifest.version == CACHE_VERSION
                && hex::encode(Sha1::digest(&data)) == manifest.pack_hash =>
        {
            pack_from_data(&skins, data)?
        }
        _ => {
            let pack = build_pack(&skins)?;
            if let Err(error) = persist(cache_dir, &skins, &pack) {
                dirty = true;
                warn!(
                    path = %cache_dir.display(),
                    "Failed to persist the rebuilt Bedrock skin pack: {error}"
                );
            }
            pack
        }
    };
    let mut revisions = HashMap::new();
    revisions.insert(pack.id, Arc::downgrade(&pack));
    Ok(SkinPackRegistry {
        skins,
        current: Some(pack),
        revisions,
        dirty,
    })
}

fn fallback_skin(player_id: Uuid) -> Skin {
    let mut skin = Skin::steve();
    let skin_id = fallback_skin_id(player_id);
    skin.skin_id.clone_from(&skin_id);
    skin.full_id = skin_id;
    skin
}

fn fallback_skin_id(player_id: Uuid) -> String {
    format!("pumpkin:fallback:{player_id}")
}

fn is_fallback_skin(player_id: Uuid, skin: &Skin) -> bool {
    skin.skin_id == fallback_skin_id(player_id)
}

fn skin_fingerprint(skin: &Skin) -> [u8; 20] {
    let mut serialized = Vec::new();
    // Writing to a Vec cannot fail; a failed write still produces a stable
    // fingerprint for the bytes written before the error.
    let _ = skin.write(&mut serialized);
    Sha1::digest(serialized).into()
}

fn skin_body_fingerprint(skin: &Skin) -> [u8; 20] {
    let mut skin = skin.clone();
    skin.cape_width = 0;
    skin.cape_height = 0;
    skin.cape_data.clear();
    skin.cape_id.clear();
    skin.persona_cape_on_classic = false;
    // Bedrock's profile hash covers the complete appearance, including the
    // cape, and therefore cannot be used to decide whether the body changed.
    skin.profile_hash.clear();
    skin_fingerprint(&skin)
}

const fn has_cape(skin: &Skin) -> bool {
    skin.cape_width != 0 && skin.cape_height != 0 && !skin.cape_data.is_empty()
}

fn build_pack(skins: &HashMap<Uuid, CachedSkin>) -> io::Result<Arc<BedrockSkinPack>> {
    let mut ordered = skins.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|(player_id, _)| player_id.as_u128());
    // Model-only changes must create a new resource-pack revision too: Java
    // clients retain the mannequin model associated with their loaded pack.
    let models = skins
        .iter()
        .map(|(&player_id, skin)| (player_id, skin.slim))
        .collect::<BTreeMap<_, _>>();
    let models = serde_json::to_vec(&models).map_err(io::Error::other)?;
    let mut files = Vec::with_capacity(ordered.len() * 2);
    for (player_id, skin) in ordered {
        files.push((
            format!(
                "assets/pumpkin/textures/bedrock_skins/{}.png",
                player_id.simple()
            ),
            skin.png.as_slice(),
        ));
        if let Some(cape) = skin.cape_png.as_deref() {
            files.push((
                format!(
                    "assets/pumpkin/textures/bedrock_skins/{}_cape.png",
                    player_id.simple()
                ),
                cape,
            ));
        }
    }
    let mut entries = Vec::with_capacity(files.len() + 2);
    entries.push(("pack.mcmeta", PACK_META));
    entries.push(("pumpkin_skin_models.json", models.as_slice()));
    entries.extend(files.iter().map(|(path, data)| (path.as_str(), *data)));
    pack_from_data(skins, stored_zip(&entries)?)
}

fn pack_from_data(
    skins: &HashMap<Uuid, CachedSkin>,
    data: Vec<u8>,
) -> io::Result<Arc<BedrockSkinPack>> {
    if skins.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot build an empty Bedrock skin pack",
        ));
    }
    let digest = Sha1::digest(&data);
    let id = Uuid::new_v3(&Uuid::nil(), digest.as_slice());
    let skins = skins
        .iter()
        .map(|(player_id, skin)| {
            (
                *player_id,
                BedrockSkin {
                    asset: format!("pumpkin:bedrock_skins/{}", player_id.simple()),
                    cape_asset: skin
                        .cape_png
                        .as_ref()
                        .map(|_| format!("pumpkin:bedrock_skins/{}_cape", player_id.simple())),
                    slim: skin.slim,
                },
            )
        })
        .collect();
    Ok(Arc::new(BedrockSkinPack {
        id,
        hash: hex::encode(digest),
        data: data.into(),
        skins,
    }))
}

fn persist(
    cache_dir: &Path,
    skins: &HashMap<Uuid, CachedSkin>,
    pack: &BedrockSkinPack,
) -> io::Result<()> {
    fs::create_dir_all(cache_dir)?;
    write_atomic(&cache_dir.join(PACK_FILE), &pack.data)?;
    let manifest = CacheManifest {
        version: CACHE_VERSION,
        pack_hash: pack.hash.clone(),
        skins: skins
            .iter()
            .map(|(player_id, skin)| {
                (
                    *player_id,
                    ManifestSkin {
                        slim: skin.slim,
                        hash: skin.hash.clone(),
                        cape: skin.cape_png.is_some(),
                    },
                )
            })
            .collect(),
    };
    let manifest = serde_json::to_vec(&manifest).map_err(io::Error::other)?;
    write_atomic(&cache_dir.join(MANIFEST_FILE), &manifest)
}

fn skin_path(cache_dir: &Path, player_id: Uuid) -> PathBuf {
    cache_dir.join(format!("{}.png", player_id.simple()))
}

fn cape_path(cache_dir: &Path, player_id: Uuid) -> PathBuf {
    cache_dir.join(format!("{}_cape.png", player_id.simple()))
}

fn persist_skin(cache_dir: &Path, player_id: Uuid, skin: &CachedSkin) -> io::Result<()> {
    write_atomic(&skin_path(cache_dir, player_id), &skin.png)?;
    let cape_path = cape_path(cache_dir, player_id);
    if let Some(cape) = &skin.cape_png {
        write_atomic(&cape_path, cape)
    } else {
        match fs::remove_file(cape_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, data)?;
    fs::rename(temporary, path)
}

#[must_use]
pub fn resource_url(
    server_address: &str,
    port: u16,
    public_url: Option<&str>,
    pack_id: Uuid,
) -> String {
    if let Some(public_url) = public_url {
        return format!(
            "{}/v1/skin-packs/{pack_id}",
            public_url.trim_end_matches('/')
        );
    }
    let address = server_address.split('\0').next().unwrap_or(server_address);
    let host = if let Some(end) = address.strip_prefix('[').and_then(|value| value.find(']')) {
        &address[..=end + 1]
    } else if address.matches(':').count() == 1 {
        address
            .rsplit_once(':')
            .filter(|(_, suffix)| suffix.parse::<u16>().is_ok())
            .map_or(address, |(host, _)| host)
    } else if address.contains(':') {
        return format!("http://[{address}]:{port}/v1/skin-packs/{pack_id}");
    } else {
        address
    };
    format!("http://{host}:{port}/v1/skin-packs/{pack_id}")
}

fn stored_zip(entries: &[(&str, &[u8])]) -> io::Result<Vec<u8>> {
    struct Entry<'a> {
        name: &'a [u8],
        crc: u32,
        size: u32,
        offset: u32,
    }

    let mut archive = Vec::new();
    let mut directory = Vec::with_capacity(entries.len());
    for &(name, data) in entries {
        let name = name.as_bytes();
        let name_len = u16::try_from(name.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP path is too long"))?;
        let size = u32::try_from(data.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP entry is too large"))?;
        let offset = u32::try_from(archive.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP is too large"))?;
        let mut crc = Crc::new();
        crc.update(data);
        let crc = crc.sum();

        push_u32(&mut archive, 0x0403_4b50);
        push_u16(&mut archive, 20);
        push_u16(&mut archive, 0);
        push_u16(&mut archive, 0);
        archive.extend_from_slice(&[0; 4]);
        push_u32(&mut archive, crc);
        push_u32(&mut archive, size);
        push_u32(&mut archive, size);
        push_u16(&mut archive, name_len);
        push_u16(&mut archive, 0);
        archive.extend_from_slice(name);
        archive.extend_from_slice(data);
        directory.push(Entry {
            name,
            crc,
            size,
            offset,
        });
    }

    let directory_offset = u32::try_from(archive.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP is too large"))?;
    for entry in &directory {
        push_u32(&mut archive, 0x0201_4b50);
        push_u16(&mut archive, 20);
        push_u16(&mut archive, 20);
        archive.extend_from_slice(&[0; 8]);
        push_u32(&mut archive, entry.crc);
        push_u32(&mut archive, entry.size);
        push_u32(&mut archive, entry.size);
        push_u16(&mut archive, entry.name.len() as u16);
        archive.extend_from_slice(&[0; 8]);
        push_u32(&mut archive, 0);
        push_u32(&mut archive, entry.offset);
        archive.extend_from_slice(entry.name);
    }
    let directory_size = u32::try_from(archive.len())
        .ok()
        .and_then(|end| end.checked_sub(directory_offset))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "ZIP is too large"))?;
    let count = u16::try_from(directory.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many ZIP entries"))?;
    push_u32(&mut archive, 0x0605_4b50);
    archive.extend_from_slice(&[0; 4]);
    push_u16(&mut archive, count);
    push_u16(&mut archive, count);
    push_u32(&mut archive, directory_size);
    push_u32(&mut archive, directory_offset);
    push_u16(&mut archive, 0);
    Ok(archive)
}

fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn persists_and_reuses_an_aggregate_skin_pack() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let skin = Skin::steve();
        let registered = packs.register(player_id, &skin).await.unwrap();
        assert_eq!(
            registered.asset,
            format!("pumpkin:bedrock_skins/{}", player_id.simple())
        );
        let first = packs.current().await.unwrap();
        assert!(first.skin(player_id).is_some());
        assert!(
            first
                .data
                .windows(PACK_META.len())
                .any(|bytes| bytes == PACK_META)
        );
        let metadata: serde_json::Value = serde_json::from_slice(PACK_META).unwrap();
        assert_eq!(metadata["pack"]["min_format"], serde_json::json!([84, 0]));
        assert_eq!(metadata["pack"]["max_format"], serde_json::json!([88, 0]));
        packs.register(player_id, &skin).await.unwrap();
        assert!(Arc::ptr_eq(&first, &packs.current().await.unwrap()));

        let second_player = Uuid::new_v4();
        packs.register(second_player, &skin).await.unwrap();
        let second = packs.current().await.unwrap();
        assert_ne!(second.id, first.id);
        assert!(second.skin(player_id).is_some());
        assert!(second.skin(second_player).is_some());
        assert!(packs.get(first.id).await.is_some());

        drop(packs);
        let reloaded = BedrockSkinPacks::load(directory.path().to_owned());
        let reloaded = reloaded.current().await.unwrap();
        assert_eq!(reloaded.id, second.id);
        assert_eq!(reloaded.hash, second.hash);
        assert!(reloaded.skin(player_id).is_some());
        assert!(reloaded.skin(second_player).is_some());
    }

    #[tokio::test]
    async fn retains_and_repairs_version_four_skins_when_migration_cannot_persist() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let mut skin = Skin::steve();
        skin.cape_width = 64;
        skin.cape_height = 32;
        skin.cape_data = vec![0x7f; 64 * 32 * 4];
        packs.register(player_id, &skin).await.unwrap();
        let expected = packs.current().await.unwrap();

        let body = fs::read(skin_path(directory.path(), player_id)).unwrap();
        let cape = fs::read(cape_path(directory.path(), player_id)).unwrap();
        let old_metadata = br#"{"pack":{"pack_format":88,"min_format":[88,0],"max_format":[88,0],"description":"Pumpkin Bedrock player skins"}}"#;
        let body_path = format!(
            "assets/pumpkin/textures/bedrock_skins/{}.png",
            player_id.simple()
        );
        let cape_path_in_pack = format!(
            "assets/pumpkin/textures/bedrock_skins/{}_cape.png",
            player_id.simple()
        );
        let old_zip = stored_zip(&[
            ("pack.mcmeta", old_metadata),
            (&body_path, &body),
            (&cape_path_in_pack, &cape),
        ])
        .unwrap();
        let manifest_path = directory.path().join(MANIFEST_FILE);
        let mut manifest: CacheManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.version = 4;
        manifest.pack_hash = hex::encode(Sha1::digest(&old_zip));
        fs::write(directory.path().join(PACK_FILE), &old_zip).unwrap();
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        drop(packs);

        let blocked_path = directory.path().join(PACK_FILE).with_extension("tmp");
        fs::create_dir(&blocked_path).unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        assert!(packs.registry.read().await.dirty);
        let migrated = packs.current().await.unwrap();
        assert_eq!(migrated.id, expected.id);
        assert_eq!(migrated.hash, expected.hash);
        assert_ne!(migrated.hash, manifest.pack_hash);
        assert!(migrated.skin(player_id).unwrap().cape_asset.is_some());
        assert_eq!(
            fs::read(skin_path(directory.path(), player_id)).unwrap(),
            body
        );
        assert_eq!(
            fs::read(cape_path(directory.path(), player_id)).unwrap(),
            cape
        );
        let unchanged_manifest: CacheManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        assert_eq!(unchanged_manifest.version, 4);
        assert_eq!(unchanged_manifest.pack_hash, manifest.pack_hash);

        fs::remove_dir(blocked_path).unwrap();
        packs.register(player_id, &skin).await.unwrap();
        assert!(!packs.registry.read().await.dirty);
        drop(packs);
        let reloaded = BedrockSkinPacks::load(directory.path().to_owned());
        let reloaded = reloaded.current().await.unwrap();
        assert_eq!(reloaded.id, migrated.id);
        assert_eq!(reloaded.hash, migrated.hash);
        assert!(reloaded.skin(player_id).unwrap().cape_asset.is_some());
        let manifest: CacheManifest =
            serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.version, CACHE_VERSION);
        assert_eq!(manifest.pack_hash, migrated.hash);
    }

    #[tokio::test]
    async fn model_only_update_changes_the_persisted_pack_revision() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let (mut skin, changed) =
            packs.accept_initial(player_id, Skin::steve(), true, Duration::ZERO);
        assert!(changed);
        assert!(!packs.register(player_id, &skin).await.unwrap().slim);
        let wide = packs.current().await.unwrap();
        let wide_png = fs::read(skin_path(directory.path(), player_id)).unwrap();

        skin.resource_patch =
            br#"{"geometry":{"default":"geometry.humanoid.customSlim"}}"#.to_vec();
        let (skin, changed) = packs.accept(player_id, skin, true, Duration::ZERO);
        assert!(changed);
        assert!(packs.register(player_id, &skin).await.unwrap().slim);
        let slim = packs.current().await.unwrap();

        assert_eq!(
            fs::read(skin_path(directory.path(), player_id)).unwrap(),
            wide_png
        );
        assert_ne!(wide.id, slim.id);
        assert_ne!(wide.hash, slim.hash);
        assert_ne!(wide.data, slim.data);
        assert!(
            !packs
                .get(wide.id)
                .await
                .unwrap()
                .skin(player_id)
                .unwrap()
                .slim
        );
        packs.register(player_id, &skin).await.unwrap();
        assert!(Arc::ptr_eq(&slim, &packs.current().await.unwrap()));

        drop(packs);
        let reloaded = BedrockSkinPacks::load(directory.path().to_owned());
        let reloaded = reloaded.current().await.unwrap();
        assert_eq!(reloaded.id, slim.id);
        assert_eq!(reloaded.hash, slim.hash);
        assert!(reloaded.skin(player_id).unwrap().slim);
    }

    #[tokio::test]
    async fn another_registration_repairs_all_assets_after_a_persistence_failure() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let first_player = Uuid::new_v4();
        let mut skin = Skin::steve();
        skin.cape_width = 64;
        skin.cape_height = 32;
        skin.cape_data = vec![0x7f; 64 * 32 * 4];

        // A directory at the image destination deterministically fails the
        // atomic rename, including when the tests run with elevated privileges.
        let blocked_path = skin_path(directory.path(), first_player);
        fs::create_dir(&blocked_path).unwrap();
        packs.register(first_player, &skin).await.unwrap();
        assert!(packs.registry.read().await.dirty);
        assert!(!directory.path().join(MANIFEST_FILE).exists());
        fs::remove_dir(blocked_path).unwrap();

        let second_player = Uuid::new_v4();
        packs.register(second_player, &Skin::steve()).await.unwrap();
        assert!(!packs.registry.read().await.dirty);
        assert!(skin_path(directory.path(), first_player).is_file());
        assert!(cape_path(directory.path(), first_player).is_file());
        let expected = packs.current().await.unwrap();

        drop(packs);
        let reloaded = BedrockSkinPacks::load(directory.path().to_owned());
        let reloaded = reloaded.current().await.unwrap();
        assert_eq!(reloaded.id, expected.id);
        assert_eq!(reloaded.hash, expected.hash);
        assert!(reloaded.skin(first_player).unwrap().cape_asset.is_some());
        assert!(reloaded.skin(second_player).is_some());
    }

    #[tokio::test]
    async fn exports_classic_cape_as_a_separate_java_texture() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let mut skin = Skin::steve();
        skin.cape_width = 64;
        skin.cape_height = 32;
        skin.cape_data = vec![0x7f; 64 * 32 * 4];

        let registered = packs.register(player_id, &skin).await.unwrap();
        let cape_asset = format!("pumpkin:bedrock_skins/{}_cape", player_id.simple());
        assert_eq!(registered.cape_asset.as_deref(), Some(cape_asset.as_str()));

        let registry = packs.registry.read().await;
        let cached = &registry.skins[&player_id];
        let body = image::load_from_memory(&cached.png).unwrap();
        assert_eq!((body.width(), body.height()), (64, 64));
        let cape = image::load_from_memory(cached.cape_png.as_ref().unwrap()).unwrap();
        assert_eq!((cape.width(), cape.height()), (64, 32));
    }

    #[tokio::test]
    async fn normalizes_nonstandard_bedrock_cape_atlases() {
        let packs = BedrockSkinPacks::default();
        let player_id = Uuid::new_v4();
        let mut skin = Skin::steve();
        skin.cape_width = 64;
        skin.cape_height = 64;
        skin.cape_data = vec![0x7f; 64 * 64 * 4];

        let registered = packs.register(player_id, &skin).await.unwrap();
        assert!(registered.cape_asset.is_some());
        let registry = packs.registry.read().await;
        let cape =
            image::load_from_memory(registry.skins[&player_id].cape_png.as_ref().unwrap()).unwrap();
        assert_eq!((cape.width(), cape.height()), (64, 32));
    }

    #[tokio::test]
    async fn fallback_does_not_replace_a_cached_skin() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let mut skin = Skin::steve();
        skin.skin_data.fill(0x7f);
        skin.cape_width = 64;
        skin.cape_height = 32;
        skin.cape_data = vec![0x7f; 64 * 32 * 4];

        let registered = packs.register(player_id, &skin).await.unwrap();
        let pack = packs.current().await.unwrap();
        let pack_id = pack.id;
        drop(pack);
        drop(packs);

        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let fallback = fallback_skin(player_id);
        let preserved = packs.register(player_id, &fallback).await.unwrap();

        assert_eq!(preserved.asset, registered.asset);
        assert_eq!(preserved.cape_asset, registered.cape_asset);
        assert_eq!(packs.current().await.unwrap().id, pack_id);
    }

    #[tokio::test]
    async fn rejects_non_classic_java_geometry() {
        let packs = BedrockSkinPacks::default();
        let mut skin = Skin::steve();
        skin.is_persona = true;
        assert!(packs.register(Uuid::new_v4(), &skin).await.is_none());

        skin.is_persona = false;
        skin.resource_patch = br#"{"geometry":{"default":"geometry.custom"}}"#.to_vec();
        assert!(packs.register(Uuid::new_v4(), &skin).await.is_none());

        skin.resource_patch = br#"{"geometry":{"default":"geometry.humanoid.custom"}}"#.to_vec();
        skin.image_width = 129;
        skin.image_height = 129;
        skin.skin_data = vec![0; 129 * 129 * 4];
        assert!(packs.register(Uuid::new_v4(), &skin).await.is_none());
    }

    #[tokio::test]
    async fn accepts_formatted_classic_resource_patch() {
        let packs = BedrockSkinPacks::default();
        let mut skin = Skin::steve();
        skin.resource_patch =
            br#"{ "geometry": { "default": "geometry.humanoid.customSlim" } }"#.to_vec();

        let registered = packs.register(Uuid::new_v4(), &skin).await.unwrap();
        assert!(registered.slim);
    }

    #[test]
    fn applies_skin_trust_and_change_cooldown() {
        let packs = BedrockSkinPacks::default();
        let player_id = Uuid::new_v4();
        let skin = Skin::steve();
        let (accepted, changed) = packs.accept(player_id, skin, true, Duration::from_mins(1));
        assert!(changed);

        let mut replacement = accepted.clone();
        replacement.skin_id = "replacement".to_string();
        let (rate_limited, changed) =
            packs.accept(player_id, replacement, true, Duration::from_mins(1));
        assert!(!changed);
        assert_eq!(rate_limited.skin_id, accepted.skin_id);

        let mut untrusted = accepted;
        untrusted.skin_id = "untrusted".to_string();
        untrusted.is_trusted = false;
        let (fallback, changed) = packs.accept(player_id, untrusted, true, Duration::from_mins(1));
        assert!(changed);
        assert_eq!(fallback.skin_id, fallback_skin_id(player_id));
        assert!(fallback.is_trusted);
    }

    #[tokio::test]
    async fn completes_preserves_and_removes_a_cape_through_the_persisted_cache() {
        let directory = tempfile::tempdir().unwrap();
        let packs = BedrockSkinPacks::load(directory.path().to_owned());
        let player_id = Uuid::new_v4();
        let skin = Skin::steve();
        let (_, changed) =
            packs.accept_initial(player_id, skin.clone(), true, Duration::from_mins(1));
        assert!(changed);

        let mut with_cape = skin.clone();
        with_cape.cape_width = 64;
        with_cape.cape_height = 32;
        with_cape.cape_data = vec![0x7f; 64 * 32 * 4];
        with_cape.cape_id = "delayed-cape".to_string();
        with_cape.profile_hash = "appearance-with-cape".to_string();
        let (accepted, changed) = packs.accept(player_id, with_cape, true, Duration::from_mins(1));
        assert!(changed);
        assert!(has_cape(&accepted));
        assert!(
            packs
                .register(player_id, &accepted)
                .await
                .unwrap()
                .cape_asset
                .is_some()
        );

        let with_cape = packs.current().await.unwrap();
        let (preserved, changed) =
            packs.accept_initial(player_id, skin.clone(), true, Duration::ZERO);
        assert!(!changed);
        assert!(has_cape(&preserved));
        packs.register(player_id, &preserved).await.unwrap();
        assert!(Arc::ptr_eq(&with_cape, &packs.current().await.unwrap()));

        let (limited, changed) =
            packs.accept(player_id, skin.clone(), true, Duration::from_mins(1));
        assert!(!changed);
        assert!(has_cape(&limited));

        let (without_cape, changed) = packs.accept(player_id, skin, true, Duration::ZERO);
        assert!(changed);
        assert!(!has_cape(&without_cape));
        assert!(
            packs
                .register(player_id, &without_cape)
                .await
                .unwrap()
                .cape_asset
                .is_none()
        );
        let without_cape = packs.current().await.unwrap();
        assert_ne!(with_cape.id, without_cape.id);
        assert!(!cape_path(directory.path(), player_id).exists());

        drop(packs);
        let reloaded = BedrockSkinPacks::load(directory.path().to_owned());
        let reloaded = reloaded.current().await.unwrap();
        assert_eq!(reloaded.id, without_cape.id);
        assert_eq!(reloaded.hash, without_cape.hash);
        assert!(reloaded.skin(player_id).unwrap().cape_asset.is_none());
    }

    #[test]
    fn resource_url_reuses_the_java_handshake_host() {
        assert_eq!(
            resource_url("example.org:25565", 19132, None, Uuid::nil()),
            "http://example.org:19132/v1/skin-packs/00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            resource_url("[2001:db8::1]:25565", 19132, None, Uuid::nil()),
            "http://[2001:db8::1]:19132/v1/skin-packs/00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            resource_url(
                "ignored.invalid",
                19132,
                Some("https://packs.example.org/pumpkin/"),
                Uuid::nil()
            ),
            "https://packs.example.org/pumpkin/v1/skin-packs/00000000-0000-0000-0000-000000000000"
        );
    }
}
