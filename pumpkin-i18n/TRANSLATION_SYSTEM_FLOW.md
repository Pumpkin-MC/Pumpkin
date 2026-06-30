# 🌐 Pumpkin 翻译系统完整运作流程

---

## 一、编译流程（Build Time）

### 1.1 文件内嵌

`pumpkin-i18n/build.rs` 在编译时执行，**仅嵌入 `en_us` 英语翻译**到最终二进制文件中，禁止其他语言内嵌。

```
cargo build
  │
  └─► pumpkin-i18n/build.rs
        │
        ├─ 扫描 assets/translations/pumpkin/*.json（128 个文件）
        │   └─► 生成 generated_locale_codes.rs
        │        fn locale_code(idx: usize) -> &'static str
        │        枚举序号 → 语言代码字符串映射
        │        例: AfZa → "af_za", ZhCn → "zh_cn"
        │
        └─ 仅嵌入 3 个 en_us 文件 → generated_store.rs
              │
              ├─ pumpkin/en_us.json
              │   → namespace "pumpkin:" 前缀
              │   → 例: "pumpkin:server.log.starting_server"
              │
              ├─ vanilla/en_us_java.json
              │   → namespace "java_minecraft:" 前缀
              │
              └─ vanilla/en_us_bedrock.lang
                  → namespace "bedrock_minecraft:" 前缀
                  → key=value 逐行解析，key 小写化
```

**关键设计**：编译期只嵌入 `en_us`（英语）三种翻译文件。其余 127 种语言在运行时按需下载。`assets/translations/`
目录中的其他语言文件仅作为远程镜像的源数据，不进入二进制。

```rust
// generated_store.rs 输出结构
pub(crate) fn load_all_translations()
    -> [HashMap<String, String>; Locale::COUNT]  // 128 个槽位
{
    let mut array: [HashMap; 128] = std::array::from_fn(|_| HashMap::new());

    // ✅ EnUs 槽位: 注入 pumpkin: + java_minecraft: + bedrock_minecraft: 条目
    // ❌ 其余 127 个槽位: 空 HashMap，运行时动态填充
}
```

---

## 二、启动流程（Server Startup）

### 2.1 解析服务端 Locale

```
resolve_server_locale(config_value)
  │
  ├─ "auto" → detect_system_locale()
  │   ├─ Linux:   读取 LANG / LC_ALL / LC_MESSAGES 环境变量
  │   ├─ Windows: GetUserDefaultLocaleName() API
  │   └─ 失败:    EnUs 回退
  │
  └─ "zh-CN" → parse_locale_value("zh-CN") → Locale::ZhCn

↓
set_server_global_locale(locale)  // 存入 OnceLock<Locale>
```

### 2.2 翻译文件下载

启动后优先加载本地 `data/translation/{locale}/` 目录内的缓存文件，缺失时从远程镜像下载。

```
spawn_blocking {  // 不阻塞 tokio runtime

  ┌─ Step 1: 尝试磁盘缓存 ─────────────────────────────┐
  │  load_cached_translations(locale, cache_root)       │
  │    → 读 data/translation/zh_cn/pumpkin.json         │
  │    → 读 data/translation/zh_cn/java_minecraft.json  │
  │    → 读 data/translation/zh_cn/bedrock_minecraft.json│
  │    → 全部存在 → 直接返回 ✅ (跳过下载)               │
  └────────────────────────────────────────────────────┘
                        ↓ 缓存未命中
  ┌─ Step 2: 远程下载 ─────────────────────────────────┐
  │  download_locale(&config, locale)                   │
  │    → GET {mirror}/pumpkin/zh_cn.json                │
  │    → GET {mirror}/vanilla/zh_cn_java.json           │
  │    → GET {mirror}/vanilla/zh_cn_bedrock.lang        │
  │                                                     │
  │  每个文件独立下载，部分失败可容忍                    │
  │  SHA256 哈希校验 (.sha256 文件):                    │
  │    ├─ 校验文件存在 + 哈希匹配 → 接受 ✅              │
  │    ├─ 校验文件存在 + 哈希不匹配 → 拒绝 ❌            │
  │    └─ 校验文件不存在 → 降级接受 ⚠️                  │
  └────────────────────────────────────────────────────┘
                        ↓
  ┌─ Step 3: 保存到磁盘 ───────────────────────────────┐
  │  save_downloaded_translations(downloaded, locale,   │
  │                               cache_root)           │
  │    → 创建 data/translation/zh_cn/ 目录              │
  │    → 写入 pumpkin.json                              │
  │    → 写入 java_minecraft.json                       │
  │    → 写入 bedrock_minecraft.json                    │
  └────────────────────────────────────────────────────┘
}
```

#### 下载超时处理

每个 HTTP 请求有独立超时（默认 1000ms，可通过 `pumpkin.toml` 配置）：

- **超时/失败** → 该文件的翻译不被加载，释放内嵌的 `en_us` 文件作为回退
- **全部成功** → 三个 namespace 的翻译都加载
- **部分成功** → 成功的 namespace 加载对应翻译，失败的用英语回退

### 2.3 注入翻译引擎

```
load_downloaded(&downloaded, locale)
  │
  ├─ pumpkin 条目   → add_translation_file("pumpkin", json, locale)
  ├─ java 条目      → add_translation_file("java_minecraft", json, locale)
  └─ bedrock 条目   → add_translation("bedrock_minecraft", key, val, locale)
                        (逐条插入)

↓ 翻译引擎内部

TranslationEngine {
    stores: ArcSwap<Box<[FstLocaleStore; 128]>>  // FST 不可变查找
    overrides: Box<[DashMap; 128]>               // 运行时动态注入
    cache: DashMap                               // lock-free 缓存
}
```

### 2.4 初始化后台加载器

```
init_translation_loader(download_config, cache_root)
  → 存储到 LOADER_STATE (OnceLock)
  → 供后续玩家加入时的 `ensure_locale_translations()` 使用
```

---

## 三、玩家加入流程（Player Join）

### 3.1 翻译文件已存在

```
玩家加入 (locale = zh_cn)
  │
  ├─ set_player_locale(uuid, "zh_cn", config)
  │   → 缓存到 PLAYER_CACHE: uuid → Locale::ZhCn
  │
  ├─ ensure_locale_translations(ZhCn)  // 后台 spawn_blocking
  │   ├─ 检查 LOADED_LOCALES 集合 → 已加载 → 跳过
  │   └─ (已由服务端启动时加载)
  │
  └─ 翻译查询直接命中 ZhCn FST → 返回中文翻译 ✅
```

### 3.2 翻译文件不存在（首次加入的新语言）

```
玩家加入 (locale = ja_jp) — 日文翻译未下载
  │
  ├─ set_player_locale(uuid, "ja_jp", config)
  │   → 缓存到 PLAYER_CACHE
  │
  ├─ ensure_locale_translations(JaJp)  // 后台 spawn_blocking
  │   ├─ 检查 LOADED_LOCALES → 未加载 → 标记为加载中
  │   ├─ load_cached_translations(JaJp, cache_root) → 磁盘无缓存
  │   ├─ download_locale(&config, JaJp) → 从镜像下载日文翻译
  │   ├─ save_downloaded_translations(...) → 保存到磁盘
  │   └─ load_downloaded(...) → 注入引擎 + 清除 DashMap 缓存
  │
  └─ 玩家立即正常加入，不等待下载完成
      翻译查询走三级回退：
        Tier 1: JaJp FST → 未命中
        Tier 2: EnUs FST → 命中 ✅ → 显示英语文本
        Tier 3: 原始 key → 兜底
      
      后台下载完成后：
        → engine.add_translations() 清除缓存
        → 下一次查询命中 JaJp FST → 显示日文翻译 ✅
```

### 3.3 三级回退查找

```
resolve_translation(key="pumpkin:welcome.back", locale=JaJp)
  │
  ├─ Tier 1: JaJp override map  ──── 未命中
  ├─ Tier 1: JaJp FST 查找       ──── 未命中 (未下载)
  │
  ├─ Tier 2: EnUs override map   ──── 未命中
  ├─ Tier 2: EnUs FST 查找       ──── 命中 → warn! + 返回英语 ✅
  │
  └─ Tier 3: 原始 key            ──── error! + 返回 "pumpkin:welcome.back"
```

---

## 四、磁盘缓存结构

```
{exec_dir}/                         ← 程序工作目录
└── data/                           ← 可配置 (translation_cache_dir)
    └── translation/
        ├── en_us/                  ← en_us 仅作为缓存副本（编译期已内嵌，无需下载）
        │   ├── pumpkin.json
        │   ├── java_minecraft.json
        │   └── bedrock_minecraft.json
        ├── zh_cn/                  ← 服务端启动时加载
        │   ├── pumpkin.json
        │   ├── java_minecraft.json
        │   └── bedrock_minecraft.json
        └── ja_jp/                  ← 首位日本玩家加入时后台下载
            ├── pumpkin.json
            ├── java_minecraft.json
            └── bedrock_minecraft.json
```

---

## 五、配置项参考 (`pumpkin.toml`)

```toml
[advanced.locale]
# 服务端日志/控制台语言 ("auto" = 跟随系统)
server_global = "auto"

# Java 版玩家语言解析策略
client_java_edition = "auto"

# Bedrock 版玩家语言解析策略
client_bedrock_edition = "auto"

# 翻译镜像 URL（空 = 使用默认 GitHub 镜像）
mirror_url = ""

# 单次 HTTP 请求超时 (ms)
timeout = 1000

# 跳过 SHA256 校验
skip_checksum = false

# 翻译缓存目录（相对路径基于工作目录，绝对路径直接使用）
translation_cache_dir = "data/translation"
```

---

## 六、数据流全景图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           编译期 (Build Time)                            │
│                                                                         │
│  build.rs                                                               │
│    │ assets/translations/pumpkin/en_us.json ───── include_str! ─────┐   │
│    │ assets/translations/vanilla/en_us_java.json ── include_str! ─┐ │   │
│    │ assets/translations/vanilla/en_us_bedrock.lang ─ include_str! │ │   │
│    ▼                                                                ▼ ▼   │
│  generated_store.rs: load_all_translations() → [HashMap; 128]           │
│  (仅 EnUs 槽位有数据)                                                    │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │ LazyLock 延迟初始化
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          启动期 (Startup)                                │
│                                                                         │
│  main.rs                                                                │
│    ├─ resolve_server_locale() ─→ set_server_global_locale()             │
│    ├─ [disk] load_cached_translations(server_locale)                    │
│    │    └─ 命中 → 跳过下载                                               │
│    ├─ [HTTP] download_locale(server_locale)                             │
│    │    └─ SHA256 校验 → 接受/拒绝                                       │
│    ├─ [disk] save_downloaded_translations(server_locale)                │
│    ├─ [engine] load_downloaded() → TranslationEngine                    │
│    └─ init_translation_loader(config, cache_root)                       │
│         └─ 存储到 LOADER_STATE 供后台使用                                │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    玩家加入期 (Player Join)                               │
│                                                                         │
│  lib.rs / play.rs                                                       │
│    ├─ set_player_locale(uuid, reported_locale, config)                  │
│    │    └─ PLAYER_CACHE: {uuid → Locale}                                │
│    │                                                                    │
│    └─ spawn_blocking { ensure_locale_translations(player_locale) }     │
│         ├─ LOADED_LOCALES 去重检查                                       │
│         ├─ [disk] load_cached_translations()                            │
│         ├─ [HTTP] download_locale()                                     │
│         ├─ [disk] save_downloaded_translations()                        │
│         └─ [engine] load_downloaded() → 清除缓存                         │
│                                                                         │
│  玩家不等待下载 → 立即加入 → 使用英语回退                                 │
│  下载完成后 → 下次查询自动使用目标语言                                     │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        运行时查询 (Hot Path)                              │
│                                                                         │
│  localized_log("server.log.starting")                                   │
│    → "pumpkin:server.log.starting"                                      │
│    → translation_engine().resolve(locale, key)                          │
│                                                                         │
│    查询链路:                                                             │
│      DashMap 缓存 (lock-free, ~99% 命中)                                │
│        → FST (Finite State Transducer) O(log n)                        │
│          → 三级回退: 目标locale → EnUs → raw key                        │
│            → 预编译 Token 流 (零解析开销)                                │
│                                                                         │
│    格式化:                                                               │
│      ResolvedTranslation::Tokenized {                                   │
│        tokens: [Text("Hello "), Var(0), Text("!")]                     │
│      }                                                                  │
│      format_tokens(tokens, &["World"]) → "Hello World!"                │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 七、关键模块清单

| 文件                                | 职责                                                                          |
|-----------------------------------|-----------------------------------------------------------------------------|
| `pumpkin-i18n/build.rs`           | 编译期嵌入 en_us，生成 locale_code 映射                                               |
| `pumpkin-i18n/src/locale.rs`      | 128 变体 Locale 枚举，from_str/to_code/normalize                                 |
| `pumpkin-i18n/src/download.rs`    | HTTP 下载、SHA256 校验、磁盘缓存、后台加载器、`mark_locale_loaded` 去重                        |
| `pumpkin-i18n/src/engine.rs`      | FST 构建/查找、DashMap 缓存、预编译 Token、`value_or_raw` 回退                            |
| `pumpkin-i18n/src/store.rs`       | 全局 TRANSLATIONS、translation_engine、动态注入 API                                 |
| `pumpkin-i18n/src/token.rs`       | `%s` / `{name}` 占位符解析、预编译 TokenStream                                       |
| `pumpkin-i18n/src/client.rs`      | 玩家 UUID→Locale 缓存、client locale 解析                                          |
| `pumpkin-i18n/src/server.rs`      | 服务端 locale 全局状态、系统语言检测                                                      |
| `pumpkin-i18n/src/lib.rs`         | 模块声明、公共 API 导出、`PUMPKIN_NAMESPACE`、`pumpkin_translation_key`                |
| `pumpkin-config/src/locale.rs`    | 用户可配置的 TOML locale 设置                                                       |
| `pumpkin-util/src/translation.rs` | `translate_plain` / `translate_format` / `localized_log` / `localized_text` |
| `pumpkin/src/main.rs`             | 启动流程编排：下载→加载→初始化后台加载器                                                       |
| `pumpkin/src/lib.rs`              | 玩家登录时触发后台翻译下载                                                               |
| `pumpkin/src/net/java/play.rs`    | 设置变更时同步 locale + 触发下载                                                       |
