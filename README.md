<div align="center">

# 🎃 Pumpkin

**A fast, efficient, and customizable Minecraft server written entirely in Rust.**

[![CI](https://github.com/Pumpkin-MC/Pumpkin/actions/workflows/rust.yml/badge.svg)](https://github.com/Pumpkin-MC/Pumpkin/actions)
[![Discord](https://img.shields.io/discord/1268592337445978193?label=Discord&logo=discord&logoColor=white&color=7389D8&labelColor=6A7EC2)](https://discord.gg/wT8XjrjKkf)
[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-yellow.svg)](https://opensource.org/licenses/gpl-3-0)
[![Website](https://img.shields.io/badge/Website-pumpkinmc.org-blue)](https://pumpkinmc.org/)

[Website](https://pumpkinmc.org/) • [Documentation](https://docs.pumpkinmc.org/#quick-start) • [Discord](https://discord.gg/wT8XjrjKkf) • [Donate](https://pumpkinmc.org/donate/)

---

![Pumpkin Chunk Loading](./assets/pumpkin-chunk-loading.webp)

</div>

> [!IMPORTANT]
> **Pumpkin is currently under heavy development.**  
> Track progress towards the official 1.0.0 milestone on [Issue #449](https://github.com/Pumpkin-MC/Pumpkin/issues/449).

---

##  Goals

* **🎃 Performance:** Built from the ground up leveraging Rust's multi-threading for maximum speed and low overhead.
* **🎃 Compatibility:** Full support for modern Java & Bedrock Edition clients while faithfully replicating Vanilla mechanics.
* **🎃 Security:** Proactively designed to mitigate common server exploits and vulnerabilities.
* **🎃 Flexibility:** Highly modular configuration—enable only the features your server needs.
* **🎃 Extensibility:** Comprehensive foundation and API for custom plugin development.

---

##  Features

| Category | Tracking / Issue | Included Features |
| :--- | :--- | :--- |
| **Protocol** | [#1401](https://github.com/Pumpkin-MC/Pumpkin/issues/1401) | Server Status/Ping, Encryption, Packet Compression, Java Edition, Bedrock Edition *(W.I.P)* |
| **World Engine** | [#1403](https://github.com/Pumpkin-MC/Pumpkin/issues/1403) | Player Tab-list, Scoreboard, World Loading & Saving, Time, World Borders, Lighting, Entity Spawning, Bossbar, Chunk Loading/Saving (Vanilla, Linear, Pump), [Chunk Generation](https://github.com/Pumpkin-MC/Pumpkin/issues/36), [Redstone](https://github.com/Pumpkin-MC/Pumpkin/issues/1402), Liquid Physics |
| **Player System** | [#1405](https://github.com/Pumpkin-MC/Pumpkin/issues/1405) | Skins, Teleportation, Movement, Animations, Inventory, [Combat](https://github.com/Pumpkin-MC/Pumpkin/issues/1404), Experience, Hunger, Off-Hand, Advancements *(W.I.P)*, Eating |
| **Entities** | — | Non-Living (Minecarts, Eggs, etc.) *(W.I.P)*, Entity Effects, Players, Mobs *(W.I.P)*, Animals *(W.I.P)*, [Entity AI](https://github.com/Pumpkin-MC/Pumpkin/issues/1406), Bosses *(W.I.P)*, Villagers *(W.I.P)*, Entity Saving |
| **Server Operations** | [#1407](https://github.com/Pumpkin-MC/Pumpkin/issues/1407) | Plugins, Query, RCON, Inventories, Particles, Chat, [Commands](https://github.com/Pumpkin-MC/Pumpkin/issues/15), Permissions, Translations, Config (`.toml`) |
| **Proxy Support** | — | [BungeeCord](https://github.com/SpigotMC/BungeeCord), [BungeeGuard](https://github.com/lucko/BungeeGuard), [Velocity](https://github.com/PaperMC/Velocity) |

---

##  Getting Started

* **Quick Start Guide:** Follow the step-by-step setup instructions on our [Documentation Site](https://docs.pumpkinmc.org/#quick-start).
* **Community:** Have questions or want to chat? Join our [Discord Server](https://discord.gg/wT8XjrjKkf).
* **Contributing:** Check out [CONTRIBUTING.md](CONTRIBUTING.md) to learn how to open issues or submit code.
* **Funding:** Help support ongoing development by visiting our [Donation Page](https://pumpkinmc.org/donate/).

---

##  License & Attribution

* **Pumpkin Server:** Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
* **Plugin API (`pumpkin-plugin-api` & `pumpkin-plugin-wit`):** Dual-licensed under [MIT](crates/pumpkin-plugin-api/LICENSE-MIT) OR [Apache-2.0](crates/pumpkin-plugin-api/LICENSE-APACHE).
* **Third-Party Assets & Data:** Bedrock mappings, protocol conversion data, and Minecraft assets are subject to their respective licenses. See [assets/NOTICE.md](assets/NOTICE.md) for full details.
