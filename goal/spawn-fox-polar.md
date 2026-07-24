# Fox / Polar bear AI + natural spawn (26.2)

## AI status

| Mob | AI? | Notes |
|---|---|---|
| **Fox** | **Yes** | Swim, panic, breed, avoid player/wolf/polar bear, leap, melee, follow parent, wander, look; targets chicken/rabbit/fish. Missing: stalk, sleep, eat berries, trust. |
| **Polar bear** | **Yes** | Swim, melee 1.25, panic, follow parent, wander, look; revenge + hunt fox. Missing: cub-player aggro, NeutralMob anger timer. |

Sources: CFR `Fox.java`, `PolarBear.java` under `/tmp/mc26.2-src`.

## Natural spawn (already supported)

Pipeline:

1. World tick → every **400 ticks** creatures (`spawn_passives`), monsters when not peaceful + gamerules
2. `get_filtered_spawning_categories` → `spawn_for_chunk` → biome `spawners.*`
3. Chunk gen: `spawn_mobs_for_chunk_generation` for creatures

### Polar bear biomes (data)

- `snowy_plains`, `ice_spikes`, `frozen_ocean`, `deep_frozen_ocean` (creature list)

### Polar bear spawn rules (CFR)

```text
checkPolarBearSpawnRules:
  if biome in POLAR_BEARS_SPAWN_ON_ALTERNATE_BLOCKS:
    light + stand on ice (POLAR_BEARS_SPAWNABLE_ON_ALTERNATE)
  else:
    Animal.checkAnimalSpawnRules (grass-like + light > 8)
```

Pumpkin now allows **ice / packed ice / blue ice / frosted ice / snow block / grass-like** underfoot for polar bears, plus light > 8.

### Why ice plains felt empty

1. Creatures only attempt ~every 400 ticks  
2. Polar bear is large (1.4×1.4) — space checks fail often  
3. Previously no ice underfoot allowed for creature rules (only grass list) — **fixed**  
4. Need gamerule `spawn_mobs` true  

## Fox biomes

Fox appears in taiga / snowy taiga / grove etc. creature lists (see `biome.rs` `minecraft:fox`).
