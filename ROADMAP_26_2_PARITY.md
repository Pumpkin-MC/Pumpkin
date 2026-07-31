# Minecraft 26.2 parity roadmap

This roadmap targets Minecraft Java Edition 26.2 (protocol 776, data version 4903)
using the official server as the behavioral oracle. The pinned oracle currently has
SHA-256 `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5`.

Parity is a test result, not a subjective feature estimate. Missing, skipped, flaky,
timed-out, or unimplemented behaviors remain in the denominator.

## Scoring and release gates

Freeze a versioned matrix of atomic behavior cells before measuring Pumpkin. Weight
the matrix by subsystem:

| Subsystem | Weight |
| --- | ---: |
| Protocol and connectivity | 15 |
| Persistence and world I/O | 15 |
| Blocks, redstone, and fluids | 15 |
| Entities, AI, and combat | 15 |
| World generation, dimensions, and lighting | 12 |
| Items, inventory, recipes, and effects | 12 |
| Commands, administration, and permissions | 8 |
| Rules, time, weather, and progression | 5 |
| Robustness and resource handling | 3 |

For cell `i` with weight `w_i`:

- `covered_i` is 1 only when all required oracle and Pumpkin observations ran.
- `pass_i` is 1 only when every mandatory assertion passed.
- Coverage is `100 * sum(w_i * covered_i) / sum(w_i)`.
- Parity is `100 * sum(w_i * covered_i * pass_i) / sum(w_i)`.

A 95% claim requires at least 98% coverage, at least 95% weighted parity, at least
90% in every subsystem, and no critical crash, corruption, protocol, persistence,
cross-load, or test-control failure. A 100% claim means all frozen cells pass; it is
called 100% test-defined conformance rather than proof over every possible world and
packet stream.

## M0: Differential conformance lab

Build a hermetic runner that launches vanilla and Pumpkin in separate temporary
directories with equivalent fixed configurations and worlds. Add a minimal independent
protocol-776 client, RCON driver, deterministic tick barriers, canonical NBT/region
observers, packet transcripts, and replay bundles.

The harness must fail when its own control primitives are broken. It must never use a
Pumpkin command as the sole judge of Pumpkin state.

Exit criteria:

- One command runs a scenario against both servers and emits a field-level diff.
- Oracle hash, Java version, Pumpkin commit/binary hash, configuration, seed, logs,
  packets, and snapshots are recorded with every result.
- Repeated deterministic vanilla runs produce identical canonical output.
- Normalizers have positive and negative tests proving they cannot hide gameplay data.

## M1: Protocol schema and session state

Generate a state/direction inventory from official 26.2 packet classes and classify
every packet as implemented, safely ignored, cleanly rejected, or missing. Complete
field codecs, bounds checks, login/configuration/play transitions, registries, tags,
known packs, recipes, secure chat, resource packs, transfer/cookies, proxy validation,
queue limits, and slow-client behavior.

Exit criteria:

- Every official packet type is classified with no accidental omissions.
- Implemented packets match official bytes and decoded fields.
- Threshold, fragmentation, encryption-transition, truncation, overlong-VarInt, and
  maximum-size cases pass without panic or unbounded allocation.
- One million protocol fuzz inputs produce no crash.
- 10,000 login/disconnect cycles produce no state desynchronization or resource leak.

## M2: Test control plane

Make the fixture-building surface trustworthy: console/RCON world context, command
result values, `execute` modifiers and conditions, block-state arguments, real forced
chunk loading, deterministic tick stepping, `randomTickSpeed`, and light-update
observation.

Exit criteria:

- Player, console, RCON, and command-block sources have differential command tests.
- Forced chunks tick without players.
- Random-tick tests use controlled stepping or a predeclared statistically sufficient
  observation window.
- Command success, failure, integer result, feedback, and side effect all match.

## M3: Lossless vanilla world I/O

Use one data-version constant everywhere. Implement exact named block-state and biome
disk palettes. Model behaviorally significant chunk fields and preserve unknown NBT
verbatim. Preserve structures, blending/retrogen/upgrade data, post-processing,
carving masks, scheduled ticks, block entities, entities/passengers, POI, player data,
scoreboards, statistics, and advancements. Make writes atomic and crash recoverable.

Exit criteria:

- Vanilla to Pumpkin no-op save to vanilla preserves canonical state for at least 100
  representative chunks in all dimensions.
- Unknown synthetic tags survive at every supported NBT scope.
- Every emitted data file uses version 4903.
- Fault injection at each write/flush/rename boundary leaves an old or new valid file,
  never a corrupt hybrid.
- Unsupported input versions are rejected before mutation.

## M4: Canonical lighting

Define one in-memory nibble order and section coordinate system. Share serialization
between initial chunk data and incremental updates, including below/above-world padding
sections. Track dirty sections, chunk watchers, border propagation, and import validity.

Exit criteria:

- Chunk-with-light and light-update packets decode through one reference implementation
  and match vanilla masks and arrays.
- Nibble probes at every coordinate parity and boundary land at the correct position.
- Opaque/emissive placement and removal match after quiescence at chunk/world edges.
- Vanilla sparse-light imports do not create dark columns.
- Save/restart preserves canonical light values.

## M5: Inventory transaction kernel

Complete and model-test every slot action before expanding item/container breadth:
left/right pickup, quick move, hotbar/offhand swap, clone, throw, quick craft, pickup
all, invalid indices, stale state IDs, cursor handling, component equality, close, and
disconnect. Add player, equipment, and ender-inventory persistence.

Exit criteria:

- Every slot action has exhaustive table tests.
- Property tests prove item/component conservation and stack-size invariants.
- Malformed or stale clicks cannot panic or duplicate items and trigger authoritative
  resynchronization.
- All inventory types survive save/restart.

## M6: Shared items and effects

Centralize consumption, Creative exceptions, durability and break notification,
container remainders, cooldowns, food, projectiles, potion payloads, and entity
classification. Complete milk, tipped/spectral arrows, bucket-contained mobs, fishing,
fireworks, tools, and all generated component codecs.

Exit criteria:

- Each item-behavior family has unit and differential interaction tests.
- Survival consumes exactly once; Creative preserves items where vanilla does.
- Milk clears effects and yields the correct remainder.
- Tipped arrows preserve components through bow/crossbow, entity serialization, and hit.
- Instant effects invert for every undead type.
- Durability, cooldown, and save/reload behavior match.

## M7: Blocks, fluids, and redstone

Audit behavior families rather than identifiers: placement/state preservation, support,
neighbor updates, scheduled/random ticks, fluids/waterlogging, loot, multiblocks,
weather/light, and block-entity persistence. Then cover wire/update order, torches,
repeaters, comparators, observers, pistons, rails, plates/buttons/targets, daylight
detectors, sensors, hoppers, dispensers/droppers, and crafters.

Exit criteria:

- Every block/state round-trips through network and disk representations.
- Family matrices cover orientation, waterlogging, support, drops, updates, and restart.
- Exact-tick redstone fixtures match pulse timing and update order.
- Pistons, rails, comparators, hoppers, and automation preserve item components and
  block-entity state.
- Accelerated stochastic fixtures match vanilla distributions with declared confidence.

## M8: Entity foundation and mob families

Finish shared movement, collision, fluids, fire/fall, attributes, combat, effects,
projectiles, metadata, equipment, drops/XP, riding, and NBT before mob-specific AI.
Then complete pathfinding, doors, partial blocks, swimming, line of sight, target
predicates, goals, breeding/taming, villages/trading, aquatic/flying mobs, and bosses.

Exit criteria:

- Every entity type has spawn/despawn, metadata, and NBT-round-trip coverage.
- Every claimed-complete mob has a deterministic oracle trace for movement, target,
  attack, drops, and XP.
- Goal priority, Creative/spectator filtering, doors/cache invalidation, partial blocks,
  line of sight, potion projectiles, and riding have shared regression suites.
- Scaffold-only entity types remain explicitly partial and score as failures.

## M9: Deterministic world generation

Close parity stage by stage: noise router/aquifers/ore veins/biomes/terrain/heightmaps,
surface rules and carvers, configured/placed features, structures and references,
block entities/loot/post-processing, flat/custom dimensions, and blending. Preserve
exact RNG derivation and order under parallel generation.

Exit criteria:

- Fixed seed/chunk corpora in all dimensions match exact blocks, fluids, biomes,
  heightmaps, structures, references, loot seeds, and block-entity NBT.
- Results are identical at worker counts 1, 2, 4, and maximum and independent of chunk
  request order.
- Mid-generation restart neither duplicates nor changes output.

## M10: Full command and configuration surfaces

Diff the official command-tree packet and implement every literal/argument leaf after
its backing subsystem exists. Compare parsers, suggestions, redirects, forks,
permissions, result values, feedback, and side effects. Map every official server
property to a Pumpkin key, intentional extension, or documented unsupported behavior;
provide a Java-only vanilla-26.2 profile.

Exit criteria:

- Every vanilla command node and property is classified.
- Implemented commands pass valid, invalid, boundary, sender, permission, dimension,
  and position cases.
- Every accepted configuration field has an observable consumer.
- Invalid configuration fails without panic or partial rewrite.
- Vanilla-profile defaults match official observable behavior.

## M11: Plugins and extensions

This is Pumpkin-contract parity, not vanilla parity. Add ownership to registered
handlers, commands, services, permissions, tasks, and resources. Make unload
transactional, version the native ABI safely, complete WIT behavior, and share a native
and WASM conformance suite.

Exit criteria:

- Native and WASM conformance plugins produce equivalent documented observations.
- 1,000 load/unload cycles leave no callbacks, commands, tasks, or resources behind.
- Invalid handles and resource exhaustion cannot crash or stall the server.
- Incompatible plugins are rejected before executing code.

## M12: Integrated release qualification

Run survival lifecycles spanning join, exploration, mining, crafting, smelting, farming,
potions, combat, AI, trading, transport, automation, portals, death/respawn, saving,
restarting, and reconnecting. Include vanilla-imported and Pumpkin-generated worlds.

Exit criteria:

- Coverage/parity thresholds above are satisfied with every residual discrepancy listed.
- Critical deterministic cases pass repeated clean runs with no unexplained flake.
- Stateful fuzzing and soak tests find no panic, corruption, duplication/loss, stuck
  scheduler, or unbounded resource growth.
- Bedrock is scored separately because the Java oracle cannot validate it.

## Implementation discipline

Each behavior fix is an isolated, cherry-pickable commit with a minimized regression
case. Independent files may be implemented in parallel; shared kernels and generated
data changes are serialized. Every discovered bug first becomes a failing conformance
cell, then a fix. Roadmap and tracker status are derived from executable evidence to
avoid drift from the current branch.
