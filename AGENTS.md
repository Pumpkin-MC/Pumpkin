# Agent implementation rules

These instructions apply to every automated implementation in this repository.

## Read before coding

Prefer existing project implementations. Before editing, thoroughly read the closest
analogue and trace its complete registration chain: definition, annotation/trait,
registry match or registration call, and module declaration.

| Task | Required analogue |
| --- | --- |
| Block with entity | `campfire.rs`, `entities/campfire.rs`, then `block/registry.rs` |
| Block without entity | `dirt_path.rs`, then `block/registry.rs` |
| Item | `item/items/bucket.rs`, then `item/items/mod.rs::default_registry` |
| Mob AI goal | `entity/ai/goal/melee_attack.rs`, then an entity constructor attaching goals |
| Command | `command/commands/time.rs`, then `command/commands/mod.rs` |
| Packet handling | the most similar handler in `net/java/play.rs` |

Always defer to the current source when an example or template has drifted.

## Project patterns

- Blocks use `#[pumpkin_block(...)]` or `#[pumpkin_block_from_tag(...)]`, implement
  only required `BlockBehaviour` methods, and register in `block/registry.rs`.
- Items have no annotation macro. Implement `ItemMetadata::ids` and `ItemBehaviour`,
  then register in `item/items/mod.rs::default_registry`.
- Entities implement the required `EntityBase` and NBT methods and are mapped by
  `EntityType` in `entity/type.rs`.
- AI goals implement `Goal` and are attached through the entity's goal selector.
- Use project future aliases (`BlockFuture`, `EntityBaseFuture`, `GoalFuture`,
  `TeleportFuture`, `NbtFuture`, `ViewerFuture`, `CommandResult`) where available.
  `ItemBehaviour` deliberately uses the explicit pinned boxed future signature.
- Trait futures use matching lifetimes and `Box::pin(async move { ... })`; the project
  does not use `async_trait`.
- Name block, block-entity, item, and entity structs `XxxBlock`, `XxxBlockEntity`,
  `XxxItem`, and `XxxEntity`. Client-to-server packets are `SXxx`; server-to-client
  packets are `CXxx`.

## Prohibited practices

- Do not use `todo!`, `unreachable!`, `unimplemented!`, `dbg!`, `println!`, or
  `eprintln!`; workspace Clippy denies them. Use precise TODO comments and `tracing`.
- Never manually edit `pumpkin-data/src/generated`. Change `pumpkin-codegen` inputs or
  generators and regenerate.
- Never block a Tokio thread on Rayon. Use Rayon for CPU work and bridge with
  `tokio::sync::mpsc`, following `Level::fetch_chunks`.
- Do not use broad Clippy allows. Prefer a targeted `#[expect(...)]` on the expression
  or function, with a reason when it is not self-evident.
- Avoid `unwrap` for fallible I/O, parsing, or business state. Propagate errors or use
  an `expect` message only for a proved invariant.
- Do not add vague TODOs. State the missing behavior and intended condition.

## Concurrency

Preserve the established primitive and atomicity:

- `AtomicCell<T>` for lock-free entity values such as position and velocity.
- standard atomics for simple flags and counters.
- `ArcSwap<T>` for whole-value configuration/world swaps.
- Tokio mutexes/rwlocks for async critical sections.
- `DashMap` for high-concurrency maps.

Changes to concurrently written fields must remain minimal. Never replace an atomic
read-modify-write with a load/store pair. Deterministic vanilla tick ordering must not
be implemented by spawning mutually dependent tasks.

## Parity workflow

For vanilla behavior work, use the pinned Minecraft 26.2 JAR/decompiled source as the
primary oracle. Cite the exact vanilla class/method in the task or regression. Read the
closest Pumpkin implementation completely before editing. Add a minimized regression
test, make one logical change, and keep commits cherry-pickable. Do not claim live or
stochastic verification without a sufficient declared observation window.

## Pre-submit checks

Every implementation must complete, or explicitly report why it could not complete:

1. `cargo fmt` or the narrow package formatting check.
2. Focused regression tests.
3. `RUSTFLAGS="-D warnings" cargo clippy -p <package> --all-targets`.
4. Relevant package/workspace tests.
5. A build proportional to the change.
6. `git diff --check` and inspection for debug/dead code.

The coordinator runs a combined workspace test, strict Clippy, and release build after
parallel workers stop editing.
