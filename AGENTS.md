# AI Agent Development Guidelines

> Behavioral guidelines for AI agents working on the Pumpkin project. Agents must follow this document when contributing.
> Also must comply with [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 1. Submitting PRs

- PR title format: `<type>: <short description>` (e.g. `feat(auth): Add multi-Yggdrasil authentication support`)
- Append `🤖🤖🤖` to the PR title to enter the AI agent fast-merge pipeline.
- **Search for existing similar PRs** before submitting. Don't resubmit nearly identical implementations.
- **One PR, one thing** — don't mix unrelated changes together.

---

## 2. Read Before You Code

1. Prefer existing implementations in the project; avoid reinventing the wheel.

2. Before writing any code, find the most similar existing implementation and **read it thoroughly**.

| Task                         | Read these files first                                               |
|------------------------------|----------------------------------------------------------------------|
| Add a block (with entity)    | `campfire.rs` + `entities/campfire.rs` → `registry.rs` registration  |
| Add a block (no entity)      | `dirt_path.rs` → `registry.rs` registration                          |
| Add an item                  | `bucket.rs` → `items/mod.rs` `default_registry()`                    |
| Add a mob AI Goal            | `melee_attack.rs` → how goals are attached in entity                 |
| Add a command                | `time.rs` → `commands/mod.rs` registration                           |
| Add packet handling          | `net/java/play.rs` (find a similar packet's handler)                 |

Then trace the registration chain: struct definition → macro annotation → registry registration → mod.rs declaration.

---

## 3. Code Templates

These templates are distilled from the project's **actual code** — illustrative only; always defer to the real codebase.

### 3.1 Blocks

```rust
// Single block
#[pumpkin_block("minecraft:dirt_path")]
pub struct DirtPathBlock;

// Group of blocks (via tag)
#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl BlockBehaviour for DirtPathBlock {
    fn on_entity_step<'a>(&'a self, args: OnEntityStepArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move { /* ... */ })
    }
    // Use default trait impls for other methods; override only what you need.
    // BlockBehaviour has ~28 methods, most with default implementations.
    // Note: mirror() and rotate() are not async and do not return BlockFuture.
}
```

Registration: add `manager.register(YourBlock);` in `pumpkin/src/block/registry.rs`'s `default_registry()`.

### 3.2 Block Entities

```rust
pub struct CampfireBlockEntity {
    position: BlockPos,
    // custom fields
}

impl BlockEntity for CampfireBlockEntity {
    // The following 5 methods have no default impl — must be hand-written
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { /* serialize to NBT */ })
    }
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self { /* construct from NBT */ }
    fn resource_location(&self) -> &'static str { "minecraft:campfire" }
    fn get_position(&self) -> BlockPos { self.position }
    fn as_any(&self) -> &dyn Any { self }

    // tick has a default empty impl; override only when needed
    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { /* per-tick logic */ })
    }
}
```

Create and register in the parent block's `BlockBehaviour::placed()` via `world.add_block_entity(Arc::new(entity))`.

### 3.3 Items

```rust
pub struct BucketItem;

impl ItemMetadata for BucketItem {
    fn ids() -> Box<[u16]> {
        // Return all item IDs this behaviour covers
        Box::new([Item::BUCKET.id, Item::WATER_BUCKET.id, /* ... */])
    }
}

impl ItemBehaviour for BucketItem {
    fn normal_use<'a>(&'a self, item: &'a Item, player: &'a Player)
        -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    {
        Box::pin(async move { /* ... */ })
    }

    fn use_on_block<'a>(
        &'a self, item: &'a mut ItemStack, player: &'a Player,
        location: BlockPos, face: BlockDirection, cursor_pos: Vector3<f32>,
        block: &'a Block, server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { /* ... */ })
    }
}
```

Registration: `manager.register(YourItem);` in `pumpkin/src/item/items/mod.rs`'s `default_registry()`.

> **Note**: there is no `#[pumpkin_item]` macro. Items implement the `ItemMetadata` trait manually. For why `ItemBehaviour` doesn't use type aliases, see [6.1 Use Project Type Aliases](#61-use-project-type-aliases).

### 3.4 Entities

```rust
pub struct BatEntity {
    entity: Entity,              // base entity (position, velocity, UUID…)
    living_entity: LivingEntity, // living properties (health, effects…)
    mob_entity: Mob,             // mob properties (AI, Goals, navigation…)
  //mob: Arc<dyn MobBase>,       // if a Mob subtype, holds an Arc back-pointer
}

impl NBTStorage for BatEntity { /* NBT read/write */ }

impl EntityBase for BatEntity {
    // The following 3 methods have no default impl — must be hand-written
    fn get_entity(&self) -> &Entity { &self.entity }
    fn get_living_entity(&self) -> Option<&LivingEntity> { Some(&self.living_entity) }
    fn cast_any(&self) -> &dyn std::any::Any { self }

    fn tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, server: &'a Server)
        -> EntityBaseFuture<'a, ()>
    {
        Box::pin(async move {
            self.get_entity().tick(caller, server).await;
            self.living_entity.tick(caller, server).await;
            // ...
        })
    }
}
```

Registration: map by `EntityType` ID in the match arm at `pumpkin/src/entity/type.rs`:
```rust
id if id == EntityType::BAT.id => BatEntity::new(entity),
```

### 3.5 AI Goals

```rust
pub struct MeleeAttackGoal {
    mob: Arc<dyn Mob>,       // holds Mob reference for method calls
    speed: f64,
    cooldown: i32,
    // ...
}

impl Goal for MeleeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { /* check whether to start */ })
    }
    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { /* check whether to continue */ })
    }
    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { /* start action */ })
    }
    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { /* stop action */ })
    }
    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { /* per-tick logic */ })
    }
}
```

Registration: attach in the entity's constructor via `mob_entity.goals_selector`:
```rust
mob_entity.goals_selector.add_goal(MeleeAttackGoal { mob: mob.clone(), ... });
```

---

## 4. Naming Conventions

| Category              | Pattern          | Examples                                   |
|-----------------------|------------------|--------------------------------------------|
| Block struct          | `XxxBlock`       | `CampfireBlock`, `DirtPathBlock`           |
| Block entity struct   | `XxxBlockEntity` | `CampfireBlockEntity`, `ChestBlockEntity`  |
| Item struct           | `XxxItem`        | `BucketItem`, `FishingRodItem`             |
| Entity struct         | `XxxEntity`      | `BatEntity`, `EndermanEntity`              |
| C→S packet            | `SXxx`           | `SKeepAlive`, `SChatMessage`               |
| S→C packet            | `CXxx`           | `CKeepAlive`, `CChatMessage`               |

---

## 5. Prohibited Practices

| Forbidden                                          | Reason       | Do this instead                                           |
|----------------------------------------------------|--------------|-----------------------------------------------------------|
| `todo!()` / `unreachable!()` / `unimplemented!()`  | Clippy deny  | `// TODO: describe what needs to be done`                 |
| `dbg!()` / `println!()` / `eprintln!()`            | Clippy deny  | `tracing::debug!()` / `info!()` / `warn!()` / `error!()`  |
| Manually edit `pumpkin-data/src/generated/`        | Overwritten  | Edit `pumpkin-codegen/src/`                               |
| Block on Rayon from a Tokio thread                 | Deadlock     | Bridge via `tokio::sync::mpsc`; see `Level::fetch_chunks` |

---

## 6. Coding Conventions

### 6.1 Use Project Type Aliases

```rust
// ✅ Correct
fn tick<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> { ... }

// ❌ Avoid hand-writing
fn tick<'a>(&'a self, mob: &'a dyn Mob) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> { ... }
```

Available type aliases:

| Alias                     | Defined in                            | Description                                                  |
|---------------------------|---------------------------------------|--------------------------------------------------------------|
| `BlockFuture<'a, T>`      | `pumpkin/src/block/mod.rs`            | Block trait method return type                               |
| `EntityBaseFuture<'a, T>` | `pumpkin/src/entity/mod.rs`           | Entity trait method return type                              |
| `GoalFuture<'a, T>`       | `pumpkin/src/entity/ai/goal/mod.rs`   | AI Goal trait method return type                             |
| `TeleportFuture`          | `pumpkin/src/entity/mod.rs`           | No `'a` lifetime (teleport takes `Arc<Self>` ownership)      |
| `NbtFuture<'a, T>`        | `pumpkin/src/entity/mod.rs`           | NBT serialization return type                                |
| `ViewerFuture<'a, T>`     | `pumpkin/src/block/viewer.rs`         | Block view update                                            |
| `CommandResult<'a>`       | `pumpkin/src/command/mod.rs`          | Command execution result                                     |

> Note: the `ItemBehaviour` trait does **not** use any type aliases. Hand-write the full `Pin<Box<dyn Future<...>>>` every time.

### 6.2 Future Signature Rules

The project does not use `async_trait`. All async trait methods return `Pin<Box<dyn Future<Output = T> + Send + 'a>>`:

```rust
// ✅ Correct pattern: &'a self, borrow params with matching lifetime, Box::pin(async move {...})
fn normal_use<'a>(&'a self, item: &'a Item, player: &'a Player)
    -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
{
    Box::pin(async move { /* ... */ })
}

// Use '_ when params don't need to be borrowed into the future
fn get_use_duration(&self) -> i32 { 0 }
```

### 6.3 unwrap and expect

- `unwrap()` **is used in practice**, primarily on lock operations (`mutex.lock().unwrap()`) and infallible cases.
- For fallible business-logic values, prefer `?` propagation or annotate with `expect("reason")`.
- Don't abuse `unwrap()` on fallible I/O or parsing operations.

### 6.4 Clippy Lints

```rust
// ✅ Targeted expectation: suppress only the expression that triggers the lint
#[expect(clippy::cast_possible_truncation)]
fn convert(value: u64) -> u32 { value as u32 }

// ❌ Avoid global allow
#[allow(clippy::cast_possible_truncation)]
```

### 6.5 TODO Comments

```rust
// ✅ Good TODO: describes what's missing and the expected behavior
// TODO: Check light level and isOpaqueFullCube when sky light is available.
// TODO: Use actual loot tables once LootTable system is fully implemented. For now, give raw cod.

// ❌ Bad TODO: too vague
// TODO
// TODO: fix later
```

### 6.6 Logging

```rust
use tracing::{info, warn, error, debug, trace};
info!("Player {} connected from {}", name, addr);
warn!("Chunk at {:?} failed to load: {}", pos, err);
```

### 6.7 Import Order (Recommended)

No enforced import order in the project, but keep this layering with each block sorted alphabetically:

```rust
use std::sync::Arc;              // std first

use tokio::sync::Mutex;           // third-party crates
use arc_swap::ArcSwap;

use pumpkin_data::Block;          // workspace crates
use pumpkin_util::math::Vector3;

use crate::entity::player::Player; // crate internals
use super::registry::ItemRegistry;
```

---

## 7. Pre-Submit Checklist

```
[ ] cargo clippy --all-targets    → zero warnings
[ ] cargo test                    → all pass
[ ] cargo build                   → compiles
[ ] Tests or unit tests for the feature
[ ] No leftover debug code or commented-out dead code
```

---

## 8. Architecture Quick Reference

```
pumpkin/                main crate (entrypoint, blocks, entities, items, commands, networking, plugins)
├── pumpkin-data/       generated static data (do NOT edit manually)
├── pumpkin-world/      world generation, chunk management, saves (Anvil/Linear)
├── pumpkin-protocol/   network protocol definitions (Java C/S + Bedrock)
├── pumpkin-inventory/  inventory (containers, recipes, crafting)
├── pumpkin-config/     TOML config loading
├── pumpkin-nbt/        NBT serialization
├── pumpkin-util/        utilities (math, text, noise, rng)
├── pumpkin-macros/     procedural macros
├── pumpkin-codegen/    build-time code generation
├── pumpkin-codecs/     optional codec
├── pumpkin-plugin-api/ Wasm plugin API
└── pumpkin-api-macros/ plugin macros
```

Dependency direction (bottom → top): `pumpkin-data` → `pumpkin-util` → `pumpkin-nbt` → `pumpkin-world` / `pumpkin-protocol` / `pumpkin-inventory` → `pumpkin-config` → `pumpkin/`

---

## 9. Concurrency Guide

Internal entity state relies heavily on lock-free atomics — this is the project's defining concurrency pattern:

| Primitive                               | Use case                                                              | Examples                                               |
|-----------------------------------------|-----------------------------------------------------------------------|--------------------------------------------------------|
| `crossbeam::atomic::AtomicCell<T>`      | **Main primitive for entity state fields** (15+ fields: pos, vel, …)  | `Entity.pos`, `Entity.velocity`, `Entity.yaw`          |
| `std::sync::atomic::AtomicBool/I32/U8`  | Simple flags and counters (~12 fields)                                | `Entity.on_ground`, `Entity.fire_ticks`, `Entity.age`  |
| `ArcSwap<T>`                            | Whole-value atomic swap (dimension switch, config hot-reload)         | `Entity.world`, `Player.config`                        |
| `tokio::sync::Mutex<T>`                 | Async critical sections (passengers, portals)                         | `Entity.passengers`, `Entity.portal_manager`           |
| `tokio::sync::RwLock<T>`                | Read-heavy server data                                                | `PermissionRegistry`, `BannedIpList`                   |
| `dashmap::DashMap`                      | High-concurrency HashMap                                              | map indices in World                                   |

**Key rule**: keep Tokio and Rayon separate. Offload CPU-bound work to `rayon::spawn` or Rayon parallel iterators — never block a Tokio thread. Bridge the two runtimes with `tokio::sync::mpsc`.

---

## 10. Key File Index

| Purpose               | Path                                               |
|-----------------------|----------------------------------------------------|
| Server entrypoint     | `pumpkin/src/lib.rs`                               |
| Server core           | `pumpkin/src/server/mod.rs`                        |
| World management      | `pumpkin/src/world/mod.rs`                         |
| Block registry        | `pumpkin/src/block/registry.rs`                    |
| Item registry         | `pumpkin/src/item/items/mod.rs` (default_registry) |
| Java packet handling  | `pumpkin/src/net/java/play.rs`                     |
| Command dispatch      | `pumpkin/src/command/commands/mod.rs`              |
| Entity type matching  | `pumpkin/src/entity/type.rs`                       |
| Config definition     | `pumpkin-config/src/lib.rs`                        |
| World generation      | `pumpkin-world/src/generation/`                    |
| Code generation       | `pumpkin-codegen/src/main.rs`                      |
