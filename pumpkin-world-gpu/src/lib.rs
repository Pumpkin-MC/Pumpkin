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
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct OctaveParams {
    pub x_origin: f32,
    pub y_origin: f32,
    pub z_origin: f32,
    pub amplitude: f32,
    pub persistence: f32,
    pub lacunarity: f32,
    padding0: f32,
    padding1: f32,
}

impl OctaveParams {
    #[must_use]
    pub const fn new(
        x_origin: f32,
        y_origin: f32,
        z_origin: f32,
        amplitude: f32,
        persistence: f32,
        lacunarity: f32,
    ) -> Self {
        Self {
            x_origin,
            y_origin,
            z_origin,
            amplitude,
            persistence,
            lacunarity,
            padding0: 0.0,
            padding1: 0.0,
        }
    }
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
            params.push(OctaveParams::new(
                x_origin as f32,
                y_origin as f32,
                z_origin as f32,
                data.amplitude as f32,
                data.persistence as f32,
                data.lacunarity as f32,
            ));
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

        // The graph pipeline binds more storage buffers than wgpu's conservative
        // defaults allow, so ask for what this adapter actually supports. An adapter
        // too limited for the pipeline fails here and the caller falls back to the CPU.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("pumpkin-world-gpu noise device"),
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await
            .ok()?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("octave_perlin"),
            source: wgpu::ShaderSource::Wgsl(include_str!("octave_perlin.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    storage_entry(5, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(6, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(7, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(8, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(9, wgpu::BufferBindingType::Storage { read_only: true }),
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
    pub fn evaluate_graph(&self, compiled: &graph::CompiledGraph, points: &[[f32; 3]]) -> Vec<f32> {
        let instructions = &compiled.instructions;
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

        // WGSL storage bindings must be non-empty even when a graph uses no noise
        // nodes, so fall back to a single zeroed element rather than a 0-byte buffer.
        let samplers_buffer =
            self.storage_or_placeholder("graph samplers", &compiled.samplers.samplers);
        let octaves_buffer =
            self.storage_or_placeholder("graph octaves", &compiled.samplers.octaves);
        let permutations_buffer =
            self.storage_or_placeholder("graph permutations", &compiled.samplers.permutations);
        let spline_points_buffer =
            self.storage_or_placeholder("graph spline points", &compiled.spline_points);
        let interpolated_buffer = self.storage_or_placeholder(
            "graph interpolated samplers",
            &compiled.samplers.interpolated,
        );

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("density_graph bind group"),
            layout: &self.graph_bind_group_layout,
            entries: &[
                bind_entry(0, &dims_buffer),
                bind_entry(1, &instructions_buffer),
                bind_entry(2, &points_buffer),
                bind_entry(3, &scratch_buffer),
                bind_entry(4, &output_buffer),
                bind_entry(5, &samplers_buffer),
                bind_entry(6, &octaves_buffer),
                bind_entry(7, &permutations_buffer),
                bind_entry(8, &spline_points_buffer),
                bind_entry(9, &interpolated_buffer),
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

    /// Uploads `data` as a storage buffer, substituting one zeroed element when it is
    /// empty: WGSL rejects zero-sized storage bindings, and the placeholder has to be a
    /// full element wide or validation rejects the size mismatch instead.
    fn storage_or_placeholder<T: Pod>(&self, label: &str, data: &[T]) -> wgpu::Buffer {
        let placeholder;
        let contents = if data.is_empty() {
            placeholder = vec![0u8; std::mem::size_of::<T>().max(4)];
            &placeholder[..]
        } else {
            bytemuck::cast_slice(data)
        };
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::STORAGE,
            })
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
            let cpu_value = cpu_sampler.sample(
                f64::from(point[0]),
                f64::from(point[1]),
                f64::from(point[2]),
            );
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

        let compiled = crate::graph::CompiledGraph {
            instructions: crate::graph::test::sample_graph(),
            samplers: crate::graph::SamplerPool::default(),
            spline_points: Vec::new(),
        };

        // Spread y across the gradient's clamped range and both saturated ends.
        let points: Vec<[f32; 3]> = (0..512)
            .map(|i| {
                let y = -128.0 + (i as f32) * 0.75;
                [i as f32 * 0.5, y, i as f32 * -0.25]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph(&compiled, &points);
        assert_eq!(gpu_results.len(), points.len());

        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let cpu_value = crate::graph::evaluate_cpu(
                &compiled.instructions,
                &compiled.samplers,
                &compiled.spline_points,
                point[0],
                point[1],
                point[2],
            );
            assert!(
                (cpu_value - gpu_value).abs() < 1e-5,
                "graph mismatch at {point:?}: cpu={cpu_value} gpu={gpu_value}"
            );
        }
    }

    /// End-to-end check of the Noise opcode: a graph containing a real seeded sampler,
    /// evaluated on the GPU, against the CPU `DoublePerlinNoiseSampler` it came from.
    /// This is what proves the sampler pool is uploaded and indexed correctly.
    #[test]
    fn gpu_noise_opcode_matches_real_cpu_sampler() {
        use pumpkin_data::chunk::DoublePerlinNoiseParameters;
        use pumpkin_world::generation::noise::perlin::DoublePerlinNoiseSampler;
        use pumpkin_world::generation::noise::router::proto_noise_router::DoublePerlinNoiseBuilder;

        const AMPLITUDES: &[f64] = &[1.0, 1.0, 1.0];

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        let params = DoublePerlinNoiseParameters::new(
            0,
            -7,
            AMPLITUDES,
            0x5F3B_1A77,
            0x91E4_C2D0,
            DoublePerlinNoiseSampler::get_amplitude(AMPLITUDES),
        );
        let deriver = crate::graph::test::test_deriver();
        let cpu_sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(&deriver, &params);

        let mut samplers = crate::graph::SamplerPool::default();
        let (first, second) = cpu_sampler.samplers();
        let sampler_index = samplers.push_double_perlin(first, second, cpu_sampler.amplitude());

        let mut noise = crate::graph::Instruction::noise(0, sampler_index, 1.0, 1.0);
        noise.input1 = 0;
        let compiled = crate::graph::CompiledGraph {
            instructions: vec![noise],
            samplers,
            spline_points: Vec::new(),
        };

        let points: Vec<[f32; 3]> = (0..1000)
            .map(|i| {
                let f = i as f32;
                [f * 1.7, f * -0.9, f * 2.3]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph(&compiled, &points);

        let mut max_diff = 0.0f64;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let expected = cpu_sampler.sample(
                f64::from(point[0]),
                f64::from(point[1]),
                f64::from(point[2]),
            );
            max_diff = max_diff.max((expected - f64::from(gpu_value)).abs());
        }

        assert!(
            max_diff < 1e-3,
            "GPU Noise opcode diverged from the CPU sampler by {max_diff}"
        );
    }
    /// `ShiftA`/`ShiftB` feed the sampler rotated, partly-zeroed coordinates, so a
    /// transcription slip there is invisible unless checked against vanilla's exact
    /// argument order. `ShiftedNoise` then offsets the point by three input nodes.
    #[test]
    fn gpu_shift_opcodes_match_cpu_semantics() {
        use crate::graph::{Instruction, OpCode};
        use pumpkin_data::chunk::DoublePerlinNoiseParameters;
        use pumpkin_world::generation::noise::perlin::DoublePerlinNoiseSampler;
        use pumpkin_world::generation::noise::router::proto_noise_router::DoublePerlinNoiseBuilder;

        const AMPLITUDES: &[f64] = &[1.0, 1.0];

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        let params = DoublePerlinNoiseParameters::new(
            0,
            -5,
            AMPLITUDES,
            0x2C1D_9B04,
            0x77A0_5E31,
            DoublePerlinNoiseSampler::get_amplitude(AMPLITUDES),
        );
        let deriver = crate::graph::test::test_deriver();
        let cpu_sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(&deriver, &params);

        let mut samplers = crate::graph::SamplerPool::default();
        let (first, second) = cpu_sampler.samplers();
        let sampler_index = samplers.push_double_perlin(first, second, cpu_sampler.amplitude());

        // Node 0: ShiftA, node 1: ShiftB, node 2: ShiftedNoise offset by both.
        let mut shift_a = Instruction::new_for_test(OpCode::ShiftA, 0);
        shift_a.sampler_index = sampler_index;
        let mut shift_b = Instruction::new_for_test(OpCode::ShiftB, 1);
        shift_b.sampler_index = sampler_index;
        let mut shifted = Instruction::new_for_test(OpCode::ShiftedNoise, 2);
        shifted.sampler_index = sampler_index;
        shifted.input0 = 0;
        shifted.input1 = 1;
        shifted.input2 = 0;
        shifted.param0 = 0.25;
        shifted.param1 = 0.125;

        let compiled = crate::graph::CompiledGraph {
            instructions: vec![shift_a, shift_b, shifted],
            samplers,
            spline_points: Vec::new(),
        };

        let points: Vec<[f32; 3]> = (0..600)
            .map(|i| {
                let f = i as f32;
                [f * 2.1, f * -1.3, f * 0.7]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph(&compiled, &points);

        // Independent expectation, written straight from density_function/noise.rs
        // rather than reusing the crate's own interpreter.
        let shift_sample_3d = |x: f64, y: f64, z: f64| -> f64 {
            cpu_sampler.sample(x * 0.25, y * 0.25, z * 0.25) * 4.0
        };

        let mut max_diff = 0.0f64;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let (x, y, z) = (
                f64::from(point[0]),
                f64::from(point[1]),
                f64::from(point[2]),
            );
            let a = shift_sample_3d(x, 0.0, z);
            let b = shift_sample_3d(z, x, 0.0);
            let expected =
                cpu_sampler.sample(x.mul_add(0.25, a), y.mul_add(0.125, b), z.mul_add(0.25, a));
            max_diff = max_diff.max((expected - f64::from(gpu_value)).abs());
        }

        assert!(
            max_diff < 1e-2,
            "GPU shift opcodes diverged from vanilla semantics by {max_diff}"
        );
    }
    /// Splines nest: a knot's value can be another spline. Flattening them into
    /// separate instructions is the whole trick that removes runtime recursion, so this
    /// checks a two-level spline against an expectation transcribed from
    /// `density_function/spline.rs` rather than from this crate's own interpreter.
    #[test]
    fn gpu_nested_spline_matches_vanilla_semantics() {
        // Independent transcription of Spline::sample for a two-knot spline.
        fn eval_spline(loc: f32, knots: [(f32, f32, f32); 2]) -> f32 {
            let outside = |k: (f32, f32, f32), last: f32| -> f32 {
                if k.1 == 0.0 {
                    last
                } else {
                    k.1 * (loc - k.0) + last
                }
            };
            if loc < knots[0].0 {
                return outside(knots[0], knots[0].2);
            }
            if loc >= knots[1].0 {
                return outside(knots[1], knots[1].2);
            }
            let (lower, upper) = (knots[0], knots[1]);
            let dist = upper.0 - lower.0;
            let x_scale = (loc - lower.0) / dist;
            let delta = upper.2 - lower.2;
            let extrap_lo = lower.1 * dist - delta;
            let extrap_hi = -upper.1 * dist + delta;
            let lerp = |t: f32, a: f32, b: f32| a + t * (b - a);
            (x_scale * (1.0 - x_scale)) * lerp(x_scale, extrap_lo, extrap_hi)
                + lerp(x_scale, lower.2, upper.2)
        }

        use crate::graph::{CompiledGraph, GpuSplinePoint, Instruction, OpCode};

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        // Node 0: location = ClampedYGradient over y, giving a spread of locations.
        let mut location = Instruction::new_for_test(OpCode::ClampedYGradient, 0);
        location.param0 = 0.0;
        location.param1 = 100.0;
        location.param2 = 0.0;
        location.param3 = 1.0;

        // Nodes 1-2: constants used by the inner spline's knots.
        let mut inner_lo = Instruction::new_for_test(OpCode::Constant, 1);
        inner_lo.param0 = -0.5;
        let mut inner_hi = Instruction::new_for_test(OpCode::Constant, 2);
        inner_hi.param0 = 2.0;

        // Node 3: inner spline over the same location, knots at 0.2 / 0.8.
        let mut inner = Instruction::new_for_test(OpCode::Spline, 3);
        inner.input0 = 0;
        inner.aux0 = 0;
        inner.aux1 = 2;

        // Node 4: constant used as the outer spline's other knot.
        let mut outer_far = Instruction::new_for_test(OpCode::Constant, 4);
        outer_far.param0 = 5.0;

        // Node 5: outer spline whose first knot's value IS the inner spline.
        let mut outer = Instruction::new_for_test(OpCode::Spline, 5);
        outer.input0 = 0;
        outer.aux0 = 2;
        outer.aux1 = 2;

        let spline_points = vec![
            // inner spline knots
            GpuSplinePoint::new(0.2, 0.0, 1),
            GpuSplinePoint::new(0.8, 1.5, 2),
            // outer spline knots; knot 0 pulls its value from the inner spline (node 3)
            GpuSplinePoint::new(0.3, 0.5, 3),
            GpuSplinePoint::new(0.9, 0.0, 4),
        ];

        let compiled = CompiledGraph {
            instructions: vec![location, inner_lo, inner_hi, inner, outer_far, outer],
            samplers: crate::graph::SamplerPool::default(),
            spline_points,
        };

        let points: Vec<[f32; 3]> = (0..400)
            .map(|i| [0.0, i as f32 * 0.3 - 10.0, 0.0])
            .collect();

        let gpu_results = ctx.evaluate_graph(&compiled, &points);

        let mut max_diff = 0.0f32;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            // ClampedYGradient(y) over [0,100] -> [0,1]
            let loc = (point[1] / 100.0).clamp(0.0, 1.0);
            let inner_value = eval_spline(loc, [(0.2, 0.0, -0.5), (0.8, 1.5, 2.0)]);
            let expected = eval_spline(loc, [(0.3, 0.5, inner_value), (0.9, 0.0, 5.0)]);
            max_diff = max_diff.max((expected - gpu_value).abs());
        }

        assert!(
            max_diff < 1e-4,
            "GPU nested spline diverged from vanilla semantics by {max_diff}"
        );
    }
    /// `InterpolatedNoiseSampler` is the one node that samples with a non-zero vertical
    /// scale, which triggers the Y-quantization branch of `sample_no_fade`, and that
    /// walks its octaves in reverse against halving fractions. Checked against the real
    /// CPU sampler rather than this crate's interpreter.
    #[test]
    fn gpu_interpolated_noise_matches_cpu_sampler() {
        use crate::graph::{CompiledGraph, Instruction, OpCode, SamplerPool};
        use pumpkin_data::noise_router::InterpolatedNoiseSamplerData;
        use pumpkin_util::random::xoroshiro128::Xoroshiro;
        use pumpkin_world::generation::noise::router::density_function::{
            StaticIndependentChunkNoiseFunctionComponentImpl, noise::InterpolatedNoiseSampler,
        };

        // Vanilla's overworld values.
        static DATA: InterpolatedNoiseSamplerData = InterpolatedNoiseSamplerData {
            scaled_xz_scale: 0.25,
            scaled_y_scale: 0.125,
            xz_factor: 80.0,
            y_factor: 160.0,
            smear_scale_multiplier: 8.0,
        };

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        let mut random = Xoroshiro::from_seed(9876);
        let cpu_sampler = InterpolatedNoiseSampler::new(&DATA, &mut random);

        let mut samplers = SamplerPool::default();
        let sampler_index = samplers.push_interpolated(&cpu_sampler);

        let mut node = Instruction::new_for_test(OpCode::InterpolatedNoise, 0);
        node.sampler_index = sampler_index;

        let compiled = CompiledGraph {
            instructions: vec![node],
            samplers,
            spline_points: Vec::new(),
        };

        // Block coordinates in a realistic range; f32 loses ground far from origin, so
        // this stays where a server actually generates terrain.
        let points: Vec<[f32; 3]> = (0..300)
            .map(|i| {
                let f = i as f32;
                [f * 3.0 - 400.0, (f % 64.0) * 4.0 - 64.0, f * 2.0 - 300.0]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph(&compiled, &points);

        let mut max_diff = 0.0f64;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let pos = pumpkin_util::math::vector3::Vector3::new(
                point[0] as i32,
                point[1] as i32,
                point[2] as i32,
            );
            let expected = cpu_sampler.sample(&pos);
            max_diff = max_diff.max((expected - f64::from(gpu_value)).abs());
        }

        // Looser than the other opcodes: this one multiplies coordinates by ~684 before
        // sampling, so f32 rounding is amplified well beyond a plain noise lookup.
        assert!(
            max_diff < 5e-2,
            "GPU interpolated noise diverged from the CPU sampler by {max_diff}"
        );
    }
}
