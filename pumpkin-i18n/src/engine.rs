use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Write};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use fst::{Map, MapBuilder};
use tracing::{error, warn};
use xxhash_rust::xxh64::Xxh64;

use crate::token::{self, Token, TokenStream};

// ---------------------------------------------------------------------------
// Resolved translation
// ---------------------------------------------------------------------------

/// The result of a translation key lookup.
///
/// Variants distinguish between plain strings (no placeholders) and
/// precompiled token streams so that callers can skip the formatting
/// step when there is nothing to substitute.
#[derive(Clone, Debug)]
pub enum ResolvedTranslation {
    /// A plain string with no formatting placeholders.
    Static(Arc<str>),
    /// A precompiled token stream ready for [`format_tokens`].
    Tokenized {
        /// Original untranslated template.
        template: Arc<str>,
        /// Precompiled placeholder stream.
        tokens: TokenStream,
    },
    /// A missing key fallback.
    Missing(Arc<str>),
}

impl ResolvedTranslation {
    /// Builds a resolved translation from a raw template string.
    #[must_use]
    pub fn from_template(value: &str) -> Self {
        let template: Arc<str> = Arc::from(value);
        match token::precompile(value) {
            Some(tokens) => Self::Tokenized { template, tokens },
            None => Self::Static(template),
        }
    }

    /// Returns the original translation template.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) | Self::Missing(s) => s,
            Self::Tokenized { template, .. } => template,
        }
    }

    /// Returns the precompiled token stream, if this template contains
    /// formatting placeholders or escaped percent literals.
    #[must_use]
    pub fn tokens(&self) -> Option<&[Token]> {
        match self {
            Self::Tokenized { tokens, .. } => Some(tokens),
            Self::Static(_) | Self::Missing(_) => None,
        }
    }

    /// Returns whether this is the raw-key missing fallback.
    #[must_use]
    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// Convenience: format this resolved translation into `buf`.
    ///
    /// For [`Static`](Self::Static) this is a simple `push_str`;
    /// for [`Tokenized`](Self::Tokenized) it streams tokens.
    pub fn write_to(&self, args: &[String], buf: &mut String) {
        match self {
            Self::Static(s) | Self::Missing(s) => buf.push_str(s),
            Self::Tokenized { tokens, .. } => format_tokens(tokens, args, buf),
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting engine (zero‑regex, streaming writes)
// ---------------------------------------------------------------------------

/// Stream precompiled [`Token`]s into `buf` using the supplied `args`.
///
/// * [`Token::Text`] is copied byte‑for‑byte.
/// * [`Token::Var`] indexes into `args`; missing indices produce an empty
///   string.
///
/// The function writes directly to the buffer without intermediate
/// allocations.
pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String) {
    for token in tokens {
        match token {
            Token::Text(s) => buf.push_str(s),
            Token::Var(idx) => {
                if let Some(arg) = args.get(*idx) {
                    buf.push_str(arg);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Per‑locale FST store
// ---------------------------------------------------------------------------

/// Immutable FST‑backed store for a single locale.
struct FstLocaleStore {
    /// FST mapping `key.as_bytes()` → index into `entries`.
    fst: Map<Vec<u8>>,
    /// Precompiled entries indexed by FST output value.
    entries: Box<[ResolvedTranslation]>,
}

impl FstLocaleStore {
    /// Build an [`FstLocaleStore`] from a flat `key → translation` map.
    fn build(data: &HashMap<String, String>) -> Self {
        // 1. Collect and sort keys for deterministic FST construction.
        let mut sorted: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        sorted.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut entries: Vec<ResolvedTranslation> = Vec::with_capacity(sorted.len());

        // 2. Build FST: key → entry_index (as u64).
        let fst_bytes = {
            let mut builder = MapBuilder::memory();
            for (idx, (key, value)) in sorted.iter().enumerate() {
                let _ = builder.insert(key, idx as u64);

                entries.push(ResolvedTranslation::from_template(value));
            }
            builder.into_inner().expect("FST build failed")
        };

        let fst = Map::new(fst_bytes).expect("FST load failed");

        Self {
            fst,
            entries: entries.into_boxed_slice(),
        }
    }

    /// Look up a key in the FST. Returns `None` when the key is unknown.
    fn lookup(&self, key: &str) -> Option<&ResolvedTranslation> {
        let idx = self.fst.get(key.as_bytes())? as usize;
        self.entries.get(idx)
    }
}

// ---------------------------------------------------------------------------
// Translation engine (multi‑locale, cached, lock‑free reads)
// ---------------------------------------------------------------------------

type FastHasher = BuildHasherDefault<Xxh64>;
type ResolvedMap = DashMap<String, Arc<ResolvedTranslation>, FastHasher>;
#[cfg(not(debug_assertions))]
type SeenLogMap = DashMap<String, (), FastHasher>;

/// A high‑performance translation engine with FST‑based key lookup,
/// precompiled format tokens, and a concurrent cache.
///
/// # Design
/// * One [`FstLocaleStore`] per locale, stored behind an [`ArcSwap`] so
///   locale data can be reloaded atomically without blocking readers.
/// * A [`DashMap`] (sharded lock‑free map) with XXH64 hashing caches
///   resolved translations keyed by `"locale:namespace:key"`.
pub struct TranslationEngine {
    /// Per‑locale FST stores, atomically swappable.
    stores: ArcSwap<Box<[FstLocaleStore]>>,
    /// Runtime/plugin overrides. Checked before built-in FST data.
    overrides: Box<[ResolvedMap]>,
    /// Cache for resolved translations. Key format: `"<locale_idx>:<key>"`.
    cache: ResolvedMap,
    /// Release-build log limiter for requested-locale → English fallback.
    #[cfg(not(debug_assertions))]
    fallback_log_once: SeenLogMap,
    /// Release-build log limiter for complete translation misses.
    #[cfg(not(debug_assertions))]
    missing_log_once: SeenLogMap,
}

impl TranslationEngine {
    /// Build the engine from an array of per‑locale translation maps.
    ///
    /// `data[locale_idx]` is the flat key‑value map for that locale.
    pub fn build(data: &[HashMap<String, String>]) -> Self {
        let stores: Box<[FstLocaleStore]> = data
            .iter()
            .map(FstLocaleStore::build)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            stores: ArcSwap::from_pointee(stores),
            overrides: build_override_maps(data.len()),
            cache: DashMap::with_hasher(BuildHasherDefault::default()),
            #[cfg(not(debug_assertions))]
            fallback_log_once: DashMap::with_hasher(BuildHasherDefault::default()),
            #[cfg(not(debug_assertions))]
            missing_log_once: DashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    /// Resolve a translation key for the given locale.
    ///
    /// # Fallback strategy (identical to [`crate::store::get_translation`])
    /// 1. **Requested locale** — silent, no log.
    /// 2. **`EnUs`** — logs [`warn!`] when the key was not found in step 1.
    /// 3. **Raw key** — logs [`error!`] when neither locale contains the key.
    ///
    /// The result is cached behind [`DashMap`] so subsequent lookups are
    /// lock‑free. The return value is never `None` — at minimum the raw key
    /// is wrapped as [`ResolvedTranslation::Missing`].
    ///
    /// # Arguments
    /// * `locale_idx` — Index of the locale (use `locale as usize`).
    /// * `key` — The fully‑qualified translation key (`"namespace:entry"`).
    pub fn resolve(&self, locale_idx: usize, key: &str) -> Arc<ResolvedTranslation> {
        let cache_key = make_cache_key(locale_idx, key);

        // Fast path: cache hit (no lock contention).
        if let Some(entry) = self.cache.get(&cache_key) {
            return entry.value().clone();
        }

        let stores = self.stores.load();

        // Tier 1 – requested locale (silent)
        if let Some(entry) = self.lookup_override(locale_idx, key) {
            self.cache.insert(cache_key, entry.clone());
            return entry;
        }

        if let Some(entry) = stores.get(locale_idx).and_then(|store| store.lookup(key)) {
            let resolved = Arc::new(entry.clone());
            self.cache.insert(cache_key, resolved.clone());
            return resolved;
        }

        // Tier 2 – EnUs fallback
        if locale_idx != crate::locale::Locale::EnUs as usize {
            if let Some(entry) = self.lookup_override(crate::locale::Locale::EnUs as usize, key) {
                self.log_english_fallback(locale_idx, key);
                self.cache.insert(cache_key, entry.clone());
                return entry;
            }

            if let Some(entry) = stores
                .get(crate::locale::Locale::EnUs as usize)
                .and_then(|store| store.lookup(key))
            {
                self.log_english_fallback(locale_idx, key);
                let resolved = Arc::new(entry.clone());
                self.cache.insert(cache_key, resolved.clone());
                return resolved;
            }
        }

        // Tier 3 – raw key
        self.log_missing_translation(locale_idx, key);
        let resolved = Arc::new(ResolvedTranslation::Missing(Arc::from(key)));
        self.cache.insert(cache_key, resolved.clone());
        resolved
    }

    /// Adds or replaces a runtime translation entry.
    ///
    /// Overrides are checked before the immutable built-in FST stores. This
    /// keeps plugin/custom translation loading cheap and avoids rebuilding all
    /// locale data for a single write.
    pub fn add_translation(&self, locale_idx: usize, key: &str, translation: String) {
        if let Some(store) = self.overrides.get(locale_idx) {
            store.insert(
                key.to_owned(),
                Arc::new(ResolvedTranslation::from_template(&translation)),
            );
            self.cache.clear();
        }
    }

    /// Adds or replaces several runtime translation entries for one locale.
    pub fn add_translations<I>(&self, locale_idx: usize, entries: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let Some(store) = self.overrides.get(locale_idx) else {
            return;
        };

        for (key, translation) in entries {
            store.insert(
                key,
                Arc::new(ResolvedTranslation::from_template(&translation)),
            );
        }
        self.cache.clear();
    }

    /// Reload translation data atomically.
    ///
    /// Builds new [`FstLocaleStore`]s and swaps them in without blocking
    /// concurrent readers (wait‑free).
    pub fn reload(&self, data: &[HashMap<String, String>]) {
        let stores: Box<[FstLocaleStore]> = data
            .iter()
            .map(FstLocaleStore::build)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        self.stores.store(Arc::new(stores));
        for overrides in &self.overrides {
            overrides.clear();
        }
        #[cfg(not(debug_assertions))]
        {
            self.fallback_log_once.clear();
            self.missing_log_once.clear();
        }
        // Clear the cache so that new stores are used immediately.
        self.cache.clear();
    }

    fn lookup_override(&self, locale_idx: usize, key: &str) -> Option<Arc<ResolvedTranslation>> {
        self.overrides
            .get(locale_idx)?
            .get(key)
            .map(|entry| entry.value().clone())
    }

    #[cfg(debug_assertions)]
    fn log_english_fallback(&self, locale_idx: usize, key: &str) {
        warn!(
            locale_idx,
            key, "translation key not found in requested locale – falling back to English"
        );
    }

    #[cfg(not(debug_assertions))]
    fn log_english_fallback(&self, _locale_idx: usize, key: &str) {
        if self.fallback_log_once.insert(key.to_owned(), ()).is_none() {
            warn!(
                key,
                "translation key not found in requested locale – falling back to English"
            );
        }
    }

    #[cfg(debug_assertions)]
    fn log_missing_translation(&self, locale_idx: usize, key: &str) {
        error!(
            locale_idx,
            key, "translation key not found in any locale – returning raw key"
        );
    }

    #[cfg(not(debug_assertions))]
    fn log_missing_translation(&self, _locale_idx: usize, key: &str) {
        if self.missing_log_once.insert(key.to_owned(), ()).is_none() {
            error!(
                key,
                "translation key not found in any locale – returning raw key"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[inline]
fn make_cache_key(locale_idx: usize, key: &str) -> String {
    // Use a compact representation: "<locale_idx>:<key>"
    let mut buf = String::with_capacity(4 + key.len() + 1);
    let _ = write!(buf, "{locale_idx}:{key}");
    buf
}

fn build_override_maps(len: usize) -> Box<[ResolvedMap]> {
    (0..len)
        .map(|_| DashMap::with_hasher(BuildHasherDefault::default()))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
