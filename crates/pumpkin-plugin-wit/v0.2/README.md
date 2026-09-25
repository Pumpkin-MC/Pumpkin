# `pumpkin:plugin@0.2.0` ABI

This directory defines the `plugin` world for the stackless async ABI. It is a
separate WIT package from `pumpkin:plugin@0.1.0`, so the host and plugin must
use bindings for the same package version. At load time, the host selects the
ABI from the component's versioned WIT imports and exports, not from
`metadata.version` (which is the plugin's own version). Unknown or mixed
package versions must be rejected. v0.1 plugins keep using the v0.1 world and
its synchronous behavior.

All 14 callbacks in the `plugin` world are `async func`: initialization,
load/unload, events, commands and suggestions, tasks, IPC, AI goals, and chunk
generation. A callback may await host imports; it completes when it returns
its result. `handle-event` takes and returns an owned event value. For a
`blocking` handler, the host applies changes from the returned value after the
callback completes. A nonblocking handler's returned event is ignored. The
host must not lend mutable event state across an awaited import.

The package declares 955 async interface functions and three sync functions:
the metadata export and two UUID conversions. The host must not block the
guest executor while an async call waits for I/O, its owning domain, or another
plugin. Resource handles stay opaque. The host routes calls by handle ownership;
physical domains and scheduler IDs do not appear in WIT. The six explicit
`borrow<T>` parameters in v0.1 now take owned `T`
values. Four resource constructors became async static `create` functions
returning the resource because WIT has no async constructor form. Guest and
host bindings need to be regenerated for these changes.

## Function classification

The table covers every declared function, including interfaces used only for
type reuse by the current world. Counts include resource methods and static
functions. An async function may read host state, perform I/O, wait for its
resource's owning domain, or reenter a plugin. Even a simple resource getter
may need that routing, so implementations cannot assume it finishes in the
same call stack. The three sync functions have a run-to-completion contract.

| Interface or world | Async | Sync | Reason |
| --- | ---: | ---: | --- |
| `plugin` callbacks | 14 | 0 | Lifecycle and callback entrypoints can call the host. |
| `block-entity` | 173 | 0 | World-owned block entity state. |
| `boss-bar` | 15 | 0 | Mutable display state and viewers. |
| `command` | 28 | 0 | Command graph, sender state, feedback, and permission checks. |
| `context` | 6 | 0 | Registrations and server context. |
| `datapack` | 9 | 0 | Datapack registry and data access. |
| `display` | 67 | 0 | Entity-owned display state. |
| `enchantments` | 4 | 0 | Registry access and registration. |
| `gui` | 12 | 0 | Inventory and screen state. |
| `i18n` | 2 | 0 | Translation storage and loading. |
| `inventory` | 28 | 0 | Player or container-owned mutable state. |
| `ipc` | 1 | 0 | Cross-plugin request and response. |
| `item-stack` | 29 | 0 | Mutable item state and registry lookup. |
| `logging` | 2 | 0 | Log sinks may perform I/O. |
| `metadata` | 0 | 1 | `get-metadata` returns an immutable guest-owned snapshot and must not call host imports or perform I/O. |
| `player` | 191 | 0 | Player-owned state, packets, and permission checks. |
| `recipe` | 3 | 0 | Recipe registration. |
| `scheduler` | 3 | 0 | Task registration and cancellation. |
| `scoreboard` | 28 | 0 | Mutable scoreboard state. |
| `server` | 64 | 0 | Global state, I/O, world management, and command dispatch. |
| `text` | 37 | 0 | Opaque resource methods may cross a host boundary. |
| `uuid` | 1 | 2 | `generate` obtains randomness; `parse` and `to-string` only convert owned data in memory. |
| `%world` | 252 | 0 | World, entity, chunk, and registry operations. |

These interfaces define types but no functions: `advancement`, `attributes`,
`bedrock-packets`, `biomes`, `common`, `damage-types`, `data-components`,
`entity`, `entity-statuses`, `entity-types`, `event`, `forms`, `game-events`,
`game-rules`, `java-dialogs`, `java-packets`, `particles`, `permission`,
`potions`, `screens`, `sounds`, `statistics`, and `status-effect`.
