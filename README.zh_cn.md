<div align="center">

# Pumpkin

![CI](https://github.com/Pumpkin-MC/Pumpkin/actions/workflows/rust.yml/badge.svg)
[![Discord](https://img.shields.io/discord/1268592337445978193.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/wT8XjrjKkf)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-yellow.svg)](https://opensource.org/licenses/gpl-3-0)

**简体中文** · [English](README.en_us.md) · [开发进度](PROGRESS.zh_cn.md)

</div>

[Pumpkin](https://pumpkinmc.org/) 是一个完全用 Rust 编写的 Minecraft 服务端，追求高性能、可定制，并在核心机制上贴近原版体验。

<div align="center">

![Pumpkin 区块加载](./assets/pumpkin-chunk-loading.webp)

</div>

## 目标

- **性能**：充分利用多线程，追求速度与效率的上限。
- **兼容**：支持最新的 Java 版与基岩版协议（本分支正式连接固定为 26.2），玩法内容以 Java 版原版机制为准。
- **安全**：默认防范已知安全漏洞。
- **灵活**：高度可配置，可关闭不需要的功能。
- **可扩展**：为插件开发提供基础。

> [!IMPORTANT]
> Pumpkin 仍在密集开发中。
>
> [1.0.0 发布前的待办清单](https://github.com/Pumpkin-MC/Pumpkin/issues/449)
>
> 本分支的详细进度见 [PROGRESS.zh_cn.md](PROGRESS.zh_cn.md)。

## 语言

控制台语言在 `pumpkin.toml` 中配置：

```toml
[logging]
# en_us | zh_cn | zh_en（中英双语）
locale = "en_us"
```

| 取值 | 控制台 |
|---|---|
| `en_us` | English |
| `zh_cn` | 简体中文 |
| `zh_en` / `bilingual` | 中文 / English 双语并列 |

玩家客户端界面使用各自的游戏语言；原版死亡消息、物品名等由客户端语言决定（命令反馈走翻译键，中文客户端自动显示中文）。

## 配置模板

将 [config/pumpkin.zh_cn.toml](config/pumpkin.zh_cn.toml)（简体中文注释）或
[config/pumpkin.en_us.toml](config/pumpkin.en_us.toml)（英文注释）复制为 `pumpkin.toml` 使用。
两个模板的配置项与默认值完全一致，仅注释语言不同。

## 功能一览

- [x] 配置系统（toml）
- 协议（[跟踪 issue](https://github.com/Pumpkin-MC/Pumpkin/issues/1401)）
  - [x] 服务器状态 / Ping
  - [x] 加密、数据包压缩
  - [x] Java 版 · 基岩版（开发中）
- 世界（[跟踪 issue](https://github.com/Pumpkin-MC/Pumpkin/issues/1403)）
  - [x] 区块加载/保存（Vanilla、Linear、Pump）、光照、世界边界、世界时间与存档
  - [x] 实体生成（按群系加权、spawn cost、局部上限）
  - [x] 红石（线路/活塞/中继器/比较器/观察者/传感器/振动）（[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/1402)）
  - [x] 液体物理
  - 区块生成：村庄（多群系）、古代城市、矿井、要塞、地狱堡垒、末地城、林地府邸、试炼密室等（[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/36)）
- 玩家（[跟踪 issue](https://github.com/Pumpkin-MC/Pumpkin/issues/1405)）
  - [x] 皮肤、传送、移动、动画、背包、经验、饥饿、副手、进食
  - [x] 进度系统（开发中）
  - 战斗：冷却/暴击/横扫/盾牌/击退（[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/1404)）
- 实体
  - [x] 生物 AI（75+ 生物原版 goal 结构，[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/1406)）
  - [x] 村民（交易/日程/感染治愈）、Boss（末影龙/凋灵）、实体存档、状态效果
- 服务端
  - [x] Query、RCON、聊天、粒子、权限、翻译
  - 插件（[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/1407)）· 命令（[跟踪](https://github.com/Pumpkin-MC/Pumpkin/issues/15)）
- 代理
  - [x] Bungeecord、Velocity

## 运行方式

参见官方 [快速开始](https://docs.pumpkinmc.org/#quick-start) 指南。

## 参与贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 文档

官方文档：<https://pumpkinmc.org/>

## 交流

加入 [Discord 服务器](https://discord.gg/wT8XjrjKkf) 获取动态并与社区交流。

## 赞助

如果想支持项目，请查看 [捐赠页面](https://pumpkinmc.org/donate/)。
