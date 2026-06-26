# pumpkin-i18n & Text & Translation API Documentation

> **生成日期**: 2026-06-26  
> **Rust Edition**: 2024 | **MSRV**: 1.95  
> **版本**: 0.1.0-dev+26.2

---

## 目录

- [统一的翻译入口层 (pumpkin-util/src/translation.rs)](#统一的翻译入口层-pumpkin-utilsrctranslationrs)
  - [localized_log / localized_log_format / localized_text](#localized_log--localized_log_format--localized_text)
  - [何时使用哪个函数](#何时使用哪个函数)
- [pumpkin-i18n](#pumpkin-i18n)
    - [架构概览](#pumpkin-i18n-架构概览)
    - [模块结构](#pumpkin-i18n-模块结构)
    - [Locale — 语言环境](#1-locale---语言环境)
    - [Server — 服务端语言](#2-server---服务端语言)
    - [Client — 客户端语言](#3-client---客户端语言)
    - [Store — 翻译存储](#4-store---翻译存储)
    - [Engine — 翻译引擎（高级）](#5-engine---翻译引擎高级)
    - [Token — 格式化占位符预编译](#6-token---格式化占位符预编译)
    - [内部工具函数](#7-内部工具函数)
- [pumpkin-util/src/text](#pumpkin-utilsrctext)
    - [架构概览](#pumpkin-utilsrctext-架构概览)
    - [模块结构](#text-模块结构)
    - [TextComponent — 文本组件](#1-textcomponent---文本组件)
    - [TextComponentBase — 组件基类](#2-textcomponentbase---组件基类)
    - [TextContent — 内容类型](#3-textcontent---内容类型)
    - [Style — 样式](#4-style---样式)
    - [Color / NamedColor / RGBColor / ARGBColor — 颜色系统](#5-color--namedcolor--rgbcolor--argbcolor---颜色系统)
    - [ClickEvent — 点击事件](#6-clickevent---点击事件)
    - [HoverEvent — 悬浮事件](#7-hoverevent---悬浮事件)
    - [Translation 辅助函数](#8-translation-辅助函数)
- [翻译键命名规范](#翻译键命名规范)
- [使用示例](#完整使用示例)---

# 统一的翻译入口层 (pumpkin-util/src/translation.rs)

**文件**: `pumpkin-util/src/translation.rs`

`pumpkin-util::translation` 是整个 Pumpkin 项目中**唯一的翻译函数定义位置**。所有 crate 统一从此模块导入，
不再在 `pumpkin` crate 中重复定义。

## localized_log / localized_log_format / localized_text

这三个函数是服务端代码使用 i18n 的主要入口。它们内部调用 `pumpkin_i18n` 的底层函数，
并自动使用 `server_global_locale()` 作为语言参数。

### localized_log — 纯文本日志

```rust
use pumpkin_util::translation::localized_log;

pub fn localized_log(key: &str) -> String;
```

- 将 key 自动加上 `pumpkin:` 命名空间前缀
- 调用 `pumpkin_i18n::get_translation(key, server_global_locale())` 查找翻译
- 翻译缺失时返回原始 key 字符串
- 用于日志、panic 消息、错误信息等纯文本场景

### localized_log_format — 格式化日志

```rust
use pumpkin_util::translation::localized_log_format;

pub fn localized_log_format(key: &str, args: &[String]) -> String;
```

- 与 `localized_log` 类似，但额外支持占位符替换
- 调用 `pumpkin_i18n::format_translation(key, server_global_locale(), args)`
- args 中的 **纯字符串**（非 `TextComponent`）会按索引替换翻译模板中的 `%s` 占位符
- 示例：`localized_log_format("server.log.build_info", &[os, arch, debug_flag])`

### localized_text — 带染色子组件的翻译

```rust
use pumpkin_util::translation::localized_text;

pub fn localized_text<W: Into<Vec<TextComponent>>>(key: &'static str, with: W) -> TextComponent;
```

- 创建 `TextComponent::custom("pumpkin", key, server_global_locale(), with)`
- 子组件 `with` 会被插入到翻译模板的占位符位置，**保留颜色和样式**
- 返回 `TextComponent`，可以继续链式调用 `.to_pretty_console()` 等方法
- ❗**不要**将 `.to_pretty_console()` 的结果传给 `localized_log_format` — 那会导致 ANSI 码嵌套错误

## 何时使用哪个函数

| 场景         | 推荐函数                       | 原因                       |
|------------|----------------------------|--------------------------|
| 控制台纯文本日志   | `localized_log`            | 最简路径，无额外开销               |
| 带参数的格式化日志  | `localized_log_format`     | 支持 `%s` 占位符 + 参数         |
| 带颜色的启动横幅   | `localized_text`           | 子组件保留染色                  |
| 玩家聊天消息     | `TextComponent::translate` | 客户端翻译，非服务端               |
| 服务端自定义翻译消息 | `localized_text`           | `TextContent::Custom` 变体 |

### 完整调用链路

```
代码中调用                    翻译入口层                       i18n 引擎
──────────────────────────────────────────────────────────────────────────
localized_log("key")         → get_translation("pumpkin:key", locale)  → resolve(key, locale)
localized_log_format("k",a)  → format_translation("pumpkin:k", l, a)  → resolve → tokens → write
localized_text("k", [c])     → TextComponent::custom → .to_pretty()   → resolve → tokens → render
```

### 导入规范

所有 crate 统一从 `pumpkin-util` 导入：

```rust
// ✅ 正确 — 所有 crate 统一使用此路径
use pumpkin_util::translation::{localized_log, localized_log_format, localized_text};

// ❌ 错误 — pumpkin crate 中不再有这些函数的定义
use crate::localized_log;
use pumpkin::localized_log;
```

---

# pumpkin-i18n

## pumpkin-i18n 架构概览

`pumpkin-i18n` 是 Pumpkin Minecraft 服务端的国际化（i18n）核心库，负责：

- **128 种语言**的翻译键存储与检索（分布在 `assets/translations/pumpkin/` 下）
- **1121 个翻译键**，覆盖服务端日志、命令系统、世界生成、认证等所有模块
- 服务端日志语言解析（系统环境检测 / 配置文件覆盖）
- 玩家语言缓存（UUID → Locale 映射）
- 零正则、流式输出的格式化占位符预编译（`%s`, `%1$s`, `{}`, `{0}`）
- 基于 FST (Finite State Transducer) 的高性能翻译引擎

> **注意**: 业务代码不应直接调用 `pumpkin_i18n` 的函数。请使用 `pumpkin-util::translation` 中的
> `localized_log` / `localized_log_format` / `localized_text` 作为统一入口。
> 这些函数自动处理命名空间前缀和 `server_global_locale()` 解析。

### 依赖

| 依赖            | 用途                   |
|---------------|----------------------|
| `arc-swap`    | 无锁原子替换翻译数据           |
| `dashmap`     | 并发安全的缓存 HashMap      |
| `fst`         | FST 索引加速键查找          |
| `serde_json`  | 解析翻译 JSON 文件         |
| `tracing`     | 丢失翻译键的 warn/error 日志 |
| `xxhash-rust` | 高速哈希（DashMap 用）      |

---

## pumpkin-i18n 模块结构

```
pumpkin-i18n/src/
├── lib.rs          # crate root, re-exports, SubstitutionRange, parse_locale_value
├── locale.rs       # Locale 枚举 (128 variants) + FromStr
├── server.rs       # server_global_locale(), set_server_global_locale(), detect_system_locale()
├── client.rs       # player_locale(), set_player_locale(), resolve_client_locale()
├── store.rs        # TRANSLATIONS 全局存储, get_translation(), add_translation()
├── engine.rs       # TranslationEngine (FST + DashMap cache)
└── token.rs        # Token 枚举, precompile(), format_tokens()
```

---

## 1. Locale — 语言环境

**文件**: `pumpkin-i18n/src/locale.rs`

### 枚举定义

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locale {
    // 128 个变体，按字母序:
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
    EnUs,  // ← EnUs 是默认回退
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

### 常量

| 方法      | 签名                       | 说明        |
|---------|--------------------------|-----------|
| `COUNT` | `pub const COUNT: usize` | 语言总数（128） |

### FromStr 实现

```rust
impl FromStr for Locale {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
```

- 解析 `"en_us"`, `"zh_cn"`, `"de_de"` 等蛇形命名
- **不区分大小写**
- 未匹配时回退到 `Locale::EnUs`（不返回 Err）
- 每个变体有内联注释标注语言全称

---

## 2. Server — 服务端语言

**文件**: `pumpkin-i18n/src/server.rs`

### 公开 API

```rust
// 获取当前服务端日志语言
pub fn server_global_locale() -> Locale;

// 设置服务端日志语言（由 pumpkin server crate 在启动时调用）
pub fn set_server_global_locale(locale: Locale);

// 自动检测系统语言
pub fn detect_system_locale() -> Locale;

// 解析配置值并定位语言
pub fn resolve_server_locale(config_value: &str) -> Locale;
```

### 详细说明

| 函数                           | 行为                                                                                                                                  |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------------------|
| `server_global_locale()`     | 返回 `OnceLock` 中存储的语言，未初始化时回退 `EnUs`                                                                                                 |
| `set_server_global_locale()` | **仅首次调用生效**（`OnceLock`），后续调用被静默忽略                                                                                                   |
| `detect_system_locale()`     | **Linux/macOS**: 读取 `LANG` → `LC_ALL` → `LC_MESSAGES` 环境变量<br>**Windows**: 调用 `GetUserDefaultLocaleName` API<br>**其他平台**: 回退 `EnUs` |
| `resolve_server_locale(cfg)` | 若 `cfg == "auto"` 调用 `detect_system_locale()`；否则解析配置值                                                                               |

### 与服务端配置集成

`server_global_locale()` 由 `pumpkin` crate 在启动时通过 `pumpkin_config` 配置初始化：

```rust
// pumpkin/src/main.rs — 启动流程
use pumpkin_i18n::{resolve_server_locale, set_server_global_locale};

let config = PumpkinConfig::load();  // 包含 advanced.locale.server_global 字段
let server_global_locale = resolve_server_locale( & config.advanced.locale.server_global);
set_server_global_locale(server_global_locale);
```

配置结构定义在 `pumpkin-config/src/locale.rs`：

```rust
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
  /// 服务端日志和控制台输出的语言 ("auto" 或语言代码)
  pub server_global: String,
  /// Java Edition 客户端语言解析策略
  pub client_java_edition: String,
  /// Bedrock Edition 客户端语言解析策略
  pub client_bedrock_edition: String,
}
```

默认使用 `"auto"`（自动检测系统语言）。设置为 `"zh_cn"` 等具体语言代码可强制覆盖。

### 用法示例

```rust
use pumpkin_i18n::{detect_system_locale, resolve_server_locale, set_server_global_locale};

// 启动时
let locale = resolve_server_locale("auto");  // 或 "zh_cn"
set_server_global_locale(locale);
```

---

## 3. Client — 客户端语言

**文件**: `pumpkin-i18n/src/client.rs`

### 公开 API

```rust
// 登录时缓存玩家语言
pub fn set_player_locale(uuid: &str, player_reported_locale: &str, config_value: &str) -> Locale;

// 获取已缓存的玩家语言
pub fn player_locale(uuid: &str) -> Locale;

// 玩家离开时清除缓存
pub fn remove_player_locale(uuid: &str);

// 纯函数：根据配置和客户端上报值计算最终语言
pub fn resolve_client_locale(player_locale: &str, config_value: &str) -> Locale;

// 将 Locale 转换为日志用字符串 (如 "en_us")
pub fn locale_to_log_string(locale: Locale) -> String;
```

### 内部结构

```rust
// 基于 DashMap + XXH64 的全局玩家语言缓存
static PLAYER_CACHE: LazyLock<DashMap<String, Locale, BuildHasherDefault<Xxh64>>>;
```

### 语言解析逻辑

```
config_value == "auto" ?
  ├── 是 → 使用玩家上报的语言
  └── 否 → 覆盖为配置的语言（如全部玩家使用 zh_cn）
```

### 用法示例

```rust
use pumpkin_i18n::{set_player_locale, player_locale, remove_player_locale};

// 玩家加入
let loc = set_player_locale("550e8400-...", "zh_cn", "auto");

// 翻译时获取
let locale = player_locale("550e8400-...");

// 玩家离开
remove_player_locale("550e8400-...");
```

---

## 4. Store — 翻译存储

**文件**: `pumpkin-i18n/src/store.rs`

### 全局存储

```rust
pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::COUNT]>>;
```

- 编译时通过 `build.rs` 将所有 JSON 翻译文件嵌入二进制
- 运行时 `Mutex` 保护（写锁），允许插件动态添加翻译

### 公开 API

```rust
pub fn get_translation(key: &str, locale: Locale) -> String;

pub fn add_translation<P: Into<String>>(namespace: P, key: P, translation: P, locale: Locale);

pub fn add_translation_file<P: Into<String>>(namespace: P, file_path: P, locale: Locale);
```

### get_translation — 三级回退策略

```
Tier 1: 请求的 locale → 命中直接返回 (无日志)
Tier 2: EnUs 回退       → warn! 日志 + 返回英文字符串
Tier 3: 原始 key       → error! 日志 + 返回 key 自身
```

> **注意**: key 比较时会被 `to_ascii_lowercase()` 处理

### add_translation / add_translation_file

```rust
// 单条添加
add_translation("pumpkin", "welcome", "Willkommen", Locale::DeDe);

// 从 JSON 字符串批量加载
add_translation_file(
"pumpkin",
r#"{"welcome": "Willkommen", "goodbye": "Auf Wiedersehen"}"#,
Locale::DeDe,
);
```

- `add_translation()`: 单键值对插入，key 自动拼接为 `"namespace:key"`
- `add_translation_file()`: 从 JSON 字符串解析 `HashMap` 并批量插入
    - JSON 为空或解析失败时静默返回（TODO: 需要更健壮的错误处理）

---

## 5. Engine — 翻译引擎（高级）

**文件**: `pumpkin-i18n/src/engine.rs`

### 设计目的

为高频读取场景（每个聊天消息、每个 UI 文本都需翻译）提供极致性能：

- **FST 索引**: O(key_length) 的键查找，相比 HashMap 有更小的内存占用
- **ArcSwap 存储**: 写者替换整个语言数据，读者无锁读取
- **DashMap 缓存**: 命中后 lock-free，XxHash64 降低哈希碰撞
- **预编译 Token**: 翻译字符串中的 `%s` 占位符在加载时编译为 `TokenStream`

### 核心类型

```rust
pub struct TranslationEngine {
    stores: ArcSwap<Box<[FstLocaleStore]>>,   // 每种语言一个 FST 存储
    cache: DashMap<String, Arc<ResolvedTranslation>, BuildHasherDefault<Xxh64>>,
}

pub enum ResolvedTranslation {
    Static(Arc<str>),       // 无占位符的纯文本
    Tokenized(TokenStream), // 预编译的 token 流
}
```

### 公开 API

```rust
impl TranslationEngine {
    /// 从每语言翻译映射表构建引擎
    pub fn build(data: &[HashMap<String, String>]) -> Self;

    /// 解析翻译键（三级回退+缓存）
    pub fn resolve(&self, locale_idx: usize, key: &str) -> Arc<ResolvedTranslation>;

    /// 原子重载翻译数据
    pub fn reload(&self, data: &[HashMap<String, String>]);
}

impl ResolvedTranslation {
    /// 将格式化结果写入缓冲区
    pub fn write_to(&self, args: &[String], buf: &mut String);
}

/// 流式格式化预编译 Token 到缓冲区
pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String);
```

### resolve() 三级回退

与 `store::get_translation()` 完全一致的回退逻辑，但结果缓存在 `DashMap` 中。

### 缓存键格式

`"<locale_idx>:<key>"` — 例如 `"32: pumpkin:welcome"`（32 = EnUs 的枚举索引）

---

## 6. Token — 格式化占位符预编译

**文件**: `pumpkin-i18n/src/token.rs`

### Token 枚举

```rust
pub enum Token {
    Text(Arc<str>),  // 纯文本片段，直接输出
    Var(usize),      // 变量占位符，索引指向 args 数组
}

pub type TokenStream = Arc<[Token]>;
```

### precompile()

```rust
pub fn precompile(template: &str) -> Option<TokenStream>;
```

解析翻译字符串中的占位符，返回预编译的 Token 序列。

**支持的占位符格式**:

| 格式               | 说明                | 示例                                          |
|------------------|-------------------|---------------------------------------------|
| `%%`             | 转义字面 `%`          | `"100%%"` → `Token::Text("100%")`           |
| `%s`, `%d`, `%f` | 顺序索引 (0, 1, 2, …) | `"%s joined"` → `[Var(0), Text(" joined")]` |
| `%1$s`, `%2$d`   | 显式 1-based 索引     | `"%2$s → %1$s"` → 参数反转                      |

**返回值**: `None` 表示字符串不含任何 `%`（调用方可直接使用原字符串）。

### format_tokens()

```rust
pub fn format_tokens(tokens: &[Token], args: &[String], buf: &mut String);
```

- 零分配流式写入
- `Var(idx)` 若越界则输出空字符串（不 panic）

---

## 7. 内部工具函数

**文件**: `pumpkin-i18n/src/lib.rs`

### parse_locale_value (pub(crate))

```rust
pub(crate) fn parse_locale_value(raw: &str) -> Locale;
```

- 将 `-` 标准化为 `_`
- 解析失败回退 `Locale::EnUs`
- 被 `server` 和 `client` 模块使用

### SubstitutionRange

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    pub start: usize,  // 起始字节索引（含）
    pub end: usize,    // 结束字节索引（含）
}
impl SubstitutionRange {
    pub const fn len(&self) -> usize;      // (end - start) + 1
    pub const fn is_empty(&self) -> bool;  // start == end
}
```

用于标记翻译字符串中占位符的字节区间，被 `pumpkin-util/src/text/translation.rs` 使用。

---

# pumpkin-util/src/text

## pumpkin-util/src/text 架构概览

`pumpkin-util/src/text` 实现了 Minecraft 聊天组件系统，包括：

- JSON ↔ NBT 序列化（与 Minecraft 协议兼容）
- 控制台彩色输出（利用 ANSI 转义码）
- Bedrock Edition 字符串生成
- 文本渐变（gradient）和彩虹（rainbow）效果
- 富文本样式（粗体、斜体、下划线、删除线、混淆）
- 事件系统（点击、悬浮）

### 依赖 (text 相关)

`pumpkin-i18n`（用于翻译键查找）、`pumpkin-nbt`（NBT 序列化）、`serde` / `serde_json`、`colored`（控制台 ANSI 颜色）

---

## Text 模块结构

```
pumpkin-util/src/
├── translation.rs # localized_log, localized_log_format, localized_text (统一翻译入口)
└── text/
    ├── mod.rs         # TextComponent, TextComponentBase, TextContent, 测试
    ├── color.rs       # Color, NamedColor, RGBColor, ARGBColor, hsv_to_rgb
    ├── style.rs       # Style (颜色、粗体、斜体、下划线、删除线、混淆、插入、点击、悬浮、字体、阴影)
    ├── click.rs       # ClickEvent 枚举
    ├── hover.rs       # HoverEvent 枚举
    └── translation.rs # reorder_substitutions, translation_to_pretty, get_translation_text
```

> `pumpkin-util/src/translation.rs` 是**所有 crate 的统一翻译入口**。它封装 `pumpkin_i18n` 的底层函数，
> 自动处理 `pumpkin:` 命名空间前缀和 `server_global_locale()` 解析。
> `pumpkin-util/src/text/translation.rs` 是 `TextComponent` 渲染所需的低级翻译辅助函数。

---

## 1. TextComponent — 文本组件

**文件**: `pumpkin-util/src/text/mod.rs:32`

### 定义

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextComponent(pub TextComponentBase);
```

`TextComponent` 是一个 newtype 包装器，持有内部的 `TextComponentBase`。

### 序列化行为

**Deserialize**: 接受三种 JSON 格式

- **字符串**: `"Hello"` → 纯文本组件
- **数组**: `[comp1, comp2]` → 空内容 + `extra` 包含所有元素
- **对象**: `{"text": "...", "color": "red"}` → 标准组件

**Serialize**: 通过 `to_translated()` 先解析所有翻译，再序列化为 JSON 对象。

### 构造方法

```rust
impl TextComponent {
    pub fn empty() -> Self;                          // 空组件（用于收集子组件）
    pub fn text<P: Into<Cow<'static, str>>>(p) -> Self;  // 纯文本
    pub fn translate<K, W>(key: K, with: W) -> Self; // 客户端翻译（key 如 "multiplayer.player.joined"）
    pub fn translate_cross<K1, K2, W>(java_key, bedrock_key, with) -> Self; // 跨平台翻译
    pub fn custom<K, W>(namespace, key, locale, with) -> Self; // 自定义翻译（服务端解析）
    pub fn from_legacy_string(input: &str) -> Self;  // 解析 § 格式的遗留字符串
    pub fn from_content(content: TextContent) -> Self; // 从 TextContent 创建
    pub fn chat_decorated(format, player_name, content) -> Self; // 聊天消息格式化
}
```

### 链式修改器

每个方法返回 `Self`，支持链式调用。

```rust
impl TextComponent {
    // --- 颜色 ---
    pub fn color(self, color: Color) -> Self;
    pub fn color_named(self, color: NamedColor) -> Self;
    pub fn color_rgb(self, color: RGBColor) -> Self;
    pub fn gradient(self, colors: &[RGBColor]) -> Self;       // RGB 渐变
    pub fn gradient_named(self, colors: &[NamedColor]) -> Self; // 命名颜色渐变
    pub fn rainbow(self) -> Self;                               // 彩虹效果

    // --- 样式 ---
    pub fn bold(self) -> Self;
    pub fn italic(self) -> Self;
    pub fn underlined(self) -> Self;
    pub fn strikethrough(self) -> Self;
    pub fn obfuscated(self) -> Self;

    // --- 高级 ---
    pub fn font(self, resource_location: String) -> Self;       // 设置字体
    pub fn shadow_color(self, color: ARGBColor) -> Self;         // 阴影颜色
    pub fn insertion(self, text: String) -> Self;                // Shift点击文本
    pub fn click_event(self, event: ClickEvent) -> Self;
    pub fn hover_event(self, event: HoverEvent) -> Self;

    // --- 拼接 ---
    pub fn add_child(self, child: Self) -> Self;                 // 追加子组件
    pub fn add_text<P: Into<Cow<'static, str>>>(self, text) -> Self; // 追加纯文本

    // --- 换行 & 括号 ---
    pub fn new_line(self) -> Self;                                // 追加换行
    pub fn wrap_in_square_brackets(self) -> Self;                 // 用 [ ] 包裹

    // --- 输出 ---
    pub fn to_pretty_console(self) -> String;                     // 控制台彩色字符串
    pub fn get_text(self) -> String;                              // 纯文本 (EnUs)
    pub fn encode(&self) -> Box<[u8]>;                            // NBT 序列化
}
```

### 静态方法

```rust
impl TextComponent {
    pub fn join(elements: Vec<Self>, separator: &Self) -> Self;   // 通用拼接
    pub fn join_with_comma(elements: Vec<Self>) -> Self;           // 逗号+空格拼接 (灰色)
}
```

### 内部方法

```rust
fn apply_color_effect<F>(self, color_gen: F) -> Self
where
    F: Fn(usize, usize) -> RGBColor;
```

- 将文本拆分为逐字符，对每个字符调用 `color_gen(i, total_len)` 获得独立颜色
- 原文本内容清空，彩色字符放入 `extra` 字段
- `gradient()`, `rainbow()` 均基于此实现

---

## 2. TextComponentBase — 组件基类

**文件**: `pumpkin-util/src/text/mod.rs:88`

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

### 序列化说明

- `content` 和 `style` 使用 `#[serde(flatten)]`，序列化时字段平铺到同一层级
- `extra` 为空时不序列化
- 序列化为 camelCase（如 `clickEvent`, `hoverEvent`, `shadowColor`）

### 输出方法

```rust
impl TextComponentBase {
    pub fn to_pretty_console(self) -> String;           // ANSI 控制台输出
    pub fn to_bedrock_string(self) -> String;            // Bedrock % 翻译键格式
    pub fn to_bedrock_legacy(self, locale: Locale) -> String; // Bedrock § 格式码 + 翻译
    pub fn get_text(self, locale: Locale) -> String;     // 纯文本（指定语言）
    pub fn to_translated(self) -> Self;                   // 解析所有翻译（递归）
}
```

### to_pretty_console 输出流程

```
TextContent → 纯文本
  ├── Text/EntityNames/Keybind → 直接输出
  ├── Translate → 查询 minecraft:key 的 EnUs 翻译
  └── Custom → 查询 namespace:key 的翻译
→ 应用 color.console_color()
→ 应用 bold/italic/underline/strikethrough (ANSI 转义)
→ 若有 OpenUrl/OpenFile 点击事件 → 包裹 OSC 8 链接
→ 递归处理 extra 子组件
```

---

## 3. TextContent — 内容类型

**文件**: `pumpkin-util/src/text/mod.rs:1108`

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
    #[serde(skip)]  // 不直接序列化，需先 to_translated()
    Custom {
        key: Cow<'static, str>,
        locale: Locale,
        with: Vec<TextComponentBase>,
    },
}
```

### 各变体说明

| 变体            | JSON 形式                                          | 说明                               |
|---------------|--------------------------------------------------|----------------------------------|
| `Text`        | `{"text": "Hello"}`                              | 纯文本                              |
| `Translate`   | `{"translate": "chat.type.text", "with": [...]}` | 客户端翻译                            |
| `EntityNames` | `{"selector": "@a"}`                             | 实体选择器结果                          |
| `Keybind`     | `{"keybind": "key.forward"}`                     | 按键绑定                             |
| `Custom`      | ❌ 不序列化                                           | 服务端自定义翻译，序列化前需 `to_translated()` |

### 注意

`TextContent` 使用 `#[serde(untagged)]` 自动推断变体，但因 `Custom` 被标记为 `#[serde(skip)]`，序列化时不会遇到它。反序列化时，
`Custom` 不被 serde 识别（需要通过 `TextComponent::custom()` 构造）。

---

## 4. Style — 样式

**文件**: `pumpkin-util/src/text/style.rs`

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

### 设计要点

- 所有字段均为 `Option`，`None` 表示"不从父组件继承时使用默认值"
- 所有字段 `skip_serializing_if = "Option::is_none"`
- 提供链式 builder 方法（大部分是 `const fn`）

### Builder 方法

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

---

## 5. Color / NamedColor / RGBColor / ARGBColor — 颜色系统

**文件**: `pumpkin-util/src/text/color.rs`

### Color 枚举

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

### 自定义 Deserialize

```rust
impl<'de> Deserialize<'de> for Color {
    // "reset" → Reset
    // "#RRGGBB" → Rgb(RGBColor)
    // "red", "dark_blue", ... → Named(NamedColor)
}
```

反序列化失败时的错误消息已 i18n（翻译键以 `"pumpkin:text.color."` 为前缀）。

### 方法

```rust
impl Color {
    pub fn console_color(&self, text: &str) -> ColoredString; // ANSI 终端颜色
    pub const fn from_legacy_code(code: char) -> Option<Self>; // §0-§f → Color
    pub fn from_hex_str(hex: &str) -> Option<Self>;             // "FF55AA" → Rgb
}
```

**`console_color` 映射关系**:

| NamedColor      | ANSI 映射              |
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

> ⚠️ Gray 和 DarkGray 都映射到 `bright_black()`，在 ANSI 16 色终端中无法区分。

### NamedColor — 16 种 Minecraft 标准颜色

```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black = 0,       // #000000
    DarkBlue,        // #0000AA
    DarkGreen,       // #00AA00
    DarkAqua,        // #00AAAA
    DarkRed,         // #AA0000
    DarkPurple,      // #AA00AA
    Gold,            // #FFAA00
    Gray,            // #AAAAAA
    DarkGray,        // #555555
    Blue,            // #5555FF
    Green,           // #55FF55
    Aqua,            // #55FFFF
    Red,             // #FF5555
    LightPurple,     // #FF55FF
    Yellow,          // #FFFF55
    White,           // #FFFFFF
}
```

### NamedColor 方法

```rust
impl NamedColor {
    pub const fn to_rgb(&self) -> RGBColor;          // → 对应的 RGB 值
    pub const fn to_legacy_char(&self) -> char;      // → '0'..'f'
}
impl TryFrom<&str> for NamedColor { /* ... */ }      // 从 snake_case 字符串解析
```

### RGBColor

```rust
#[derive(Debug, Deserialize, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
impl RGBColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self;
}
impl Serialize for RGBColor { /* → "#RRGGBB" */ }
```

### ARGBColor

```rust
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
pub struct ARGBColor {
    alpha: u8,   // 注意：私有
    red: u8,     // 注意：私有
    green: u8,   // 注意：私有
    blue: u8,    // 注意：私有
}
impl ARGBColor {
    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self;
}
impl Serialize for ARGBColor { /* → [alpha, red, green, blue] 字节数组 */ }
```

> ⚠️ `ARGBColor` 字段为私有，外部无法读取分量值（可能有意为之）。

### hsv_to_rgb — HSV→RGB 转换

```rust
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8);
```

- `h`: 色相 (0-360°)
- `s`: 饱和度 (0.0-1.0)
- `v`: 明度 (0.0-1.0)

---

## 6. ClickEvent — 点击事件

**文件**: `pumpkin-util/src/text/click.rs`

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

### JSON 示例

| 变体                | JSON                                            |
|-------------------|-------------------------------------------------|
| `OpenUrl`         | `{"action":"open_url","url":"https://..."}`     |
| `RunCommand`      | `{"action":"run_command","command":"/help"}`    |
| `CopyToClipboard` | `{"action":"copy_to_clipboard","value":"text"}` |

---

## 7. HoverEvent — 悬浮事件

**文件**: `pumpkin-util/src/text/hover.rs`

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
        id: Cow<'static, str>,           // 实体类型 (如 "minecraft:pig")
        uuid: Cow<'static, str>,         // UUID 字符串
        name: Option<Vec<TextComponentBase>>,
    },
}
```

### 便捷构造方法

```rust
impl HoverEvent {
    pub fn show_text(text: TextComponent) -> Self;
    pub fn show_entity<P: Into<Cow<'static, str>>>(
        uuid: P,
        kind: P,
        name: Option<TextComponent>,
    ) -> Self;
}
```

---

## 8. Translation 辅助函数

**文件**: `pumpkin-util/src/text/translation.rs`

### reorder_substitutions

```rust
pub fn reorder_substitutions(
    translation: &str,
    with: Vec<TextComponentBase>,
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>);
```

- 解析翻译字符串中的 `%s` 和 `%1$s` 占位符
- 按索引重新排列 `with` 组件，使其与翻译字符串中的占位符顺序一致
- 返回重排后的组件 + 每个占位符的字节范围

### translation_to_pretty

```rust
pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String;
```

- 查询翻译键，替换占位符，返回控制台友好的彩色字符串
- 用于 `TextComponentBase::to_pretty_console()` 中的 `Translate` / `Custom` 变体

### get_translation_text

```rust
pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String;
```

- 与 `translation_to_pretty` 类似，但输出纯文本（无 ANSI 颜色码）
- 用于 `TextComponentBase::get_text()` 和 `to_bedrock_legacy()`

---

# 翻译键命名规范

整个项目使用 **namespace:key** 格式组织翻译键，namespace 通常为 crate 名称或功能模块名。

### 当前命名空间（15 个，1121 keys）

| Namespace     | Keys | 用途                 | 示例键                                                |
|---------------|------|--------------------|----------------------------------------------------|
| `auth`        | 35   | JWT/OIDC 认证消息      | `auth.jwt.failed_read_response`                    |
| `client`      | 8    | 客户端断开和错误消息         | `client.disconnect.error_reading_incoming_packet`  |
| `commands`    | 162  | 命令系统（描述、错误、参数）     | `commands.args.bounded_num.must_not_be_less`       |
| `config`      | 10   | 配置文件加载消息           | `config.load.convert_merged_failed`                |
| `crash`       | 37   | 崩溃报告和标签            | `crash.backtrace_label`                            |
| `debug`       | 127  | 断言、expect、panic 消息 | `debug.expect.loot_table_mutex_not_poisoned`       |
| `inventory`   | 8    | 物品栏和容器消息           | `inventory.furnace_output_slot.on_take_item`       |
| `network`     | 12   | 认证网络 URL           | `network.authentication.mojang_authentication_url` |
| `permissions` | 42   | 权限节点描述             | `permissions.ban.description`                      |
| `plugin`      | 13   | 插件加载和依赖消息          | `plugin.initialization.failed`                     |
| `protocol`    | 20   | 协议验证和错误消息          | `protocol.bedrock.invalid_action_id`               |
| `server`      | 329  | 服务端日志、启动、关闭        | `server.log.starting_server`                       |
| `text`        | 6    | 文本组件颜色解析错误         | `text.color.hex_format_invalid`                    |
| `util`        | 27   | 通用工具消息             | `util.math.expected_2_elements`                    |
| `world`       | 276  | 世界生成、区块、结构         | `world.chunk.anvil.appending_chunk_eof`            |
| `minecraft:`  | -    | Minecraft 原生翻译键    | `minecraft:chat.type.text`                         |

### 翻译文件位置

```
assets/translations/pumpkin/<locale>.json   (128 files, 1121 keys each)
assets/translations/vanilla/en_us_java.json (Minecraft native keys)
```

- 128 个 JSON 文件，每个对应一种 `Locale`
- 扁平的键值结构，如 `"server.log.starting_server": "Starting %s %s Minecraft (Protocol %s)"`
- 全部文件通过 `build.rs` 在编译期嵌入二进制，运行时零磁盘 I/O

---

# 完整使用示例

### 1. 初始化 i18n

```rust
use pumpkin_i18n::{resolve_server_locale, set_server_global_locale};

// 服务端启动时（通常在 main.rs 中）
let config = PumpkinConfig::load();
let locale = resolve_server_locale( & config.advanced.locale.server_global); // "auto" 或 "zh_cn"
set_server_global_locale(locale);
```

### 2. 纯文本日志翻译 (localized_log)

```rust
use pumpkin_util::translation::localized_log;

// 简单日志 — 自动使用 server_global_locale()
let msg = localized_log("server.log.started_accepting_connections");
info!("{}", msg);
// → "Stopped accepting incoming connections" (EnUs)
```

### 3. 格式化日志翻译 (localized_log_format)

```rust
use pumpkin_util::translation::localized_log_format;

// 带参数的日志
let msg = localized_log_format(
"server.log.build_info",
& [os.to_string(), arch.to_string(), debug_flag.to_string()],
);
info!("{}", msg);
// → "Build info: FAMILY: \"unix\", OS: \"linux\", ARCH: \"x86_64\", BUILD: \"Debug\""
```

### 4. 带染色的启动横幅 (localized_text)

```rust
use pumpkin_util::translation::localized_text;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

// ✅ 正确 — 使用 localized_text，子组件保留染色
let msg = localized_text(
"server.log.starting_server",  // 翻译模板: "Starting %s %s Minecraft (Protocol %s)"
[
TextComponent::text("Pumpkin").color_named(NamedColor::Gold),
TextComponent::text(CARGO_PKG_VERSION).color_named(NamedColor::Green),
TextComponent::text(protocol_version).color_named(NamedColor::DarkBlue),
],
);
info!("{}", msg.to_pretty_console());
// → "Starting \x1b[33mPumpkin\x1b[0m \x1b[32m0.1.0-dev\x1b[0m Minecraft (Protocol \x1b[34m766\x1b[0m)"

// ❌ 错误 — 不要把 .to_pretty_console() 传入 localized_log_format
// localized_log_format("server.log.starting_server", &[
//     TextComponent::text("Pumpkin").color_named(NamedColor::Gold).to_pretty_console(),
// ]); // ANSI 码会被嵌套破坏！
```

### 5. 构建并发送聊天消息

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

// NBT 序列化发送给客户端
let bytes: Box<[u8] > = msg.encode();
```

### 6. 控制台日志（彩色）

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

let msg = TextComponent::text("Server started!")
.color_named(NamedColor::Green)
.bold();

println!("{}", msg.to_pretty_console());
```

### 7. 客户端语言缓存

```rust
use pumpkin_i18n::{set_player_locale, player_locale, remove_player_locale};

// 玩家登录
let locale = set_player_locale(
"550e8400-e29b-41d4-a716-446655440000", // UUID
"zh_cn",                                 // 客户端上报
"auto",                                  // 服务端配置
);

// 翻译时获取
let locale = player_locale("550e8400-e29b-41d4-a716-446655440000");

// 玩家离开
remove_player_locale("550e8400-e29b-41d4-a716-446655440000");
```

### 8. 文本渐变效果

```rust
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

let msg = TextComponent::text("Welcome to the server!")
.gradient_named( & [NamedColor::Red, NamedColor::Gold, NamedColor::Green]);
```

### 9. 彩虹文字

```rust
use pumpkin_util::text::TextComponent;

let msg = TextComponent::text("RAINBOW TEXT").rainbow();
```

### 10. 富文本 + 事件

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

### 11. 动态添加翻译

```rust
use pumpkin_i18n::{add_translation, add_translation_file, Locale};

// 单条翻译
add_translation("myplugin", "welcome", "欢迎!", Locale::ZhCn);

// 批量加载
add_translation_file(
"myplugin",
r#"{
        "welcome": "欢迎!",
        "goodbye": "再见!",
        "error.not_found": "未找到玩家"
    }"#,
Locale::ZhCn,
);
```

### 12. 使用高级翻译引擎

```rust
use pumpkin_i18n::engine::TranslationEngine;
use std::collections::HashMap;

// 构建引擎
let data: Vec<HashMap<String, String> > = vec![/* 每种语言一个 map */];
let engine = TranslationEngine::build( & data);

// 高频翻译（直接使用引擎，适合极高吞吐量场景）
let resolved = engine.resolve(Locale::EnUs as usize, "pumpkin:welcome");
let mut buf = String::new();
resolved.write_to( & ["Steve".into()], & mut buf);
// buf → "Welcome, Steve"  （若翻译键为 "Welcome, %s"）

// 大多数场景使用表层 API 即可：
use pumpkin_util::translation::localized_log_format;
let msg = localized_log_format("welcome", & ["Steve".to_string()]);
```
