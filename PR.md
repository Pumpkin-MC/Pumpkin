## 概述

> [!IMPORTANT]
> 基于[gerofurlani07-gif/gpu-noise-acceleration](https://github.com/gerofurlani07-gif/Pumpkin/tree/gpu-noise-acceleration)二次开发，添加光照、地表、洞穴/裂谷等生成加速
> 本PR为实验性功能，仅用于追踪问题和测试环境，相关问题请在本PR下回复，请勿合并！！！

使用 wgpu 的 GPU 计算加速，目前实现世界生成的 4 个阶段：

1. **噪声阶段** — GPU 在单次调度中评估整个区块所有位置的密度函数图，将密度值映射为方块状态
2. **地表阶段** — GPU 批量预计算每列噪声值（512 次 CPU 调用 → 4 次 GPU 调度），以 O(1) 数组查找替代 CPU 逐列采样
3. **洞穴/裂谷阶段** — 通过 `Arc` 共享并复用表面噪声批次，加速 `top_material()` 地表材料恢复
4. **光照阶段** — 将 18×18×N 区域的天空光和方块光的扫描卸载到 GPU 计算

通过 Cargo `gpu` 特性条件编译，GPU 不可用时自动回退 CPU 路径。GPU 专用访问器方法通过 `#[cfg(feature = "gpu")]` 门控。

```
RUSTFLAGS='-C target-cpu=native' cargo build --release -p pumpkin --features gpu
```

---

## 新 crate：`pumpkin-gpu`

| 文件                                       | 用途                                                |
|--------------------------------------------|-----------------------------------------------------|
| `pumpkin-gpu/src/world/gpu.rs`             | GPU 上下文，缓冲区缓存调度，图评估，回读            |
| `pumpkin-gpu/src/world/graph.rs`           | 噪声路由器 → GPU 指令编译器，CPU 参考评估器         |
| `pumpkin-gpu/src/world/noise.rs`           | 区块噪声密度评估回调，密度映射，beardifier 扭曲     |
| `pumpkin-gpu/src/world/surface.rs`         | 地表/洞穴裂谷噪声批量 GPU 回调，DoublePerlin 预处理 |
| `pumpkin-gpu/src/world/light.rs`           | 天空/方块光 GPU 扫描 + 全局 GPU 上下文单例          |
| `pumpkin-gpu/src/world/chunk.rs`           | 区块级 GPU 调度辅助，完整管线编排                   |
| `pumpkin-gpu/src/world/graph.wgsl`         | GPU 密度函数图着色                                  |
| `pumpkin-gpu/src/world/octave_perlin.wgsl` | GPU 八度柏林噪声着色器                              |
| `pumpkin-gpu/src/world/light.wgsl`         | GPU 光照传播着色器                                  |

### 其他变更

| 文件                                           | 变更内容                                     |
|------------------------------------------------|----------------------------------------------|
| `pumpkin-config/src/gpu.rs`                    | GPU 配置结构体、设备选择、后端强制           |
| `pumpkin-world/src/generation/surface/mod.rs`  | SurfaceNoiseBatch 类型、GPU 回调注册入口     |
| `pumpkin-world/src/generation/noise/mod.rs`    | NoiseGpuFn 类型、GPU 回调注册入口            |
| `pumpkin-world/src/lighting/mod.rs`            | SkyLightGpuFn 类型、GPU 回调注册入口         |
| `pumpkin-world/src/generation/carver/mod.rs`   | precompute_carver_noise_batch GPU 快速路径   |
| `pumpkin-world/src/generation/proto_chunk.rs`  | 地表构建 GPU 快速路径                        |
| `pumpkin-world/src/generation/noise/perlin.rs` | `#[cfg(feature = "gpu")]` 门控暴露访问器方法 |
| `pumpkin/src/server/mod.rs`                    | GPU 初始化、回调注册、配置驱动的功能开关     |

---

## 调用链：GPU vs CPU 路径

### 特性标志链

```
pumpkin --features gpu
  ├── pumpkin-gpu (可选依赖)
  │     └── pumpkin-world/gpu (解锁访问器方法)
  └── pumpkin-world/gpu (传播)
```

### 回调注册（`server/mod.rs`）

```
init_global_gpu_with_config() → has_global_gpu()?
  ├── light_acceleration    → register_sky_light_gpu()       (光照)
  ├── surface_acceleration  → register_surface_noise_gpu()   (地表 + 洞穴/裂谷)
  └── noise_acceleration    → register_noise_gpu()           (噪声)
```

### 各阶段调度

| 阶段          | GPU 入口                                                                           | 回退                                          |   状态    |
|---------------|------------------------------------------------------------------------------------|-----------------------------------------------|:---------:|
| **噪声**      | `step_to_noise()` → `get_noise_gpu()` → `evaluate_graph_with()`（一次调度 98K 点） | `ChunkNoiseGenerator`（CPU，完整含水层/矿石） | ✅ 已接入 |
| **地表**      | `build_surface()` → `get_surface_noise_gpu()` → `sample_batch()` × 4               | 逐列 CPU 采样                                 | ✅ 已接入 |
| **洞穴/裂谷** | `carve()` → `precompute_carver_noise_batch()` → 复用同一地表回调                   | CPU 采样                                      | ✅ 已接入 |
| **光照**      | `convert_light()` → `get_sky_light_gpu()` → `scan_sky_light_raw()`                 | CPU 列扫描                                    | ✅ 已接入 |

### 回退保障

- `OnceLock::get()` 未注册时返回 `None` → 所有阶段回退 CPU
- `get_global_gpu()?` GPU 未初始化时返回 `None` → 回调返回 `None`
- `has_global_gpu()` 注册前检查 → 无 GPU 不注册回调
- 边界检查 `lx < 16 && lz < 16` → OOB 坐标 CPU 回退
- 无 panic，无静默失败

---

## 测试覆盖

### GPU crate（44 测试）

| 类别       | 数量 | 示例                                                                |
|---------- --|:----:|----------------------------- ----------------------------------------|
| 图编译     |  8   | `compile_reports_unsupported_nodes`，`real_routers_lower_end_to_end` |
| 噪声操作   |  8   | `gpu_noise_opcode_matches_real_cpu_sampler`，`nested_spline_matches_vanilla` |
| 天空光     |  8   | `gpu_sky_light_matches_cpu_reference_varied`，`sky_light_fully_solid_column` |
| 主世界路由 |  3   | `gpu_matches_cpu_on_the_full_overworld_router`，`emits_every_router_output` |
| 末地岛屿   |  2   | `gpu_end_islands_no_overflow_at_extreme_coords`                     |
| 区块管线   |  5   | `gpu_chunk_pipeline_matches_cpu_reference`，`overworld_router_compiles` |
| Beardifier  |  1   | `gpu_beardifier_matches_cpu_with_real_structures`                   |
| 地表回调   |  4   | `surface_callback_returns_none_without_gpu`，`double_perlin_scale_matches_cpu` |
| 噪声回调   |  7   | `noise_callback_returns_none_without_gpu`，`density_mapping_*`      |

### World crate（GPU 相关测试）

| 类别      | 数量 | 示例                                                               |
|-----------|:----:|------------- ------------------------------------------------------|
| 地表批次  |  4   | `no_gpu_callback_returns_none_by_default`，`batch_oob_indexing_does_not_panic` |
| 洞穴/裂谷 |  10  | `restores_surface`，`overworld_has_aquifer`，`skips_surface_restore`  |

---

## 基准测试结果

### 测试环境

- **GPU**：NVIDIA GeForce GTX 1060（GP106M，Pascal，6 GB）
- **CPU**：Intel Core i7 Coffee Lake-H（移动端）
- **Rust**：1.97.1，`--release`，LTO thin，codegen-units=1

### GPU 加速收益

| 计算类型                          | CPU      | GPU（GTX 1060） |   加速比   |
|-----------------------------------|----------|-----------------|:----------:|
| 八度柏林噪声（1,200 点）          | 2.17 ms  | 0.22 ms         |  **9.7×**  |
| 下界路由器（1,200 点）            | 2.40 ms  | 0.36 ms         |  **6.7×**  |
| 主世界路由器（1 区块，1,200 点）  | 20.74 ms | 1.02 ms         | **20.3×**  |
| 主世界路由器（267 区块，320K 点） | 6,916 ms | 47 ms           | **146.9×** |

### CPU 路径对比（GPU-accelerated vs master）

| 基准测试                | master   | GPU-accelerated |  差异  |
|-------------------------|----------|-----------------|:------:|
| `noise_generation`      | 33.74 ms | 28.09 ms        | −16.8% |
| `noise_router_creation` | 105.1 µs | 90.5 µs         | −13.9% |

> 差异在系统测量方差范围内（未修改 crate 间观测到 5–20%）。`#[cfg(feature = "gpu")]` 门控使 CPU-only 构建与 master 结构相同。

### 端到端估算（每区块，总计约 41.9 ms）

| 阶段      | 占比  | GPU 就绪  | 阶段加速比 | 端到端增益 |
|-----------|:-----:|:---------:|:----------:|:----------:|
| 光照      | 37.4% | ✅ 已接入 |   ~2–4×    | 1.17–1.30× |
| 噪声      | 35.9% | ✅ 已接入 |  20–147×   | 1.52–1.55× |
| 地表      | 9.4%  | ✅ 已接入 | 批量预计算 |    边际    |
| 洞穴/裂谷 | 5.2%  | ✅ 已接入 | 批量预计算 |    边际    |

- **全部四个阶段 GPU 加速**：理论上限 **~3.7×**
- **光照 + 地表 + 噪声同时启用**：预估 **1.8–2.5×** 每区块

---

## 配置系统

### 最小配置

默认值对大多数用户已够用。以下是最小可工作的配置：

```toml
[gpu]
enabled = true
```

完整配置及每个字段的详细说明见下文。

---

### `gpu.backend` — 图形 API 后端选择

wgpu 通过你系统的原生图形 API 访问 GPU。`backend` 字段控制在**所有可用 GPU** 上使用哪一个后端。

#### `"auto"`（默认）的优先级

当设为 `"auto"` 时，wgpu 按以下顺序探测：

| 平台        |  优先级 1  | 优先级 2 | 优先级 3 |
|-------------|:----------:|:--------:|:--------:|
| **Linux**   |   Vulkan   |    GL    |    —     |
| **macOS**   |   Metal    |    —     |    —     |
| **Windows** | DirectX 12 |  Vulkan  |    GL    |

即：在 Linux 上，Vulkan 优先，GL (OpenGL/GLES) 作为后备；在 macOS 上仅 Metal；在 Windows 上 DX12 优先于 Vulkan。

#### 强制指定后端

如果自动探测选错了后端或你需要为 CI/容器指定某一特定后端：

```toml
[gpu]
backend = "vulkan"   # 可选项：auto | vulkan | metal | dx12 | gl
```

| 值         | 对应 wgpu 后端          | 适用平台           |
|------------|-------------------------|--------------------|
| `"auto"`   | 平台自动（见上表）      | 所有               |
| `"vulkan"` | `wgpu::Backend::Vulkan` | Linux, Windows     |
| `"metal"`  | `wgpu::Backend::Metal`  | macOS              |
| `"dx12"`   | `wgpu::Backend::Dx12`   | Windows            |
| `"gl"`     | `wgpu::Backend::Gl`     | 所有（兼容性最好） |

> **注意**：强制后端会跳过 wgpu 的自动探测，直接枚举**该后端的所有适配器**，按设备类型排序（独立 GPU > 集成 GPU > CPU），再结合 `device.strategy` 筛选。

---

### `gpu.device.strategy` — 设备选择策略

系统可能有多个 GPU（例如笔记本的独立显卡 + 集成显卡），`device.strategy` 决定选哪个。

#### `"auto"`（默认）

调用 wgpu 高层 `request_adapter` API。行为取决于 `PowerPreference`：

- 通常选择：`PowerPreference::HighPerformance` → wgpu 优先返回**独立显卡**
- 仅当 `strategy = "integrated"` 时使用 `PowerPreference::LowPower`
- 如果返回的适配器不匹配 `name`/`index` 过滤，则回退到枚举**所有后端的所有适配器**

**适用场景**：绝大多数用户 — 让 wgpu 自己做最优选择。

#### `"index"` — 按索引选择

```toml
[gpu.device]
strategy = "index"
index = 0    # 第一个 GPU（通常为独立显卡）
# index = 1  # 第二个 GPU（通常为集成显卡）
```

按 wgpu 枚举顺序的零基索引选取。枚举顺序取决于驱动和平台，**不一定稳定**（系统更新后顺序可能变化）。

**适用场景**：多 GPU 服务器，你确切知道顺序且需要在两张同型号卡中选特定的那张。

#### `"name"` — 按名称子串匹配

```toml
[gpu.device]
strategy = "name"
name = "GTX 1060"    # 大小写不敏感，匹配适配器名称的任意子串
```

在所有适配器的 `adapter.get_info().name` 中**大小写不敏感子串匹配**。多个匹配时返回第一个（通常已按独立 > 集成排序）。

**适用场景**：适配器顺序不可预测（跨重启/驱动更新），但你知道 GPU 名称中包含的特定字符串。这是**最稳定的选择策略**。

#### `"integrated"` — 优先集成显卡

```toml
[gpu.device]
strategy = "integrated"
```

选择 `DeviceType::IntegratedGpu` 的第一个适配器。找不到集成 GPU 时回退到第一个可用适配器。同时使用 `PowerPreference::LowPower` 实现节能。

**适用场景**：笔记本省电模式、独立显卡需要留给显示器或其他应用。

---

### 字段速查表

| 字段                       | 类型   | 默认     | 说明                                                  |
|----------------------------|--------|----------|-------------------------------------------------------|
| `gpu.enabled`              | bool   | `true`   | 总开关，`false` 时行为与无 GPU 构建完全相同           |
| `gpu.noise_acceleration`   | bool   | `true`   | GPU 区块密度函数图评估（单次调度 98K 点）             |
| `gpu.light_acceleration`   | bool   | `true`   | GPU 天空光/方块光列扫描                               |
| `gpu.surface_acceleration` | bool   | `true`   | GPU 地表 + 洞穴/裂谷 DoublePerlin 批量噪声            |
| `gpu.backend`              | enum   | `"auto"` | 图形后端：`auto` / `vulkan` / `metal` / `dx12` / `gl` |
| `gpu.device.strategy`      | enum   | `"auto"` | 设备选择：`auto` / `index` / `name` / `integrated`    |
| `gpu.device.index`         | u32    | —        | `strategy = "index"` 时的适配器索引                   |
| `gpu.device.name`          | string | —        | `strategy = "name"` 时的名称子串（大小写不敏感）      |

---

### 实例配置：双 GPU 笔记本（我本人本机：GTX 1060 + UHD 630）

以下示例基于本机环境 — 一台同时拥有 NVIDIA GeForce GTX 1060（独立显卡）和 Intel UHD Graphics 630（集成显卡）的笔记本，运行 Arch Linux。

#### 查看系统中可用的 GPU

```bash
# Linux 列出所有 GPU 硬件
lspci -nn | grep -i 'vga\|3d\|display'
# 输出示例：
# 00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 630
# 01:00.0 VGA compatible controller: NVIDIA Corporation GP106M [GeForce GTX 1060 Mobile]

# 获取 Vulkan 设备名称（用于 strategy = "name"）
vulkaninfo --summary 2>/dev/null | grep deviceName
# 输出示例：
#     deviceName = Intel(R) UHD Graphics 630 (CFL GT2)
#     deviceName = NVIDIA GeForce GTX 1060

# NVIDIA 专有驱动用户还可以用 nvidia-smi
nvidia-smi --query-gpu=name,uuid --format=csv,noheader
# 输出示例：
# NVIDIA GeForce GTX 1060, GPU-38f549ee-b884-b9ac-e134-da8017e0336c
```

#### 场景 1：使用独立显卡获得最佳性能（推荐默认）

```toml
[gpu]
enabled = true
backend = "auto"           # Linux 上自动选 Vulkan

[gpu.device]
strategy = "name"
name = "GTX 1060"          # 匹配 "NVIDIA GeForce GTX 1060"
```

#### 场景 2：省电 — 使用集成显卡

```toml
[gpu]
enabled = true

[gpu.device]
strategy = "integrated"    # 选 Intel UHD Graphics 630
```

#### 场景 3：强制 Vulkan + 索引选择

```toml
[gpu]
enabled = true
backend = "vulkan"

[gpu.device]
strategy = "index"
index = 0                  # GTX 1060（通常先被枚举）
```

#### 场景 4：无头服务器（仅计算，无显示）

```toml
[gpu]
enabled = true
backend = "vulkan"         # 无头环境推荐明确指定 Vulkan

[gpu.device]
strategy = "name"
name = "RTX 4090"          # 按名称稳定匹配
```

#### 场景 5：容器/CI 环境

```toml
[gpu]
enabled = false            # 无 GPU 可用时直接禁用
```

> **提示**：不确定 `name` 写什么？先使用 `strategy = "index"` + `index = 0` 启动一次，查看启动日志中输出了哪个适配器名称，再改回 `strategy = "name"` 固定下来。

---

## 已实施优化

| # | 优化                                        | 影响                                        |
|---|---------------------------------------------|---------------------------------------------|
| 1 | `sample_batch()` DashMap 缓冲区缓存         | 相同采样器跨调度复用 GPU 缓冲区             |
| 2 | `PreparedGraph` point_capacity=256          | 避免首次调度立即重新分配                    |
| 3 | 移除占位符绑定组                            | 每个图节省 17 个 GPU 缓冲区                 |
| 4 | 合并 `read_back_range`/`read_back_u8_range` | 消除约 40 行重复代码                        |
| 5 | 提升 OctaveBatch 提取                       | 4× 调用 → 每个采样器一次                    |
| 6 | Arc 共享噪声批次                            | 洞穴/裂谷 top_material 从 4KB 复制变为 O(1) |
| 7 | 优雅的轮询错误处理                          | 设备丢失时返回空结果，不触发 panic          |

---

## 验证清单

- [x] `cargo test -p pumpkin-gpu --release` — **44/44** 通过
- [x] `cargo test -p pumpkin-world -- carver` — **10/10** 通过
- [x] `cargo test -p pumpkin-world -- gpu_batch` — **4/4** 通过
- [x] 全部 4 个阶段均有 GPU 回调及干净的 CPU 回退
- [x] 全部 3 个 `OnceLock` 注册表正确门控
- [x] 洞穴/裂谷 OOB 坐标边界检查安全网
- [x] `Arc<[f64]>` 共享批次（O(1) 克隆）
- [x] GPU 缓冲区缓存消除重复上传
- [x] `noise_acceleration` / `surface_acceleration` / `light_acceleration` 标志均已生效
- [x] 地表 + 洞穴/裂谷测试验证批次 vs CPU 路径
