# Translation Key Naming Convention

Pumpkin translation keys use a **dot-separated hierarchical naming** structure and must strictly adhere to the following
format.

## General Rules

- Separate hierarchy levels with `.`: `namespace.category.feature.detail`
- Use all lowercase, and connect words with underscores: `commands.pumpkin.stop.error_invalid_args`
- The namespace (`pumpkin:` or `minecraft:`) is appended automatically by the code and **must not appear in translation
  files**.
- The translation files are located at `assets/translations`

## Format Quick Reference

| Purpose                         | Format                                     | Example                                        |
|---------------------------------|--------------------------------------------|------------------------------------------------|
| Overall command description     | `commands.<command>.description`           | `commands.pumpkin.description.*`               |
| Command sub-feature hover       | `commands.<command>.<feature>.hover`       | `commands.pumpkin.stop.hover.*`                |
| Command sub-feature description | `commands.<command>.<feature>.description` | `commands.pumpkin.stop.description.*`          |
| Specific command output text    | `commands.<command>.<feature>.<detail>`    | `commands.pumpkin.version.response.*`          |
| Error/exception messages        | `commands.<command>.<scenario>.error`      | `commands.pumpkin.load.error_missing_config.*` |
| URLs and configurable params    | `commands.<command>.<param>`               | `commands.pumpkin.github_api_url.*`            |
| Server log messages             | `server.<module>.<event>`                  | `server.startup.complete.*`                    |
| Configuration-related prompts   | `config.<module>.<key>`                    | `config.networking.port_in_use.*`              |
| General player messages         | `chat.<event>`                             | `chat.player_joined.*`                         |
| Plugin messages                 | `plugin.<plugin>.<path>`                   | `plugin.myplugin.greeting.*`                   |

## Hierarchy Breakdown

```
commands.pumpkin.version.response
  │        │       │       └── Specific message identifier
  │        │       └── Feature name (e.g., version, stop, help)
  │        └── Command name
  └── Top-level category (commands, config, chat, server)
```

## Example File

```json
{
  "commands.pumpkin.version": "Pumpkin %s\n",
  "commands.pumpkin.description": "Empowering everyone to host fast \nand efficient Minecraft servers.\n",
  "commands.pumpkin.version.hover": "Click to Copy Version",
  "commands.pumpkin.github": "[Github Repository]",
  "commands.pumpkin.github.hover": "Click to open repository."
}
```

## Adding New Translations

1. Add the new key in English to `en_us.json`.
2. Add the corresponding translation (or keep the English placeholder) in all other language files.
3. Ensure all translation files contain the exact same set of keys.
4. Keep keys sorted alphabetically for easier maintenance.
5. Ensure URLs and configurable parameters are also stored as text strings.