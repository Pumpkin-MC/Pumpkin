//! GPU-accelerated (wgpu) noise sampling prototype for `pumpkin-world`.
//!
//! Ports [`pumpkin_util::noise::perlin::OctavePerlinNoiseSampler`] to a WGSL compute
//! shader (see `octave_perlin.wgsl`) so a batch of independent sample points can be
//! evaluated in parallel instead of one at a time on the CPU.
//!
//! This intentionally runs in f32, not f64 like the CPU sampler: it will not reproduce
//! vanilla Minecraft terrain bit-for-bit for a given seed, only its own output
//! deterministically. GPU availability is never assumed — [`GpuNoiseContext::try_new`]
//! returns `None` when there is no compatible adapter, and callers must fall back to
//! the CPU sampler in that case.

use bytemuck::{Pod, Zeroable};
use pumpkin_util::noise::perlin::OctavePerlinNoiseSampler;
use wgpu::util::DeviceExt;

pub mod graph;

const WORKGROUP_SIZE: u32 = 64;

/// GPU-side mirror of one octave's parameters (see `SamplerData` in
/// `pumpkin-util/src/noise/perlin.rs`). Padded to 32 bytes to keep
/// `array<OctaveParams>` naturally aligned in the WGSL storage buffer.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct OctaveParams {
    pub x_origin: f32,
    pub y_origin: f32,
    pub z_origin: f32,
    pub amplitude: f32,
    pub persistence: f32,
    pub lacunarity: f32,
    _pad0: f32,
    _pad1: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Dims {
    num_points: u32,
    num_octaves: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GraphDims {
    num_points: u32,
    num_instructions: u32,
}

/// A GPU-ready snapshot of an [`OctavePerlinNoiseSampler`]'s state: per-octave
/// parameters plus each octave's 256-entry permutation table, flattened.
pub struct OctaveBatch {
    pub params: Vec<OctaveParams>,
    /// `[octave_index * 256 + permutation_index]`, values in `0..256`.
    pub permutations: Vec<u32>,
}

impl OctaveBatch {
    /// Extracts GPU-ready parameters from a real, seeded CPU sampler, so the GPU path
    /// mirrors an actual sampler instance instead of synthetic test data.
    #[must_use]
    pub fn from_cpu_sampler(sampler: &OctavePerlinNoiseSampler) -> Self {
        let mut params = Vec::with_capacity(sampler.samplers.len());
        let mut permutations = Vec::with_capacity(sampler.samplers.len() * 256);

        for data in &sampler.samplers {
            let (x_origin, y_origin, z_origin) = data.sampler.origin();
            params.push(OctaveParams {
                x_origin: x_origin as f32,
                y_origin: y_origin as f32,
                z_origin: z_origin as f32,
                amplitude: data.amplitude as f32,
                persistence: data.persistence as f32,
                lacunarity: data.lacunarity as f32,
                _pad0: 0.0,
                _pad1: 0.0,
            });
            permutations.extend(data.sampler.permutation().iter().map(|&b| u32::from(b)));
        }

        Self {
            params,
            permutations,
        }
    }

    #[must_use]
    pub const fn num_octaves(&self) -> usize {
        self.params.len()
    }
}

/// An initialized GPU compute context for octave-noise sampling.
///
/// Construction never panics on missing/unsupported hardware — use [`Self::try_new`]
/// and fall back to the CPU sampler when it returns `None`.
pub struct GpuNoiseContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    graph_pipeline: wgpu::ComputePipeline,
    graph_bind_group_layout: wgpu::BindGroupLayout,
    pub adapter_name: String,
    pub adapter_is_discrete: bool,
}

impl GpuNoiseContext {
    /// Tries to initialize a GPU compute context, blocking until adapter/device
    /// request completes. Returns `None` if no compatible GPU is available.
    #[must_use]
    pub fn try_new() -> Option<Self> {
        pollster::block_on(Self::try_new_async())
    }

    async fn try_new_async() -> Option<Self> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok()?;

        let info = adapter.get_info();
        let adapter_name = info.name.clone();
        let adapter_is_discrete = info.device_type == wgpu::DeviceType::DiscreteGpu;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("pumpkin-world-gpu noise device"),
                ..Default::default()
            })
            .await
            .ok()?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("octave_perlin"),
            source: wgpu::ShaderSource::Wgsl(include_str!("octave_perlin.wgsl").into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("octave_perlin bind group layout"),
                entries: &[
                    storage_entry(0, wgpu::BufferBindingType::Uniform),
                    storage_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(3, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(4, wgpu::BufferBindingType::Storage { read_only: false }),
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("octave_perlin pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("octave_perlin pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("sample_octaves"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let graph_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("density_graph"),
            source: wgpu::ShaderSource::Wgsl(include_str!("graph.wgsl").into()),
        });

        let graph_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("density_graph bind group layout"),
                entries: &[
                    storage_entry(0, wgpu::BufferBindingType::Uniform),
                    storage_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(3, wgpu::BufferBindingType::Storage { read_only: false }),
                    storage_entry(4, wgpu::BufferBindingType::Storage { read_only: false }),
                ],
            });

        let graph_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("density_graph pipeline layout"),
                bind_group_layouts: &[Some(&graph_bind_group_layout)],
                immediate_size: 0,
            });

        let graph_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("density_graph pipeline"),
            layout: Some(&graph_pipeline_layout),
            module: &graph_shader,
            entry_point: Some("evaluate_graph"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Some(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            graph_pipeline,
            graph_bind_group_layout,
            adapter_name,
            adapter_is_discrete,
        })
    }

    /// Evaluates a compiled density-function graph (see [`graph::compile`]) at every
    /// point in `points`, returning the root node's value per point.
    #[must_use]
    pub fn evaluate_graph(
        &self,
        instructions: &[graph::Instruction],
        points: &[[f32; 3]],
    ) -> Vec<f32> {
        if points.is_empty() || instructions.is_empty() {
            return vec![0.0; points.len()];
        }

        let dims = GraphDims {
            num_points: points.len() as u32,
            num_instructions: instructions.len() as u32,
        };

        let dims_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("graph dims"),
                contents: bytemuck::bytes_of(&dims),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let instructions_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("graph instructions"),
                    contents: bytemuck::cast_slice(instructions),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let flat_points: Vec<f32> = points.iter().flat_map(|p| p.iter().copied()).collect();
        let points_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("graph points"),
                contents: bytemuck::cast_slice(&flat_points),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let scratch_size = (instructions.len() * points.len() * std::mem::size_of::<f32>()) as u64;
        let scratch_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph scratch"),
            size: scratch_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let output_size = (points.len() * std::mem::size_of::<f32>()) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph out_density"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph out_density staging"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("density_graph bind group"),
            layout: &self.graph_bind_group_layout,
            entries: &[
                bind_entry(0, &dims_buffer),
                bind_entry(1, &instructions_buffer),
                bind_entry(2, &points_buffer),
                bind_entry(3, &scratch_buffer),
                bind_entry(4, &output_buffer),
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("density_graph encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("density_graph pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.graph_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(dims.num_points.div_ceil(WORKGROUP_SIZE), 1, 1);
        };
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_size);
        self.queue.submit(Some(encoder.finish()));

        self.read_back(&staging_buffer)
    }

    fn read_back(&self, staging_buffer: &wgpu::Buffer) -> Vec<f32> {
        let slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll failed while waiting for GPU readback");
        rx.recv()
            .expect("map_async callback dropped without sending a result")
            .expect("failed to map GPU output buffer for readback");

        let data = slice
            .get_mapped_range()
            .expect("buffer was mapped but get_mapped_range failed");
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();
        result
    }

    /// Samples `batch` at every point in `points`, returning one density value per
    /// point in the same order.
    #[must_use]
    pub fn sample_batch(&self, batch: &OctaveBatch, points: &[[f32; 3]]) -> Vec<f32> {
        if points.is_empty() || batch.num_octaves() == 0 {
            return vec![0.0; points.len()];
        }

        let dims = Dims {
            num_points: points.len() as u32,
            num_octaves: batch.num_octaves() as u32,
        };

        let dims_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dims"),
                contents: bytemuck::bytes_of(&dims),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let octaves_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("octaves"),
                contents: bytemuck::cast_slice(&batch.params),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let permutations_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("permutations"),
                    contents: bytemuck::cast_slice(&batch.permutations),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let flat_points: Vec<f32> = points.iter().flat_map(|p| p.iter().copied()).collect();
        let points_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("points"),
                contents: bytemuck::cast_slice(&flat_points),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let output_size = (points.len() * std::mem::size_of::<f32>()) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out_density"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out_density staging"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("octave_perlin bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                bind_entry(0, &dims_buffer),
                bind_entry(1, &octaves_buffer),
                bind_entry(2, &permutations_buffer),
                bind_entry(3, &points_buffer),
                bind_entry(4, &output_buffer),
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("octave_perlin encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("octave_perlin pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = dims.num_points.div_ceil(WORKGROUP_SIZE);
            pass.dispatch_workgroups(workgroups, 1, 1);
        };
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_size);
        self.queue.submit(Some(encoder.finish()));

        let slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll failed while waiting for GPU readback");
        rx.recv()
            .expect("map_async callback dropped without sending a result")
            .expect("failed to map GPU output buffer for readback");

        let data = slice
            .get_mapped_range()
            .expect("buffer was mapped but get_mapped_range failed");
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        result
    }
}

const fn storage_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bind_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

#[cfg(test)]
mod test {
    use super::{GpuNoiseContext, OctaveBatch};
    use pumpkin_util::{
        noise::perlin::OctavePerlinNoiseSampler,
        random::{RandomDeriverImpl, RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro},
    };

    fn make_reference_sampler() -> OctavePerlinNoiseSampler {
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(1234));
        let splitter = rand.next_splitter();
        let mut rand = splitter.split_string("minecraft:terrain");
        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
        OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true)
    }

    /// GPU (f32) output must land close to the CPU (f64) reference for the same
    /// sampler and points — not bit-exact (that's the accepted tradeoff), but close
    /// enough that a real algorithmic bug (wrong permutation indexing, wrong gradient
    /// table, wrong fade curve, ...) would fail this.
    #[test]
    fn gpu_matches_cpu_within_f32_tolerance() {
        let Some(ctx) = GpuNoiseContext::try_new() else {
            // No GPU in this environment (CI containers, headless builds): the CPU path
            // is what ships by default, so skipping keeps the suite green there.
            return;
        };

        let cpu_sampler = make_reference_sampler();
        let batch = OctaveBatch::from_cpu_sampler(&cpu_sampler);

        let points: Vec<[f32; 3]> = (0..2000)
            .map(|i| {
                let fx = f64::from(i) * 3.7;
                let fy = f64::from(i) * -1.9;
                let fz = f64::from(i) * 5.3;
                [fx as f32, fy as f32, fz as f32]
            })
            .collect();

        let gpu_results = ctx.sample_batch(&batch, &points);
        assert_eq!(gpu_results.len(), points.len());

        let mut max_abs_diff = 0.0f64;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let cpu_value =
                cpu_sampler.sample(f64::from(point[0]), f64::from(point[1]), f64::from(point[2]));
            let diff = (cpu_value - f64::from(gpu_value)).abs();
            max_abs_diff = max_abs_diff.max(diff);
        }

        // Noise values here are roughly in [-2, 2] (max_value affects the exact bound);
        // 1e-3 is generous for f32 vs f64 but tight enough to catch a real algorithm bug.
        assert!(
            max_abs_diff < 1e-3,
            "GPU/CPU noise diverged by {max_abs_diff}, larger than f32 precision alone should cause"
        );
    }

    /// The WGSL graph interpreter must agree with the CPU reference interpreter in
    /// `graph.rs` for every supported opcode — both run in f32, so this is where a
    /// mistranslated opcode (wrong operand order, missing short-circuit, ...) shows up.
    #[test]
    fn gpu_graph_matches_cpu_reference() {
        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        let instructions = crate::graph::test::sample_graph();

        // Spread y across the gradient's clamped range and both saturated ends.
        let points: Vec<[f32; 3]> = (0..512)
            .map(|i| {
                let y = -128.0 + (i as f32) * 0.75;
                [i as f32 * 0.5, y, i as f32 * -0.25]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph(&instructions, &points);
        assert_eq!(gpu_results.len(), points.len());

        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let cpu_value = crate::graph::evaluate_cpu(&instructions, point[0], point[1], point[2]);
            assert!(
                (cpu_value - gpu_value).abs() < 1e-5,
                "graph mismatch at {point:?}: cpu={cpu_value} gpu={gpu_value}"
            );
        }
    }
}
