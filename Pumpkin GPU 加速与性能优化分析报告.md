# Pumpkin 项目 GPU 加速与性能优化分析报告

> 日期: 2026-08-03 | 分支: GPU-accelerated | 测试: 29/29 通过

---

## 一、pumpkin-gpu 模块分析

### 1.1 定位

`pumpkin-gpu` 是独立 GPU 加速 crate，使用 wgpu v30.0.0，与 `pumpkin-world` / `pumpkin` 零耦合。通过 `cargo build -p pumpkin-gpu` 显式构建，不进入默认构建路径。

### 1.2 目录结构

```
pumpkin-gpu/src/world/
├── mod.rs              # pub mod {chunk, gpu, graph, light}
├── gpu.rs              # GPU 基础设施 (GpuNoiseContext, 4 个 pipeline)
├── graph.rs            # 密度函数图编译器 (28 操作码) + CPU 参考解释器
├── light.rs            # 光扫描类型 + 全局 GPU 上下文
├── chunk.rs            # 完整区块噪声 GPU 管线 (OverworldNoisePipeline)
├── graph.wgsl          # 密度函数图 compute shader
├── octave_perlin.wgsl  # Perlin 噪声 compute shader
└── light.wgsl          # 光扫描 compute shader (sky + block)
```

### 1.3 四个 Compute Pipeline

| # | Shader | 入口点 | 功能 | 并行度 |
|---|--------|--------|------|--------|
| 1 | `octave_perlin.wgsl` | `sample_octaves` | 批量八度 Perlin 噪声 | 每点独立 |
| 2 | `graph.wgsl` | `evaluate_graph` | 28 操作码密度函数图 | 每点独立 |
| 3 | `light.wgsl` | `scan_sky_light` | 天空光列扫描 | 每列独立 (324 列) |
| 4 | `light.wgsl` | `scan_block_light` | 方块光扫描 | 每位置独立 |

### 1.4 GPU vs CPU 正确性验证

**29 个单元测试全部通过**，覆盖 GPU 输出与 CPU 参考的逐项对比：

| 模块 | 测试数 | 对比方式 |
|------|--------|---------|
| graph (图编译器) | 5 | CPU 软件解释器 vs 预期值 |
| gpu (噪声) | 10 | GPU vs CPU f64 参考实现 |
| gpu (天空光) | 8 | GPU vs CPU 列扫描逐字节比较 |
| gpu (路由器) | 1 | GPU vs CPU 全 overworld router |
| chunk (区块管线) | 5 | GPU 全区块 256 位置 vs CPU 参考 |

**关键对比结果**:

| 对比项 | 规模 | 结果 |
|--------|------|------|
| 天空光 | 324×384 (~124K 位置) | 逐字节一致，零差异 |
| 噪声路由器 | 500 点 × 10 输出 | f32 容差内完全一致 |
| 区块管线 | 256 位置 × 10 输出 | 各输出在对应容差内一致 |

---

## 二、模块详解

### 2.1 密度函数图求值 (`graph.wgsl` + `graph.rs`)

**原理**: 将 `BaseNoiseRouter` 的 217 节点拓扑排序后扁平化为指令列表，GPU shader 作为字节码解释器逐指令执行。

**支持 28 种操作码**: 噪声系列 (Noise, ShiftA/B, ShiftedNoise, InterpolatedNoise, EndIslands)、算术 (Constant, PassThrough, LinearAdd/Mul)、一元 (Abs/Square/Cube/HalfNegative/QuarterNegative/Squeeze/Invert)、二元 (Add/Mul/Min/Max)、条件 (RangeChoice, IntervalSelect)、钳制 (Clamp, ClampedYGradient, ClampedYIdentity)、样条 (Spline)、地形 (Beardifier)。

**API**:
```rust
let ctx = GpuNoiseContext::try_new()?;
let compiled = compile_router(&router, &config)?;
let results = ctx.evaluate_graph_with(&compiled, &points, &beardifier);
// results: [output][point], 10 values per point
```

### 2.2 天空光列扫描 (`light.wgsl` + `light.rs`)

**原理**: `convert_light()` 的列扫描部分——324 列 × ~384 高度，每列从地表向下依次减去不透明度。GPU shader 将 324 列全部分配到独立 work item 并行执行。

**数据流**: Cache → `opacity[324×384]` + `heightmap[324]` → GPU buffer → shader 并行 → readback → `Vec<u8>`

**API**:
```rust
let ctx = GpuNoiseContext::try_new()?;
let sky = ctx.scan_sky_light_raw(&opacity, &heightmap, 324, 384, -64);
```

**全局上下文** (用于可选激活):
```rust
pumpkin_gpu::world::light::init_global_gpu();  // 无 GPU 时静默跳过
let sky = pumpkin_gpu::world::light::try_sky_light_gpu(&input);
```

### 2.3 完整区块噪声管线 (`chunk.rs`)

**原理**: 将 `compile_router` + `PreparedGraph` 封装为可复用管线，一次编译后反复求值任意 3D 位置集合。单次 GPU dispatch 产出全部 10 个路由器输出。

**API**:
```rust
let pipeline = OverworldNoisePipeline::new(seed, legacy)?;

// 98,304 个区块位置
let positions = OverworldNoisePipeline::chunk_block_positions(0, 0, -64, 320);

// 批量求值
let outputs = pipeline.evaluate(&ctx, &positions);

// 提取 final_density
let densities = OverworldNoisePipeline::collect_output(&outputs, FINAL_DENSITY, positions.len());

// 重复 dispatch（上传图表一次）
let mut prepared = pipeline.prepare(&ctx);
```

**10 个路由器输出**: barrier_noise, fluid_level_floodedness_noise, fluid_level_spread_noise, lava_noise, erosion, depth, **final_density**, vein_toggle, vein_ridged, vein_gap。

---

## 三、性能数据

| 指标 | 数值 |
|------|------|
| 噪声 GPU vs CPU | **12-30x** |
| 完整区块加速上限 (仅噪声) | **~1.56x** |
| 预计噪声+光照 GPU 后 | **~3.7x** |
| 区块生成时间分布 | 光照 37.4% / 噪声 35.9% / 地表 9.4% / 雕刻 5.2% |

---

## 四、GPU 可加速的剩余逻辑

| 优先级 | 模块 | 占比 | 状态 |
|--------|------|------|------|
| 🟢 | 天空光列扫描 | 光照一部分 | ✅ GPU 已实现 |
| 🟢 | 密度函数图求值 | 噪声 35.9% | ✅ GPU 已实现 |
| 🟢 | 完整区块噪声管线 | 噪声 35.9% | ✅ GPU 已实现 |
| 🟡 | 表面构建 | 9.4% | Spline 操作码已支持 |
| ⚪ | 含水层采样器 | — | 密度输入已 GPU |
| ⚪ | 洞穴雕刻 | 5.2% | ROI 不高 |

**不适合 GPU**: 路径查找 (顺序 A*)、实体 AI (已 tokio 并发)、调色板序列化 (仅 4096 元素)、高度图 (256 列)。

---

## 五、优化方案

### 5.1 CPU 端优化 (建议优先实施)

| # | 优化项 | 文件 | 收益 | 难度 |
|---|--------|------|------|------|
| 1 | convert_light 每列锁一次替代每方块锁 | `lighting/engine.rs` | 消除 ~64K 锁操作 | 简单 |
| 2 | BFS 预计算不透明度数组 | `lighting/engine.rs` | 消除数千次锁 | 中 |
| 3 | 地表噪声采样器缓存 | `surface/mod.rs` | 消除 ~98K 次重建 | 简单 |
| 4 | surface_heights HashMap→数组 | `lighting/engine.rs` | 减少分配 | 简单 |
| 5 | convert_light 列扫描 rayon 并行 | `lighting/engine.rs` | 多核加速 | 简单 |
| 6 | propagate_light 初始扫描并行 | `lighting/engine.rs` | 多核加速 | 简单 |

**预估**: 第 1-6 项共计减少区块生成时间 15-25%。

### 5.2 GPU 端优化 (已实现)

| # | 优化项 | 状态 |
|---|--------|------|
| 1 | GPU 辅助方法提取 (`uniform_from_dims` 等) | ✅ gpu.rs |
| 2 | 消除 `sky_light_gpu_callback` Vec 拷贝 | ✅ light.rs |
| 3 | CPU 参考实现 vs GPU 逐字节测试 | ✅ 29 tests |
| 4 | 全局 GPU 上下文优雅降级 | ✅ light.rs |

### 5.3 架构建议

1. **GPU 接入生产管线**: 通过函数指针注入 (如 `LightEngine::with_gpu(fn)`)，避免循环依赖
2. **区块噪声全 GPU**: 使用 `OverworldNoisePipeline` 在 `populate_noise()` 中批量求值替代逐点 CPU 遍历
3. **SIMD**: 项目目前零 SIMD 内联函数，对暂不 GPU 化的光照传播路径可引入 SSE/AVX

---

## 附录

### A. 代码中已标记的性能 TODO

| 文件 | 内容 |
|------|------|
| `chunk/palette.rs:243` | "Don't use HashMap's here, because its slow" |
| `feature/features/ore.rs:181` | "TODO: using a section would be faster" |
| `surface/mod.rs:308` | "TODO: we want to cache these" |
| `surface_height_sampler.rs:127` | "TODO: It seems kind of wasteful to iter over all components" |
| `lighting/engine.rs:118` | "Skip if already visited (critical early-exit optimization)" |

### B. 激活方式

```bash
cargo build -p pumpkin-gpu --release
cargo test -p pumpkin-gpu --lib   # 29 tests
```

所有 GPU 功能独立于 `pumpkin-world` / `pumpkin`，通过显式调用使用。无 GPU 时自动降级，无需配置。
