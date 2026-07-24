# Vanilla 26.2 Redstone System — CFR ground truth

Decompiled **64** redstone-related Java sources under `/tmp/mc26.2-src`.
Total tree: **712** java files.

## Core packages

### `net.minecraft.world.level.redstone`
- `CollectingNeighborUpdater.java` (214 lines)
- `DefaultRedstoneWireEvaluator.java` (60 lines)
- `ExperimentalRedstoneUtils.java` (36 lines)
- `ExperimentalRedstoneWireEvaluator.java` (232 lines)
- `InstantNeighborUpdater.java` (49 lines)
- `NeighborUpdater.java` (96 lines)
- `Orientation.java` (201 lines)
- `Redstone.java` (12 lines)
- `RedstoneWireEvaluator.java` (61 lines)

### Key blocks
- `VANILLA_METHODS/redstone/SignalGetter.java` (120 lines)
- `net/minecraft/world/level/block/RedStoneWireBlock.java` (489 lines)
- `net/minecraft/world/level/block/DiodeBlock.java` (224 lines)
- `net/minecraft/world/level/block/ComparatorBlock.java` (233 lines)
- `net/minecraft/world/level/block/RepeaterBlock.java` (128 lines)
- `net/minecraft/world/level/block/ObserverBlock.java` (144 lines)
- `net/minecraft/world/level/block/RedstoneTorchBlock.java` (175 lines)
- `net/minecraft/world/level/block/PoweredRailBlock.java` (184 lines)
- `net/minecraft/world/level/block/BaseRailBlock.java` (375 lines)
- `net/minecraft/world/level/block/LightningRodBlock.java` (160 lines)
- `net/minecraft/world/level/block/LeverBlock.java` (179 lines)
- `net/minecraft/world/level/block/ButtonBlock.java` (211 lines)
- `VANILLA_METHODS/redstone/DefaultRedstoneWireEvaluator.java` (60 lines)
- `net/minecraft/world/level/redstone/ExperimentalRedstoneWireEvaluator.java` (232 lines)
- `net/minecraft/world/level/redstone/CollectingNeighborUpdater.java` (214 lines)
- `VANILLA_METHODS/redstone/NeighborUpdater.java` (96 lines)

## SignalGetter (canonical power API)

| Method | Meaning |
|---|---|
| `getSignal(pos, dir)` | Weak power from block at pos toward dir; if conductor, max with `getDirectSignalTo` |
| `getDirectSignal(pos, dir)` | Strong power |
| `getDirectSignalTo(pos)` | Max strong from all 6 neighbors into pos |
| `hasNeighborSignal(pos)` | Any neighbor `getSignal > 0` |
| `getBestNeighborSignal(pos)` | Max neighbor signal (cap 15) |
| `getControlInputSignal` | Side input for diodes / wire POWER / redstone block |

Default `BlockBehaviour.getSignal` → `ownSignal` (0 unless overridden).

## Wire power (DefaultRedstoneWireEvaluator)

```
target = getBlockSignal(pos)  // shouldSignal=false → getBestNeighborSignal
if target==15 return 15
return max(target, getIncomingWireSignal(pos))  // horizontal wires + diagonals, then -1
```

Pumpkin `calculate_power` mirrors this (block power without dust + wire max−1).

## Powered / Activator rail (PoweredRailBlock.updateState)

```
shouldPower = hasNeighborSignal(pos)
           || findPoweredRailSignal(forward, 0)
           || findPoweredRailSignal(backward, 0)
```
- Search depth < 8
- `isSameRailWithPower` only continues on **already POWERED** same-orientation rails
- Returns true if chain hits a rail with `hasNeighborSignal`
- **Only updates self**; neighbors recompute via block updates

## Lightning rod

- `ownSignal` / weak: 15 if powered (via default getSignal→ownSignal)
- Strong: 15 only if `facing == direction`
- Powered for 8 ticks after strike

## Pumpkin alignment notes

| Component | Status vs 26.2 CFR |
|---|---|
| Wire power formula | Aligned (calculate_power ≈ Default evaluator) |
| Wire turbo BFS | MCHPRS-based accelerator (MC-81098), not experimental evaluator |
| Diode/repeater/comparator structure | Present; item-frame analog still TODO |
| Powered/activator rail update | **Fixed**: self-only updateState cascade (was BFS over-propagate) |
| NeighborUpdater orientation | Not fully ported (Orientation / CollectingNeighborUpdater) |

## Method extracts

See `/tmp/mc26.2-src/VANILLA_METHODS/redstone/` and full sources under `/tmp/mc26.2-src/net/minecraft/world/level/{block,redstone}/`.
