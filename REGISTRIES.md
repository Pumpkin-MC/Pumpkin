# Registry migration overview

This document tracks Pumpkin's migration to `pumpkin-registry` on the `data-migration` branch.

## Legend

| Kind | Meaning |
|---|---|
| **Static** | `StaticRegistry<T>`: compile-time entries with a fixed identifier/id mapping. |
| **Frozen** | `FrozenRegistry<T>`: assembled from bootstrap providers and immutable after construction. |
| **Reloadable** | `ReloadableRegistry<T>`: a registry whose frozen backing data can be replaced. |
| **—** | Not represented by `pumpkin-registry` yet. |

For **Ported**:

- **Yes** means the registry is represented by a typed `pumpkin-registry` registry and normal runtime lookup uses it.
- **No** means it still uses the legacy/generated representation.
- The generated client registry-sync tables in `generated/registry.rs` are a separate serialization concern. For an unported registry their current item representation is encoded NBT (`StaticRegistryEntry { data: &'static [u8] }`), not a typed `pumpkin-registry` item.

There are currently **no production `ReloadableRegistry<T>` instances**. The type and builder support exist in `pumpkin-registry`, but no game registry has been migrated to it yet.

## Current registry tree

```text
minecraft:root                                      Frozen<Arc<dyn Registry>>
├── minecraft:dimension_type                       Static<Dimension>
└── minecraft:worldgen                             Frozen<Arc<dyn Registry>>
    ├── minecraft:noise_settings                   Static<GenerationSettings>
    ├── minecraft:structure_set                    Static<StructureSet>
    ├── minecraft:world_preset                     Static<WorldPreset>
    ├── minecraft:biome_source_type                Frozen<BiomeSourceType>
    └── minecraft:chunk_generator_type             Frozen<ChunkGeneratorType>
```

`minecraft:worldgen/biome` is **not** in the typed tree yet. Biomes still use the generated `Biome` data/lookup code and the legacy registry-sync representation.

## Ported `pumpkin-registry` registries

| Registry | Kind | Item type | Ported | Notes |
|---|---|---|---|---|
| `minecraft:root` | Frozen | `Arc<dyn Registry>` | Yes | Root/container registry. Built after bootstrap providers have been registered. |
| `minecraft:dimension_type` | Static | `Dimension` | Yes | Generated vanilla dimension types. |
| `minecraft:worldgen` | Frozen | `Arc<dyn Registry>` | Yes | Container for world-generation registries. |
| `minecraft:worldgen/noise_settings` | Static | `GenerationSettings` | Yes | Noise-generation settings, including the noise router. |
| `minecraft:worldgen/structure_set` | Static | `StructureSet` | Yes | Structure sets and their placements. Runtime generation resolves these through the registry. |
| `minecraft:worldgen/world_preset` | Static | `WorldPreset` | Yes | Generated vanilla world presets and generator-settings metadata. |
| `minecraft:worldgen/biome_source_type` | Frozen | `BiomeSourceType` | Yes | Extensible decoder/dispatch registry (`fixed`, `multi_noise`, `the_end`, ...). |
| `minecraft:worldgen/chunk_generator_type` | Frozen | `ChunkGeneratorType` | Yes | Extensible decoder/dispatch registry (`noise`, `flat`, ...). |

### Why the extension registries are Frozen

`BiomeSourceType` and `ChunkGeneratorType` are assembled from bootstrap providers instead of generated as one fixed static slice. This leaves room for plugins to contribute additional implementations before the root registry is frozen.

## Vanilla/client registry-sync registries

These are the registries currently present in `crates/pumpkin-data/src/generated/registry.rs`.

For rows marked **No**, the current registry-sync item is an encoded NBT payload (`&'static [u8]`). The **Item type** column gives the typed runtime representation where one has already been migrated; otherwise it shows the current legacy representation.

| Registry | Kind | Item type | Ported | Notes |
|---|---|---|---|---|
| `minecraft:banner_pattern` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:cat_sound_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:cat_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:chat_type` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:chicken_sound_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:chicken_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:cow_sound_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:cow_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:damage_type` | — | Encoded NBT (`StaticRegistryEntryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:dialog` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:dimension_type` | Static | `Dimension` | Yes | Typed registry exists; generated registry-sync payload still exists for protocol serialization. |
| `minecraft:enchantment` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:frog_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:instrument` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:jukebox_song` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:painting_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:pig_sound_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:pig_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:sulfur_cube_archetype` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:test_environment` | — | Encoded NBT (`StaticRegistryEntryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:test_instance` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:timeline` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:trim_material` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:trim_pattern` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:wolf_sound_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:wolf_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:world_clock` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |
| `minecraft:worldgen/biome` | — | Generated `Biome` / encoded NBT for sync | No | Biome data is strongly typed in generated code, but lookup has not been moved to `pumpkin-registry`. |
| `minecraft:zombie_nautilus_variant` | — | Encoded NBT (`StaticRegistryEntry`) | No | Legacy registry-sync table only. |

## Typed worldgen registries not present in the registry-sync table

These registries are Pumpkin runtime/worldgen registries and do not correspond to rows in the generated client registry-sync table above.

|Registry | Kind | Item type | Ported |
|---|---|---|---|
| `minecraft:worldgen/noise_settings` | Static | `GenerationSettings` | Yes |
| `minecraft:worldgen/structure_set` | Static | `StructureSet` | Yes |
| `minecraft:worldgen/world_preset` | Static | `WorldPreset` | Yes |
| `minecraft:worldgen/biome_source_type` | Frozen | `BiomeSourceType` | Yes |
| `minecraft:worldgen/chunk_generator_type` | Frozen | `ChunkGeneratorType` | Yes |

## Migration notes

- Prefer `DataKey<T>`/typed registry lookup over generated `from_name`, direct constant lookup, or bespoke registry maps when migrating a registry.
- Vanilla data that is immutable for the life of the process is a good fit for **Static** when it can be emitted directly by codegen.
- Registries whose entries are contributed by bootstrap providers/plugins are a good fit for **Frozen**.
- Datapack-backed registries that must change during a reload are expected to use **Reloadable**, but none have been wired up yet.
- The legacy `StaticRegistry`/`StaticRegistryEntry` names in `pumpkin-data::generated::registry` are protocol registry-sync data structures. They are **not** `pumpkin_registry::StaticRegistry<T>` and should not be confused with the new registry kind.
