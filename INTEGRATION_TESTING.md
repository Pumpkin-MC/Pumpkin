# Creating a GameTest

Minecraft has a built-in GameTest system. It places a test structure in the world, runs the test, and reports a pass or failure. A test instance JSON file names the structure and sets the test type and time limit. See the [Test Instance Definition](https://minecraft.wiki/w/Test_instance_definition) for the JSON format.

- **Block based test:** The structure contains test blocks. A START block starts the test. An ACCEPT block marks a pass. A FAIL block marks a failure. This type does not need a Rust test function.
- **Function test:** The test instance names a registered function. In Pumpkin, the function uses Rust code to set up the test and check the result. It can use `minecraft:empty` as its structure.

## Block based tests

Block based tests use redstone and test blocks inside a saved structure. [This video on test blocks](https://www.youtube.com/watch?v=6elLSg0y7X4) shows the in-game controls.

1. Join a Pumpkin server as an operator in Creative mode. Run `/test create pumpkin:redstone_example 5 4 5` to make a test area and a test instance block. Build the behavior you want to check inside the area.
2. Get test blocks with `/give @s minecraft:test_block`. Place exactly one in **START** mode. It emits a one-tick redstone pulse when the test begins. Wire the expected outcome to a test block in **ACCEPT** mode and any incorrect outcome to one in **FAIL** mode. A **LOG** block can record a message when powered. Set each block's mode and optional message in its in-game screen. Include at least one ACCEPT block; the test times out if none receives power.
3. Use the test instance block's **Save** action. It writes `pumpkin:redstone_example` to `<world>/generated/pumpkin/structure/redstone_example.nbt`. Copy that file to `assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/structure/redstone_example.nbt`.
4. Create `assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/test_instance/redstone_example.json`:

   ```json
   {
     "type": "minecraft:block_based",
     "environment": "minecraft:default",
     "structure": "pumpkin:redstone_example",
     "max_ticks": 20
   }
   ```

The structure ID must match the saved structure path. Set `max_ticks` long enough for the redstone circuit to reach ACCEPT or FAIL.

## Function tests

A builtin function test needs a Rust function, a registry entry, and a test instance JSON file. Use a nearby test in `crates/pumpkin-core/src/server/builtin_test_functions/` as a model.

### 1. Write the test function

Add the function to an existing module or a new module under `crates/pumpkin-core/src/server/builtin_test_functions/`. A new module also needs a `mod` declaration in that directory's `mod.rs`.

```rust
use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

pub(super) async fn test_stone_example(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let position = BlockPos::new(2, 1, 2);
    test.set_block_default(&position, &Block::STONE).await?;
    test.assert_block_present(&position, &Block::STONE).await?;

    test.succeed();
    Ok(())
}
```

Call `set_dimensions(width, height, depth)` first. Coordinates are relative to the test structure, starting at `(0, 0, 0)`. If the structure is larger than the requested dimensions, the larger value wins on each axis. `build_floor` fails if its width or depth exceeds the test dimensions. Placing a block outside the test dimensions also fails the test. For a condition that must become true on a later tick, use `succeed_when` or a sequence instead of calling `succeed` immediately.

### 2. Register the function

Add a line inside `ensure_builtin_test_functions_registered()` in `crates/pumpkin-core/src/server/builtin_test_functions/mod.rs`:

```rust
register_test_function("pumpkin:stone_example", example::test_stone_example);
```

The resource ID must match the `function` value in the JSON file.

### 3. Add the test instance

Create `assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/test_instance/stone_example.json`:

```json
{
  "type": "minecraft:function",
  "environment": "minecraft:default",
  "structure": "minecraft:empty",
  "function": "pumpkin:stone_example",
  "max_ticks": 20,
  "setup_ticks": 0
}
```

Use `minecraft:empty` for tests that build their own blocks. Choose `max_ticks` long enough for any scheduled assertions. If the test uses a saved structure, set `structure` to its resource ID; `set_dimensions` will keep the larger size on each axis.

## Regenerate and check

From the repository root, regenerate the test data after changing the datapack:

```bash
cargo run --locked -p pumpkin-codegen -- test_instance test_environment test_function structure_template
```

If you changed Rust code, run `cargo fmt --all` and the affected crates' Clippy and test checks. Build the server and run the new GameTest from the repository root, using its test instance ID:

```bash
cargo run --bin pumpkin -- --tests "pumpkin:stone_example"
```

For the block based example, run `cargo run --bin pumpkin -- --tests "pumpkin:redstone_example"`. You can also run `/test run pumpkin:redstone_example` in a server.

Use `*` in a selector to run a group of tests. For example, this command runs tests whose IDs start with `pumpkin:redstone_`:

```bash
cargo run --bin pumpkin -- --tests "pumpkin:redstone_*"
```

Run all tests that are not marked `manual_only` by leaving the selector empty:

```bash
cargo run --bin pumpkin -- --tests
```

You can also run `/test run pumpkin:stone_example` in a server.

## Frequently asked questions

### Why can't the server find my test?

The test instance data is embedded in the Pumpkin binary through code generation. If you add or change a test instance JSON file or a saved structure, regenerate the data and rebuild the server. Check that the JSON file name matches the test ID in the command. For a block based test, check that `structure` matches the NBT path and that the saved structure contains START and ACCEPT blocks. For a function test, check that `function` matches the registered function ID.

```bash
cargo run --locked -p pumpkin-codegen -- test_instance test_environment test_function structure_template
cargo build --bin pumpkin
```

### When must I regenerate the test data?

Run code generation each time you add or change a test instance JSON file or a saved structure. Do this before you run the test. A rebuild alone does not update the generated test data.
