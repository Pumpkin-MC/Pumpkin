# Pumpkin Development Progress (English)

> 简体中文版本：[PROGRESS.zh_cn.md](PROGRESS.zh_cn.md)
> Updated: 2026-07-26 · Active branch: `fix/gameplay-ai-spawn-net`

## What this project is

Pumpkin is a high-performance Minecraft server written in Rust:

- **Vanilla parity first**: every gameplay behavior is grounded in the official 26.2 decompiled source (`/root/Vanilla`) — constants, conditions, and ordering are mirrored one-to-one and cited (file + line) in comments. Where infrastructure is still missing, the gap is stated explicitly in doc comments instead of shipping invented values.
- **Performance track**: Rust without GC, atomic state, allocation-conscious code; preventing memory leaks is a hard requirement.
- **Both editions**: Java Edition and Bedrock connect; game content always follows Java Edition.
- **Version policy**: gameplay connections are pinned to protocol **26.2** (older clients are cleanly refused at handshake); status pings answer for any version.
- **Workflow**: multiple branches merge into `fix/gameplay-ai-spawn-net`; no local builds — every push is verified by GitHub Actions (clippy with `-Dwarnings` + ARM64 release build).

## System completeness

| System | Status | Notes |
|---|---|---|
| Mob AI | ~85% | 75+ mobs registered with vanilla goal structure; door breaking/opening, reinforcements, zombification, caravans, fish schools, schedules landed |
| Redstone | ~90% | Wire propagation, pistons, repeater/comparator, observer, sensors, full click sounds |
| Vibration/Sculk | ~90% | Frequency table, wool occlusion, distance power, calibrated filter, 17 emit points; travel delay & resonance pending |
| Mob spawning | ~90% | Per-biome weighted pools + spawn costs + local caps; baby zombies / chicken jockeys / reinforcements |
| Biomes/Worldgen | ~85% | Noise terrain, carvers, villages in five biomes, ancient city, mineshaft, stronghold, fortress, end city, mansion, trial chambers |
| Combat | ~85% | Attack cooldown, crits, sweeping, shields (axe disable, vanilla durability rule), knockback |
| Networking | stable | Per-tick block acks, chunk batching, block entity filtering; single-version 26.2 |
| Persistence | stable | Base NBT round-trips for every mob; feature state (zombification, door breaking, babies, villager data) persisted |
| Commands | ~80% | Major commands work; output uses client translation keys (Chinese clients see Chinese automatically) |
| Chinese support | good | Bilingual server console; Simplified-Chinese config template |

## Mob status highlights

- **Zombie family** (zombie/husk/drowned/zombie villager): door breaking (Hard), night village pressure, reinforcements (Hard), 5% babies with +50% speed, chicken jockeys, full villager infection ↔ golden-apple cure loop, drowned trident ranged attack
- **Skeleton family**: bow-draw pose, sun avoidance, stray slowness arrows, bogged poison arrows
- **Piglins/brutes/hoglins**: 300-tick overworld zombification (vanilla equipment semantics), door opening, bartering, group anger
- **Villagers**: trading/restock/breeding/gossip/job-block binding, bed claiming and sleep, scheduled walks to bed and job site, door use
- **Neutral/passive**: wolves (taming/defense/variants), cats, foxes, pandas, goats, bees, axolotls, llama **caravans**, schooling fish, wandering trader night invisibility, and more
- **Bosses/special**: ender dragon (phase machine), wither, warden (sonic boom), shulker, creaking, breeze

## Known gaps (by priority)

1. **Horse/donkey/mule riding & taming** — needs the saddle equipment slot plus client steering protocol as one coherent block
2. **Raids** (captains, patrols, bells)
3. Village POI graph (currently anchored on villagers' claimed beds)
4. Vibration travel delay (1 block/tick) and amethyst resonance
5. Trader-llama caravan state, pig riding, full beehive cycle
6. Enderman light-dependent magic value (sky light stands in for now)

## Recent changes (2026-07-26 batch)

One-block step-jump pathfinding fix (fleeing mobs clear ledges), door interaction goal family, zombie reinforcements, baby zombies + chicken jockeys, piglin-family zombification, drowned tridents, llama caravans, fish schools, villager schedules, wandering trader invisibility (vanilla potion duration/sounds), enderman rain damage and daylight teleport (vanilla formula), creeper lingering cloud, sculk vibration system, full interaction-block click sounds, daylight detector night underflow fix, vanilla shield durability, persistence fix for 63 mobs, 26.2 version pin, upstream sync.
