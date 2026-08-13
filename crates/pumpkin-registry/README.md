# pumpkin-registry

Registry infrastructure for Pumpkin.

This crate provides the registry types used to store and resolve game data, including immutable, static, and reloadable registries. Registries are keyed by `Identifier`s and can also be nested through the type-erased `Registry` interface.

## Registry types

- `FrozenRegistry<T>` — immutable registry with fast indexed lookups.
- `StaticRegistry<T>` — registry for data that is populated once and then kept for the lifetime of the server.
- `ReloadableRegistry<T>` — registry whose backing `FrozenRegistry` can be replaced, intended for reloadable data such as datapacks.
- `RegistryBuilder<T>` — constructs registries from registered bootstrap providers.
- `DataKey<T>` — resolves a typed value through a path of nested registries.

All registry values must be `Send + Sync + 'static`.

## Bootstrap providers

Builtin registry contents can be declared with `bootstrap_provider!`. Providers are discovered and collected automatically by the linker.

```rust
use pumpkin_registry::bootstrap_provider;

#[derive(Debug)]
struct Block(u32);

bootstrap_provider! {
    BLOCKS: Block => "minecraft:block" => {
        "minecraft:stone" => Block(1),
        "minecraft:dirt" => Block(2),
    }
}
```

A provider can also use a function or closure returning `Vec<RegistryEntry<T>>` when entries need to be produced programmatically.

Registries are then be built from all providers targeting the same registry identifier:

```rust
use pumpkin_registry::{BOOTSTRAP, RegistryBuilder};
use pumpkin_registry::bootstrap::BootstrapManager;
use pumpkin_util::identifier::Identifier;

let _ = BOOTSTRAP.set(BootstrapManager::new());

let blocks = RegistryBuilder::<Block>::frozen(
    &Identifier::parse_static("minecraft:block"),
)?;
```

Duplicate identifiers are rejected during bootstrap.

## Nested registry access

`DataKey<T>` represents a path through one or more registries. For example, a key such as:

```text
minecraft:root/minecraft:blocks/minecraft:stone
```

walks the registry tree and resolves the final value as `T`. Resolution validates each registry and value type along the way and returns a `DataKeyGetError` when a registry, identifier, value, or expected type does not match.

## Global registries

The crate exposes two global initialization points:

- `BOOTSTRAP` — the global `BootstrapManager`.
- `ROOT` — the root `FrozenRegistry<Arc<dyn Registry>>` used for nested registry lookup.

Both use `OnceLock` and are expected to be initialized during server startup.
