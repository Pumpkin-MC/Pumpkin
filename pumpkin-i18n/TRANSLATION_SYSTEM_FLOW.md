# 🌐 Pumpkin Translation System Complete Workflow

> **Last Updated**: 2026-06-30
> **Rust Edition**: 2024 | **MSRV**: 1.95

---

## Table of Contents

- [1. System Architecture Overview](#1-system-architecture-overview)
- [2. Build Phase](#2-build-phase)
- [3. Startup Phase](#3-startup-phase)
- [4. Player Join Flow](#4-player-join-flow)
- [5. Runtime Query Hot Path](#5-runtime-query-hot-path)
- [6. Translation Engine Internals](#6-translation-engine-internals)
- [7. Runtime Download System](#7-runtime-download-system)
- [8. Disk Cache Structure](#8-disk-cache-structure)
- [9. Configuration Reference](#9-configuration-reference)
- [10. Data Flow Panorama](#10-data-flow-panorama)
- [11. Core Module Inventory](#11-core-module-inventory)

---

## 1. System Architecture Overview

`pumpkin-i18n` is the core internationalization (i18n) library for the Pumpkin Minecraft server, responsible for:

- Storage and retrieval of translation key-values across **128 languages** (located under
  `assets/translations/pumpkin/`)
- **1122 translation keys** covering all modules including server logs, command system, world generation, and
  authentication
- Server log language resolution (system environment detection / config file override)
- Player language cache (UUID → Locale mapping)
- Zero-regex, streaming-output format placeholder precompilation (`%s`, `%1$s`, `{}`, `{0}`)
- High-performance translation engine based on FST (Finite State Transducer)

### Layered Architecture

```
┌───────────────────────────────────────────────────────────┐
│  Business Code (pumpkin, pumpkin-server, plugins)         │
│    ↓ Unified entry point                                  │
│  pumpkin-util::translation                                │
│    translate_plain / translate_format                     │
│    localized_log / localized_log_format / localized_text  │
│    ↓ Auto-prepends pumpkin: namespace prefix              │
│  pumpkin-i18n (Core Engine)                               │
│    store → engine → token                                 │
│    locale / server / client / download                    │
│    ↓                                                      │
│  FST Index + DashMap Cache + ArcSwap Lock-Free Store      │
└───────────────────────────────────────────────────────────┘
```

### Dependencies

| Dependency    | Purpose                                            |
|---------------|----------------------------------------------------|
| `arc-swap`    | Lock-free atomic replacement of translation data   |
| `dashmap`     | Concurrency-safe high-performance HashMap          |
| `fst`         | FST index for accelerated key lookup               |
| `serde_json`  | Parsing translation JSON files                     |
| `sha2`        | SHA256 integrity verification of downloaded files  |
| `tracing`     | warn/error logging for missing translation keys    |
| `ureq`        | HTTP client (downloading remote translation files) |
| `xxhash-rust` | High-speed hashing (for DashMap)                   |

---

## 2. Build Phase

### 2.1 File Embedding

`pumpkin-i18n/build.rs` executes at compile time, **embedding only `en_us` English translations** into the final
binary (across three namespaces: pumpkin, java_minecraft, bedrock_minecraft). Other languages are not embedded.

```
cargo build
  │
  └─► pumpkin-i18n/build.rs
        │
        └─ Only embed 3 en_us files → generated_store.rs
              │
              ├─ pumpkin/en_us.json
              │   → Prepends "pumpkin:" namespace prefix
              │   → e.g. "pumpkin:server.log.starting_server"
              │
              ├─ vanilla/en_us_java.json
              │   → Prepends "java_minecraft:" namespace prefix
              │
              └─ vanilla/en_us_bedrock.lang
                  → Prepends "bedrock_minecraft:" namespace prefix
                  → Parse key=value line by line, lowercase the key
```

### 2.2 Generated Code Structure

```rust
// generated_store.rs output structure
pub(crate) fn load_all_translations()
    -> [HashMap<String, String>; Locale::COUNT]  // 142 slots
{
    let mut array: [HashMap; 128] = std::array::from_fn(|_| HashMap::new());

    // ✅ EnUs slot: inject pumpkin: + java_minecraft: + bedrock_minecraft: entries
    // ❌ Remaining 127 slots: empty HashMap, populated dynamically at runtime
}
```

**Key Design**: Only `en_us` (English) translation files across the three namespaces (pumpkin, java_minecraft,
bedrock_minecraft) are embedded at build time. The remaining 127 languages are downloaded on demand at runtime (only
pumpkin and java_minecraft are downloaded; bedrock is not downloaded). Other language files in the
`assets/translations/` directory serve only as source data for remote mirrors and are not included in the binary.

---

## 3. Startup Phase

### 3.1 Resolve Server Locale

```
resolve_server_locale(config_value)
  │
  ├─ "auto" → detect_system_locale()
  │   ├─ Linux:   Read LANG / LC_ALL / LC_MESSAGES environment variables
  │   ├─ Windows: GetUserDefaultLocaleName() API
  │   └─ Failure: EnUs fallback
  │
  └─ "zh-CN" → parse_locale_value("zh-CN") → Locale::ZhCn

↓
set_server_global_locale(locale)  // Store in OnceLock<Locale>
```

- `set_server_global_locale()` **only takes effect on first call** (`OnceLock`); subsequent calls are silently ignored
- `server_global_locale()` falls back to `EnUs` when not initialized
- `server_command_locale()` is a semantic alias, equivalent to `server_global_locale()`

### 3.2 Config File Integration

The server reads configuration from `pumpkin_config` at startup:

```rust
// pumpkin/src/main.rs — Startup flow
use pumpkin_i18n::{resolve_server_locale, set_server_global_locale};

let config = PumpkinConfig::load();  // contains advanced.locale.server_global field
let server_global_locale = resolve_server_locale( & config.advanced.locale.server_global);
set_server_global_locale(server_global_locale);
```

Config struct defined in `pumpkin-config/src/locale.rs`:

```rust
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Language for server logs and console output ("auto" or a language code)
    pub server_global: String,
    /// Java Edition client language resolution strategy
    pub client_java_edition: String,
    /// Bedrock Edition client language resolution strategy
    pub client_bedrock_edition: String,
}
```

Defaults to `"auto"` (auto-detect system language). Set to a specific language code like `"zh_cn"` to force an override.

### 3.3 Translation File Download

After startup, cached files from the local `data/translation/{locale}/` directory are loaded first. If missing, they are
downloaded from the remote mirror.

```
spawn_blocking {  // Non-blocking for tokio runtime

  ┌─ Step 1: Try disk cache ───────────────────────────────┐
  │  load_cached_translations(locale, cache_root)          │
  │    → Read data/translation/zh_cn/pumpkin.json          │
  │    → Read data/translation/zh_cn/java_minecraft.json   │
  │    → All exist → Return directly ✅ (skip download)    │
  └────────────────────────────────────────────────────────┘
                        ↓ Cache miss
  ┌─ Step 2: Remote download ──────────────────────────────┐
  │  download_locale(&config, locale)                      │
  │    → GET {mirror}/pumpkin/zh_cn.json                   │
  │    → GET {mirror}/vanilla/zh_cn_java.json              │
  │                                                        │
  │  Each file is downloaded independently;                │
  │  partial failure is tolerable                          │
  │  SHA256 hash verification (.sha256 file):              │
  │    ├─ Checksum file exists + hash matches → Accept ✅  │
  │    ├─ Checksum file exists + hash mismatch → Reject ❌ │
  │    └─ Checksum file missing → Degraded accept ⚠️        │
  └────────────────────────────────────────────────────────┘
                        ↓
  ┌─ Step 3: Save to disk ─────────────────────────────────┐
  │  save_downloaded_translations(downloaded, locale,      │
  │                               cache_root)              │
  │    → Create data/translation/zh_cn/ directory          │
  │    → Write pumpkin.json                                │
  │    → Write java_minecraft.json                         │
  └────────────────────────────────────────────────────────┘
}
```

#### Download Timeout Handling

Each HTTP request has an independent timeout (default 10000ms, configurable via `pumpkin.toml`):

- **Timeout/Failure** → The translation for that file is not loaded; the embedded `en_us` file is used as fallback
- **All successful** → Translations for both namespaces are loaded
- **Partial success** → Successful namespaces load their translations; failed ones fall back to English
- **Bedrock Edition** → Not downloaded; always uses compile-time embedded `en_us` Bedrock strings

### 3.4 Inject into Translation Engine

```
load_downloaded(&downloaded, locale)
  │
  ├─ pumpkin entries → add_translation_file("pumpkin", json, locale)
  └─ java entries    → add_translation_file("java_minecraft", json, locale)
  (bedrock is not downloaded; always uses compile-time embedded en_us)

↓ Inside the translation engine

TranslationEngine {
    stores: ArcSwap<Box<[FstLocaleStore; 128]>>   // FST immutable lookup
    overrides: Box<[DashMap; 128]>                // Runtime dynamic injection
    cache: DashMap                                // Lock-free cache
    fallback_log_once / missing_log_once          // Release build log deduplication
}
```

### 3.5 Initialize Background Loader

```
init_translation_loader(download_config, cache_root)
  → Store in LOADER_STATE (OnceLock)
  → For use by ensure_locale_translations() during subsequent player joins
```

---

## 4. Player Join Flow

### 4.1 Translation Files Already Exist (Hot Path)

```
Player joins (locale = zh_cn)
  │
  ├─ set_player_locale(uuid, "zh_cn", config)
  │   → Cached in PLAYER_CACHE: uuid → Locale::ZhCn
  │
  ├─ ensure_locale_translations(ZhCn)  // Background spawn_blocking
  │   ├─ Check LOADED_LOCALES set → Already loaded → Skip
  │   └─ (Already loaded at server startup)
  │
  └─ Translation query directly hits ZhCn FST → Returns Chinese translation ✅
```

### 4.2 Translation Files Not Found (New Locale, First Join)

```
Player joins (locale = ja_jp) — Japanese translations not downloaded
  │
  ├─ set_player_locale(uuid, "ja_jp", config)
  │   → Cached in PLAYER_CACHE
  │
  ├─ ensure_locale_translations(JaJp)  // Background spawn_blocking
  │   ├─ Check LOADED_LOCALES → Not loaded → Mark as loading
  │   ├─ load_cached_translations(JaJp, cache_root) → No disk cache
  │   ├─ download_locale(&config, JaJp) → Download Japanese translations from mirror
  │   ├─ save_downloaded_translations(...) → Save to disk
  │   └─ load_downloaded(...) → Inject into engine + Clear DashMap cache
  │
  └─ Player joins immediately, without waiting for download to complete
      Translation query goes through three-tier fallback:
        Tier 1: JaJp FST → Miss
        Tier 2: EnUs FST → Hit ✅ → Display English text
        Tier 3: Raw key → Ultimate fallback

      After background download completes:
        → engine.add_translations() clears cache
        → Next query hits JaJp FST → Displays Japanese translation ✅
```

### 4.3 Player Language Resolution Logic

```
config_value == "auto" ?
  ├── Yes → Use the player's reported language
  └── No  → Override with the configured language (e.g., force all players to zh_cn)
```

### 4.4 Player Language Cache

Internally uses `DashMap<String, Locale>` based on XXH64 hashing:

- `set_player_locale(uuid, reported, config)` — Cache at login
- `player_locale(uuid)` — Get (falls back to EnUs on cache miss)
- `try_player_locale(uuid)` — Get (returns None on cache miss)
- `remove_player_locale(uuid)` — Clear when player leaves

---

## 5. Runtime Query Hot Path

### 5.1 Complete Call Chain

```
Business code
  │
  ├─ localized_log("server.log.starting")
  │     → translate_plain("server.log.starting", server_global_locale())
  │
  ├─ localized_log_format("server.log.build_info", &["linux", "x86_64", "debug"])
  │     → translate_format("server.log.build_info", server_global_locale(), &[...])
  │
  └─ localized_text("server.log.starting_server", [child1, child2])
        → TextComponent::custom("pumpkin", key, server_global_locale(), children)
            → .to_pretty_console()
                → translation_to_pretty("pumpkin:server.log.starting_server", locale, children)
  │
  ▼  All paths converge here
pumpkin_util::translation::translate_plain(key, locale)
  │
  ├─ Prepend "pumpkin:" prefix → "pumpkin:server.log.starting"
  │
  └─ pumpkin_i18n::get_translation("pumpkin:server.log.starting", locale)
        │
        └─ pumpkin_i18n::store::resolve_translation(key, locale)
              │
              └─ translation_engine().resolve(locale_idx, key)
```

### 5.2 Engine Internal Query Chain

```
engine.resolve(locale_idx=32, key="pumpkin:server.log.starting")
  │                                      ↑ EnUs enum index
  ├─ 1. DashMap cache lookup (lock-free, ~99% hit rate)
  │     cache_key = "32:pumpkin:server.log.starting"
  │     → Hit → Return Arc<ResolvedTranslation> directly ✅
  │
  ├─ 2. Override layer lookup (runtime-dynamic translations take priority)
  │     overrides[32].get(key)
  │     → Hit → Write to cache → Return
  │
  ├─ 3. FST lookup (Finite State Transducer, O(key_length))
  │     stores[32].fst.get(key)
  │     → Hit → Write to cache → Return
  │
  └─ 4. Three-tier fallback
        ├─ Tier 1: Target locale Override + FST → Miss
        ├─ Tier 2: EnUs Override + FST → Hit → warn! log + Return English
        └─ Tier 3: Raw key → error! log + Return key literal
```

### 5.3 Format Output

```rust
// Three forms of ResolvedTranslation
enum ResolvedTranslation {
    Static(Arc<str>),                              // Plain text, no placeholders
    Tokenized { template: Arc<str>, tokens: TokenStream },  // Precompiled Token stream
    Missing(Arc<str>),                              // Literal fallback for missing translation key
}

// Tokenized formatting
ResolvedTranslation::Tokenized {
    tokens: [Text("Hello "), Var(0), Text("!")]
}
format_tokens(tokens, &["World"]) → "Hello World!"
```

---

## 6. Translation Engine Internals

**File**: `pumpkin-i18n/src/engine.rs`

### 6.1 Design Purpose

Provides extreme performance for high-frequency read scenarios (every chat message, every UI text needs translation):

- **FST Index**: O(key_length) key lookup with smaller memory footprint compared to HashMap
- **ArcSwap Storage**: Writers replace entire language data sets, readers read lock-free
- **DashMap Cache**: Lock-free on hit, XxHash64 reduces hash collisions
- **Precompiled Tokens**: `%s`, `{}`, `{name}` placeholders in translation strings are compiled into `TokenStream` at
  load time
- **Override Layer**: Runtime-dynamically-injected translations take priority over compile-time-embedded FST data
- **Log Rate Limiting**: Fallback/missing translation logs are deduplicated in Release builds (to avoid log spam)

### 6.2 Core Types

```rust
pub struct TranslationEngine {
    stores: ArcSwap<Box<[FstLocaleStore]>>,    // One FST store per language
    overrides: Box<[DashMap<String, Arc<ResolvedTranslation>>]>, // Runtime overrides
    cache: DashMap<String, Arc<ResolvedTranslation>, BuildHasherDefault<Xxh64>>,
    // Release build: fallback/missing log deduplication (one log per key)
}

pub enum ResolvedTranslation {
    Static(Arc<str>),                           // Plain text with no placeholders
    Tokenized { template: Arc<str>, tokens: TokenStream }, // Precompiled token stream
    Missing(Arc<str>),                           // Literal fallback for missing translation key
}
```

### 6.3 Public API

```rust
impl TranslationEngine {
    /// Build the engine from per-language translation maps
    pub fn build(data: &[HashMap<String, String>]) -> Self;

    /// Resolve a translation key (three-tier fallback + cache), never returns None
    pub fn resolve(&self, locale_idx: usize, key: &str) -> Arc<ResolvedTranslation>;

    /// Atomically reload translation data
    pub fn reload(&self, data: &[HashMap<String, String>]);

    /// Add/override a single runtime translation
    pub fn add_translation(&self, locale_idx: usize, key: &str, translation: &str);

    /// Bulk-add/override runtime translations
    pub fn add_translations<I>(&self, locale_idx: usize, entries: I)
    where
        I: IntoIterator<Item=(String, String)>;
}

impl ResolvedTranslation {
    /// Build from a raw translation template (auto-detects whether precompilation is needed)
    pub fn from_template(value: &str) -> Self;

    /// Return the translation template string
    pub fn as_str(&self) -> &str;

    /// Return the precompiled token stream (if any)
    pub fn tokens(&self) -> Option<&[Token]>;

    /// Whether this is a fallback for a missing translation key
    pub fn is_missing(&self) -> bool;

    /// Return the translation string, or raw_key (when this is a missing fallback)
    pub fn value_or_raw(&self, raw_key: &str) -> String;

    /// Write the formatted result into a buffer
    pub fn write_to(&self, args: &[String], buf: &mut String);
}

/// Streaming format of precompiled Tokens into a buffer
pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String);
```

### 6.4 Cache Key Format

`"<locale_idx>:<key>"` — e.g. `"32: pumpkin:welcome"` (32 = EnUs's enum index)

---

## 7. Runtime Download System

**File**: `pumpkin-i18n/src/download.rs`

### 7.1 Design Purpose

Only `en_us` (English) translations are embedded at compile time; the remaining 127 languages are downloaded on demand
at runtime. The server downloads the language corresponding to `server_global_locale` at startup, and loads each
player's client language in the background when they join.

### 7.2 Core Types

```rust
/// Configuration for the translation downloader
pub struct DownloadConfig {
    pub mirror_url: String,      // Mirror URL (empty = default GitHub mirror)
    pub timeout_ms: u64,         // Timeout for a single HTTP request (default 10000ms)
    pub skip_checksum: bool,     // Skip SHA256 verification
}

/// Translation data for one language after download (organized by namespace)
pub struct DownloadedTranslations {
    pub pumpkin: HashMap<String, String>,          // pumpkin: namespace
    pub java: HashMap<String, String>,             // java_minecraft: namespace
    pub bedrock: HashMap<String, String>,          // bedrock_minecraft: namespace (en_us only)
}
```

### 7.3 Public API

```rust
// Initialize the background loader at startup
pub fn init_translation_loader(config: DownloadConfig, cache_root: PathBuf);

// Download translation files for a specified language
pub fn download_locale(config: &DownloadConfig, locale: Locale) -> DownloadedTranslations;

// Load translations from disk cache
pub fn load_cached_translations(locale: Locale, cache_root: &Path) -> Option<DownloadedTranslations>;

// Inject downloaded translations into the engine
pub fn load_downloaded(downloaded: &DownloadedTranslations, locale: Locale);

// Save downloaded translations to disk
pub fn save_downloaded_translations(downloaded: &DownloadedTranslations, locale: Locale, cache_root: &Path);

// Ensure translations for a language are loaded in the background (download + cache + inject, supports deduplication)
pub fn ensure_locale_translations(locale: Locale);

// Manually mark a language as loaded (to avoid redundant loading)
pub fn mark_locale_loaded(locale: Locale);
```

### 7.4 Download Targets

Two types of files are downloaded from the remote mirror at server startup:

1. `{mirror}/pumpkin/{code}.json` — pumpkin server translations
2. `{mirror}/vanilla/{code}_java.json` — Minecraft Java Edition native translations

> **Note**: Bedrock Edition translations are **not downloaded at runtime**; only the compile-time-embedded `en_us`
> Bedrock strings are used.

### 7.5 SHA256 Verification Flow

```
Download data → Fetch {url}.sha256 → Verify hash
  ├─ Checksum file exists + hash matches → Accept ✅
  ├─ Checksum file exists + hash mismatch → Reject ❌
  └─ Checksum file missing → Warn and accept degraded ⚠️
```

### 7.6 Background Loader Deduplication Logic

`ensure_locale_translations(locale)` supports deduplication and concurrency safety:

1. **EnUs** → no-op (already embedded at compile time)
2. **Already loaded** → no-op (internal `LOADED_LOCALES` dedup set)
3. **Full disk cache** → load and return
4. **Partial disk cache** → load what exists → download full set → overwrite-inject
5. **No cache** → download → save → inject

---

## 8. Disk Cache Structure

```
{exec_dir}/                         ← Program working directory
└── data/                           ← Configurable (translation_cache_dir)
    └── translation/
        ├── en_us/                  ← en_us is only a cached copy (already embedded at compile time; no download needed)
        │   ├── pumpkin.json
        │   └── java_minecraft.json
        ├── zh_cn/                  ← Loaded at server startup
        │   ├── pumpkin.json
        │   └── java_minecraft.json
        └── ja_jp/                  ← Background download when first Japanese player joins
            ├── pumpkin.json
            └── java_minecraft.json
```

> **Note**: Bedrock Edition translations are not downloaded or cached to disk; the compile-time embedded `en_us` Bedrock
> strings are always used.

---

## 9. Configuration Reference

`pumpkin.toml` `[advanced.locale]` section:

```toml
[advanced.locale]
# Server log/console language ("auto" = follow system)
server_global = "auto"

# Java Edition player locale resolution strategy
client_java_edition = "auto"

# Bedrock Edition player locale resolution strategy
client_bedrock_edition = "auto"

# Translation mirror URL (empty = use default GitHub mirror)
mirror_url = ""

# Single HTTP request timeout (ms)
timeout = 10000

# Skip SHA256 verification
skip_checksum = false

# Translation cache directory (relative paths are based on working directory; absolute paths used directly)
translation_cache_dir = "data/translation"
```

---

## 10. Data Flow Panorama

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          Build Phase                                     │
│                                                                          │
│  build.rs                                                                │
│    │ assets/translations/pumpkin/en_us.json ───── include_str! ───────┐  │
│    │ assets/translations/vanilla/en_us_java.json ── include_str! ──┐  │  │
│    │ assets/translations/vanilla/en_us_bedrock.lang ─ include_str! │  │  │
│    ▼                                                               ▼  ▼  │
│  generated_store.rs: load_all_translations() → [HashMap; 128]            │
│  (Only EnUs slot has data)                                               │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │ LazyLock deferred initialization
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          Startup                                         │
│                                                                          │
│  main.rs                                                                 │
│    ├─ resolve_server_locale() ─→ set_server_global_locale()              │
│    ├─ [disk] load_cached_translations(server_locale)                     │
│    │    └─ Hit → Skip download                                           │
│    ├─ [HTTP] download_locale(server_locale)                              │
│    │    └─ SHA256 verification → Accept/Reject                           │
│    ├─ [disk] save_downloaded_translations(server_locale)                 │
│    ├─ [engine] load_downloaded() → TranslationEngine                     │
│    └─ init_translation_loader(config, cache_root)                        │
│         └─ Store in LOADER_STATE for background use                      │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                     Player Join                                          │
│                                                                          │
│  lib.rs / play.rs                                                        │
│    ├─ set_player_locale(uuid, reported_locale, config)                   │
│    │    └─ PLAYER_CACHE: {uuid → Locale}                                 │
│    │                                                                     │
│    └─ spawn_blocking { ensure_locale_translations(player_locale) }       │
│         ├─ LOADED_LOCALES dedup check                                    │
│         ├─ [disk] load_cached_translations()                             │
│         ├─ [HTTP] download_locale()                                      │
│         ├─ [disk] save_downloaded_translations()                         │
│         └─ [engine] load_downloaded() → Clear cache                      │
│                                                                          │
│  Player does not wait for download → Joins immediately                   │
│  → Uses English fallback                                                 │
│  After download completes → Next query automatically uses target         │
│  language                                                                │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        Runtime Query (Hot Path)                          │
│                                                                          │
│  translate_plain("server.log.starting", locale)                          │
│  localized_log("server.log.starting")                                    │
│    → "pumpkin:server.log.starting"                                       │
│    → get_translation → resolve_translation → engine.resolve(locale, key) │
│                                                                          │
│    Query chain:                                                          │
│      DashMap cache (lock-free, ~99% hit rate)                            │
│        → FST (Finite State Transducer) O(log n)                          │
│          → Three-tier fallback: Target locale → EnUs → Raw key           │
│            → Pre-compiled Token stream (Zero parsing overhead)           │
│                                                                          │
│    Formatting:                                                           │
│      ResolvedTranslation::Tokenized {                                    │
│        tokens: [Text("Hello "), Var(0), Text("!")]                       │
│      }                                                                   │
│      format_tokens(tokens, &["World"]) → "Hello World!"                  │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 11. Core Module Inventory

| File                              | Responsibility                                                                                                |
|-----------------------------------|---------------------------------------------------------------------------------------------------------------|
| `pumpkin-i18n/build.rs`           | Compile-time embedding of en_us (pumpkin + java_minecraft + bedrock_minecraft)                                |
| `pumpkin-i18n/src/locale.rs`      | 142-variant Locale enum, from_str/to_code (hardcoded LOCALE_CODES array), normalize                           |
| `pumpkin-i18n/src/server.rs`      | Server locale global state, server_command_locale, system language detection                                  |
| `pumpkin-i18n/src/client.rs`      | Player UUID→Locale cache, client locale resolution, try_player_locale                                         |
| `pumpkin-i18n/src/store.rs`       | Global TRANSLATIONS, translation_engine, resolve_translation, dynamic injection API                           |
| `pumpkin-i18n/src/engine.rs`      | FST build/lookup, DashMap cache, override layer, pre-compiled Token, value_or_raw fallback, log rate limiting |
| `pumpkin-i18n/src/token.rs`       | `%s` / `{name}` / `{}` placeholder parsing, pre-compiled TokenStream                                          |
| `pumpkin-i18n/src/download.rs`    | HTTP download, SHA256 verification, disk cache, background loader, mark_locale_loaded dedup                   |
| `pumpkin-i18n/src/lib.rs`         | Module declarations, public API exports, PUMPKIN_NAMESPACE, pumpkin_translation_key                           |
| `pumpkin-config/src/locale.rs`    | User-configurable TOML locale settings                                                                        |
| `pumpkin-util/src/translation.rs` | translate_plain / translate_format / localized_log / localized_text unified entry point                       |
| `pumpkin/src/main.rs`             | Startup flow orchestration: Download → Load → Initialize background loader                                    |
| `pumpkin/src/lib.rs`              | Trigger background translation download on player login                                                       |
| `pumpkin/src/net/java/play.rs`    | Sync locale on settings change + Trigger download                                                             |
