# Pumpkin 原版对照差距总表（Parity Gaps）

> 更新：2026-07-27 · 分支：`fix/gameplay-ai-spawn-net`
> 本文档是给**所有开发者与 AI agent**的共享参考：工作规则、架构地图、逐项差距清单（附原版出处）。
> 原版依据：`/root/Vanilla`（Minecraft 26.2 官方反编译）。所有行号引用以该副本为准。

---

## 0. 工作规则（必读）

1. **原版保真**：任何玩法行为（常量、条件、执行顺序、随机数抽取顺序）必须逐行对照 `/root/Vanilla` 反编译源实现，代码注释中标注 `文件名:行号`。基础设施不足时，在文档注释里显式声明缺口，**禁止填入猜测数值**。
2. **不在本地构建**：开发机是 11GB 内存的手机，只允许 `cargo fmt`。推送后由 GitHub Actions 验证（clippy `-D warnings` + `-D clippy::option-if-let-else` 等 pedantic lint + ARM64 release 构建）。写代码时防御性规避 clippy：Option 上用 `map_or`/`map_or_else` 而非 if-let/else 等。
3. **分支/提交**：全部工作最终合并到 `fix/gameplay-ai-spawn-net`；提交信息用 conventional commits（`fix(spawn): ...`），正文引用原版出处。
4. **补丁脚本**：任何脚本化批量替换必须断言目标串存在——静默 no-op 曾导致坏枚举上线（93d7332b 修复）。
5. **性能路线**：防内存泄漏（注意 `Arc` 循环，实体移除时必须切断挂载/骑乘关系）、原子状态、避免不必要分配。
6. **双端**：Java 版 + 基岩版接入，游戏内容一律以 Java 版为准；协议固定 26.2，旧客户端握手即拒。

## 1. 架构地图（关键入口）

| 子系统 | 位置 |
|---|---|
| 自然刷怪 | `pumpkin/src/world/natural_spawner.rs`（1500+ 行，NaturalSpawner 移植）；tick 驱动 `pumpkin/src/world/mod.rs` ~1595 |
| 刷怪规则 | `pumpkin/src/entity/type.rs` `check_spawn_rules`（SpawnPlacements 等价物） |
| 刷怪笼 | `pumpkin/src/block/entities/mob_spawner.rs`（BaseSpawner 移植） |
| 寻路 | `pumpkin/src/entity/ai/pathfinder/`（A* + WalkNodeEvaluator） |
| AI goal | `pumpkin/src/entity/ai/goal/`（62 个 goal 文件）；注册在各生物构造器 |
| 红石 | `pumpkin/src/block/blocks/redstone/`（39 文件 8300 行）+ `piston/`；更新管线 `neighbor_updater.rs` |
| 结构生成 | `pumpkin-world/src/generation/structure/`；jigsaw：`structures/jigsaw.rs` + `jigsaw_placement.rs`；模板缓存 `template/cache.rs` |
| 结构资产 | `pumpkin-world/assets/structures/`（1181 个 NBT）+ `assets/worldgen/`（template_pool/processor_list JSON），build.rs 编译期嵌入 |
| 地形噪声 | `pumpkin-world/src/generation/`（noise router、proto_chunk.rs、beardifier 地形适配已实现） |
| 生成数据 | `pumpkin-data/src/generated/`（**已提交进仓库**，改数据需同步改 `pumpkin-codegen/src/` 源） |

## 2. 寻路差距清单（2026-07-27 审计，35 项）

已核对为正确：4 基本方向 + 4 对角扩展、`is_neighbor_valid`、WALKABLE_DOOR 对角禁行、宽度<0.5 栅栏柱穿行、26 个 PathType malus 数值、门类型映射（WALKABLE_DOOR/UNPASSABLE_RAIL 规则）。

**高危（修复中/待修）：**

| # | 问题 | Pumpkin | 原版 |
|---|---|---|---|
| 16 | BLOCKED 判定用 `is_full_cube`，楼梯/半砖/箱子/玻璃板被当 OPEN→WALKABLE，生物撞墙抖动 | `pathfinding_context.rs:168` | `WalkNodeEvaluator.java:489-491`（isPathfindable(LAND)） |
| 17 | 开门检测用 `collision_shapes.is_empty()`（永假），开着的门被当墙 | `pathfinding_context.rs:138` | `WalkNodeEvaluator.java:473-479`（OPEN 属性） |
| 25 | A* 失败后非原版 `direct_walk_toward` 直线冲目标 → 跳崖/进岩浆 | `mod.rs:491-545` | `PathNavigation.java:191-195`（清路径站住） |
| 1 | 启发系数 1.0，原版 FUDGING=1.5 | `mod.rs:96,302` | `PathFinder.java:36,105` |
| 12 | 缺 `canReachWithoutCollision` 扫掠 AABB（栅栏/关门源格），生物顶栅栏角 | `walk_node_evaluator.rs:115` | `WalkNodeEvaluator.java:181-191,223` |
| 13 | `tryJumpOn` 非递归且缺宽度<1 天花板 AABB 检查（对着半砖悬檐跳不停） | `walk_node_evaluator.rs:151-198` | `WalkNodeEvaluator.java:267-283` |
| 35 | 只有陆地导航——无 Swim/Fly/Amphibious NodeEvaluator，鱼/鱿鱼/守卫者/幻翼等全用陆地寻路 | 整个 pathfinder/ | `navigation/` + `pathfinder/` 全家族 |

**中危**：maxVisitedNodes 固定 560（应为 `floor(max(followRange,16)*16)`，`PathNavigation.java:77-89`）；到达判定缺 reachRange 参数（`PathFinder.java:87`）；弹出节点未标 closed 重复扩展（`PathFinder.java:85`）；大体型对角禁行缺失（`WalkNodeEvaluator.java:160`）；getStart 水面上浮/BB 四角回退缺失（`:75-117`）；全生物统一僵尸 malus 表（应各生物自设，`Mob.java:204-213`）；小生物 BB 被夹到 0.6×1.95 过不了 1 格缝；卡住检测非速度缩放（`PathNavigation.java:288-316`）；无方块变更触发重算（`:391-401`）；无 canCutCorner 切角（`:255-286`）；游泳位置上浮（`GroundPathNavigation.java:91-104`）与 avoidSun trimPath（`:107-120`）缺失。

**低危**：步下代价整形非原版、水柱下探 16 格上限（原版到 minY）、燃烧方块集合略偏、checkNeighbourBlocks 遍历顺序、宽体路点偏移未用 `Path.getNextEntityPos`、BIG_MOBS_CLOSE_TO_DANGER 枚举缺失、trimPath 炼药锅修正缺失、`set_can_float/can_walk_over_fences` 无调用方。

## 3. 红石差距清单（2026-07-27 审计）

已核对为正确：线路功率计算（15 短路、15→0 衰减、对角上下连接）、强/弱充能区分与线尘抑制、中继器全套（延迟/锁存/脉冲缩短/三档 tick 优先级）、比较器（比较/减法、隔方块读取含物品展示框路径、容器公式 `floor(1+14·fill)`）、活塞 12 上限 + PistonStructureResolver + 粘液/蜂蜜黏连 + 活塞/发射器 QC、观察者 2gt+2gt、CollectingNeighborUpdater 更新序（W,E,D,U,N,S + 层级重入 + 0x80 wire-skip）。

**高危（修复中/待修）：**

| # | 问题 | Pumpkin | 原版 |
|---|---|---|---|
| 1 | 红石火把无烧毁（60t 内 8 次翻转→熄灭+烟+160t 复查） | `redstone_torch.rs` | `RedstoneTorchBlock.java:38-42,86-91,142-153` |
| 2 | 比较器/门侧输入读任意方块弱功率；原版只认线(POWER)/红石块(15)/`isSignalSource` | `abstract_redstone_gate.rs:289-311` | `SignalGetter.java:45-60` |
| 3 | 标靶只响应箭；原版所有弹射物（非箭 8gt，箭 20gt） | 仅 `projectile/arrow.rs:377` 调用 | `TargetBlock.java:50-58` |
| 4 | 朝下活塞会被自己面向方向的方块充能 | `piston.rs:306-334` | `PistonBaseBlock.java:144-158`（跳过 facing 方向） |
| 5 | 移动中的活塞方块无碰撞箱，实体穿透 | `piston_extension.rs:16-35` | `MovingPistonBlock.java:110-116` |
| 6 | 活塞不推玩家、无粘液块弹射 | `entities/piston.rs:89-91` | `PistonMovingBlockEntity.java:130-165` |

**中危**：方块事件下一 tick 才 flush（活塞链慢 1gt，`world/mod.rs:578,1052` vs ServerLevel.runBlockEvents 排空语义）；短脉冲吐块缺 `lastTicked` 条件（`PistonBaseBlock.java:137`）；回拉方向错误致带釉陶瓦被拉（`:208`）；`is_movable` 缺世界高度界检查（`:224-237`）；观察者被活塞移动的重脉冲/断电边角（`ObserverBlock.java:115-124`）；物品展示框比较器值恒 1（应 `rotation%8+1`，`ItemFrame.java:402-407`）；缺失模拟量源：重生锚（整个方块未实现）/末地传送门框架/蜡烛蛋糕/铜灯/饰纹陶罐/嘎枝之心/幽匿感测体频率/书架；线不能放漏斗上（`RedStoneWireBlock.java:239-241`）；线的向上连接规则细节（活板门分支等，`:213-230`）；绊线钩拆除只单邻居更新；绊线实体检测用整方块 AABB 且不过滤旁观者。

**低危**：`willTickThisTick` 近似、wire prepare 多余更新（幻影 BUD）、活塞头挖掉基座 SKIP_DROPS 丢物品、比较器隔块判定 `is_solid_block` vs `isRedstoneConductor`、模式切换无音效、比较器容器变更最迟 1gt 延迟等。

## 4. 生物刷新（2026-07-27 已修 + 余留）

**今日已修：**
- `WATER_CREATURE.is_persistent` 错为 true（原版 `MobCategory.java:19` 为 false）→ 鱿鱼/海豚 399/400 tick 被过滤，几乎不刷。（fbbb3cce）
- 刷怪笼缺 MaxNearbyEntities(6)/RequiredPlayerRange(16)/光照规则/位置公式 `(rand-rand)`——地牢骷髅无限堆刷的直接原因。（d957a38c）
- 水生生物逐种 SpawnPlacements 规则补齐：鱿鱼/海豚/鳕鱼/鲑鱼/河豚海平面窗口、热带鱼、发光鱿鱼深度+全黑（此前发光鱿鱼刷满浅海）、美西螈地面 tag、溺尸 1/40+深度（河流 1/15）。（ea39d324）

**余留 TODO（natural_spawner.rs 行号）：**
- `:1033-1034` 结构刷怪覆盖：下界堡垒 FORTRESS_ENEMIES、海底神殿守卫者、女巫小屋、前哨站——需 `structureManager.getAllStructuresAt` 等价物
- `:1137` `getSpawnBox`（史莱姆/岩浆怪特殊箱）
- `:1169` `blockState.allowsSpawning` 未接入
- `mob/mod.rs:377` 通用 `checkMobSpawnRules`（脚下方块 isValidSpawn）未实现
- 特殊刷怪器缺：猫（村庄/女巫小屋）、流浪商人、掠夺者巡逻、僵尸围城
- 骷髅同点 5-6 只：刷怪笼修复应解决大部分；自然刷怪的包机制与上限均已对照正确；若仍复现，下一步查 wander goal 扩散与光照缓存

## 5. 结构生成

**全部 35 个 StructureKeys 均已注册**，jigsaw 数据驱动（5 村庄/古城/堡垒遗迹/前哨/trail_ruins/试炼密室），1181 个 NBT 模板已嵌入。

**占位/简化实现（需完整移植）：**
- `ocean_monument.rs:66-108` —— 程序化阶梯金字塔占位，无房间网格/远古守卫者/海绵房（原版 `OceanMonumentPieces` 12 种 piece）
- `mansion.rs:86-145` —— 只摆入口+两面墙（73 个模板只用 4 个），无 LayoutGenerator/FlagMatrix 楼层房间屋顶
- `end_city.rs:37-41` —— 只用 5/20 模板，无桥/fat_tower/二三层/屋顶
- `mineshaft.rs` —— 移植中（原版 `MineshaftPieces.java` 782 行：走廊铁轨/蛛网/洞穴蜘蛛刷怪笼/运输矿车宝箱/支撑柱、房间、十字、楼梯）
- `jigsaw_placement.rs:56` `PoolAliasLookup` 为 stub → 试炼密室池别名未生效
- 模板处理器缺：`gravity`、`block_ignore`、`jigsaw_replacement`、`block_age`、`lava_submerged_block`、`blackstone_replace`；`capped` 的 limit 是 no-op（`processor.rs:149-152`）
- `mod.rs` 两个 dispatch 函数逐字重复（drift 风险，待合并）

## 6. 地形生成（用户报告，诊断中）

用户观察到：悬空岛、石头堆中孤立草方块、石头+沙/陶土混杂表面。怀疑方向：surface rules 的 stone-depth/floor 判定、群系采样坐标、噪声实例错配。诊断 agent 进行中，结论出来后补充本节。
（注：beard_thin/beard_box/bury/encapsulate 地形适配已在 `proto_chunk.rs:563-` 实现，村庄悬空不是 beardifier 缺失所致。）

## 7. 其他已知缺口（历史）

- 袭击系统（袭击队长/巡逻/钟声）
- 村庄 POI 图（现以村民认领床近似）
- Brain/行为树系统缺失——猪灵/村民/监守者等用 goal 近似
- 行商羊驼商队状态、蜂巢完整周期
- 马驴骡：骑乘已落地（59acff5d+e8e266e8），鞍具背包界面 TODO

## 8. 时间线（2026-07-27 批次）

- e331ddbe `fix(lint)` processor.rs option_if_let_else
- fbbb3cce `fix(spawn)` WATER_CREATURE 非持久化
- d957a38c `fix(spawner)` 刷怪笼 BaseSpawner 对照重写
- ea39d324 `feat(spawn)` 水生生物刷新规则全套
- e8e266e8 `fix(build)` horse.rs 编译错误
- 进行中：寻路高危修复（agent）、红石高危修复（agent）、矿井完整移植（agent）、地形诊断（agent）
