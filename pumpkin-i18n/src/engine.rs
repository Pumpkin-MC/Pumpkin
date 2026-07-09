use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use fst::{Map, MapBuilder};
use tracing::{debug, error};
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

    /// Returns the translated string, or `raw_key` when the key is missing.
    #[must_use]
    pub fn value_or_raw(&self, raw_key: &str) -> String {
        if self.is_missing() {
            raw_key.to_owned()
        } else {
            self.as_str().to_owned()
        }
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
                } else {
                    debug!(
                        idx,
                        args_len = args.len(),
                        "translation placeholder out of range — skipping"
                    );
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
        // 1. Lowercase keys for case-insensitive lookup, then sort.
        let mut sorted: Vec<(String, &str)> = data
            .iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v.as_str()))
            .collect();
        sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        // Deduplicate keys that collide after lowercasing (e.g. "Foo" / "foo").
        sorted.dedup_by(|a, b| a.0 == b.0);

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
/// precompiled format tokens, and a concurrent per‑locale cache.
///
/// # Design
/// * One [`FstLocaleStore`] per locale, stored behind an [`ArcSwap`] so
///   locale data can be reloaded atomically without blocking readers.
/// * A per‑locale [`DashMap`] (sharded lock‑free map, XXH64-hashed) avoids
///   allocating a composite cache key on every resolve call and enables O(1)
///   locale‑wide cache eviction.
pub struct TranslationEngine {
    /// Per‑locale FST stores, atomically swappable.
    stores: ArcSwap<Box<[FstLocaleStore]>>,
    /// Runtime/plugin overrides. Checked before built-in FST data.
    overrides: Box<[ResolvedMap]>,
    /// Per‑locale resolved-translation cache.  `cache[locale_idx]` holds
    /// entries keyed by the normalised (lowercased) translation key.
    cache: Box<[ResolvedMap]>,
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
    #[must_use]
    pub fn build(data: &[HashMap<String, String>]) -> Self {
        let stores = build_stores(data);
        let num_locales = data.len();
        Self {
            stores: ArcSwap::from_pointee(stores),
            overrides: build_override_maps(num_locales),
            cache: build_cache_maps(num_locales),
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
    /// 2. **`EnUs`** — logs [`debug!`] when the key was not found in step 1.
    /// 3. **Raw key** — logs [`error!`] when neither locale contains the key.
    ///
    /// The result is cached in the per‑locale [`DashMap`] so subsequent
    /// lookups are lock‑free and require **zero allocation** on cache hit.
    /// The return value is never `None` — at minimum the raw key is wrapped
    /// as [`ResolvedTranslation::Missing`].
    ///
    /// # Arguments
    /// * `locale_idx` — Index of the locale (use `locale as usize`).
    /// * `key` — The fully‑qualified translation key (`"namespace:entry"`).
    pub fn resolve(&self, locale_idx: usize, key: &str) -> Arc<ResolvedTranslation> {
        let key = normalize_key(key);
        let cache = &self.cache[locale_idx];

        // Fast path: cache hit — zero allocation, no composite-key format.
        if let Some(entry) = cache.get(key.as_ref()) {
            return entry.value().clone();
        }

        let stores = self.stores.load();

        // Helper: insert into the requesting locale's cache and return.
        let insert = |k: &str, v: Arc<ResolvedTranslation>| -> Arc<ResolvedTranslation> {
            cache.insert(k.to_owned(), Arc::clone(&v));
            v
        };

        // Tier 1 – requested locale (silent)
        if let Some(entry) = self.lookup_override(locale_idx, key.as_ref()) {
            return insert(key.as_ref(), entry);
        }

        if let Some(entry) = stores
            .get(locale_idx)
            .and_then(|store| store.lookup(key.as_ref()))
        {
            return insert(key.as_ref(), Arc::new(entry.clone()));
        }

        // Tier 2 – EnUs fallback
        if locale_idx != crate::locale::Locale::EnUs as usize {
            if let Some(entry) =
                self.lookup_override(crate::locale::Locale::EnUs as usize, key.as_ref())
            {
                self.log_english_fallback(locale_idx, key.as_ref());
                return insert(key.as_ref(), entry);
            }

            if let Some(entry) = stores
                .get(crate::locale::Locale::EnUs as usize)
                .and_then(|store| store.lookup(key.as_ref()))
            {
                self.log_english_fallback(locale_idx, key.as_ref());
                return insert(key.as_ref(), Arc::new(entry.clone()));
            }
        }

        // Tier 3 – raw key
        self.log_missing_translation(locale_idx, key.as_ref());
        let raw: Arc<str> = Arc::from(key.into_owned());
        insert(
            raw.as_ref(),
            Arc::new(ResolvedTranslation::Missing(Arc::clone(&raw))),
        )
    }

    /// Adds or replaces a runtime translation entry.
    ///
    /// Overrides are checked before the immutable built-in FST stores. This
    /// keeps plugin/custom translation loading cheap and avoids rebuilding all
    /// locale data for a single write.
    pub fn add_translation(&self, locale_idx: usize, key: &str, translation: &str) {
        if let Some(store) = self.overrides.get(locale_idx) {
            let normalized = normalize_key(key).into_owned();
            store.insert(
                normalized.clone(),
                Arc::new(ResolvedTranslation::from_template(translation)),
            );
            self.cache[locale_idx].remove(&normalized);
        }
    }

    /// Adds or replaces several runtime translation entries for one locale.
    ///
    /// Clears the entire per‑locale cache afterwards (O(1)) so that future
    /// lookups pick up the fresh overrides.  Other locales are unaffected.
    pub fn add_translations<I>(&self, locale_idx: usize, entries: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let Some(store) = self.overrides.get(locale_idx) else {
            return;
        };

        for (key, translation) in entries {
            store.insert(
                key.to_ascii_lowercase(),
                Arc::new(ResolvedTranslation::from_template(&translation)),
            );
        }
        self.cache[locale_idx].clear();
    }

    /// Reload translation data atomically.
    ///
    /// Builds new [`FstLocaleStore`]s and swaps them in without blocking
    /// concurrent readers (wait‑free).
    pub fn reload(&self, data: &[HashMap<String, String>]) {
        let stores = build_stores(data);
        self.stores.store(Arc::new(stores));
        for overrides in &self.overrides {
            overrides.clear();
        }
        #[cfg(not(debug_assertions))]
        self.fallback_log_once.clear();
        #[cfg(not(debug_assertions))]
        self.missing_log_once.clear();
        for cache in &self.cache {
            cache.clear();
        }
    }

    fn lookup_override(&self, locale_idx: usize, key: &str) -> Option<Arc<ResolvedTranslation>> {
        self.overrides
            .get(locale_idx)?
            .get(key)
            .map(|entry| entry.value().clone())
    }

    #[allow(clippy::unused_self)] // `self` unused in debug builds (no log limiter)
    fn log_english_fallback(&self, _locale_idx: usize, key: &str) {
        #[cfg(not(debug_assertions))]
        if self.fallback_log_once.insert(key.to_owned(), ()).is_some() {
            return;
        }
        debug!(
            key,
            "translation key not found in requested locale – falling back to English"
        );
    }

    #[allow(clippy::unused_self)] // `self` unused in debug builds (no log limiter)
    fn log_missing_translation(&self, _locale_idx: usize, key: &str) {
        #[cfg(not(debug_assertions))]
        if self.missing_log_once.insert(key.to_owned(), ()).is_some() {
            return;
        }
        error!(
            key,
            "translation key not found in any locale – returning raw key"
        );
    }
}

// ---------------------------------------------------------------------------
// Normalize key
// ---------------------------------------------------------------------------

/// Normalise a translation key to lowercase for case-insensitive lookup.
/// Avoids allocation when the key is already fully lowercase.
#[inline]
fn normalize_key(key: &str) -> Cow<'_, str> {
    if key.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(key.to_ascii_lowercase())
    } else {
        Cow::Borrowed(key)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_stores(data: &[HashMap<String, String>]) -> Box<[FstLocaleStore]> {
    data.iter()
        .map(FstLocaleStore::build)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn build_override_maps(len: usize) -> Box<[ResolvedMap]> {
    (0..len)
        .map(|_| DashMap::with_hasher(BuildHasherDefault::default()))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn build_cache_maps(len: usize) -> Box<[ResolvedMap]> {
    (0..len)
        .map(|_| DashMap::with_hasher(BuildHasherDefault::default()))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
