# 📖 Pumpkin Translation & Text API Reference

> **Last Updated**: 2026-06-30
> **Rust Edition**: 2024 | **MSRV**: 1.95
> **Version**: 0.1.0-dev+26.3

---

## Table of Contents

- [1. Import Convention](#1-import-convention)
- [2. Unified Translation Entry Layer (pumpkin-util::translation)](#2-unified-translation-entry-layer-pumpkin-utiltranslation)
    - [2.1 translate_plain / translate_format — Explicit Locale Translation](#21-translate_plain--translate_format--explicit-locale-translation)
    - [2.2 localized_log / localized_log_format / localized_text](#22-localized_log--localized_log_format--localized_text)
    - [2.3 When to Use Which Function](#23-when-to-use-which-function)
    - [2.4 Complete Call Chain](#24-complete-call-chain)
- [3. pumpkin-i18n Module APIs](#3-pumpkin-i18n-module-apis)
    - [3.1 Locale — Language Environment](#31-locale--language-environment)
    - [3.2 Server — Server Language](#32-server--server-language)
    - [3.3 Client — Client Language](#33-client--client-language)
    - [3.4 Store — Translation Store](#34-store--translation-store)
    - [3.5 Engine — Translation Engine](#35-engine--translation-engine)
    - [3.6 Token — Format Placeholder Precompilation](#36-token--format-placeholder-precompilation)
    - [3.7 Download — Runtime Download](#37-download--runtime-download)
    - [3.8 Internal Utility Functions](#38-internal-utility-functions)
- [4. Text Component System (pumpkin-util::text)](#4-text-component-system-pumpkin-utiltext)
    - [4.1 TextComponent — Text Component](#41-textcomponent--text-component)
    - [4.2 TextComponentBase — Component Base](#42-textcomponentbase--component-base)
    - [4.3 TextContent — Content Type](#43-textcontent--content-type)
    - [4.4 Style — Style](#44-style--style)
    - [4.5 Color System](#45-color-system)
    - [4.6 ClickEvent — Click Event](#46-clickevent--click-event)
    - [4.7 HoverEvent — Hover Event](#47-hoverevent--hover-event)
    - [4.8 Translation Helper Functions](#48-translation-helper-functions)
- [5. Complete Usage Examples](#5-complete-usage-examples)

---

## 1. Import Convention

All crates import translation functions uniformly from `pumpkin-util`:

```rust
// ✅ Correct — All crates use this path uniformly
use pumpkin_util::translation::{
    localized_log, localized_log_format, localized_text,
    translate_plain, translate_format,
};

// ❌ Wrong — These functions are no longer defined in the pumpkin crate
use crate::localized_log;
use pumpkin::localized_log;
```

> **Note**: Business code should not call `pumpkin_i18n` functions directly. Use the functions in
`pumpkin-util::translation` as the unified entry point.
> These functions automatically handle namespace prefixing and `server_global_locale()` resolution.

---

## 2. Unified Translation Entry Layer (pumpkin-util::translation)

**File**: `pumpkin-util/src/translation.rs`

`pumpkin-util::translation` is the **only place where translation functions are defined** in the entire Pumpkin project.
All crates import uniformly from this module.

`translate_plain` / `translate_format` provide explicit locale parameter support, while `localized_log` /
`localized_log_format` / `localized_text` are convenience wrappers that automatically use `server_global_locale()`.

### 2.1 translate_plain / translate_format — Explicit Locale Translation

```rust
use pumpkin_util::translation::{translate_plain, translate_format};

/// Plain text translation with specified locale
pub fn translate_plain(key: &str, locale: Locale) -> String;

/// Formatted translation with specified locale (supports placeholder substitution)
pub fn translate_format(key: &str, locale: Locale, args: &[String]) -> String;
```

`translate_plain`:

- Automatically prepends the `pumpkin:` namespace prefix to the key
- Calls `pumpkin_i18n::get_translation(key, locale)` to look up the translation
- Returns the original key string when the translation is missing
- Used for scenarios that require translation into a specific language (e.g., command replies)

`translate_format`:

- Similar to `translate_plain`, but additionally supports placeholder substitution
- Calls `pumpkin_i18n::format_translation(key, locale, args)`

### 2.2 localized_log / localized_log_format / localized_text

```rust
use pumpkin_util::translation::{localized_log, localized_log_format, localized_text};

/// Plain text log (auto-uses server_global_locale())
pub fn localized_log(key: &str) -> String;

/// Formatted log (auto-uses server_global_locale())
pub fn localized_log_format(key: &str, args: &[String]) -> String;

/// Translation with colored child components
pub fn localized_text<W>(key: &'static str, with: W) -> TextComponent
where
    W: Into<Vec<TextComponent>>;
```

`localized_log`:

- Internally calls `translate_plain(key, server_global_locale())`
- Used for plain text scenarios such as logs, panic messages, and error messages

`localized_log_format`:

- Internally calls `translate_format(key, server_global_locale(), args)`
- **Plain strings** (not `TextComponent`) in `args` will replace `%s` placeholders in the translation template by index
- Example: `localized_log_format("server.log.build_info", &[os, arch, debug_flag])`

`localized_text`:

- Creates `TextComponent::custom("pumpkin", key, server_global_locale(), with)`
- Child components `with` are inserted at placeholder positions in the translation template, **preserving colors and
  styles**
- Returns `TextComponent`, allowing further chained calls like `.to_pretty_console()`
- ❗**Do not** pass the result of `.to_pretty_console()` to `localized_log_format` — that would cause nested ANSI code
  corruption

### 2.3 When to Use Which Function

| Scenario                       | Recommended Function                   | Reason                                                           |
|--------------------------------|----------------------------------------|------------------------------------------------------------------|
| Console plain text log         | `localized_log`                        | Automatically uses server_global_locale()                        |
| Formatted log with parameters  | `localized_log_format`                 | Automatically uses server_global_locale()                        |
| Specified language translation | `translate_plain` / `translate_format` | Explicitly passes Locale parameter, suitable for command replies |
| Colored startup banner         | `localized_text`                       | Child components retain coloring                                 |
| Player chat message            | `TextComponent::translate`             | Client-side translation, not server-side                         |
| Server custom translation msg  | `localized_text`                       | `TextContent::Custom` variant                                    |

### 2.4 Complete Call Chain

```
Code call                       Translation entry layer                    i18n engine
─────────────────────────────────────────────────────────────────────────────────────────────────────
translate_plain("key", loc)   → get_translation("pumpkin:key", loc)      → resolve(key, locale)
translate_format("k",l,a)     → format_translation("pumpkin:k", l, a)    → resolve → tokens → write
localized_log("key")          → translate_plain("key", srv_locale())     → (same as above)
localized_log_format("k",a)   → translate_format("key", srv_locale(),a)  → (same as above)
localized_text("k", [c])      → TextComponent::custom → .to_pretty()     → resolve → tokens → render
```

---

## 3. pumpkin-i18n Module APIs

### 3.1 Locale — Language Environment

**File**: `pumpkin-i18n/src/locale.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locale {
    // 128 variants, in alphabetical order:
    AfZa,
    ArSa,
    AstEs,
    AzAz,
    Bar,
    BaRu,
    BeBy,
    BgBg,
    Brb,
    BrFr,
    BsBa,
    CaEs,
    CsCz,
    CyGb,
    DaDk,
    DeAt,
    DeCh,
    DeDe,
    ElGr,
    EnAu,
    EnCa,
    EnGb,
    EnNz,
    Enp,
    EnPt,
    EnUd,
    EnUs,  // ← EnUs is the default fallback
    Enws,
    EoUy,
    Esan,
    EsAr,
    EsCl,
    EsEc,
    EsEs,
    EsMx,
    EsUy,
    EsVe,
    EtEe,
    EuEs,
    FaIr,
    FiFi,
    FilPh,
    FoFo,
    FrCa,
    FrFr,
    FraDe,
    FurIt,
    FyNl,
    GaIe,
    GdGb,
    GlEs,
    HawUs,
    HeIl,
    HiIn,
    HrHr,
    HuHu,
    HyAm,
    IdId,
    IgNg,
    IoEn,
    IsIs,
    Isv,
    ItIt,
    JaJp,
    JboEn,
    KaGe,
    KkKz,
    KnIn,
    KoKr,
    Ksh,
    KwGb,
    LaLa,
    LbLu,
    LiLi,
    Lmo,
    LoLa,
    LolUs,
    LtLt,
    LvLv,
    Lzh,
    MkMk,
    MnMn,
    MsMy,
    MtMt,
    Nah,
    NdsDe,
    NlBe,
    NlNl,
    NnNo,
    NoNo,
    OcFr,
    Ovd,
    PlPl,
    PtBr,
    PtPt,
    QyaAa,
    RoRo,
    Rpr,
    RuRu,
    RyUa,
    SahSah,
    SeNo,
    SkSk,
    SlSi,
    SoSo,
    SqAl,
    SrCs,
    SrSp,
    SvSe,
    Sxu,
    Szl,
    TaIn,
    ThTh,
    TlhAa,
    TlPh,
    Tok,
    TrTr,
    TtRu,
    UkUa,
    ValEs,
    VecIt,
    ViVn,
    YiDe,
    YoNg,
    ZhCn,
    ZhHk,
    ZhTw,
    ZlmArab,
}
```

| Method  | Signature                | Description                |
|---------|--------------------------|----------------------------|
| `COUNT` | `pub const COUNT: usize` | Total language count (128) |

**FromStr Implementation**:

```rust
impl FromStr for Locale {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
```

- Parses snake_case names like `"en_us"`, `"zh_cn"`, `"de_de"`
- **Case-insensitive**
- Falls back to `Locale::EnUs` on no match (does not return Err)

---

### 3.2 Server — Server Language

**File**: `pumpkin-i18n/src/server.rs`

```rust
// Get the current server log language
pub fn server_global_locale() -> Locale;

// Get the language for command output (semantic alias, same as server_global_locale)
pub fn server_command_locale() -> Locale;

// Set the server log language (called by the pumpkin server crate at startup)
pub fn set_server_global_locale(locale: Locale);

// Auto-detect the system language
pub fn detect_system_locale() -> Locale;

// Parse a config value and resolve the language
pub fn resolve_server_locale(config_value: &str) -> Locale;
```

| Function                     | Behavior                                                                                                                                                                         |
|------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `server_global_locale()`     | Returns the language stored in `OnceLock`; falls back to `EnUs` when not initialized                                                                                             |
| `server_command_locale()`    | Semantic alias for `server_global_locale()`; using this name in the command module is clearer                                                                                    |
| `set_server_global_locale()` | **Only takes effect on first call** (`OnceLock`); subsequent calls are silently ignored                                                                                          |
| `detect_system_locale()`     | **Linux/macOS**: reads `LANG` → `LC_ALL` → `LC_MESSAGES` environment variables<br>**Windows**: calls `GetUserDefaultLocaleName` API<br>**Other platforms**: falls back to `EnUs` |
| `resolve_server_locale(cfg)` | If `cfg == "auto"`, calls `detect_system_locale()`; otherwise parses the config value                                                                                            |

---

### 3.3 Client — Client Language

**File**: `pumpkin-i18n/src/client.rs`

```rust
// Cache the player's language at login
pub fn set_player_locale(uuid: &str, player_reported_locale: &str, config_value: &str) -> Locale;

// Get the cached player language (falls back to EnUs on cache miss)
pub fn player_locale(uuid: &str) -> Locale;

// Get the cached player language (returns None on cache miss)
pub fn try_player_locale(uuid: &str) -> Option<Locale>;

// Clear the cache when a player leaves
pub fn remove_player_locale(uuid: &str);

// Pure function: compute the final language from config and client-reported value
pub fn resolve_client_locale(player_locale: &str, config_value: &str) -> Locale;

// Convert a Locale to a log string (e.g. "en_us")
pub fn locale_to_log_string(locale: Locale) -> String;
```

**Internal Structure**:

```rust
// Global player language cache based on DashMap + XXH64
static PLAYER_CACHE: LazyLock<DashMap<String, Locale, BuildHasherDefault<Xxh64>>>;
```

**Language Resolution Logic**:

```
config_value == "auto" ?
  ├── Yes → Use the player's reported language
  └── No  → Override with the configured language (e.g., force all players to zh_cn)
```

---

### 3.4 Store — Translation Store

**File**: `pumpkin-i18n/src/store.rs`

```rust
// Global store
pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::COUNT]>>;

// Get the global translation engine (FST + DashMap cache, high-performance reads)
pub fn translation_engine() -> &'static TranslationEngine;

// Resolve a translation key using the engine (returns Arc<ResolvedTranslation>)
pub fn resolve_translation(key: &str, locale: Locale) -> Arc<ResolvedTranslation>;

// Get a translation string (three-tier fallback)
pub fn get_translation(key: &str, locale: Locale) -> String;

// Get and format a translation (supports placeholder substitution)
pub fn format_translation(key: &str, locale: Locale, args: &[String]) -> String;

// Add a single translation
pub fn add_translation<N, K, V>(namespace: N, key: K, translation: V, locale: Locale)
where
    N: AsRef<str>,
    K: AsRef<str>,
    V: Into<String>;

// Bulk-load translations from JSON
pub fn add_translation_file<N, J>(namespace: N, json: J, locale: Locale)
where
    N: AsRef<str>,
    J: AsRef<str>;
```

**get_translation — Three-Tier Fallback Strategy**:

```
Tier 1: Requested locale → Hit returns directly (no log)
Tier 2: EnUs fallback    → warn! log + returns English string
Tier 3: Raw key          → error! log + returns the key itself
```

> **Note**: Keys are compared after `to_ascii_lowercase()` processing

**Dynamic Injection Examples**:

```rust
// Add a single entry
add_translation("pumpkin", "welcome", "Willkommen", Locale::DeDe);

// Bulk-load from a JSON string
add_translation_file(
"pumpkin",
r#"{"welcome": "Willkommen", "goodbye": "Auf Wiedersehen"}"#,
Locale::DeDe,
);
```

- `add_translation()`: Inserts a single key-value pair; the key is auto-joined as `"namespace:key"`, written to both the
  engine override and the TRANSLATIONS map
- `add_translation_file()`: Parses a `HashMap` from a JSON string and bulk-inserts; emits a `warn!` log when the JSON is
  empty or parsing fails; writes to both the engine override (clearing the DashMap cache) and the TRANSLATIONS map
- `format_translation()`: First looks up via `resolve_translation`, then writes the token stream + args into a buffer

---

### 3.5 Engine — Translation Engine

**File**: `pumpkin-i18n/src/engine.rs`

> For most scenarios, use the surface-level API from `pumpkin-util::translation`. Direct engine calls are only for
> extreme throughput scenarios.

```rust
pub struct TranslationEngine {
    stores: ArcSwap<Box<[FstLocaleStore]>>,    // One FST store per language
    overrides: Box<[DashMap<String, Arc<ResolvedTranslation>>]>, // Runtime overrides
    cache: DashMap<String, Arc<ResolvedTranslation>, BuildHasherDefault<Xxh64>>,
}

pub enum ResolvedTranslation {
    Static(Arc<str>),                           // Plain text with no placeholders
    Tokenized { template: Arc<str>, tokens: TokenStream }, // Precompiled token stream
    Missing(Arc<str>),                           // Literal fallback for missing translation key
}
```

```rust
impl TranslationEngine {
    pub fn build(data: &[HashMap<String, String>]) -> Self;
    pub fn resolve(&self, locale_idx: usize, key: &str) -> Arc<ResolvedTranslation>;
    pub fn reload(&self, data: &[HashMap<String, String>]);
    pub fn add_translation(&self, locale_idx: usize, key: &str, translation: &str);
    pub fn add_translations<I>(&self, locale_idx: usize, entries: I)
    where
        I: IntoIterator<Item=(String, String)>;
}

impl ResolvedTranslation {
    pub fn from_template(value: &str) -> Self;
    pub fn as_str(&self) -> &str;
    pub fn tokens(&self) -> Option<&[Token]>;
    pub fn is_missing(&self) -> bool;
    pub fn value_or_raw(&self, raw_key: &str) -> String;
    pub fn write_to(&self, args: &[String], buf: &mut String);
}

pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String);
```

---

### 3.6 Token — Format Placeholder Precompilation

**File**: `pumpkin-i18n/src/token.rs`

```rust
pub enum Token {
    Text(Arc<str>),  // Plain text fragment, output directly
    Var(usize),      // Variable placeholder, index into the args array
}

pub type TokenStream = Arc<[Token]>;
```

```rust
/// Parses placeholders in a translation string and returns a precompiled Token sequence
pub fn precompile(template: &str) -> Option<TokenStream>;

/// Zero-allocation streaming write
pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String);
```

**Supported Placeholder Formats**:

| Format               | Description                                      | Example                                         |
|----------------------|--------------------------------------------------|-------------------------------------------------|
| `%%`                 | Escaped literal `%`                              | `"100%%"` → `Token::Text("100%")`               |
| `%s`, `%d`, `%f`     | Sequential index (0, 1, 2, ...)                  | `"%s joined"` → `[Var(0), Text(" joined")]`     |
| `%1$s`, `%2$d`       | Explicit 1-based index                           | `"%2$s → %1$s"` → argument reversal             |
| `{}`, `{:?}`         | Rust-style sequential index                      | `"{} + {}"` → `[Var(0), Text(" + "), Var(1)]`   |
| `{0}`, `{0:?}`       | Rust-style explicit 0-based index                | `"{1} → {0}"` → argument reversal               |
| `{name}`, `{name:?}` | Named placeholder (numbered by first appearance) | `"{name} joined"` → `[Var(0), Text(" joined")]` |
| `{{`, `}}`           | Escaped literal `{` / `}`                        | `"{{escaped}}"` → `Token::Text("{escaped}")`    |

**Return value**: `None` means the string contains no placeholders (the caller can use the original string directly).

`format_tokens` outputs an empty string when `Var(idx)` is out of bounds (no panic).

---

### 3.7 Download — Runtime Download

**File**: `pumpkin-i18n/src/download.rs`

```rust
pub struct DownloadConfig {
    pub mirror_url: String,      // Mirror URL (empty = default GitHub mirror)
    pub timeout_ms: u64,         // Timeout for a single HTTP request (default 10000ms)
    pub skip_checksum: bool,     // Skip SHA256 verification
}

pub struct DownloadedTranslations {
    pub pumpkin: HashMap<String, String>,          // pumpkin: namespace
    pub java: HashMap<String, String>,             // java_minecraft: namespace
    pub bedrock: HashMap<String, String>,          // bedrock_minecraft: namespace (en_us only)
}
```

```rust
pub fn init_translation_loader(config: DownloadConfig, cache_root: PathBuf);
pub fn download_locale(config: &DownloadConfig, locale: Locale) -> DownloadedTranslations;
pub fn load_cached_translations(locale: Locale, cache_root: &Path) -> Option<DownloadedTranslations>;
pub fn load_downloaded(downloaded: &DownloadedTranslations, locale: Locale);
pub fn save_downloaded_translations(downloaded: &DownloadedTranslations, locale: Locale, cache_root: &Path);
pub fn ensure_locale_translations(locale: Locale);
pub fn mark_locale_loaded(locale: Locale);
```

---

### 3.8 Internal Utility Functions

**File**: `pumpkin-i18n/src/lib.rs`

```rust
// Parse a config value into a Locale (pub(crate))
pub(crate) fn parse_locale_value(raw: &str) -> Locale;

// Placeholder substitution range marker
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    pub start: usize,  // Starting byte index (inclusive)
    pub end: usize,    // Ending byte index (inclusive)
}
impl SubstitutionRange {
    pub const fn len(&self) -> usize;      // (end - start) + 1
    pub const fn is_empty(&self) -> bool;  // start == end
}
```

`parse_locale_value` normalizes `-` to `_`, falling back to `EnUs` on parse failure. `SubstitutionRange` marks the byte
ranges of placeholders in translation strings; consumed by `pumpkin-util/src/text/translation.rs`.

---

## 4. Text Component System (pumpkin-util::text)

`pumpkin-util/src/text` implements Minecraft's chat component system, including JSON ↔ NBT serialization, colored
console output, Bedrock Edition string generation, text gradient and rainbow effects, rich text styles, and event
system.

**Module Structure**:

```
pumpkin-util/src/
├── translation.rs  # translate_plain, translate_format, localized_log, localized_log_format, localized_text
└── text/
    ├── mod.rs          # TextComponent, TextComponentBase, TextContent, tests
    ├── color.rs        # Color, NamedColor, RGBColor, ARGBColor, hsv_to_rgb
    ├── style.rs        # Style (color, bold, italic, underline, strikethrough, obfuscation, insertion, click, hover, font, shadow)
    ├── click.rs        # ClickEvent enum
    ├── hover.rs        # HoverEvent enum
    └── translation.rs  # reorder_substitutions, translation_to_pretty, get_translation_text
```

### 4.1 TextComponent — Text Component

**File**: `pumpkin-util/src/text/mod.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextComponent(pub TextComponentBase);
```

**Serialization Behavior**:

- **Deserialize** accepts three JSON formats:
    - **String**: `"Hello"` → plain text component
    - **Array**: `[comp1, comp2]` → empty content + `extra` containing all elements
    - **Object**: `{"text": "...", "color": "red"}` → standard component
- **Serialize**: Resolves all translations via `to_translated()` first, then serializes as a JSON object

**Constructor Methods**:

```rust
impl TextComponent {
    pub fn empty() -> Self;                          // Empty component (for collecting children)
    pub fn text<P: Into<Cow<'static, str>>>(p) -> Self;  // Plain text
    pub fn translate<K, W>(key: K, with: W) -> Self; // Client-side translation (key like "multiplayer.player.joined")
    pub fn translate_cross<K1, K2, W>(java_key, bedrock_key, with) -> Self; // Cross-platform translation
    pub fn custom<K, W>(namespace, key, locale, with) -> Self; // Custom translation (server-side resolution)
    pub fn from_legacy_string(input: &str) -> Self;  // Parse §-format legacy string
    pub fn from_content(content: TextContent) -> Self; // Create from TextContent
    pub fn chat_decorated(format, player_name, content) -> Self; // Chat message formatting
}
```

**Chainable Modifiers** (each method returns `Self`, enabling chained calls):

```rust
impl TextComponent {
    // --- Color ---
    pub fn color(self, color: Color) -> Self;
    pub fn color_named(self, color: NamedColor) -> Self;
    pub fn color_rgb(self, color: RGBColor) -> Self;
    pub fn gradient(self, colors: &[RGBColor]) -> Self;       // RGB gradient
    pub fn gradient_named(self, colors: &[NamedColor]) -> Self; // Named color gradient
    pub fn rainbow(self) -> Self;                               // Rainbow effect

    // --- Style ---
    pub fn bold(self) -> Self;
    pub fn italic(self) -> Self;
    pub fn underlined(self) -> Self;
    pub fn strikethrough(self) -> Self;
    pub fn obfuscated(self) -> Self;

    // --- Advanced ---
    pub fn font(self, resource_location: String) -> Self;       // Set font
    pub fn shadow_color(self, color: ARGBColor) -> Self;         // Shadow color
    pub fn insertion(self, text: String) -> Self;                // Shift-click insertion text
    pub fn click_event(self, event: ClickEvent) -> Self;
    pub fn hover_event(self, event: HoverEvent) -> Self;

    // --- Concatenation ---
    pub fn add_child(self, child: Self) -> Self;                 // Append a child component
    pub fn add_text<P: Into<Cow<'static, str>>>(self, text) -> Self; // Append plain text

    // --- Newlines & Brackets ---
    pub fn new_line(self) -> Self;                                // Append a newline
    pub fn wrap_in_square_brackets(self) -> Self;                 // Wrap with [ ]

    // --- Output ---
    pub fn to_pretty_console(self) -> String;                     // Console colored string
    pub fn get_text(self) -> String;                              // Plain text (EnUs)
    pub fn encode(&self) -> Box<[u8]>;                            // NBT serialization
}
```

**Static Methods**:

```rust
impl TextComponent {
    pub fn join(elements: Vec<Self>, separator: &Self) -> Self;   // General-purpose join
    pub fn join_with_comma(elements: Vec<Self>) -> Self;           // Comma+space join (gray)
}
```

### 4.2 TextComponentBase — Component Base

**File**: `pumpkin-util/src/text/mod.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TextComponentBase {
    #[serde(flatten)]
    pub content: Box<TextContent>,
    #[serde(flatten)]
    pub style: Box<Style>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Self>,
}
```

- `content` and `style` use `#[serde(flatten)]`, so their fields are serialized at the same nesting level
- `extra` is omitted from serialization when empty
- Serialized in camelCase (e.g., `clickEvent`, `hoverEvent`, `shadowColor`)

**Output Methods**:

```rust
impl TextComponentBase {
    pub fn to_pretty_console(self) -> String;           // ANSI console output
    pub fn to_bedrock_string(self) -> String;            // Bedrock % translation key format
    pub fn to_bedrock_legacy(self, locale: Locale) -> String; // Bedrock § format codes + translation
    pub fn get_text(self, locale: Locale) -> String;     // Plain text (specified language)
    pub fn to_translated(self) -> Self;                   // Resolve all translations (recursive)
}
```

**to_pretty_console Output Flow**:

```
TextContent → Plain text
  ├── Text/EntityNames/Keybind → Output directly
  ├── Translate → Look up EnUs translation of minecraft:key
  └── Custom → Look up translation of namespace:key
→ Apply color.console_color()
→ Apply bold/italic/underline/strikethrough (ANSI escapes)
→ If OpenUrl/OpenFile click event → Wrap with OSC 8 hyperlink
→ Recursively process extra child components
```

### 4.3 TextContent — Content Type

**File**: `pumpkin-util/src/text/mod.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TextContent {
    Text {
        text: Cow<'static, str>,
    },
    Translate {
        translate: Cow<'static, str>,
        #[serde(skip, default)]
        bedrock_translate: Option<Cow<'static, str>>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<TextComponentBase>,
    },
    EntityNames {
        selector: Cow<'static, str>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Cow<'static, str>>,
    },
    Keybind {
        keybind: Cow<'static, str>,
    },
    #[serde(skip)]  // Not serialized directly; requires to_translated() first
    Custom {
        key: Cow<'static, str>,
        locale: Locale,
        with: Vec<TextComponentBase>,
    },
}
```

| Variant       | JSON Form                                        | Description                                                                     |
|---------------|--------------------------------------------------|---------------------------------------------------------------------------------|
| `Text`        | `{"text": "Hello"}`                              | Plain text                                                                      |
| `Translate`   | `{"translate": "chat.type.text", "with": [...]}` | Client-side translation                                                         |
| `EntityNames` | `{"selector": "@a"}`                             | Entity selector result                                                          |
| `Keybind`     | `{"keybind": "key.forward"}`                     | Key binding                                                                     |
| `Custom`      | ❌ Not serialized                                 | Server-side custom translation; requires `to_translated()` before serialization |

`TextContent` uses `#[serde(untagged)]` to auto-infer the variant, but because `Custom` is marked `#[serde(skip)]`, it
is never encountered during serialization. During deserialization, `Custom` is not recognized by serde (it must be
constructed via `TextComponent::custom()`).

### 4.4 Style — Style

**File**: `pumpkin-util/src/text/style.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub color: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub insertion: Option<String>,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub font: Option<String>,
    #[serde(rename = "shadow_color")]
    pub shadow_color: Option<ARGBColor>,
}
```

- All fields are `Option`; `None` means "use the default when not inherited from a parent component"
- All fields use `skip_serializing_if = "Option::is_none"`
- Chainable builder methods are provided (most are `const fn`)

**Builder Methods**:

```rust
impl Style {
    pub const fn color(self, color: Color) -> Self;
    pub const fn color_named(self, color: NamedColor) -> Self;
    pub const fn bold(self) -> Self;
    pub const fn italic(self) -> Self;
    pub const fn underlined(self) -> Self;
    pub const fn strikethrough(self) -> Self;
    pub const fn obfuscated(self) -> Self;
    pub const fn shadow_color(self, color: ARGBColor) -> Self;
    pub fn insertion(self, text: String) -> Self;
    pub fn click_event(self, event: ClickEvent) -> Self;
    pub fn hover_event(self, event: HoverEvent) -> Self;
    pub fn font(self, resource_location: String) -> Self;
}
```

### 4.5 Color System

**File**: `pumpkin-util/src/text/color.rs`

```rust
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Color {
    #[default]
    Reset,
    Rgb(RGBColor),
    Named(NamedColor),
}
```

**Custom Deserialize**:

```rust
// "reset" → Reset
// "#RRGGBB" → Rgb(RGBColor)
// "red", "dark_blue", ... → Named(NamedColor)
```

Error messages on deserialization failure are i18n-ed (translation keys prefixed with `"pumpkin:text.color."`).

**Methods**:

```rust
impl Color {
    pub fn console_color(&self, text: &str) -> ColoredString; // ANSI terminal color
    pub const fn from_legacy_code(code: char) -> Option<Self>; // §0-§f → Color
    pub fn from_hex_str(hex: &str) -> Option<Self>;             // "FF55AA" → Rgb
}
```

**NamedColor — 16 Minecraft Standard Colors**:

```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black = 0,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}
```

**console_color ANSI Mapping**:

| NamedColor      | ANSI Mapping         |
|-----------------|----------------------|
| Black           | `black()`            |
| DarkBlue        | `blue()`             |
| DarkGreen       | `green()`            |
| DarkAqua        | `cyan()`             |
| DarkRed         | `red()`              |
| DarkPurple      | `purple()`           |
| Gold            | `yellow()`           |
| Gray / DarkGray | `bright_black()` ⚠️  |
| Blue            | `bright_blue()`      |
| Green           | `bright_green()`     |
| Aqua            | `bright_cyan()`      |
| Red             | `bright_red()`       |
| LightPurple     | `bright_purple()`    |
| Yellow          | `bright_yellow()`    |
| White           | `white()`            |
| Rgb(r, g, b)    | `truecolor(r, g, b)` |

> ⚠️ Gray and DarkGray both map to `bright_black()`, making them indistinguishable on ANSI 16-color terminals.

**RGBColor**:

```rust
#[derive(Debug, Deserialize, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}
impl Serialize for RGBColor { /* → "#RRGGBB" */ }
```

**ARGBColor** (fields are private):

```rust
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
pub struct ARGBColor {
    alpha: u8,
    red: u8,
    green: u8,
    blue: u8
}
impl Serialize for ARGBColor { /* → [alpha, red, green, blue] byte array */ }
```

**hsv_to_rgb**:

```rust
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8);
// h: Hue (0-360°), s: Saturation (0.0-1.0), v: Value (0.0-1.0)
```

### 4.6 ClickEvent — Click Event

**File**: `pumpkin-util/src/text/click.rs`

```rust
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClickEvent {
    OpenUrl { url: Cow<'static, str> },
    OpenFile { path: Cow<'static, str> },
    RunCommand { command: Cow<'static, str> },
    SuggestCommand { command: Cow<'static, str> },
    ChangePage { page: u32 },
    CopyToClipboard { value: Cow<'static, str> },
}
```

| Variant           | JSON                                            |
|-------------------|-------------------------------------------------|
| `OpenUrl`         | `{"action":"open_url","url":"https://..."}`     |
| `RunCommand`      | `{"action":"run_command","command":"/help"}`    |
| `CopyToClipboard` | `{"action":"copy_to_clipboard","value":"text"}` |

### 4.7 HoverEvent — Hover Event

**File**: `pumpkin-util/src/text/hover.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum HoverEvent {
    ShowText {
        value: Vec<TextComponentBase>,
    },
    ShowItem {
        id: Cow<'static, str>,
        count: Option<i32>,
    },
    ShowEntity {
        id: Cow<'static, str>,           // Entity type (e.g., "minecraft:pig")
        uuid: Cow<'static, str>,         // UUID string
        name: Option<Vec<TextComponentBase>>,
    },
}
```

**Convenience Constructors**:

```rust
impl HoverEvent {
    pub fn show_text(text: TextComponent) -> Self;
    pub fn show_entity<P: Into<Cow<'static, str>>>(
        uuid: P, kind: P, name: Option<TextComponent>,
    ) -> Self;
}
```

### 4.8 Translation Helper Functions

**File**: `pumpkin-util/src/text/translation.rs`

```rust
/// Parses %s / %1$s placeholders in the translation string and reorders `with` components by index
pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>);

/// Looks up a translation key, substitutes placeholders, returns a console-friendly colored string
pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String;

/// Similar to translation_to_pretty, but outputs plain text (no ANSI color codes)
pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String;
```

- `translation_to_pretty` used for the `Translate` / `Custom` variants in `TextComponentBase::to_pretty_console()`
- `get_translation_text` used for `TextComponentBase::get_text()` and `to_bedrock_legacy()`

---

## 5. Complete Usage Examples

### 5.1 Initialize i18n

```rust
use pumpkin_i18n::{resolve_server_locale, set_server_global_locale};

// At server startup (typically in main.rs)
let config = PumpkinConfig::load();
let locale = resolve_server_locale( & config.advanced.locale.server_global); // "auto" or "zh_cn"
set_server_global_locale(locale);
```

### 5.2 Explicit Locale Translation (translate_plain / translate_format)

```rust
use pumpkin_util::translation::{translate_plain, translate_format};
use pumpkin_i18n::Locale;

let sender_locale = player_locale("550e8400-...");

// Plain text
let msg = translate_plain("commands.pumpkin.stop.success", sender_locale);
// → "Server stopped" (EnUs) / "服务器已停止" (ZhCn)

// Formatted with parameters
let msg = translate_format(
"commands.pumpkin.tp.success",
sender_locale,
& ["Alice".to_string(), "Bob".to_string()],
);
```

### 5.3 Plain Text Log (localized_log)

```rust
use pumpkin_util::translation::localized_log;

// Simple log — automatically uses server_global_locale()
let msg = localized_log("server.log.started_accepting_connections");
info!("{}", msg);
// → "Stopped accepting incoming connections" (EnUs)
```

### 5.4 Formatted Log (localized_log_format)

```rust
use pumpkin_util::translation::localized_log_format;

// Log with parameters
let msg = localized_log_format(
"server.log.build_info",
& [os.to_string(), arch.to_string(), debug_flag.to_string()],
);
info!("{}", msg);
// → "Build info: FAMILY: \"unix\", OS: \"linux\", ARCH: \"x86_64\", BUILD: \"Debug\""
```

### 5.5 Colored Startup Banner (localized_text)

```rust
use pumpkin_util::translation::localized_text;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

// ✅ Correct — Use localized_text, child components retain coloring
let msg = localized_text(
"server.log.starting_server",  // Translation template: "Starting %s %s Minecraft (Protocol %s)"
[
TextComponent::text("Pumpkin").color_named(NamedColor::Gold),
TextComponent::text(CARGO_PKG_VERSION).color_named(NamedColor::Green),
TextComponent::text(protocol_version).color_named(NamedColor::DarkBlue),
],
);
info!("{}", msg.to_pretty_console());
// → "Starting \x1b[33mPumpkin\x1b[0m \x1b[32m0.1.0-dev\x1b[0m Minecraft (Protocol \x1b[34m766\x1b[0m)"

// ❌ Wrong — Do not pass .to_pretty_console() into localized_log_format
// localized_log_format("server.log.starting_server", &[
//     TextComponent::text("Pumpkin").color_named(NamedColor::Gold).to_pretty_console(),
// ]); // ANSI codes will be corrupted by nesting!
```

### 5.6 Build and Send Chat Messages

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

let msg = TextComponent::empty()
.add_child(
TextComponent::text("[Server] ")
.color_named(NamedColor::Gold)
.bold()
)
.add_child(
TextComponent::translate(
"multiplayer.player.joined",
[TextComponent::text("Steve")]
)
.color_named(NamedColor::Yellow)
);

// NBT-serialize and send to client
let bytes: Box<[u8] > = msg.encode();
```

### 5.7 Console Logging (Colored)

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

let msg = TextComponent::text("Server started!")
.color_named(NamedColor::Green)
.bold();

println!("{}", msg.to_pretty_console());
```

### 5.8 Client Language Caching

```rust
use pumpkin_i18n::{set_player_locale, player_locale, remove_player_locale};

// Player login
let locale = set_player_locale(
"550e8400-e29b-41d4-a716-446655440000", // UUID
"zh_cn",                                 // Client-reported
"auto",                                  // Server config
);

// Retrieve for translation
let locale = player_locale("550e8400-e29b-41d4-a716-446655440000");

// Player leaves
remove_player_locale("550e8400-e29b-41d4-a716-446655440000");
```

### 5.9 Text Gradient Effect

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

let msg = TextComponent::text("Welcome to the server!")
.gradient_named( & [NamedColor::Red, NamedColor::Gold, NamedColor::Green]);
```

### 5.10 Rainbow Text

```rust
use pumpkin_util::text::TextComponent;

let msg = TextComponent::text("RAINBOW TEXT").rainbow();
```

### 5.11 Rich Text + Events

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::color::NamedColor;
use std::borrow::Cow;

let msg = TextComponent::text("Click me!")
.color_named(NamedColor::Aqua)
.bold()
.underlined()
.click_event(ClickEvent::OpenUrl {
url: Cow::Borrowed("https://example.com")
})
.hover_event(HoverEvent::show_text(
TextComponent::text("Go to example.com")
.color_named(NamedColor::Gray)
));
```

### 5.12 Dynamically Adding Translations

```rust
use pumpkin_i18n::{add_translation, add_translation_file, Locale};

// Single translation
add_translation("myplugin", "welcome", "Welcome!", Locale::ZhCn);

// Bulk load
add_translation_file(
"myplugin",
r#"{
        "welcome": "Welcome!",
        "goodbye": "Goodbye!",
        "error.not_found": "Player not found"
    }"#,
Locale::ZhCn,
);
```

### 5.13 Using the Translation Engine Directly (Advanced)

> Only for extreme throughput scenarios; use the surface-level API for most cases.

```rust
use pumpkin_i18n::engine::TranslationEngine;
use std::collections::HashMap;

// Build engine
let data: Vec<HashMap<String, String> > = vec![/* one map per language */];
let engine = TranslationEngine::build( & data);

// High-frequency translation
let resolved = engine.resolve(Locale::EnUs as usize, "pumpkin:welcome");
let mut buf = String::new();
resolved.write_to( & ["Steve".into()], & mut buf);
// buf → "Welcome, Steve"  (if translation key is "Welcome, %s")

// For most scenarios, use the surface-level API instead:
use pumpkin_util::translation::localized_log_format;
let msg = localized_log_format("welcome", & ["Steve".to_string()]);
```
