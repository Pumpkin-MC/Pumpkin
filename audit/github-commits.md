# Pumpkin Commits History & Parity Audit

## Local Protected Commits (Ahead of upstream master)

- Total ahead commits: **9**
- Status: **100% PRESERVED & INTEGRATED**

| Commit Hash | Author | Date | Subject | Scope / Vanilla Parity Impact |
|-------------|--------|------|---------|-------------------------------|
| `8644e5fcfe` | sonng | 2026-10-15 | fix(entity): synchronize data tracker after reading item frame custom nbt | Local Custom & Parity Enhancement |
| `0c4ee8805a` | sonng | 2026-10-15 | feat(blocks): support loot table unpacking in decorated pots | Local Custom & Parity Enhancement |
| `412f55d6f7` | sonng | 2026-10-15 | feat(loot): add LootTable support to BarrelBlockEntity for structure parity | Local Custom & Parity Enhancement |
| `413a536cfd` | sonng | 2026-10-15 | fix(worldgen): restore village villagers and structure entities with vanilla parity | Local Custom & Parity Enhancement |
| `15f84b9ebc` | sonng | 2026-10-15 | fix(spawner): vanilla mob spawning parity, night light calculation, and mob cap | Local Custom & Parity Enhancement |
| `2280a4c65f` | sonng | 2026-10-15 | fix: sync upstream audit fixes (issues #3885 #3881 #3888, PRs #4067 #4061 #4060 #4058 #4055 #4053 #4042) and restore vanilla jigsaw parity | Local Custom & Parity Enhancement |
| `f9d2955381` | sonng | 2026-10-15 | fix: resolve piston 1M update recursion, client jitter desync, and upstream parity fixes | Local Custom & Parity Enhancement |
| `d9b72f6937` | sonng | 2026-10-15 | merge: update to upstream master while preserving all full parity fixes | Local Custom & Parity Enhancement |
| `7c0eb67158` | sonng | 2026-10-15 | feat: complete vanilla parity fixes (mob spawning, villager generation, mob AI, aquatic physics, pistons, commands, etc.) | Local Custom & Parity Enhancement |

## Upstream Commits Sample (Recent 100 commits)

| Commit Hash | Author | Date | Subject |
|-------------|--------|------|---------|
| `8644e5fcfe` | sonng | 2026-10-15 | fix(entity): synchronize data tracker after reading item frame custom nbt |
| `0c4ee8805a` | sonng | 2026-10-15 | feat(blocks): support loot table unpacking in decorated pots |
| `412f55d6f7` | sonng | 2026-10-15 | feat(loot): add LootTable support to BarrelBlockEntity for structure parity |
| `413a536cfd` | sonng | 2026-10-15 | fix(worldgen): restore village villagers and structure entities with vanilla parity |
| `15f84b9ebc` | sonng | 2026-10-15 | fix(spawner): vanilla mob spawning parity, night light calculation, and mob cap |
| `2280a4c65f` | sonng | 2026-10-15 | fix: sync upstream audit fixes (issues #3885 #3881 #3888, PRs #4067 #4061 #4060 #4058 #4055 #4053 #4042) and restore vanilla jigsaw parity |
| `f9d2955381` | sonng | 2026-10-15 | fix: resolve piston 1M update recursion, client jitter desync, and upstream parity fixes |
| `d9b72f6937` | sonng | 2026-10-15 | merge: update to upstream master while preserving all full parity fixes |
| `7c0eb67158` | sonng | 2026-10-15 | feat: complete vanilla parity fixes (mob spawning, villager generation, mob AI, aquatic physics, pistons, commands, etc.) |
| `5b53f064af` | Cayden Tisdale | 2026-10-14 | feat(plugin): put the v0.2 wasm host behind a plugin-v0_2 feature (#3852) |
| `618784535b` | Megalith | 2026-10-14 | fix(loot): cook mob drops when the mob burns or dies to fire aspect (#4040) |
| `de67cd357d` | Megalith | 2026-10-14 | fix(player): respawn hardcore players in spectator mode (#4041) |
| `9707ae4528` | SplashGG | 2026-10-14 | fix(entity): use vanilla suffocation predicates so leaves don't suffocate (#4049) |
| `7ceb6af513` | Megalith | 2026-10-14 | fix(entity): use the entity's own dimensions for poses and eye height (#4043) |
| `36f9341f27` | Megalith | 2026-10-13 | feat(block): implement conduit behavior (#4018) |
| `5d9aa5e074` | Megalith | 2026-10-13 | fix(block): correct stem growth and farmland updates (#4013) |
| `b0c48b2d76` | Alexander Medvedev | 2026-10-12 | fix: many generation mismatches |
| `a4e52c30e5` | Hypnos | 2026-10-11 | fix(inventory): keep spectators out of chests whose loot isn't rolled yet (#4016) |
| `7dac745662` | Missing_Love | 2026-10-12 | fix: decode 26.3 spectator actions as optional entity IDs (#3705) |
| `3ad20f428b` | Tom | 2026-10-11 | feat(block): add BlockBehaviour::attack hook for when a player starts digging (#3884) |
| `308a5dcc18` | Tom | 2026-10-11 | fix(block): add dragon egg world event on teleport (particles) (#3883) |
| `05319feab6` | Megalith | 2026-10-11 | fix(block): correct sea pickle bonemeal spread (#4012) |
| `e0cb7c01fb` | Megalith | 2026-10-10 | docs(contributing): tighten AI, capture and hardcoding rules (#4004) |
| `987b979d25` | Denys Shatin | 2026-10-10 | fix(entity): add attack animation for iron golem (#3967) |
| `d2eac55dab` | Cayden Tisdale | 2026-10-09 | fix(world): prevent recursive stack overflow during block updates (#3924) |
| `ac616e5a4f` | Saranxzi | 2026-10-10 | fix(block): hydrate farmland during rain (#3932) |
| `f175b3a514` | DylanClouden28 | 2026-10-09 | fix(block): let mob heads be placed on top of note blocks (#3966) |
| `9809468847` | Batuhan | 2026-10-10 | fix(world): validate chunk record and array lengths from region files (#3916) |
| `4a5cf54697` | Batuhan | 2026-10-10 | fix(net): gate login phase transitions on a completed login (#3912) |
| `d44f0c6a44` | Alexander Medvedev | 2026-10-08 | chore(chunk-gen): add more tests |
| `fc35283df9` | Umut Cevdet Koçak | 2026-10-08 | perf(data): look up structure metadata through a phf map (#3787) |
| `5ede4c217f` | Tom | 2026-10-08 | fix(block): add delay to dragon egg falling (#3882) |
| `5f0a165de4` | Mean | 2026-10-08 | fix(plugin): handle blocking round-trip for BlockDamageAbortEvent (#3898) |
| `1a6a6e6f15` | Alexander Medvedev | 2026-10-07 | feat(codegen): map NBT to nbt-tree in WIT |
| `634f21feb5` | Alexander Medvedev | 2026-10-07 | feat(protocol): implement block properties in Bedrock StartGame packet |
| `f1c0871f49` | xnacly | 2026-10-06 | fix(nix): fetch the vanilla server jar for the sandboxed build (#3892) |
| `3b1abcded5` | ydw1904 | 2026-10-06 | fix(locate): resolve /locate structure per structure (#3862) |
| `efdab1847a` | rennex | 2026-10-06 | fix(data): prevent map decoration NBT corruption (#3747) |
| `3df2a6b11c` | Mean | 2026-10-06 | fix(plugin): give block burn events a world and position (#3857) |
| `2c7931aa1f` | Tom | 2026-10-05 | fix(block): don't play break effects when leaves decay (#3828) |
| `b930e7db4a` | Tom | 2026-10-05 | fix(entity): require a mace for Over-overkill advancement (#3825) |
| `def9cad31f` | Mean | 2026-10-05 | fix(portal): write nether portal blocks without neighbour shape updates (#3842) |
| `36378f3981` | Kron | 2026-10-05 | fix(block): scale weighted pressure plate signal with entity count (#3877) |
| `d1c71a8588` | Kron | 2026-10-05 | fix(block): place copper chains along the clicked axis (#3870) |
| `6b3950c242` | Kyle Kovalchick | 2026-10-05 | fix(block): match vanilla daylight detector signal calculations (#3786) |
| `894f675cdb` | ydw1904 | 2026-10-05 | fix(entity): stop falling blocks from replacing blocks they can't land in (#3864) |
| `86683caa41` | Zinedin Polic | 2026-10-05 | fix(item-stacking-damage): vanilla dropped items + tag-based damage (#3711) |
| `93f63c33c5` | Alexander Medvedev | 2026-10-05 | fix: https://github.com/Pumpkin-MC/Pumpkin/issues/2472 |
| `34a85ae94a` | Alexander Medvedev | 2026-10-05 | feat(net): replace rcon with minecraft server management protocol |
| `b8fda56053` | Kyle Kovalchick | 2026-10-05 | perf(build): cut fat-LTO release link memory by 24% (#3866) |
| `8adc29feb5` | Kyle Davis | 2026-10-04 | feat(wit): add GameTest API definitions for v0.2 (#3770) |
| `a06daf06ac` | Kyle Kovalchick | 2026-10-03 | Fix perms on command aliases (#3801) |
| `a8db585158` | kedor | 2026-10-04 | fixing missing check inside progress_changed (#3856) |
| `53c29a967c` | Cayden Tisdale | 2026-10-03 | Merge pull request #3850 from Kron5620/fix/packet-error-log-level |
| `67abc63db0` | Kron | 2026-10-02 | fix(net): log play packet errors at their own severity |
| `1859221e7a` | Cayden Tisdale | 2026-10-01 | build: split pumpkin into core and wasm host crates and move full LTO releases to a nightly (#3824) |
| `742beaf6f9` | rennex | 2026-10-01 | fix(build): stop unnecessary pumpkin rebuilds (#3746) |
| `361c34c4dc` | Demetrius Kanios | 2026-09-29 | feat(plugin): Branch a new v0.2 plugin API from the v0.1 API (#3766) |
| `4426d1113a` | Kurenai | 2026-09-28 | fix(server): ensure hardcore difficulty consistency across worlds and clients (#3501) |
| `5a7714e8c4` | Matheus Tavares | 2026-09-27 | fix(command): accept score holders in execute if score (#3596) |
| `5353caa71d` | Matheus Tavares | 2026-09-27 | fix(data): read and write book content as filterable text (#3572) |
| `5746077df6` | Zinedin Polic | 2026-09-28 | fix(ai): NeutralMob anger, neutral targets, daylight sun-burn (#3568) |
| `b0371559e7` | rennex | 2026-09-27 | fix(config): support vanilla ops field names (#3736) |
| `7e01885e43` | Zinedin Polic | 2026-09-27 | feat(net): 50ms Java tick flush and vanilla 20 TPS (#3249) |
| `fc780ec7cb` | Zinedin Polic | 2026-09-26 | fix(tnt): vanilla parity for fuse, drag and priming (#3419) |
| `003d3c49ea` | KBDL | 2026-09-26 | fix(block): notify clients when blocks break on their own (#3722) |
| `fbe19575e6` | GFoniX | 2026-09-25 | fix(protocol): tag partial profiles as false in the profile component (#3731) |
| `a4d6465edb` | PigTurtle | 2026-09-25 | refactor(WASM): Unified resource handling & cleanup (#3258) |
| `9d7b325512` | Demetrius Kanios | 2026-09-25 | fix(deps): Revert bad Wasmtime update pending investigation (#3733) |
| `dbcfeeedf8` | Matheus Tavares | 2026-09-24 | fix(item): only open signed books through the server (#3570) |
| `467c7f8f3b` | Matheus Tavares | 2026-09-24 | fix(text): use Java translations for console and RCON output (#3709) |
| `8c2122b4c9` | xRookieFight | 2026-09-24 | refactor(bedrock): derive enum packet codecs and fix protocol naming (#3423) |
| `76b2ce3ca0` | Megalith | 2026-09-24 | docs: add AGENTS.md with guidelines for coding agents (#3714) |
| `868ffee0c8` | Alexander Medvedev | 2026-09-24 | chore: make more stuff more data driven |
| `10a2ad56b8` | Matheus Tavares | 2026-09-24 | feat(command): implement scoreboard players subcommands (#3595) |
| `d9c6878a16` | Luke Evanson | 2026-09-24 | fix(protocol): reject VarInt/VarUInt/VarLong/VarULong encodings with overflowing final-byte bits 🤖🤖🤖 (#3536) |
| `f5304d2465` | yAstro | 2026-09-24 | fix(plugin): stop the hot-reload watcher on shutdown (#3670) |
| `d9dd4c6297` | Cayden Tisdale | 2026-09-24 | fix(plugin): handle store shutdown race (#3727) |
| `b97ff75c1a` | Cayden Tisdale | 2026-09-23 | fix(docker): make data persistence reliable (#3692) |
| `bfd024ce80` | Matheus Tavares | 2026-09-23 | fix(command): accept namespaced function names and tags in /function (#3597) |
| `d69bbfb88f` | Cayden Tisdale | 2026-09-23 | feat(scheduler): add stackless global domain scheduling (#3453) |
| `4151d17ca1` | Megalith | 2026-09-23 | perf(world): allocate proto chunk blocks only up to the highest non-air section (#3710) |
| `43bcd7b108` | Zinedin Polic | 2026-09-22 | fix(entity resync): vanilla sendChanges resync through the entity tracker (#3401) |
| `7c657ea2b0` | Matheus Tavares | 2026-09-22 | fix(world): read chunks without yPos (#3488) |
| `5d3d634e63` | Bryan | 2026-09-22 | refactor(net): remove the dead java config module (#3207) |
| `8808493cfb` | Alexander Medvedev | 2026-09-20 | ci: make sign seperate step |
| `204a94ed89` | Alexander Medvedev | 2026-09-20 | chore: bump version to 0.2.0+26.3-26.51 |
| `9bab5ea4f5` | Alexander Medvedev | 2026-09-20 | chore: extract compostable, cooking_fuel and use_remainder from data components |
| `e413623e9d` | Megalith | 2026-09-20 | fix(sculk): port shrieker warning and warden summon flow (#3599) |
| `5ab64781bc` | Cayden Tisdale | 2026-09-19 | fix(ci): allow builds after skipped optional check (#3604) |
| `70c39335f9` | Alexander Medvedev | 2026-09-19 | refactor: move multi-version to pumpkin-java-multiversion |
| `891345af2c` | Cayden Tisdale | 2026-09-19 | fix: cancel test CI jobs correctly (#3598) |
| `3889be7a3b` | Cayden Tisdale | 2026-09-19 | ci: Optimize CI for Pull Requests w/ affected checks only (#3445) |
| `218a7fe467` | Alexander Medvedev | 2026-09-19 | feat: add posteffect command |
| `12f5687629` | Alexander Medvedev | 2026-09-19 | feat: add compute command |
| `8813176fdd` | Alexander Medvedev | 2026-09-19 | feat: add swing command |
| `c31eb83616` | Thijs226 | 2026-09-19 | fix(ai): make creepers avoid cats (#3481) |
| `6a2d83389d` | Wi-Flow | 2026-09-19 | feat(entity): add the vanilla brain framework (#3490) |
| `04d02761f2` | Wi-Flow | 2026-09-19 | fix(world): save entity chunks as a snapshot instead of appending (#3494) |
| `9f1eb74412` | ZlordHUN | 2026-09-19 | Reduce Bedrock ICE datagram log noise (#3579) |
