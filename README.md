<div align="center">

# Pumpkin

![CI](https://github.com/Pumpkin-MC/Pumpkin/actions/workflows/rust.yml/badge.svg)
[![Discord](https://img.shields.io/discord/1268592337445978193.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/wT8XjrjKkf)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-yellow.svg)](https://opensource.org/licenses/gpl-3-0)

</div>

[Pumpkin](https://pumpkinmc.org/) is a Minecraft server built entirely in Rust, offering a fast, efficient,
and customizable experience. It prioritizes performance and player enjoyment while adhering to the core mechanics of the game.
<div align="center">

![Pumpkin Chunk Loading](./assets/pumpkin-chunk-loading.webp)

</div>

## Goals

- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Java & Bedrock Minecraft server version while adhering to Vanilla game mechanics.
- **Security**: Prioritizes security by preventing known security exploits.
- **Flexibility**: Highly configurable, with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.

> [!IMPORTANT]
> Pumpkin is currently under heavy development.
>
> [See what needs to be done before the 1.0.0 Release](https://github.com/Pumpkin-MC/Pumpkin/issues/449)

## Features

- [x] Configuration (toml)
- [Tracking: Protocol](https://github.com/Pumpkin-MC/Pumpkin/issues/1401)
  - [x] Server Status/Ping
  - [x] Encryption
  - [x] Packet Compression
  - [x] Java Edition
  - [x] Bedrock Edition (W.I.P)
  - ...
- [Tracking: World](https://github.com/Pumpkin-MC/Pumpkin/issues/1403)
  - [x] Player Tab-list
  - [x] Scoreboard
  - [x] World Loading
  - [x] World Time
  - [x] World Borders
  - [x] World Saving
  - [x] Lighting
  - [x] Entity Spawning
  - [x] Bossbar
  - [x] Chunk Loading (Vanilla, Linear, Pump)
  - [Chunk Generation](https://github.com/Pumpkin-MC/Pumpkin/issues/36)
  - [x] Chunk Saving (Vanilla, Linear, Pump)
  - [Redstone](https://github.com/Pumpkin-MC/Pumpkin/issues/1402)
  - [x] Liquid Physics
  - ...
- [Tracking: Player](https://github.com/Pumpkin-MC/Pumpkin/issues/1405)
  - [x] Skins
  - [x] Teleport
  - [x] Movement
  - [x] Animation
  - [x] Inventory
  - [Combat](https://github.com/Pumpkin-MC/Pumpkin/issues/1404)
  - [x] Experience
  - [x] Hunger
  - [X] Off Hand
  - [X] Advancements (W.I.P)
  - [x] Eating
  - ...
- Entities
  - [x] Non-Living (Minecart, Eggs...) (W.I.P)
  - [x] Entity Effects
  - [x] Players
  - [x] Mobs (W.I.P)
  - [x] Animals (W.I.P)
  - [Entity AI](https://github.com/Pumpkin-MC/Pumpkin/issues/1406)
  - [x] Boss (W.I.P)
  - [x] Villagers (W.I.P)
  - [X] Entity Saving
- Server
  - [Plugins](https://github.com/Pumpkin-MC/Pumpkin/issues/1407)
  - [x] Query
  - [x] RCON
  - [x] Inventories
  - [x] Particles
  - [x] Chat
  - [Commands](https://github.com/Pumpkin-MC/Pumpkin/issues/15)
  - [x] Permissions
  - [x] Translations
- Proxy
  - [x] [BungeeCord](https://github.com/SpigotMC/BungeeCord)
  - [x] [BungeeGuard](https://github.com/lucko/BungeeGuard)
  - [x] [Velocity](https://github.com/PaperMC/Velocity)

<!-- Check out our [Github Project](https://github.com/orgs/Pumpkin-MC/projects/3) to see current progress. -->

## How to run

See our [Quick Start](https://docs.pumpkinmc.org/#quick-start) guide to get Pumpkin running.

## Contributions

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Docs

Pumpkin's documentation can be found at <https://pumpkinmc.org/>

## Communication

Consider joining [our Discord server](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

## Funding

If you want to fund me and help the project, check out the [Donation Page](https://pumpkinmc.org/donate/).

## License & Attribution

* **Pumpkin Server**: Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
* **Plugin API (`pumpkin-plugin-api` & `pumpkin-plugin-wit`)**: Dual-licensed under [MIT](crates/pumpkin-plugin-api/LICENSE-MIT) OR [Apache-2.0](crates/pumpkin-plugin-api/LICENSE-APACHE) for maximum flexibility when writing plugins.
* **Third-Party Assets & Data**: Bedrock mappings, protocol conversion data, and Minecraft assets are subject to their respective licenses and attribution terms. See [assets/NOTICE.md](assets/NOTICE.md) for full details.


## 🌐 Web Resources & Interactive Index
- [INDEX3](https://quizverses-9d2f2.web.app/index3.html)
- [COLOR WAVEE](https://thelearnquesters.pages.dev/color-wavee.html)
- [DEAR ISLAND](https://studyplaying.github.io/dear-island.html)
- [ROAD CHASE SHOOTER REALISTIC GUNS](https://quizverses.pages.dev/road-chase-shooter-realistic-guns.html)
- [PICK BRAINROT 3D BATTLE](https://learnquester.github.io/pick-brainrot-3d-battle.html)
- [HEXA SORT TRICK OR TREAT](https://studyplayings.pages.dev/hexa-sort-trick-or-treat.html)
- [FIGHT TO THE END](https://quizverses.github.io/fight-to-the-end.html)
- [2048 DROP MERGE](https://thelearnquester.web.app/2048-drop-merge.html)
- [SLINKY COLOR SORT](https://learnquester.github.io/slinky-color-sort.html)
- [GOBATTLEIO](https://studyplaying.github.io/gobattleio.html)
- [CATEGORY SOLITAIRE](https://studyquesthub.web.app/category-solitaire.html)
- [EARWAX CLINIC](https://quizverses.github.io/earwax-clinic.html)
- [ONLINE PORTAL](https://learnquester.pages.dev/)
- [GOMU GOMAN](https://quizverses-9d2f2.web.app/gomu-goman.html)
- [SHAPE TRANSFORM RACE](https://studyplayings.web.app/shape-transform-race.html)
- [BATTLE TANKS FIRESTORM](https://ilearnworldjp.pages.dev/battle-tanks-firestorm.html)
- [MONSTER GIRLS BACK TO SCHOOL](https://thelearnquester.web.app/monster-girls-back-to-school.html)
- [CATEGORY CARTOON76](https://quizverses-9d2f2.web.app/category-cartoon76.html)
- [MAHJONG RIDDLES EGYPT](https://quizverses.github.io/mahjong-riddles-egypt.html)
- [CATEGORY THINKY 2](https://learnquester.pages.dev/category-thinky-2.html)
- [MATCH 3D PUZZLE SAGA](https://quizverses.github.io/match-3d-puzzle-saga.html)
- [CATEGORY CASUAL 7](https://studyquesthub.web.app/category-casual-7.html)
- [GOODS SORTING SHOPPING MASTER](https://studyplayings.pages.dev/goods-sorting-shopping-master.html)
- [MINI GOLF SAGA](https://studyplaying.github.io/mini-golf-saga.html)
- [FINGER SOCCER TOURNAMENT](https://quizverses-9d2f2.web.app/finger-soccer-tournament.html)
- [CLAP CLAP NIGHTMARE](https://quizverses-9d2f2.web.app/clap-clap-nightmare.html)
- [HEXA GO](https://quizverses.github.io/hexa-go.html)
- [PALM ISLAND SOLITAIRE](https://studyplayings.web.app/palm-island-solitaire.html)
- [CATEGORY SURVIVAL](https://studyquests.github.io/category-survival.html)
- [INDEX32](https://thelearnquesters.pages.dev/index32.html)
- [CATEGORY BRAIN261](https://quizverses.pages.dev/category-brain261.html)
- [GEOMETRY VIBES MONSTER](https://studyquesthub.web.app/geometry-vibes-monster.html)
- [CUTE CRAFT LAB](https://studyquests.github.io/cute-craft-lab.html)
- [MY CASTLE MERGE STORY](https://studyplayings.web.app/my-castle-merge-story.html)
- [PRINCESS DRESS UP RUN](https://studyquests.pages.dev/princess-dress-up-run.html)
- [CATEGORY ANIMAL216](https://thelearnquester.web.app/category-animal216.html)
- [NESTING DOLLS](https://quizverses.github.io/nesting-dolls.html)
- [IDLE DICE 3D INCREMENTAL GAME](https://quizverses.github.io/idle-dice-3d-incremental-game.html)
- [HIPPO SUPERMARKET](https://learnquester.pages.dev/hippo-supermarket.html)
- [CATEGORY SNAKE](https://quizverses.pages.dev/category-snake.html)
- [PEOPLE PLAYGROUND 3D](https://quizverses.github.io/people-playground-3d.html)
- [SWORD LIFE](https://quizverses-9d2f2.web.app/sword-life.html)
- [COLOR DOTS CHALLENGE](https://quizverses-9d2f2.web.app/color-dots-challenge.html)
- [CATEGORY BRAIN261](https://thequizzone.pages.dev/category-brain261.html)
- [BREAK BEAT](https://iskillquest.pages.dev/break-beat.html)
- [GEOMETRY ARROW 2](https://quizverses-9d2f2.web.app/geometry-arrow-2.html)
- [FLYORDIEIO](https://iskillquest.pages.dev/flyordieio.html)
- [COE SNAKE](https://thelearnquesters.pages.dev/coe-snake.html)
- [POTION MERGE WITCH](https://themindplay.github.io/potion-merge-witch.html)
- [VALLEY OF WOLVES AMBUSH](https://learnquester.pages.dev/valley-of-wolves-ambush.html)
- [CATEGORY 3D1 371](https://studyquesthub.web.app/category-3d1-371.html)
- [CHARGER CITY DRIVER](https://theskillquest.pages.dev/charger-city-driver.html)
- [CATEGORY MOUSE1 707](https://learnquesters.pages.dev/category-mouse1-707.html)
- [CELEBRITY FACE DANCE](https://quizverses-9d2f2.web.app/celebrity-face-dance.html)
- [PRINXY WINTERELLA](https://thelearnquesters.pages.dev/prinxy-winterella.html)
- [CATEGORY TRAIN YOUR BRAIN24](https://iskillquest.pages.dev/category-train-your-brain24.html)
- [SAVE LITTLE RED HOOD](https://learnquester.github.io/save-little-red-hood.html)
- [BLOCKY ARCHER RUN](https://theskillquest.pages.dev/blocky-archer-run.html)
- [CATEGORY OBSTACLE299](https://iskillquest.pages.dev/category-obstacle299.html)
- [TIMEWARRIORS](https://studyplaying.github.io/timewarriors.html)
- [BALL SORT COLOR PUZZLE](https://iskillquest.pages.dev/ball-sort-color-puzzle.html)
- [TRAIN MASTER](https://themindzone.pages.dev/train-master.html)
- [CATEGORY OBBY56](https://thelearnquesters.pages.dev/category-obby56.html)
- [MURDER STONE AGE](https://studyplaying.github.io/murder-stone-age.html)
- [CATEGORY MANAGEMENT210](https://studyplaying.github.io/category-management210.html)
- [CATEGORY SPACE57](https://learnquesters.pages.dev/category-space57.html)
- [OIL DIGGING](https://studyplayings.pages.dev/oil-digging.html)
- [CHIBI DOLL AVATAR CREATOR](https://studyquests.github.io/chibi-doll-avatar-creator.html)
- [CATEGORY CASUAL 6](https://themindplay.pages.dev/category-casual-6.html)
- [FUSION 2048](https://studyquests.github.io/fusion-2048.html)
- [CUBE STORIES ESCAPE](https://studyplayings.pages.dev/cube-stories-escape.html)
- [PUZZLE BLOCKS FILL IT COMPLETELY](https://iskillquest.pages.dev/puzzle-blocks-fill-it-completely.html)
- [FILL SORT PUZZLE](https://iskillquest.pages.dev/fill-sort-puzzle.html)
- [COSMO PET STARRY CARE](https://learnquester.pages.dev/cosmo-pet-starry-care.html)
- [SQUARE PUNKI LONG HAND](https://quizverses.github.io/square-punki-long-hand.html)
- [BRAIN TEST IQ CHALLENGE 2](https://themindzone.pages.dev/brain-test-iq-challenge-2.html)
- [KINGS AND QUEENS MAHJONG](https://thelearnquesters.pages.dev/kings-and-queens-mahjong.html)
- [EAT BLOBS SIMULATOR](https://studyplayings.pages.dev/eat-blobs-simulator.html)
- [IBIZA FOAM PARTY](https://iskillquest.pages.dev/ibiza-foam-party.html)
- [MOJO MATCH 3D](https://studyquests.github.io/mojo-match-3d.html)
- [MR MACAGI ADVENTURES](https://iskillquest.pages.dev/mr-macagi-adventures.html)
- [CATEGORY EDUCATIONAL](https://themindplay.pages.dev/category-educational.html)
- [BLOCK DIGGER](https://theskillquest.pages.dev/block-digger.html)
- [PIN PUZZLE LOVE STORY](https://iskillquest.pages.dev/pin-puzzle-love-story.html)
- [CATEGORY CRAFTING45](https://studyplaying.github.io/category-crafting45.html)
- [COOL MAN](https://studyquests.pages.dev/cool-man.html)
- [UNBLOCK IT 3D](https://quizverses.github.io/unblock-it-3d.html)
- [POP STAR](https://themindplay.github.io/pop-star.html)
- [MIND GAMES MATH CROSSWORDS](https://learnquester.github.io/mind-games-math-crosswords.html)
- [ULTIMATE ROBO DUEL 3D](https://iskillquest.pages.dev/ultimate-robo-duel-3d.html)
- [DUSTY CAT](https://themindzone.pages.dev/dusty-cat.html)
- [CATEGORY SIDE SCROLLING184](https://thelearnquesters.pages.dev/category-side-scrolling184.html)
- [ZOMBIE FRONTIER SHOOTER](https://thelearnquester.web.app/zombie-frontier-shooter.html)
- [INDEX38](https://studyquests.github.io/index38.html)
- [CATEGORY CARDS](https://studyquesthub.web.app/category-cards.html)
- [REALDRIVE FEEL THE REAL DRIVE](https://iskillquest.pages.dev/realdrive-feel-the-real-drive.html)
- [CLEAN HOUSE CLEARING TRASH AND DIRT](https://quizverses.github.io/clean-house-clearing-trash-and-dirt.html)
- [DUNGEONS N DUCKS](https://studyplaying.github.io/dungeons-n-ducks.html)
- [MIND GAMBIT](https://themindplay.github.io/mind-gambit.html)
- [CATEGORY IDLE445](https://thelearnquesters.pages.dev/category-idle445.html)
- [CATEGORY CASUAL 4](https://themindplay.pages.dev/category-casual-4.html)
- [HOUSE OF CELESTINA](https://themindplay.github.io/house-of-celestina.html)
- [SANTA GO](https://quizverses.github.io/santa-go.html)
- [CHECKERS](https://iskillquest.pages.dev/checkers.html)
- [BLOXORZ BLOCK PUZZLE 3D](https://studyplayings.web.app/bloxorz-block-puzzle-3d.html)
- [CATEGORY BOOKMARKLETS](https://iskillquest.pages.dev/category-bookmarklets.html)
- [INDEX20](https://iskillquest.pages.dev/index20.html)
- [BUBBLE RUSH](https://iskillquest.pages.dev/bubble-rush.html)
- [MONSTER TRUCK CRUSH](https://studyplaying.github.io/monster-truck-crush.html)
- [TUNG TUNG SAHUR OBBY CHALLENGE](https://theskillquest.pages.dev/tung-tung-sahur-obby-challenge.html)
- [SINGLE STROKE ENERGY LINE PUZZLE](https://themindzone.pages.dev/single-stroke-energy-line-puzzle.html)
- [CATEGORY FPS](https://themindplay.pages.dev/category-fps.html)
- [V AND N PIZZA COOKING GAME](https://thelearnquester.web.app/v-and-n-pizza-cooking-game.html)
- [BLOCK TNT BLAST](https://thelearnquesters.pages.dev/block-tnt-blast.html)
- [CLASH OF STONE](https://iskillquest.pages.dev/clash-of-stone.html)
- [ROBOT BAND FIND THE DIFFERENCES](https://themindzone.pages.dev/robot-band-find-the-differences.html)
- [CATEGORY COLOR197](https://thelearnquester.web.app/category-color197.html)
- [INDEX35](https://studyplaying.github.io/index35.html)
- [REAL RACING 3D](https://thelearnquester.web.app/real-racing-3d.html)
- [MAKEUP TRENDS THEN AND NOW](https://studyquests.github.io/makeup-trends-then-and-now.html)
- [ALIEN INTELLIGENCE TEST](https://themindplay.github.io/alien-intelligence-test.html)
- [CATEGORY JIGSAW](https://studyplaying.github.io/category-jigsaw.html)
- [BUS PARKING OUT](https://thelearnquesters.pages.dev/bus-parking-out.html)
- [ROBLOX CRAFT RUN](https://themindzone.pages.dev/roblox-craft-run.html)
- [GRANNY 2 ASYLUM HORROR HOUSE](https://theskillquest.pages.dev/granny-2-asylum-horror-house.html)
- [BUBBLE BLITZ GALAXY](https://theskillquest.pages.dev/bubble-blitz-galaxy.html)
- [DRAW CLIMB RACE THE ULTIMATE HILL CLIMBING CHALLENGE](https://iskillquest.pages.dev/draw-climb-race-the-ultimate-hill-climbing-challenge.html)
- [SNEAKY FRIENDS](https://iskillquest.pages.dev/sneaky-friends.html)
- [CATEGORY BRAIN](https://learnquesters.pages.dev/category-brain.html)
- [CATEGORY CAR 2](https://themindplay.pages.dev/category-car-2.html)
