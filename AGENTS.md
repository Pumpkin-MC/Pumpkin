# 适用于 Pumpkin 的 AI Agent 的开发指南

> 贡献要求见 [CONTRIBUTING.md](CONTRIBUTING.md)，社区行为准则见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。
> 这里主要记录代码位置和开发时需要注意的地方。

---

## 注意事项

**严禁幻觉**：不得编造不存在的 API、配置项、环境变量或依赖库。若不确定某项信息，必须通过代码搜索或文档检索进行验证，无法验证时应在 PR 描述中明确标注”待人工确认”。

**最小变更**：仅修改与当前 Issue 或任务直接相关的代码。禁止在修复 Bug 时顺手重构无关模块，或在新增功能时修改不相关的代码风格。

**安全红线**：绝对禁止在代码、注释、测试数据或 PR 描述中硬编码任何密钥、Token、密码或私人凭证。所有敏感配置必须通过环境变量或密钥管理服务注入。

**上下文感知**：在修改代码前，必须阅读相关文件及其关联的测试文件、文档和最近的 Commit 历史，确保理解现有设计意图，避免破坏隐式契约。

**透明沟通**：若在执行任务过程中遇到阻塞、需求歧义或技术不可行，必须立即在 Issue 或 PR 中留言说明，不得静默失败或强行提交有缺陷的实现。

**代码版权**：确保编写的代码不存在版权冲突，且不属于任何项目内的直接复制、粘贴或替代的代码，如有必要的情况下请标注来源并注意署名，同时遵守相应的开源协议。

---

## 开发前的准备

> 在编写任何代码之前，请先阅读项目中已有的类似实现，避免重复造轮子。版本和依赖以 [Cargo.toml](Cargo.toml)、[rust-toolchain.toml](rust-toolchain.toml) 为准。

### 开发前必读

你需要遵守如下约定：

- **环境对齐**：确保本地开发环境（Rust 版本、包管理器、Clippy 配置）与项目 CI 配置完全一致。建议先运行一次完整的 Lint 和测试套件，确认基线状态为绿色。

- **分支规范**：从最新的 Master 分支拉取新分支。分支命名必须遵循 <type>/<short-description> 格式，例如 feat/add-block 或 fix/memory-leak-parser。

- **依赖审查**：若任务需要引入新依赖，必须先检查 Cargo.toml 等清单文件，确认该依赖是否已存在。新增依赖需评估许可证兼容性、维护活跃度及包体积影响，并在 PR 描述中说明引入理由。

- **方案预沟通**：对于涉及架构变更、公共 API 修改等复杂任务，必须先在 Issue 下提交简要设计方案（Design Doc），等待至少一位维护者确认后再动手编码。

从仓库根目录构建：

```bash
cargo build --locked -p pumpkin
```

运行服务端可用 `cargo run --locked -p pumpkin --release`。它会读写运行目录中的配置和世界数据，测试时使用单独的目录或已有测试环境。

主要代码在 `crates/`，工具在 `tools/`：

| 目录                                                                                                       | 内容                                                          |
|------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------|
| [crates/pumpkin/](crates/pumpkin/)                                                                         | 服务端入口、游戏逻辑、具体命令、网络处理和插件宿主            |
| [crates/pumpkin-auth/](crates/pumpkin-auth/)                                                               | 认证、JWT、HTTP 客户端                                        |
| [crates/pumpkin-command/](crates/pumpkin-command/)                                                         | 通用命令树、参数解析、分发和补全                              |
| [crates/pumpkin-config/](crates/pumpkin-config/)                                                           | 配置定义和加载                                                |
| [crates/pumpkin-data/](crates/pumpkin-data/)                                                               | 游戏数据、注册表、物品栈和数据组件                            |
| [crates/pumpkin-protocol/](crates/pumpkin-protocol/)                                                       | Java、Bedrock、Query 协议和编解码                             |
| [crates/pumpkin-world/](crates/pumpkin-world/)                                                             | 区块、生成、光照和存档                                        |
| [crates/pumpkin-inventory/](crates/pumpkin-inventory/)                                                     | 物品栏、容器和菜单                                            |
| [crates/pumpkin-nbt/](crates/pumpkin-nbt/)、[crates/pumpkin-codecs/](crates/pumpkin-codecs/)               | NBT 和数据编解码                                              |
| [crates/pumpkin-util/](crates/pumpkin-util/)                                                               | 数学、随机数、文本、权限、版本等共用代码                      |
| [crates/pumpkin-gametest/](crates/pumpkin-gametest/)                                                       | GameTest 框架                                                 |
| [crates/pumpkin-macros/](crates/pumpkin-macros/)、[crates/pumpkin-api-macros/](crates/pumpkin-api-macros/) | 服务器和插件相关过程宏                                        |
| [crates/pumpkin-plugin-api/](crates/pumpkin-plugin-api/)                                                   | Wasm 插件 Rust SDK                                            |
| [crates/pumpkin-plugin-wit/](crates/pumpkin-plugin-wit/)                                                   | WIT 接口定义，当前目录为 `v0.1/`；不是 Cargo workspace member |
| [crates/pumpkin-host-bindings/](crates/pumpkin-host-bindings/)                                             | Wasmtime 宿主绑定                                             |
| [crates/pumpkin-plugin-runtime/](crates/pumpkin-plugin-runtime/)                                           | 插件 Store 执行、生命周期和重入处理                           |
| [crates/pumpkin-plugin-utils/](crates/pumpkin-plugin-utils/)                                               | 插件市场、许可和更新查询                                      |
| [tools/pumpkin-codegen/](tools/pumpkin-codegen/)                                                           | Rust、WIT 和 SDK 数据生成                                     |
| [tools/pumpkin-fuzzer/](tools/pumpkin-fuzzer/)                                                             | 服务端网络模糊测试                                            |

实体代码在 `crates/pumpkin/src/entity/`。NBT 和协议另有 cargo-fuzz 工程，分别在 `crates/pumpkin-nbt/fuzz/` 和 `crates/pumpkin-protocol/fuzz/`。

服务器采用 GPL-3.0；`pumpkin-plugin-api`、`pumpkin-plugin-wit` 和 `pumpkin-plugin-utils` 采用 MIT OR Apache-2.0。具体以各目录的 manifest 和许可证文件为准。

---

## 常见改动

下面的短路径以 `crates/pumpkin/src/` 为起点。

| 要改什么     | 参考实现                                                                                                                                             | 注册位置                                                      |
|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------|
| 普通方块     | [block/blocks/dirt_path.rs](crates/pumpkin/src/block/blocks/dirt_path.rs)                                                                            | `block/blocks/mod.rs`、`block/registry.rs`                    |
| 带实体的方块 | [block/blocks/campfire.rs](crates/pumpkin/src/block/blocks/campfire.rs)、[block/entities/campfire.rs](crates/pumpkin/src/block/entities/campfire.rs) | 方块注册表、放置逻辑、`block/entities/mod.rs` 的 NBT 恢复分派 |
| 物品行为     | [item/items/bucket.rs](crates/pumpkin/src/item/items/bucket.rs)                                                                                      | `item/items/mod.rs` 的 `default_registry()`                   |
| 生物         | [entity/mob/bat.rs](crates/pumpkin/src/entity/mob/bat.rs)                                                                                            | 相应模块声明、`entity/type.rs`                                |
| AI 目标      | [entity/ai/goal/melee_attack.rs](crates/pumpkin/src/entity/ai/goal/melee_attack.rs)                                                                  | 生物构造函数中的 `goals_selector` 或 `target_selector`        |
| 命令         | [command/commands/time.rs](crates/pumpkin/src/command/commands/time.rs)                                                                              | `command/commands/mod.rs` 的 `default_dispatcher()`           |
| 配方、数据包 | [server/recipe.rs](crates/pumpkin/src/server/recipe.rs)、[data/datapack/](crates/pumpkin/src/data/datapack/)                                         | 配方管理、资源加载和客户端同步                                |
| GameTest     | [server/server_test_manager.rs](crates/pumpkin/src/server/server_test_manager.rs)                                                                    | `command/commands/test.rs`                                    |

服务端入口是 [main.rs](crates/pumpkin/src/main.rs)，初始化和导出在 [lib.rs](crates/pumpkin/src/lib.rs)。世界管理从 [world/mod.rs](crates/pumpkin/src/world/mod.rs) 和 [pumpkin-world/src/level.rs](crates/pumpkin-world/src/level.rs) 开始看。

### 方块和物品

[BlockBehaviour](crates/pumpkin/src/block/mod.rs) 和 [ItemBehaviour](crates/pumpkin/src/item/mod.rs) 使用同步方法。例如 `on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId`，不需要返回 Future。

`#[pumpkin_block(...)]`、`#[pumpkin_block_from_tag(...)]` 会生成 `BlockMetadata`，不要再手写同一份实现。物品需要实现 `ItemMetadata` 和 `ItemBehaviour`，包括 `as_any()`。

[BlockEntity](crates/pumpkin/src/block/entities/mod.rs) 必须实现 `write_nbt`、`from_nbt`、`resource_location`、`get_position` 和 `as_any`。新增方块实体时，除了放置时调用 `world.add_block_entity(...)`，还要补上 `block_entity_from_nbt()` 的分派，验证世界重载后能否恢复。涉及容器时也要检查脏标记、比较器和物品组件的保存。

物品的 `use_on_block()` 返回 `BlockActionResult`。需要射线检测的物品应参考桶的 `normal_use_with_rotation()`，使用本次操作包中的 yaw / pitch。

### 实体和 AI

实体由 `Entity`、`LivingEntity`、`MobEntity` 组合。`Mob` 已有 `EntityBase` 的通用实现，`EntityBase` 也已有 `NBTStorage` 的通用实现，不要重复实现这些 trait。

生物自定义存档字段放在 `mob_write_nbt()` / `mob_read_nbt()`；其他实体使用 `write_custom_nbt()` / `read_custom_nbt()`。需要同步给客户端的字段通过 `tracked_data`、`set_synced_data()` 和 [SynchedEntityData](crates/pumpkin/src/entity/synched_entity_data.rs) 更新。

[Goal](crates/pumpkin/src/entity/ai/goal/mod.rs) 的方法也是同步的，`can_start`、`should_continue` 和 `tick` 都接收 `&mut self`。目标用 `Controls::MOVE`、`LOOK`、`JUMP`、`TARGET` 声明占用的控制项。调用 `GoalSelector::add_goal()` 时传入优先级和 `Box<G>`，如 `Box::new(MeleeAttackGoal::new(1.0, true))`。

### 命令

具体命令留在 `crates/pumpkin/src/command/`，通用解析器在 `pumpkin-command`。服务器的 `CommandExecutor::execute(&self, context: &CommandContext)` 返回 `CommandExecutorResult`，也就是 `Result<i32, CommandSyntaxError>`。添加命令时一并处理权限、补全和注册。

---

## 网络和插件

Java 的 Handshake、Status、Login、Configuration 由 [PendingConnection](crates/pumpkin/src/net/java/pending.rs) 处理；进入 Play 后由 [JavaClient](crates/pumpkin/src/net/java/mod.rs) 处理，具体处理函数在 [play/](crates/pumpkin/src/net/java/play/)。

核心 Java 包表目前按 26.3 生成，当前版本和最低版本常量都在 [生成的 packet.rs](crates/pumpkin-data/src/generated/packet.rs)。旧版本支持已拆到扩展中。[handshake.rs](crates/pumpkin/src/net/java/handshake.rs) 会根据 `PacketReceivedEvent` 处理器放行部分旧协议，但实际兼容情况仍需加载相应插件并用客户端测试。Bedrock 使用独立实现，仍在开发中。

改包时检查所属阶段、包 ID、字段顺序、长度、压缩和加密状态；读包还要考虑取消和超时。数据包命名中，`S` 表示客户端发给服务端，`C` 表示服务端发给客户端。Java 和 Bedrock 的行为分别验证。

插件加载器有 [native.rs](crates/pumpkin/src/plugin/loader/native.rs) 和 [wasm/](crates/pumpkin/src/plugin/loader/wasm/) 两种。服务器内部 API 在 `crates/pumpkin/src/plugin/api/`，Wasm 插件 SDK 在 `pumpkin-plugin-api`。

修改 Wasm 接口时，需要一起看：

- `crates/pumpkin-plugin-wit/v0.1/`：接口及 world 的导入、导出。
- `crates/pumpkin-host-bindings/src/lib.rs`：宿主绑定和调用配置。
- `crates/pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/`：Host 实现和事件转换。
- `crates/pumpkin-plugin-api/src/`：SDK 包装和导出。
- `crates/pumpkin-plugin-runtime/`：执行、重入和卸载。

检查权限及资源句柄的生命周期。原生插件的不兼容变更还要处理 `plugin/mod.rs` 中的 `PLUGIN_API_VERSION`。Wasmtime 和 WASI 当前使用固定的 Git revision，升级时保持配套。

游戏逻辑中的同步方法不代表整个项目都是同步的。网络和插件仍有 async / Future 接口，原生插件使用 `PluginFuture`，事件分派使用 `BoxFuture`；Wasm Host 按生成的 trait 实现。

---

## 写代码时

Clippy 配置在根 `Cargo.toml`，默认拒绝 `unwrap`、`expect`、`panic`、`todo!()` 和调试输出等。错误用 `Result` / `Option` 处理，日志用 `tracing`。确有必要的 lint 例外用局部 `#[expect(...)]` 并说明原因。

同步锁的中毒处理参考所在模块，只有状态允许恢复时才用 `PoisonError::into_inner`。不要持有同步锁或 `DashMap` guard 跨过 `.await`。

CPU 密集工作交给 Rayon，不要阻塞 Tokio 工作线程。异步 I/O 与同步 tick 之间的调度参考现有 `Level`、`Server` 实现；改插件执行时沿用 `pumpkin-plugin-runtime` 的 Store 管理。

实体状态常用 `AtomicCell` 和标准原子类型，共享对象替换使用 `ArcSwap`。选锁和原子类型时跟随所在模块的用法，检查锁顺序和回调重入。解析外部数据时限制长度和分配量；热路径的性能改动附上测量结果。

---

## 生成代码

不要直接改 `crates/pumpkin-data/src/generated/`。修改 `tools/pumpkin-codegen/src/` 中的生成器或 `assets/` 输入，再重新生成。入口是 [tools/pumpkin-codegen/src/main.rs](tools/pumpkin-codegen/src/main.rs)。

```bash
# 只生成 packet.rs
cargo run --locked -p pumpkin-codegen -- packet

# 生成 WIT 数据定义和宿主包映射
cargo run --locked -p pumpkin-codegen -- wit

# 全量生成 Rust、WIT 和 SDK 数据
cargo run --locked -p pumpkin-codegen
```

过滤参数可以是输出文件名，也可以省略 `.rs`。`wit` 模式还会更新宿主的 `generated_packets.rs`。SDK 的 `src/generated/` 同样由工具生成；手写 WIT 接口直接修改源文件。生成后检查 diff，避免带入无关变化。

---

## 检查和提交

- 检查单个包时用 `-p <包名>`。未安装 nextest 时可运行 `cargo test --locked --workspace`，并说明使用了不同的测试运行器。
- 仅运行部分测试或跳过 Lint 检查的 PR 将被直接关闭。
- 若修复了 Bug，必须添加一个能复现该 Bug 的失败测试用例，并确保修复后该测试通过。这防止问题在未来被无意 reintroduce。
- 明显的小改动和纯文档修改不用另写测试
- 若变更涉及解析、网络请求、内存分配或循环逻辑，必须评估其对性能的影响。必要时添加基准测试或内存泄漏检测，并在 PR 中提供对比数据。
- 在打开 PR 前，确保所有 CI 检查（构建、测试、Lint、类型检查）在本地或 fork 仓库中已通过。禁止提交”让 CI 帮我跑一下”的试探性 PR。
- 测试结果写清执行命令和失败原因。客户端测试记录客户端版本、服务器提交、所用插件、操作步骤和日志；登录成功、区块正常和长时间稳定分别说明。没有实际运行的检查直接写未运行。

### 本地测试验证

先跑相关包的检查和测试，再检查受影响的依赖方。完整检查可在根目录运行：

```bash
cargo clippy --release --all-targets --all-features
cargo fmt --all -- --check
cargo machete
cargo check --all-targets --all-features
typos
RUSTFLAGS="-Dwarnings" cargo clippy --locked --workspace --all-targets --all-features
RUSTFLAGS="-Dwarnings" cargo nextest run --locked --workspace --verbose --no-tests=pass
RUSTFLAGS="-Dwarnings" cargo test --locked --workspace --doc --verbose
RUSTFLAGS="-Dwarnings" cargo build --locked --workspace
```

### 自审清单

提交前逐项确认：

- 代码无 TODO/FIXME/HACK 遗留（或已在 Issue 中追踪）

- 无调试代码、console.log、print 语句或未使用的导入

- 所有新增公共接口均有文档，确保文档覆盖率达 80% 以上

- 测试覆盖关键路径，且无 flaky test

- PR 描述清晰说明了变更动机、实现方式及测试方法

---

## 最后向南瓜打开 PR

- 标题格式：<type>: <short description>（例如：feat(auth): Add multi-Yggdrasil authentication support）并在 PR 标题后追加 🤖🤖🤖

- 相同实现：提交前**请先搜索是否有类似的 PR**，不要重复提交几乎相同的实现，如果必要的情况下，请说明你的实现与已有的实现的区别和优缺点。

- 拆分 PR：**一个 PR 只做一件事**——不要把不相关的改动混在一起，例如一个 PR 尽量只修复一个 bug。

- 测试说明：提供足够的测试说明，如有必要请提供测试截图或者视频，

- 正文模板：[PR 模板](.github/PULL_REQUEST_TEMPLATE.md) 的 `Description` 和 `Testing`，说清问题、改法、测试及已知限制。

- 响应反馈：PR 提交后，需主动关注 CI 结果和维护者评论。若 CI 失败，必须在 24 小时内修复或说明原因；若收到修改意见，应在 48 小时内响应或协商延期。
