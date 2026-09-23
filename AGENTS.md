# AI Agent Development Guidelines

> For contribution requirements, see [CONTRIBUTING.md](CONTRIBUTING.md); for community behavior guidelines, see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
> This document primarily records code locations and key considerations during development.

---

## Guidelines

**No Hallucinations**: Never fabricate non-existent APIs, configuration options, environment variables, or dependency libraries. If you are unsure about any information, you must verify it via code search or documentation lookup. If verification is impossible, explicitly mark it as "Pending manual confirmation" in the PR description.

**Minimal Changes**: Only modify code directly related to the current Issue or task. Do not refactor unrelated modules while fixing a bug, nor alter unrelated code style when adding new features.

**Security Red Lines**: Absolutely no hardcoding of secrets, tokens, passwords, or private credentials in code, comments, test data, or PR descriptions. All sensitive configurations must be injected via environment variables or a secret management service.

**Context Awareness**: Before modifying code, you must read the relevant files, their associated tests, documentation, and recent commit history to ensure you understand the existing design intent and avoid breaking implicit contracts.

**Transparent Communication**: If you encounter blockers, ambiguous requirements, or technical infeasibility during a task, you must immediately leave a comment in the Issue or PR explaining the situation. Silent failures or forcing through flawed implementations are strictly prohibited.

**Code Copyright**: Ensure your code has no copyright conflicts and is not a direct copy-paste or replacement of code within the project. If necessary, cite the source, provide proper attribution, and comply with the corresponding open-source licenses.

---

## Preparation Before Development

> Before writing any code, please review existing similar implementations in the project to avoid reinventing the wheel. Refer to [Cargo.toml](Cargo.toml) and [rust-toolchain.toml](rust-toolchain.toml) for version and dependency specifications.

### Must-Read Before Development

You must adhere to the following conventions:

- **Environment Alignment**: Ensure your local development environment (Rust version, package manager, Clippy configuration) exactly matches the project's CI configuration. It is recommended to run the full Lint and test suite first to confirm a green baseline.

- **Branching Strategy**: Create new branches from the latest `master` branch. Branch names must follow the `<type>/<short-description>` format, e.g., `feat/add-block` or `fix/memory-leak-parser`.

- **Dependency Review**: If a task requires introducing a new dependency, first check manifest files like `Cargo.toml` to confirm it doesn't already exist. New dependencies must be evaluated for license compatibility, maintenance activity, and package size impact. The justification for introducing it must be stated in the PR description.

- **Design Pre-communication**: For complex tasks involving architectural changes or public API modifications, you must first submit a brief design document (Design Doc) under the relevant Issue and wait for confirmation from at least one maintainer before starting to code.

Build from the repository root:

```bash
cargo build --locked -p pumpkin
```

To run the server, use `cargo run --locked -p pumpkin --release`. It will read and write configuration and world data in the working directory; use a separate directory or an existing test environment when testing.

The main code resides in `crates/`, and tooling is in `tools/`:

| Directory                                                                                                  | Content                                                                              |
|:-----------------------------------------------------------------------------------------------------------|:-------------------------------------------------------------------------------------|
| [crates/pumpkin/](crates/pumpkin/)                                                                         | Server entry point, game logic, concrete commands, network handling, and plugin host |
| [crates/pumpkin-auth/](crates/pumpkin-auth/)                                                               | Authentication, JWT, HTTP client                                                     |
| [crates/pumpkin-command/](crates/pumpkin-command/)                                                         | Generic command tree, argument parsing, dispatch, and completion                     |
| [crates/pumpkin-config/](crates/pumpkin-config/)                                                           | Configuration definitions and loading                                                |
| [crates/pumpkin-data/](crates/pumpkin-data/)                                                               | Game data, registries, item stacks, and data components                              |
| [crates/pumpkin-protocol/](crates/pumpkin-protocol/)                                                       | Java, Bedrock, Query protocols and codecs                                            |
| [crates/pumpkin-world/](crates/pumpkin-world/)                                                             | Chunks, generation, lighting, and saves                                              |
| [crates/pumpkin-inventory/](crates/pumpkin-inventory/)                                                     | Inventories, containers, and screens                                                 |
| [crates/pumpkin-nbt/](crates/pumpkin-nbt/), [crates/pumpkin-codecs/](crates/pumpkin-codecs/)               | NBT and data codecs                                                                  |
| [crates/pumpkin-util/](crates/pumpkin-util/)                                                               | Shared code for math, RNG, text, permissions, versions, etc.                         |
| [crates/pumpkin-gametest/](crates/pumpkin-gametest/)                                                       | GameTest framework                                                                   |
| [crates/pumpkin-macros/](crates/pumpkin-macros/), [crates/pumpkin-api-macros/](crates/pumpkin-api-macros/) | Server and plugin-related procedural macros                                          |
| [crates/pumpkin-plugin-api/](crates/pumpkin-plugin-api/)                                                   | Wasm plugin Rust SDK                                                                 |
| [crates/pumpkin-plugin-wit/](crates/pumpkin-plugin-wit/)                                                   | WIT interface definitions; currently in `v0.1/`; not a Cargo workspace member        |
| [crates/pumpkin-host-bindings/](crates/pumpkin-host-bindings/)                                             | Wasmtime host bindings                                                               |
| [crates/pumpkin-plugin-runtime/](crates/pumpkin-plugin-runtime/)                                           | Plugin store execution, lifecycle, and reentrancy handling                           |
| [crates/pumpkin-plugin-utils/](crates/pumpkin-plugin-utils/)                                               | Plugin marketplace, licensing, and update queries                                    |
| [tools/pumpkin-codegen/](tools/pumpkin-codegen/)                                                           | Rust, WIT, and SDK data generation                                                   |
| [tools/pumpkin-fuzzer/](tools/pumpkin-fuzzer/)                                                             | Server network fuzzing                                                               |

Separate `cargo-fuzz` projects exist for NBT and protocol, located in `crates/pumpkin-nbt/fuzz/` and `crates/pumpkin-protocol/fuzz/`, respectively.

The server is licensed under GPL-3.0; `pumpkin-plugin-api`, `pumpkin-plugin-wit`, and `pumpkin-plugin-utils` are licensed under MIT OR Apache-2.0. Refer to the manifest and license files in each directory for specifics.

When looking for specific systems, start with the following files. All paths are relative to the repository root.

| Code to Find                            | Entry Points and Related Files                                                                                                                                                                                                                                         |
|:----------------------------------------|:-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Config structs, defaults, and loading   | `PumpkinConfig`, `BasicConfiguration`, `AdvancedConfiguration`, `LoadConfiguration` in [pumpkin-config/src/lib.rs](crates/pumpkin-config/src/lib.rs); specific options in the same directory and `networking/`                                                         |
| Permission definitions and queries      | [pumpkin-util/src/permission.rs](crates/pumpkin-util/src/permission.rs); commands declare required permissions upon registration                                                                                                                                       |
| Chunk loading and generation scheduling | [pumpkin-world/src/level.rs](crates/pumpkin-world/src/level.rs), [chunk_system/](crates/pumpkin-world/src/chunk_system/); loading tickets, stages, and task scheduling are in their respective submodules                                                              |
| Terrain, structures, and lighting       | [generation/generator/](crates/pumpkin-world/src/generation/generator/), [generation/feature/](crates/pumpkin-world/src/generation/feature/), [generation/structure/](crates/pumpkin-world/src/generation/structure/), [lighting/](crates/pumpkin-world/src/lighting/) |
| Chunk save formats                      | `anvil.rs`, `linear.rs`, `pump.rs` in [chunk/format/](crates/pumpkin-world/src/chunk/format/); file I/O in [chunk/io/](crates/pumpkin-world/src/chunk/io/)                                                                                                             |
| Player chunk sending, entity tracking   | [world/chunker.rs](crates/pumpkin/src/world/chunker.rs), [world/entity_tracker.rs](crates/pumpkin/src/world/entity_tracker.rs)                                                                                                                                         |
| Container interaction                   | [pumpkin-inventory/src/screen_handler.rs](crates/pumpkin-inventory/src/screen_handler.rs) and [player/](crates/pumpkin-inventory/src/player/); specific containers also invoke server-side block entities                                                              |
| Datapack resource loading               | [data/datapack/mod.rs](crates/pumpkin/src/data/datapack/mod.rs); recipes, functions, loot tables, and GameTests are parsed by loaders in the same directory                                                                                                            |
| Java & Bedrock network configuration    | [networking/java.rs](crates/pumpkin-config/src/networking/java.rs), [networking/bedrock.rs](crates/pumpkin-config/src/networking/bedrock.rs); connection handling in `crates/pumpkin/src/net/`                                                                         |

---

## Common Modifications

The short paths below are relative to `crates/pumpkin/src/`.

| What to Modify       | Reference Implementation                                                                                                                             | Registration Location                                                                |
|:---------------------|:-----------------------------------------------------------------------------------------------------------------------------------------------------|:-------------------------------------------------------------------------------------|
| Normal Blocks        | [block/blocks/dirt_path.rs](crates/pumpkin/src/block/blocks/dirt_path.rs)                                                                            | `block/blocks/mod.rs`, `block/registry.rs`                                           |
| Blocks with Entities | [block/blocks/campfire.rs](crates/pumpkin/src/block/blocks/campfire.rs), [block/entities/campfire.rs](crates/pumpkin/src/block/entities/campfire.rs) | Block registry, placement logic, NBT restoration dispatch in `block/entities/mod.rs` |
| Item Behaviors       | [item/items/bucket.rs](crates/pumpkin/src/item/items/bucket.rs)                                                                                      | `default_registry()` in `item/items/mod.rs`                                          |
| Mobs                 | [entity/mob/bat.rs](crates/pumpkin/src/entity/mob/bat.rs)                                                                                            | Corresponding module declarations, `entity/type.rs`                                  |
| AI Goals             | [entity/ai/goal/melee_attack.rs](crates/pumpkin/src/entity/ai/goal/melee_attack.rs)                                                                  | `goals_selector` or `target_selector` in mob constructors                            |
| Commands             | [command/commands/time.rs](crates/pumpkin/src/command/commands/time.rs)                                                                              | `default_dispatcher()` in `command/commands/mod.rs`                                  |
| Recipes & Datapacks  | [server/recipe.rs](crates/pumpkin/src/server/recipe.rs), [data/datapack/](crates/pumpkin/src/data/datapack/)                                         | Recipe management, resource loading, and client synchronization                      |
| GameTest             | [server/server_test_manager.rs](crates/pumpkin/src/server/server_test_manager.rs)                                                                    | `command/commands/test.rs`                                                           |

The server entry point is [main.rs](crates/pumpkin/src/main.rs); initialization and exports are handled in [lib.rs](crates/pumpkin/src/lib.rs); and world management is located in [world/mod.rs](crates/pumpkin/src/world/mod.rs).

### Blocks and Items

[BlockBehaviour](crates/pumpkin/src/block/mod.rs) and [ItemBehaviour](crates/pumpkin/src/item/mod.rs) use synchronous methods.

`#[pumpkin_block(...)]` and `#[pumpkin_block_from_tag(...)]` will generate `BlockMetadata`; do not manually write duplicate implementations.

The following templates demonstrate normal blocks, block entities, and items, respectively. `Example*` and `example.rs` are placeholder names to be replaced; they are not existing types or files in the repository. The examples borrow vanilla IDs; registering the same ID will override existing behaviors without adding unregistered blocks or items to the client. When modifying existing functionality, extend the original implementation directly.

**Normal Blocks**

This can be placed in a newly added `block/blocks/example.rs`. The template retains the default placement state and interaction result:

```rust
use pumpkin_data::BlockStateId;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, NormalUseArgs, OnPlaceArgs, registry::BlockActionResult};

#[pumpkin_block("minecraft:stone")]
pub struct ExampleBlock;

impl BlockBehaviour for ExampleBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        args.block.default_state.id
    }

    fn normal_use(&self, _args: NormalUseArgs<'_>) -> BlockActionResult {
        BlockActionResult::Pass
    }
}
```

Declare `pub mod example;` in [block/blocks/mod.rs](crates/pumpkin/src/block/blocks/mod.rs), import `ExampleBlock` in [block/registry.rs](crates/pumpkin/src/block/registry.rs), and call `manager.register(ExampleBlock)` within `default_registry()`.

**Saving and Restoring Block Entities**

The following implements [BlockEntity](crates/pumpkin/src/block/entities/mod.rs), borrowing the `CustomName` field from the enchanting table to demonstrate NBT reading and writing. This can be placed in a newly added `block/entities/example.rs`. It only demonstrates saving and does not include runtime renaming or client synchronization:

```rust
use std::any::Any;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::block::entities::BlockEntity;

pub struct ExampleBlockEntity {
    position: BlockPos,
    custom_name: Option<String>,
}

impl ExampleBlockEntity {
    pub const ID: &'static str = "minecraft:enchanting_table";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: None,
        }
    }
}

impl BlockEntity for ExampleBlockEntity {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(name) = &self.custom_name {
            nbt.put_string("CustomName", name.clone());
        }
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        Self {
            position,
            custom_name: nbt.get_string("CustomName").map(str::to_owned),
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }
    fn get_position(&self) -> BlockPos {
        self.position
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

The corresponding block can be placed in a newly added `block/blocks/example_entity.rs`, using the same enchanting table ID as the block entity:

```rust
use std::sync::Arc;

use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, PlacedArgs, entities::example::ExampleBlockEntity};

#[pumpkin_block("minecraft:enchanting_table")]
pub struct ExampleEntityBlock;

impl BlockBehaviour for ExampleEntityBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(ExampleBlockEntity::new(*args.position)));
    }
}
```

Declare `example` and `example_entity` in the `mod.rs` files of their respective directories, and register the block type in `block/registry.rs`. In the `match id` block of `block/entities/mod.rs::block_entity_from_nbt()`, replace the branch for the original enchanting table ID to return `Some(Arc::new(example::ExampleBlockEntity::from_nbt(nbt, pos)))`. Verify that it can be restored after a world reload; when containers are involved, also check the saving of dirty flags, comparators, and item components.

`write_internal()` already writes `id`, `x`, `y`, and `z`; `write_nbt()` should only write its own fields. NBT required by the client is provided by `chunk_data_nbt()`; refer to [World::update_block_entity()](crates/pumpkin/src/world/mod.rs) and [enchanting_table.rs](crates/pumpkin/src/block/entities/enchanting_table.rs) for actual update implementations.

**Item Behaviors**

This can be placed in a newly added `item/items/example.rs`. Here, a stick is used to demonstrate the full `use_on_block()` signature:

```rust
use std::any::Any;

use pumpkin_data::{Block, BlockDirection, item::Item, item_stack::ItemStack};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{
    block::registry::BlockActionResult,
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
    server::Server,
};

pub struct ExampleItem;

impl ItemMetadata for ExampleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::STICK.id])
    }
}

impl ItemBehaviour for ExampleItem {
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        _player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        BlockActionResult::Pass
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

Declare `pub mod example;` in [item/items/mod.rs](crates/pumpkin/src/item/items/mod.rs), import `ExampleItem`, and call `manager.register(ExampleItem)` within `default_registry()`. `Pass` indicates that subsequent processing should continue; the logic for whether to attempt block placement is located in [item/registry.rs](crates/pumpkin/src/item/registry.rs). Do not arbitrarily change this to `Success`.

For items requiring raycasting, refer to the bucket's `normal_use_with_rotation()`, using the yaw/pitch from the current action packet.

### Entities and AI

Entities are composed via `Entity`, `LivingEntity`, and `MobEntity`. `Mob` already has a generic implementation of `EntityBase`, which in turn has a generic implementation of `NBTStorage`. Do not re-implement these traits.

Custom persistence fields for mobs should be placed in `mob_write_nbt()` / `mob_read_nbt()`; other entities should use `write_custom_nbt()` / `read_custom_nbt()`. Fields that need to be synchronized to the client are updated via `tracked_data`, `set_synced_data()`, and [SynchedEntityData](crates/pumpkin/src/entity/synched_entity_data.rs).

**Mob Template**

Below is a skeleton for construction and persistence, which can be placed in a newly added `entity/mob/example.rs`. `ExampleActive` is a custom NBT field for this example; spawning rules, attributes, and client metadata must be supplemented according to the actual mob.

```rust
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity,
    mob::{Mob, MobEntity},
};

pub struct ExampleEntity {
    mob_entity: MobEntity,
    active: AtomicBool,
}

impl ExampleEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        register_goals(&mob_entity);
        Arc::new(Self {
            mob_entity,
            active: AtomicBool::new(false),
        })
    }
}

impl Mob for ExampleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("ExampleActive", self.active.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(active) = nbt.get_bool("ExampleActive") {
            self.active.store(active, Ordering::Relaxed);
        }
    }
}
```

`register_goals()` is placed in the same file. Here, the existing look-around goal is attached:

```rust
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;

fn register_goals(mob: &MobEntity) {
    mob.add_goal(7, RandomLookAroundGoal::default());
}
```

[MobEntity::add_goal()](crates/pumpkin/src/entity/mob/mod.rs) and `add_target_goal()` already handle locking and boxing; simply pass the goal value directly. The former is used for general behaviors, while the latter is used for selecting targets; lower numbers indicate higher priority. Calling `GoalSelector::add_goal()` directly requires passing a `Box<G>`, e.g., `Box::new(MeleeAttackGoal::new(1.0, true))`.

Custom goals are placed in `entity/ai/goal/` with their modules declared. [Goal](crates/pumpkin/src/entity/ai/goal/mod.rs) methods are synchronous; `can_start`, `should_continue`, `start`, `stop`, and `tick` all take `&mut self` and `&dyn Mob`. `controls(&self)` returns `Controls`, declaring occupied control items via `MOVE`, `LOOK`, `JUMP`, and `TARGET`; the look-around goal above occupies `MOVE | LOOK`. Default goals do not run on every server tick; refer to `get_tick_count()` and `should_run_every_tick()` for timing.

Declare the mob module in [entity/mob/mod.rs](crates/pumpkin/src/entity/mob/mod.rs), and hook into the constructor in `from_type()` within [entity/type.rs](crates/pumpkin/src/entity/type.rs). Using the existing BAT branch as an example, replace the original branch when modifying behavior rather than adding duplicate matches. Natural spawning also involves `check_spawn_rules()` in the same file and [world/natural_spawner.rs](crates/pumpkin/src/world/natural_spawner.rs); world restoration also calls `from_type()`.

### Commands

The server command execution result, `CommandExecutorResult`, is `Result<i32, CommandSyntaxError>`. When adding a command, handle permissions, completions, and registration simultaneously.

Below is a parameterless command requiring OP level 2, which can be placed in a newly added `command/commands/example.rs`. Rename `example` and the permission node according to the actual command:

```rust
use crate::command::{
    argument_builder::{ArgumentBuilder, command},
    context::command_context::CommandContext,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

const PERMISSION: &str = "pumpkin:command.example";
const DESCRIPTION: &str = "Replies with a confirmation.";

struct ExampleCommand;

impl CommandExecutor for ExampleCommand {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        context
            .source
            .send_feedback(TextComponent::text("Ready."), false);
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));
    dispatcher.register(
        command("example", DESCRIPTION)
            .requires(PERMISSION)
            .executes(ExampleCommand),
    );
}
```

Add `mod example;` in [command/commands/mod.rs](crates/pumpkin/src/command/commands/mod.rs), and call `example::register(&mut dispatcher, registry)` in `default_dispatcher()`. Built-in commands use `register_permission_or_panic()` at startup to check for duplicate permissions; for registrations that are allowed to fail during runtime, use `register_permission()` and handle the return value.

For commands with arguments, refer to [time.rs](crates/pumpkin/src/command/commands/time.rs). The builder is located in [command/argument_builder.rs](crates/pumpkin/src/command/argument_builder.rs); server-specific argument types are in [command/argument_types/](crates/pumpkin/src/command/argument_types/), and generic argument types are in [pumpkin-command/src/argument_types/](crates/pumpkin-command/src/argument_types/).

---

## Networking and Plugins

Java's Handshake, Status, Login, and Configuration phases are handled by [PendingConnection](crates/pumpkin/src/net/java/pending.rs); once in the Play phase, handling is taken over by [JavaClient](crates/pumpkin/src/net/java/mod.rs), with specific handlers located in [play/](crates/pumpkin/src/net/java/play/).

The core Java packet table is currently generated for version 26.3. Current and minimum version constants are defined in the [generated packet.rs](crates/pumpkin-data/src/generated/packet.rs). Legacy version support has been moved to extensions. [handshake.rs](crates/pumpkin/src/net/java/handshake.rs) will allow certain legacy protocols based on the `PacketReceivedEvent` handler, but actual compatibility still requires loading the corresponding plugin and testing with a client. Bedrock uses a separate implementation that is still under development.

When modifying packets, verify the protocol phase, packet ID, field order, length, compression, and encryption status. When reading packets, also account for cancellation and timeouts. In packet naming, `S` denotes packets sent from the client to the server, and `C` denotes packets sent from the server to the client. Validate Java and Bedrock behaviors independently.

Using Keep Alive as an example, the code is distributed as follows:

| Component                                | File                                                                                                                                                                 |
|:-----------------------------------------|:---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Client-to-Server Packet & Reading        | [pumpkin-protocol/src/java/server/play/keep_alive.rs](crates/pumpkin-protocol/src/java/server/play/keep_alive.rs): `SKeepAlive`, `ServerPacket::read()`              |
| Server-to-Client Packet & Writing        | [pumpkin-protocol/src/java/client/play/keep_alive.rs](crates/pumpkin-protocol/src/java/client/play/keep_alive.rs): `CKeepAlive`, `ClientPacket::write_packet_data()` |
| Field Read/Write Utilities & Error Types | [pumpkin-protocol/src/ser/mod.rs](crates/pumpkin-protocol/src/ser/mod.rs): `NetworkReadExt`, `NetworkWriteExt`, `ReadingError`, `WritingError`                       |
| Packet ID Source                         | [pumpkin-codegen/src/packet.rs](tools/pumpkin-codegen/src/packet.rs), [assets/packets.json](assets/packets.json); `#[java_packet(...)]` uses generated constants     |
| Server Dispatch                          | [net/java/mod.rs](crates/pumpkin/src/net/java/mod.rs): matches by `to_id(version)`, calls handler after reading the packet                                           |
| Actual Handling                          | [net/java/play/keep_alive.rs](crates/pumpkin/src/net/java/play/keep_alive.rs), module declared in [play/mod.rs](crates/pumpkin/src/net/java/play/mod.rs)             |

New packets must also be declared and exported in the `mod.rs` of the corresponding protocol directory. Configuration phase packets are located in the protocol's `java/server/config/` and `java/client/config/`; do not register them in the Play dispatcher. The Keep Alive file already contains round-trip tests for different versions; refer to its field checks and add assertions for "input fully consumed."

The plugin loader has two implementations: [native.rs](crates/pumpkin/src/plugin/loader/native.rs) and [wasm/](crates/pumpkin/src/plugin/loader/wasm/). Internal server APIs are located in `crates/pumpkin/src/plugin/api/`, and the Wasm plugin SDK is in `pumpkin-plugin-api`.

When modifying Wasm interfaces, you must review the following together:

- `crates/pumpkin-plugin-wit/v0.1/`: Interface and world imports/exports.
- `crates/pumpkin-host-bindings/src/lib.rs`: Host bindings and invocation configuration.
- `crates/pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/`: Host implementation and event translation.
- `crates/pumpkin-plugin-api/src/`: SDK wrappers and exports.
- `crates/pumpkin-plugin-runtime/`: Execution, reentrancy, and unloading.

Verify permissions and the lifecycle of resource handles. Breaking changes to native plugins also require updating `PLUGIN_API_VERSION` in `plugin/mod.rs`. Wasmtime and WASI currently use pinned Git revisions; keep them aligned when upgrading.

Synchronous methods in game logic do not imply the entire project is synchronous. Networking and plugins still expose async/Future interfaces: native plugins use `PluginFuture`, and event dispatch uses `BoxFuture`; the Wasm Host implements the generated traits.

**Wasm Plugin Template**

This code belongs in the plugin's own Rust project, not in the server's `plugin/api/`. See [pumpkin-plugin-api/src/lib.rs](crates/pumpkin-plugin-api/src/lib.rs) for the current SDK definitions:

```rust
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, register_plugin};

struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "example-plugin".into(),
            version: "0.1.0".into(),
            authors: vec!["Your name".into()],
            description: "Plugin lifecycle example.".into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }
}

register_plugin!(ExamplePlugin);
```

Refer to the `Plugin` trait for interface signatures. For commands, events, and scheduled tasks, refer to the SDK's [commands.rs](crates/pumpkin-plugin-api/src/commands.rs), [events/mod.rs](crates/pumpkin-plugin-api/src/events/mod.rs), and [scheduler.rs](crates/pumpkin-plugin-api/src/scheduler.rs), respectively. Required permissions must be declared in `PluginMetadata.permissions`. Build and packaging configurations should follow the target plugin project.

---

## When Writing Code

Clippy configurations are in the root `Cargo.toml`, which denies `unwrap`, `expect`, `panic`, `todo!()`, and debug output by default. Handle errors using `Result` / `Option`, and use `tracing` for logging. Use local `#[expect(...)]` with a documented reason for any strictly necessary lint exceptions.

Refer to the respective module for handling poisoned synchronization locks; only use `PoisonError::into_inner` if the state allows recovery. Do not hold synchronization locks or `DashMap` guards across `.await` points.

Offload CPU-intensive work to Rayon; do not block Tokio worker threads. Refer to existing `Level` and `Server` implementations for scheduling between async I/O and synchronous ticks; when modifying plugin execution, follow the Store management in `pumpkin-plugin-runtime`.

Entity state commonly uses `AtomicCell` and standard atomic types, while `ArcSwap` is used for replacing shared objects. Follow the conventions of the respective module when choosing locks and atomic types, and check for lock ordering and callback reentrancy. Limit lengths and allocations when parsing external data; provide measurement results for performance changes in hot paths.

---

## Code Generation

Do not modify files in `crates/pumpkin-data/src/generated/` directly. Modify the generators in `tools/pumpkin-codegen/src/` or the inputs in `assets/`, then regenerate. The entry point is [tools/pumpkin-codegen/src/main.rs](tools/pumpkin-codegen/src/main.rs).

```bash
# Generate only packet.rs
cargo run --locked -p pumpkin-codegen -- packet

# Generate WIT data definitions and host packet mappings
cargo run --locked -p pumpkin-codegen -- wit

# Full generation of Rust, WIT, and SDK data
cargo run --locked -p pumpkin-codegen
```

Filter arguments can be the output filename, with or without the `.rs` extension. The `wit` mode will also update the host's `generated_packets.rs`. The SDK's `src/generated/` is also generated by this tool; modify source files directly for hand-written WIT interfaces. Always review the diff after generation to avoid introducing unrelated changes.

Common mappings between inputs and generators:

| Data                                | Input                                                                             | Generator                                                                                                                |
|:------------------------------------|:----------------------------------------------------------------------------------|:-------------------------------------------------------------------------------------------------------------------------|
| Blocks and State Properties         | `assets/blocks.json`, `assets/properties.json`, and mappings in `assets/bedrock/` | [block.rs](tools/pumpkin-codegen/src/block.rs)                                                                           |
| Items                               | `assets/items.json` and Bedrock item data                                         | [item.rs](tools/pumpkin-codegen/src/item.rs)                                                                             |
| Entities, Synced Fields             | `assets/entities.json`, `assets/tracked_data.json`                                | [entity_type.rs](tools/pumpkin-codegen/src/entity_type.rs), [tracked_data.rs](tools/pumpkin-codegen/src/tracked_data.rs) |
| Vanilla Recipes and Tags            | `assets/datapack/data/`                                                           | [recipes.rs](tools/pumpkin-codegen/src/recipes.rs), [tag.rs](tools/pumpkin-codegen/src/tag.rs)                           |
| SDK Types, WIT, and Packet Mappings | Game data read by respective generators                                           | [sdk/](tools/pumpkin-codegen/src/sdk/), [wit/](tools/pumpkin-codegen/src/wit/)                                           |

Vanilla built-in recipe generation and runtime datapack loading are two separate pieces of code. When modifying custom datapack recipe parsing, refer to `crates/pumpkin/src/data/datapack/recipe_loader.rs` first.
---

## Review and Submission

- Before opening a PR, ensure that all CI checks (build, test, lint, type-check) have passed locally or in your fork. PRs that only run a subset of tests or skip lint checks will be closed immediately.
- If you are fixing a bug, you must add a failing test case that reproduces the bug and ensure it passes after the fix. This prevents the issue from being accidentally reintroduced in the future.
- Obvious minor changes and pure documentation updates do not require additional tests.
- If the changes involve parsing, network requests, memory allocation, or loop logic, you must evaluate their performance impact. When necessary, add benchmarks or memory leak detection, and provide comparative data in the PR.
- Clearly state the execution commands and the reasons for failure in the test results. For client tests, record the client version, server commit, plugins used, steps to reproduce, and logs. Explicitly state the results for: successful login, normal chunk loading, and long-term stability. If a check was not actually run, explicitly mark it as "Not Run."

### Local Test Verification

First, use `-p <package_name>` to check the relevant package, then check the affected dependents. If `nextest` is not installed, you can use `cargo test --locked --workspace` and note that a different test runner was used. To run a full check, execute the following from the repository root:

```bash
cargo clippy --release --all-targets --all-features
cargo fmt --all -- --check
cargo machete
cargo check --all-targets --all-features
typos
RUSTFLAGS="-Dwarnings" cargo clippy --locked --workspace --all-targets --all-features
RUSTFLAGS="-Dwarnings" cargo nextest run --locked --workspace --verbose --no-tests=pass
RUSTFLAGS="-Dwarnings" cargo test --locked --workspace --doc --verbose
RUSTFLAGS="-Dwarnings" cargo build --locked --workspace
```

Tests are typically written in a `#[cfg(test)] mod tests` block within the target file. The following test can be appended to the end of the previously mentioned `block/entities/example.rs` to verify name restoration and the writing of basic position fields:

```rust
#[cfg(test)]
mod tests {
    use super::{BlockEntity, ExampleBlockEntity};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::position::BlockPos;

    #[test]
    fn restores_name_and_position() {
        let position = BlockPos::new(2, 64, -3);
        let mut saved = NbtCompound::new();
        saved.put_string("CustomName", "Example".to_owned());

        let entity = ExampleBlockEntity::from_nbt(&saved, position);
        let mut written = NbtCompound::new();
        entity.write_internal(&mut written);

        assert_eq!(written.get_string("CustomName"), Some("Example"));
        assert_eq!(written.get_string("id"), Some(ExampleBlockEntity::ID));
        assert_eq!(written.get_int("x"), Some(2));
        assert_eq!(written.get_int("y"), Some(64));
        assert_eq!(written.get_int("z"), Some(-3));
    }
}
```

For test placement references, see the interaction result tests in [item/registry.rs](crates/pumpkin/src/item/registry.rs), the recipe parsing tests in [recipe_loader.rs](crates/pumpkin/src/data/datapack/recipe_loader.rs), and the chunk scheduling tests in [chunk_system/tests.rs](crates/pumpkin-world/src/chunk_system/tests.rs). For scenarios requiring a full world context, use `pumpkin-gametest` and the existing `test` command.

See [.github/scripts/ci_packages.py](.github/scripts/ci_packages.py) for CI package selection rules, and [.github/workflows/rust.yml](.github/workflows/rust.yml) for check commands. Changes to WIT also require interface binding checks; for documentation, first run `typos AGENTS.md` and `git diff --check -- AGENTS.md`.

### Self-Review Checklist

Confirm the following items before submitting:

- No leftover TODO/FIXME/HACK comments in the code (or they are tracked in an Issue).
- No unused imports.
- All newly added public interfaces are documented, ensuring documentation coverage exceeds 80%.
- Tests cover critical paths, and there are no flaky tests.

---

## Open PR

- **Title Format**: `<type>: <short description>` (e.g., `feat(auth): Add multi-Yggdrasil authentication support`) and append 🤖🤖🤖 to the end of the PR title.
- **Duplicate Implementations**: Before submitting, **please search for similar existing PRs first**. Do not submit nearly identical implementations. If necessary, explain the differences, as well as the pros and cons of your implementation compared to existing ones.
- **Split PRs**: **One PR should do only one thing**. Do not mix unrelated changes; for example, a single PR should ideally fix only one bug.
- **Description Template**: Follow the `Description` and `Testing` sections in the [PR Template](.github/PULL_REQUEST_TEMPLATE.md) to explain the problem, the solution, testing, and known limitations. Attach test screenshots or videos if necessary.
- **Respond to Feedback**: After submitting a PR, you must actively monitor CI results and maintainer comments. If CI fails, you must fix it or explain the reason within 24 hours. If you receive revision requests, you should respond or negotiate an extension within 48 hours.
