# Redstone Data Reference (1.21.4)

> Extracted 2026-02-07 from Pumpkin spec data + codebase analysis.
> Sources: `blocks.json` (1.21.4), `pumpkin/src/block/blocks/redstone/`, `pumpkin/src/block/blocks/piston/`

---

## Redstone Block States (Property Tables)

### Power Sources

| Block | Properties |
|---|---|
| **redstone_block** | (no properties -- always full power 15) |
| **redstone_torch** | lit: [true\|false] |
| **redstone_wall_torch** | facing: [N\|S\|W\|E], lit: [true\|false] |
| **lever** | face: [floor\|wall\|ceiling], facing: [N\|S\|W\|E], powered: [true\|false] |
| **stone_button** | face: [floor\|wall\|ceiling], facing: [N\|S\|W\|E], powered: [true\|false] |
| **oak_button** (+ all wood variants) | face: [floor\|wall\|ceiling], facing: [N\|S\|W\|E], powered: [true\|false] |
| **stone_pressure_plate** | powered: [true\|false] |
| **oak_pressure_plate** (+ all wood variants) | powered: [true\|false] |
| **heavy_weighted_pressure_plate** | power: [0-15] |
| **light_weighted_pressure_plate** | power: [0-15] |
| **daylight_detector** | inverted: [true\|false], power: [0-15] |
| **target** | power: [0-15] |
| **trapped_chest** | type: [single\|left\|right], facing: [N\|S\|W\|E], waterlogged: [true\|false] |
| **sculk_sensor** | power: [0-15], sculk_sensor_phase: [inactive\|active\|cooldown], waterlogged: [true\|false] |
| **calibrated_sculk_sensor** | facing: [N\|S\|W\|E], power: [0-15], sculk_sensor_phase: [inactive\|active\|cooldown], waterlogged: [true\|false] |
| **tripwire_hook** | attached: [true\|false], facing: [N\|S\|W\|E], powered: [true\|false] |
| **tripwire** | attached: [true\|false], disarmed: [true\|false], east/north/south/west: [true\|false], powered: [true\|false] |
| **observer** | facing: [N\|E\|S\|W\|up\|down], powered: [true\|false] |

### Power Transmission

| Block | Properties |
|---|---|
| **redstone_wire** | east: [up\|side\|none], north: [up\|side\|none], power: [0-15], south: [up\|side\|none], west: [up\|side\|none] |
| **repeater** | delay: [1-4], facing: [N\|S\|W\|E], locked: [true\|false], powered: [true\|false] |
| **comparator** | facing: [N\|S\|W\|E], mode: [compare\|subtract], powered: [true\|false] |

### Power Consumers

| Block | Properties |
|---|---|
| **redstone_lamp** | lit: [true\|false] |
| **piston** | extended: [true\|false], facing: [N\|E\|S\|W\|up\|down] |
| **sticky_piston** | extended: [true\|false], facing: [N\|E\|S\|W\|up\|down] |
| **piston_head** | type: [normal\|sticky], facing: [N\|E\|S\|W\|up\|down], short: [true\|false] |
| **dispenser** | facing: [N\|E\|S\|W\|up\|down], triggered: [true\|false] |
| **dropper** | facing: [N\|E\|S\|W\|up\|down], triggered: [true\|false] |
| **hopper** | enabled: [true\|false], facing: [down\|N\|S\|W\|E] |
| **note_block** | instrument: [23 variants], note: [0-24], powered: [true\|false] |
| **bell** | attachment: [floor\|ceiling\|single_wall\|double_wall], facing: [N\|S\|W\|E], powered: [true\|false] |
| **iron_door** | facing: [N\|S\|W\|E], half: [upper\|lower], hinge: [left\|right], open: [true\|false], powered: [true\|false] |
| **iron_trapdoor** | facing: [N\|S\|W\|E], half: [top\|bottom], open: [true\|false], powered: [true\|false], waterlogged: [true\|false] |
| **tnt** | unstable: [true\|false] |

### Rails (Redstone-Activated)

| Block | Properties |
|---|---|
| **powered_rail** | powered: [true\|false], shape: [north_south\|east_west\|ascending_*], waterlogged: [true\|false] |
| **detector_rail** | powered: [true\|false], shape: [north_south\|east_west\|ascending_*], waterlogged: [true\|false] |
| **activator_rail** | powered: [true\|false], shape: [north_south\|east_west\|ascending_*], waterlogged: [true\|false] |

---

## Current Implementation

### Redstone Directory: `pumpkin/src/block/blocks/redstone/`

| File | Lines | Description |
|---|---|---|
| redstone_wire.rs | 861 | Wire placement, connections, power propagation |
| turbo.rs | 421 | Turbo/optimized redstone engine |
| comparator.rs | 401 | Comparator logic (compare/subtract modes) |
| tripwire_hook.rs | 374 | Tripwire hook state + activation |
| activator_rail.rs | 568 | Activator rail mechanics |
| powered_rail.rs | 569 | Powered rail boost/brake logic |
| rails/mod.rs | 335 | Rail system shared logic |
| abstract_redstone_gate.rs | 332 | Shared base for repeater/comparator |
| repeater.rs | 312 | Repeater delay + locking |
| redstone_torch.rs | 308 | Torch/wall-torch logic + burnout |
| mod.rs | 253 | Module declarations + shared redstone helpers |
| rails/common.rs | 239 | Common rail utilities |
| dropper.rs | 231 | Dropper dispensing logic |
| tripwire.rs | 212 | Tripwire string detection |
| buttons.rs | 180 | All button types (stone + wood) |
| lever.rs | 162 | Lever toggle |
| observer.rs | 145 | Observer pulse detection |
| pressure_plate/weighted.rs | 139 | Weighted pressure plate (heavy/light) |
| rails/rail.rs | 128 | Basic rail |
| pressure_plate/plate.rs | 123 | Binary pressure plate |
| pressure_plate/mod.rs | 81 | Pressure plate module |
| copper_bulb.rs | 80 | Copper bulb (1.21 addition) |
| redstone_lamp.rs | 69 | Lamp on/off |
| dispenser.rs | 55 | Dispenser (partial -- likely delegates to dropper) |
| detector_rail.rs | 55 | Detector rail |
| redstone_block.rs | 22 | Constant power source |
| target_block.rs | 15 | Target block (stub) |
| **Total** | **6,670** | |

### Piston Directory: `pumpkin/src/block/blocks/piston/`

| File | Lines | Description |
|---|---|---|
| piston.rs | 540 | Push/pull logic, block movement |
| mod.rs | 212 | Piston module + shared utilities |
| piston_extension.rs | 35 | Moving piston (block 36) |
| piston_head.rs | 35 | Piston head block state |
| **Total** | **822** | |

**Grand total redstone implementation: ~7,492 lines**

---

## Gap Analysis

### Implemented (Functional)

- Redstone wire (full power propagation + turbo engine)
- Repeater (delay, locking, directionality)
- Comparator (compare + subtract modes)
- Redstone torch + wall torch (with burnout protection)
- Lever, buttons (all variants)
- Pressure plates (binary + weighted)
- Observer
- Piston + sticky piston (push/pull with block movement)
- Rails (powered, activator, detector, basic)
- Tripwire + tripwire hook
- Dropper, dispenser
- Redstone lamp, redstone block
- Copper bulb

### Partially Implemented / Stubs

- **target_block.rs**: 15 lines -- likely a stub, needs projectile hit detection + power output
- **dispenser.rs**: 55 lines -- may delegate to dropper but needs item-specific behaviors (fire charges, water buckets, armor equipping, etc.)
- **detector_rail.rs**: 55 lines -- basic structure, may need entity detection integration

### Not Implemented (Missing Files)

- **Sculk sensor / calibrated sculk sensor**: No files found in redstone directory. These need vibration detection, frequency filtering, and wool occlusion logic.
- **Daylight detector**: No dedicated file. Needs sky light sampling + inverted mode.
- **Note block**: Not in redstone directory (may be elsewhere). Needs instrument detection from block below + note cycling + powered playback.
- **Bell**: Not in redstone directory. Needs redstone activation to ring.
- **Iron door / iron trapdoor**: Likely handled in door/trapdoor generic code, but redstone activation path needs verification.
- **Trapped chest**: Redstone output based on viewer count -- no dedicated redstone file.
- **TNT**: Redstone ignition path may exist but no dedicated file in redstone/.
- **Hopper**: Lock/unlock via redstone not confirmed in this directory.

### Key Technical Gaps

1. **Quasi-connectivity (BUD powering)**: Unknown if implemented. Pistons/dispensers/droppers should activate from blocks above them.
2. **Tick scheduling**: Repeaters and comparators need scheduled tick support for proper delay behavior. Current implementation may use immediate updates.
3. **Redstone wire power loss**: Wire should lose 1 power level per block of travel (15 -> 14 -> ... -> 0).
4. **Comparator side input**: Comparators should read signal strength from containers and measure their fullness.
5. **Update order**: Vanilla Minecraft has specific block update ordering that affects redstone circuit behavior.
