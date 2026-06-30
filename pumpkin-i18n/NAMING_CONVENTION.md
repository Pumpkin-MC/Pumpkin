# 🏷️ Pumpkin Translation Key Naming Convention

> **Last Updated**: 2026-06-30
> **Rust Edition**: 2024 | **MSRV**: 1.95
>
> Related docs: [Translation System Flow](./TRANSLATION_SYSTEM_FLOW.md) | [API Reference](./API_REFERENCE.md)

---

## Table of Contents

- [1. General Rules](#1-general-rules)
- [2. Hierarchy Structure](#2-hierarchy-structure)
- [3. Format Quick Reference](#3-format-quick-reference)
- [4. Current Namespaces](#4-current-namespaces)
- [5. Placeholder Formats](#5-placeholder-formats)
- [6. Translation File Structure](#6-translation-file-structure)
- [7. Adding New Translations](#7-adding-new-translations)

---

## 1. General Rules

Pumpkin translation keys use a **dot-separated hierarchical naming** structure and must strictly adhere to the following
format:

- Separate hierarchy levels with `.`: `namespace.category.feature.detail`
- Use all lowercase, and connect words with underscores: `commands.pumpkin.stop.error_invalid_args`
- The namespace (`pumpkin:` or `minecraft:`) is appended automatically by the code and **must not appear in translation
  files**
- All locale files **must** have the exact same key set. English (`en_us.json`) is the canonical reference
- Keys are compared case-insensitively by the engine during lookup
- The translation engine falls back to `en_us` when a key is absent in the requested locale

### Namespace Overview

The project uses two categories of namespaces:

| Namespace Prefix     | Source                         | Description                                                                                   |
|----------------------|--------------------------------|-----------------------------------------------------------------------------------------------|
| `pumpkin:`           | `assets/translations/pumpkin/` | 1122 server-side translation keys covering all modules; auto-prepended by code                |
| `java_minecraft:`    | `assets/translations/vanilla/` | Minecraft Java Edition native translation keys (e.g. `chat.type.text`); injected on download  |
| `bedrock_minecraft:` | `assets/translations/vanilla/` | Minecraft Bedrock Edition native translation keys; en_us only, embedded at compile time       |
| `minecraft:`         | Client-side only               | `TextComponent::translate()` client translations; does not go through the pumpkin-i18n engine |

---

## 2. Hierarchy Structure

```
server.log.starting_server
  │     │       └── Specific message identifier
  │     └── Module (log, startup, shutdown, network, ...)
  └── Top-level category (server, commands, config, chat, debug, ...)
```

**Level descriptions**:

- **Level 1**: System/domain — `server`, `commands`, `config`, `world`, etc.
- **Level 2**: Module/feature — `log`, `startup`, `chunk`, `network`, etc.
- **Level 3+**: Specific message identifier, succinctly describing message semantics

---

## 3. Format Quick Reference

| Purpose                      | Format                                     | Example                                            |
|------------------------------|--------------------------------------------|----------------------------------------------------|
| Overall command description  | `commands.<command>.description`           | `commands.pumpkin.description`                     |
| Sub-command hover tooltip    | `commands.<command>.<feature>.hover`       | `commands.pumpkin.stop.hover`                      |
| Sub-command description      | `commands.<command>.<feature>.description` | `commands.pumpkin.stop.description`                |
| Specific command output text | `commands.<command>.<feature>.<detail>`    | `commands.pumpkin.version.response`                |
| Error/exception messages     | `commands.<command>.<scenario>.error`      | `commands.pumpkin.load.error_missing_config`       |
| URLs and configurable params | `commands.<command>.<param>`               | `commands.pumpkin.github_api_url`                  |
| Server log messages          | `server.log.<event>`                       | `server.log.starting_server`                       |
| Server startup messages      | `server.startup.<event>`                   | `server.startup.started`                           |
| Config-related prompts       | `config.<module>.<key>`                    | `config.networking.port_in_use`                    |
| General player messages      | `chat.<event>`                             | `chat.player_joined`                               |
| Debug/expect messages        | `debug.<category>.<detail>`                | `debug.expect.loot_table_mutex_not_poisoned`       |
| Plugin messages              | `plugin.<category>.<detail>`               | `plugin.initialization.failed`                     |
| Crash report labels          | `crash.<detail>`                           | `crash.backtrace_label`                            |
| World-related messages       | `world.<module>.<event>`                   | `world.chunk.anvil.appending_chunk_eof`            |
| Network/Auth messages        | `network.<module>.<detail>`                | `network.authentication.mojang_authentication_url` |
| Protocol messages            | `protocol.<edition>.<detail>`              | `protocol.bedrock.invalid_action_id`               |
| Proxy/reverse-proxy messages | `proxy.<proxy_type>.<detail>`              | `proxy.velocity.unsupported_forward_version`       |
| Inventory messages           | `inventory.<module>.<detail>`              | `inventory.furnace_output_slot.on_take_item`       |

---

## 4. Current Namespaces

**1122 translation keys** total, distributed across 16 namespaces:

| Namespace     | Keys | Purpose                                     |
|---------------|------|---------------------------------------------|
| `server`      | 330  | Server logging, startup, shutdown           |
| `world`       | 276  | World generation, chunks, structures        |
| `commands`    | 162  | Command system (descriptions, errors, args) |
| `debug`       | 127  | Debug assertions, expects, and panics       |
| `permissions` | 42   | Permission node descriptions                |
| `crash`       | 37   | Crash report generation and labels          |
| `auth`        | 35   | JWT/OIDC authentication messages            |
| `util`        | 27   | General utility messages                    |
| `protocol`    | 20   | Protocol validation and error messages      |
| `plugin`      | 13   | Plugin loading and dependency messages      |
| `network`     | 12   | Networking authentication URLs              |
| `config`      | 10   | Configuration file loading messages         |
| `proxy`       | 9    | Proxy/reverse-proxy messages (Velocity)     |
| `client`      | 8    | Client disconnect and error messages        |
| `inventory`   | 8    | Inventory and screen handler messages       |
| `text`        | 6    | Text component color parsing errors         |

---

## 5. Placeholder Formats

Translation values support the following substitution placeholders:

| Format               | Example             | Description                                         |
|----------------------|---------------------|-----------------------------------------------------|
| `%s`                 | `"Hello %s"`        | Sequential index (0, 1, 2, ...)                     |
| `%d`, `%f`           | `"Count: %d"`       | Sequential with type hint                           |
| `%1$s`, `%2$d`       | `"%2$s → %1$s"`     | Explicit 1-based index (allows argument reordering) |
| `{}`, `{:?}`         | `"{} + {}"`         | Rust-style sequential (Debug: `{:?}`)               |
| `{0}`, `{0:?}`       | `"{1} → {0}"`       | Rust-style explicit 0-based index                   |
| `{name}`, `{name:?}` | `"{player} joined"` | Named argument (numbered by first appearance)       |
| `%%`                 | `"100%%"`           | Escaped literal `%`                                 |
| `{{`, `}}`           | `"{{escaped}}"`     | Escaped literal braces                              |

> **Prefer `%s`** for consistency with existing keys. The engine precompiles all placeholders at load time so there is
> zero runtime parsing overhead regardless of which style you use.

### Placeholder-to-Argument Mapping

Placeholder positions in the translation string correspond to argument array indices:

```
Translation template: "Starting %s %s Minecraft (Protocol %s)"
Argument array:       ["Pumpkin", "0.1.0-dev", "766"]
                        → Var(0)  → Var(1)       → Var(2)

Output: "Starting Pumpkin 0.1.0-dev Minecraft (Protocol 766)"
```

For `localized_text`, arguments are `TextComponent` arrays that retain original color and style after substitution. For
`localized_log_format`, arguments are plain strings.

---

## 6. Translation File Structure

```
assets/translations/
├── pumpkin/                       ← Pumpkin server translations (128 files)
│   ├── en_us.json                 ← Canonical reference file (embedded in binary at compile time)
│   ├── zh_cn.json                 ← Downloaded on demand at runtime
│   ├── ja_jp.json
│   ├── de_de.json
│   └── ...                        ← 128 locale files total
│
└── vanilla/                       ← Minecraft native translations
    ├── en_us_java.json            ← Java Edition (embedded at compile time)
    ├── en_us_bedrock.lang         ← Bedrock Edition (embedded at compile time; the only embedded Bedrock file)
    └── ...                        ← Other languages downloaded at runtime
```

**Key Constraints**:

- All 128 `pumpkin/*.json` files **must** have the exact same set of keys
- `en_us.json` is the canonical reference — always add new keys here first
- Keys are sorted **alphabetically** within each file for easier maintenance
- Only `en_us.json` is embedded in the binary at compile time; the remaining 127 languages are downloaded on demand at
  runtime

### JSON File Format

```json
{
  "commands.pumpkin.description": "Empowering everyone to host fast \nand efficient Minecraft servers.\n",
  "commands.pumpkin.version.hover": "Click to Copy Version",
   "config.load.creating_root_folder": "Creating new configuration root folder...",
  "server.log.starting_server": "Starting %s %s Minecraft (Protocol %s)",
  "server.log.started_server": "Started server; took %s",
  "debug.expect.loot_table_mutex_not_poisoned": "Loot table mutex should not be poisoned"
}
```

- Flat key-value structure (no nesting)
- Values may contain `\n` newlines
- Values may contain any of the supported placeholder formats above

---

## 7. Adding New Translations

Complete workflow for adding a new translation key:

1. **Add English key**: Insert the new key-value pair in alphabetical order in `en_us.json`. Follow the naming format of
   existing keys in the same category.
2. **Sync all locale files**: Add the new key (an English placeholder value is acceptable) to all 127 other language
   files.
3. **Verify consistency**: Ensure all translation files contain the exact same set of keys; `en_us.json` is the
   canonical source.
4. **Rebuild**: `cargo build` — `build.rs` will re-embed all translations into `generated_store.rs`.
5. **Use in code**:
   - Plain text log: `localized_log("category.detail")`
   - Formatted log: `localized_log_format("category.detail", &[arg1, arg2])`
   - Colored message: `localized_text("category.detail", [child_components])`
   - Specified locale: `translate_plain("category.detail", locale)`

> For detailed API usage, see [API Reference](./API_REFERENCE.md); for runtime mechanisms,
> see [Translation System Flow](./TRANSLATION_SYSTEM_FLOW.md).
