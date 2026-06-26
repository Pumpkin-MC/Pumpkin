# Translation Key Naming Convention

Pumpkin translation keys use a **dot-separated hierarchical naming** structure and must strictly adhere to the following
format.

## General Rules

- Separate hierarchy levels with `.`: `namespace.category.feature.detail`
- Use all lowercase, and connect words with underscores: `commands.pumpkin.stop.error_invalid_args`
- The namespace (`pumpkin:` or `minecraft:`) is appended automatically by the code and **must not appear in translation
  files**.
- The translation files are located at `assets/translations/pumpkin/<locale>.json` (128 locale files, 1121 keys each).
- All locale files **must** have the exact same key set. English (`en_us.json`) is the reference.
- Keys are compared case-insensitively by the engine during lookup.
- The translation engine falls back to `en_us` when a key is absent in the requested locale.

## Format Quick Reference

| Purpose                         | Format                                     | Example                                            |
|---------------------------------|--------------------------------------------|----------------------------------------------------|
| Overall command description     | `commands.<command>.description`           | `commands.pumpkin.description.*`                   |
| Command sub-feature hover       | `commands.<command>.<feature>.hover`       | `commands.pumpkin.stop.hover.*`                    |
| Command sub-feature description | `commands.<command>.<feature>.description` | `commands.pumpkin.stop.description.*`              |
| Specific command output text    | `commands.<command>.<feature>.<detail>`    | `commands.pumpkin.version.response.*`              |
| Error/exception messages        | `commands.<command>.<scenario>.error`      | `commands.pumpkin.load.error_missing_config.*`     |
| URLs and configurable params    | `commands.<command>.<param>`               | `commands.pumpkin.github_api_url.*`                |
| Server log messages             | `server.log.<event>`                       | `server.log.starting_server.*`                     |
| Server startup messages         | `server.startup.<event>`                   | `server.startup.started.*`                         |
| Configuration-related prompts   | `config.<module>.<key>`                    | `config.networking.port_in_use.*`                  |
| General player messages         | `chat.<event>`                             | `chat.player_joined.*`                             |
| Debug/expect messages           | `debug.<category>.<detail>`                | `debug.expect.loot_table_mutex_not_poisoned`       |
| Plugin messages                 | `plugin.<plugin>.<path>`                   | `plugin.myplugin.greeting.*`                       |
| Crash report labels             | `crash.<detail>`                           | `crash.backtrace_label.*`                          |
| World-related messages          | `world.<module>.<event>`                   | `world.chunk.anvil.appending_chunk_eof`            |
| Network/Auth messages           | `network.<module>.<detail>`                | `network.authentication.mojang_authentication_url` |
| Protocol messages               | `protocol.<edition>.<detail>`              | `protocol.bedrock.invalid_action_id`               |
| Inventory messages              | `inventory.<module>.<detail>`              | `inventory.furnace_output_slot.on_take_item`       |

## Hierarchy Breakdown

```
server.log.starting_server
  │     │       └── Specific message identifier
  │     └── Module (log, startup, shutdown, network, …)
  └── Top-level category (server, commands, config, chat, debug, …)
```

## Current Namespaces (1121 keys total)

| Namespace     | Keys | Purpose                                     |
|---------------|------|---------------------------------------------|
| `auth`        | 35   | JWT/OIDC authentication messages            |
| `client`      | 8    | Client disconnect and error messages        |
| `commands`    | 162  | Command system (descriptions, errors, args) |
| `config`      | 10   | Configuration file loading messages         |
| `crash`       | 37   | Crash report generation and labels          |
| `debug`       | 127  | Debug assertions, expects, and panics       |
| `inventory`   | 8    | Inventory and screen handler messages       |
| `network`     | 12   | Networking authentication URLs              |
| `permissions` | 42   | Permission node descriptions                |
| `plugin`      | 13   | Plugin loading and dependency messages      |
| `protocol`    | 20   | Protocol validation and error messages      |
| `server`      | 329  | Server logging, startup, shutdown           |
| `text`        | 6    | Text component color parsing errors         |
| `util`        | 27   | General utility messages                    |
| `world`       | 276  | World generation, chunks, structures        |

## Placeholder Formats

Translation values support these substitution placeholders:

| Format         | Example             | Description                    |
|----------------|---------------------|--------------------------------|
| `%s`           | `"Hello %s"`        | Sequential index (0, 1, 2, …)  |
| `%d`, `%f`     | `"Count: %d"`       | Sequential with type hint      |
| `%1$s`, `%2$d` | `"%2$s → %1$s"`     | Explicit 1-based index         |
| `{}`, `{0}`    | `"{} + {1}"`        | Rust-style sequential/explicit |
| `{name}`       | `"{player} joined"` | Named argument                 |
| `%%`           | `"100%%"`           | Escaped literal `%`            |
| `{{`, `}}`     | `"{{escaped}}"`     | Escaped literal braces         |

> **Prefer `%s`** for consistency with existing keys. The engine precompiles all
> placeholders at load time so there is zero runtime parsing overhead regardless
> of which style you use.

## Example File

```json
{
  "commands.pumpkin.description": "Empowering everyone to host fast \nand efficient Minecraft servers.\n",
  "commands.pumpkin.version.hover": "Click to Copy Version",
  "config.load.creating_root_folder": "Creating new configuration root folder…",
  "server.log.starting_server": "Starting %s %s Minecraft (Protocol %s)",
  "server.log.started_server": "Started server; took %s",
  "debug.expect.loot_table_mutex_not_poisoned": "Loot table mutex should not be poisoned"
}
```

## Adding New Translations

1. Add the new key in English to `en_us.json`. Reference the existing format for the category.
2. Add the corresponding translation (or keep the English placeholder) in all other 127 language files.
3. Ensure all translation files contain the exact same set of keys (`en_us.json` is the canonical source).
4. Keep keys sorted alphabetically within each file for easier maintenance.
5. Rebuild the project — `build.rs` will re-embed all translations into `generated_store.rs`.
6. Use the key in code via `localized_log("category.detail")` for plain text or
   `localized_text("category.detail", [children])` for colored messages.
