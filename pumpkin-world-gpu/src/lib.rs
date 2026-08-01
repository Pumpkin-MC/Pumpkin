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

/// Uniform header for a graph dispatch. Matches `Dims` in `graph.wgsl`; the `vec3`
/// fields there are 16-byte aligned, hence the interleaved scalars.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GraphDims {
    num_points: u32,
    num_instructions: u32,
    num_structures: u32,
    num_junctions: u32,
    affected_min: [i32; 3],
    has_affected_box: u32,
    affected_max: [i32; 3],
    padding: u32,
}

impl GraphDims {
    fn new(num_points: usize, num_instructions: usize, beardifier: &graph::BeardifierData) -> Self {
        let (affected_min, affected_max, has_affected_box) = beardifier
            .affected_box
            .map_or(([0, 0, 0], [0, 0, 0], 0), |(min, max)| {
                ([min.x, min.y, min.z], [max.x, max.y, max.z], 1)
            });
        Self {
            num_points: num_points as u32,
            num_instructions: num_instructions as u32,
            num_structures: beardifier.structures.len() as u32,
            num_junctions: beardifier.junctions.len() as u32,
            affected_min,
            has_affected_box,
            affected_max,
            padding: 0,
        }
    }
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
                    storage_entry(10, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(11, wgpu::BufferBindingType::Storage { read_only: true }),
                    storage_entry(12, wgpu::BufferBindingType::Storage { read_only: true }),
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
    /// Evaluates `compiled` with no structures nearby — the common case for terrain
    /// away from generated structures.
    #[must_use]
    pub fn evaluate_graph(&self, compiled: &graph::CompiledGraph, points: &[[f32; 3]]) -> Vec<f32> {
        self.evaluate_graph_with(compiled, points, &graph::BeardifierData::default())
    }

    /// Evaluates `compiled` against per-chunk structure data.
    ///
    /// This uploads the graph's tables on every call. For repeated dispatches use
    /// [`Self::prepare`], which uploads them once.
    #[must_use]
    pub fn evaluate_graph_with(
        &self,
        compiled: &graph::CompiledGraph,
        points: &[[f32; 3]],
        beardifier: &graph::BeardifierData,
    ) -> Vec<f32> {
        self.prepare(compiled).evaluate(points, beardifier)
    }

    /// Uploads a compiled graph's tables to the GPU once, so repeated dispatches only
    /// pay for the point batch.
    #[must_use]
    pub fn prepare<'a>(&'a self, compiled: &graph::CompiledGraph) -> PreparedGraph<'a> {
        PreparedGraph::new(self, compiled)
    }

    /// Allocates the buffers whose size follows the point count.
    fn allocate_point_buffers(
        &self,
        point_capacity: usize,
        num_instructions: usize,
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        let f32_size = std::mem::size_of::<f32>() as u64;
        let points = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph points"),
            size: point_capacity as u64 * 3 * f32_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scratch = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph scratch"),
            size: (num_instructions * point_capacity) as u64 * f32_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph out_density"),
            size: point_capacity as u64 * f32_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph out_density staging"),
            size: point_capacity as u64 * f32_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        (points, scratch, output, staging)
    }

    /// A writable storage buffer sized for `capacity` elements, contents undefined.
    fn empty_storage<T: Pod>(&self, label: &str, capacity: usize) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (std::mem::size_of::<T>() * capacity.max(1)) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Bind group used only to initialize the field before the real one is built.
    fn placeholder_bind_group(&self) -> wgpu::BindGroup {
        let dummy = self.empty_storage::<f32>("placeholder", 16);
        let uniform = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("placeholder uniform"),
            size: std::mem::size_of::<GraphDims>() as u64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("placeholder bind group"),
            layout: &self.graph_bind_group_layout,
            entries: &[
                bind_entry(0, &uniform),
                bind_entry(1, &dummy),
                bind_entry(2, &dummy),
                bind_entry(3, &dummy),
                bind_entry(4, &dummy),
                bind_entry(5, &dummy),
                bind_entry(6, &dummy),
                bind_entry(7, &dummy),
                bind_entry(8, &dummy),
                bind_entry(9, &dummy),
                bind_entry(10, &dummy),
                bind_entry(11, &dummy),
                bind_entry(12, &dummy),
            ],
        })
    }

    /// Uploads `data` as a read-only storage buffer, substituting one zeroed element
    /// when it is empty: WGSL rejects zero-sized storage bindings, and the placeholder
    /// has to be a full element wide or validation rejects the size mismatch instead.
    fn storage_from<T: Pod>(&self, label: &str, data: &[T]) -> wgpu::Buffer {
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

    fn read_back_range(&self, staging: &wgpu::Buffer, size: u64) -> Vec<f32> {
        let slice = staging.slice(..size);
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
        let result = bytemuck::cast_slice(&data).to_vec();
        drop(data);
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

/// A compiled graph with its GPU buffers already uploaded.
///
/// The graph's tables — instructions, samplers, octaves, permutation tables, spline
/// knots — never change between dispatches, so uploading them once and reusing them is
/// what makes repeated evaluation cheap. Only the point batch and the per-chunk
/// beardifier data are rewritten per call, and their buffers are grown on demand rather
/// than reallocated each time.
pub struct PreparedGraph<'a> {
    context: &'a GpuNoiseContext,
    num_instructions: u32,

    // Uploaded once, in the order the bind group expects.
    dims: wgpu::Buffer,
    instructions: wgpu::Buffer,
    samplers: wgpu::Buffer,
    octaves: wgpu::Buffer,
    permutations: wgpu::Buffer,
    spline_points: wgpu::Buffer,
    interpolated: wgpu::Buffer,
    interval_entries: wgpu::Buffer,

    // Rewritten per dispatch; reallocated only when the batch outgrows them.
    points: wgpu::Buffer,
    scratch: wgpu::Buffer,
    output: wgpu::Buffer,
    staging: wgpu::Buffer,
    structures: wgpu::Buffer,
    junctions: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    point_capacity: usize,
    structure_capacity: usize,
    junction_capacity: usize,
    /// Scratch reused across calls to flatten `[[f32; 3]]` without allocating.
    flat_points: Vec<f32>,
}

impl<'a> PreparedGraph<'a> {
    fn new(context: &'a GpuNoiseContext, compiled: &graph::CompiledGraph) -> Self {
        let device = &context.device;
        let num_instructions = compiled.instructions.len() as u32;

        let dims = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph dims"),
            size: std::mem::size_of::<GraphDims>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instructions = context.storage_from("graph instructions", &compiled.instructions);
        let samplers = context.storage_from("graph samplers", &compiled.samplers.samplers);
        let octaves = context.storage_from("graph octaves", &compiled.samplers.octaves);
        let permutations =
            context.storage_from("graph permutations", &compiled.samplers.permutations);
        let spline_points = context.storage_from("graph spline points", &compiled.spline_points);
        let interpolated = context.storage_from(
            "graph interpolated samplers",
            &compiled.samplers.interpolated,
        );
        let interval_entries =
            context.storage_from("graph interval entries", &compiled.interval_entries);

        let point_capacity = 1;
        let (points, scratch, output, staging) =
            context.allocate_point_buffers(point_capacity, num_instructions as usize);
        let structures = context.empty_storage::<graph::GpuBeardStructure>("beard structures", 1);
        let junctions = context.empty_storage::<graph::GpuBeardJunction>("beard junctions", 1);

        let mut prepared = Self {
            context,
            num_instructions,
            dims,
            instructions,
            samplers,
            octaves,
            permutations,
            spline_points,
            interpolated,
            interval_entries,
            points,
            scratch,
            output,
            staging,
            structures,
            junctions,
            bind_group: context.placeholder_bind_group(),
            point_capacity,
            structure_capacity: 1,
            junction_capacity: 1,
            flat_points: Vec::new(),
        };
        prepared.rebuild_bind_group();
        prepared
    }

    fn rebuild_bind_group(&mut self) {
        self.bind_group = self
            .context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("density_graph bind group"),
                layout: &self.context.graph_bind_group_layout,
                entries: &[
                    bind_entry(0, &self.dims),
                    bind_entry(1, &self.instructions),
                    bind_entry(2, &self.points),
                    bind_entry(3, &self.scratch),
                    bind_entry(4, &self.output),
                    bind_entry(5, &self.samplers),
                    bind_entry(6, &self.octaves),
                    bind_entry(7, &self.permutations),
                    bind_entry(8, &self.spline_points),
                    bind_entry(9, &self.interpolated),
                    bind_entry(10, &self.structures),
                    bind_entry(11, &self.junctions),
                    bind_entry(12, &self.interval_entries),
                ],
            });
    }

    /// Evaluates the graph for `points`, reusing every buffer that is still big enough.
    #[must_use]
    pub fn evaluate(
        &mut self,
        points: &[[f32; 3]],
        beardifier: &graph::BeardifierData,
    ) -> Vec<f32> {
        if points.is_empty() || self.num_instructions == 0 {
            return vec![0.0; points.len()];
        }

        let mut bind_group_stale = points.len() > self.point_capacity;

        if bind_group_stale {
            let (p, s, o, st) = self
                .context
                .allocate_point_buffers(points.len(), self.num_instructions as usize);
            self.points = p;
            self.scratch = s;
            self.output = o;
            self.staging = st;
            self.point_capacity = points.len();
        }
        if beardifier.structures.len() > self.structure_capacity {
            self.structures = self.context.empty_storage::<graph::GpuBeardStructure>(
                "beard structures",
                beardifier.structures.len(),
            );
            self.structure_capacity = beardifier.structures.len();
            bind_group_stale = true;
        }
        if beardifier.junctions.len() > self.junction_capacity {
            self.junctions = self.context.empty_storage::<graph::GpuBeardJunction>(
                "beard junctions",
                beardifier.junctions.len(),
            );
            self.junction_capacity = beardifier.junctions.len();
            bind_group_stale = true;
        }
        if bind_group_stale {
            self.rebuild_bind_group();
        }

        let queue = &self.context.queue;
        let dims = GraphDims::new(points.len(), self.num_instructions as usize, beardifier);
        queue.write_buffer(&self.dims, 0, bytemuck::bytes_of(&dims));

        self.flat_points.clear();
        self.flat_points
            .extend(points.iter().flat_map(|p| p.iter().copied()));
        queue.write_buffer(&self.points, 0, bytemuck::cast_slice(&self.flat_points));

        if !beardifier.structures.is_empty() {
            queue.write_buffer(
                &self.structures,
                0,
                bytemuck::cast_slice(&beardifier.structures),
            );
        }
        if !beardifier.junctions.is_empty() {
            queue.write_buffer(
                &self.junctions,
                0,
                bytemuck::cast_slice(&beardifier.junctions),
            );
        }

        let output_size = (points.len() * std::mem::size_of::<f32>()) as u64;
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("density_graph encoder"),
                });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("density_graph pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.graph_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(dims.num_points.div_ceil(WORKGROUP_SIZE), 1, 1);
        };
        encoder.copy_buffer_to_buffer(&self.output, 0, &self.staging, 0, output_size);
        queue.submit(Some(encoder.finish()));

        let result = self.context.read_back_range(&self.staging, output_size);
        self.staging.unmap();
        result
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
            interval_entries: Vec::new(),
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
                &compiled,
                &crate::graph::BeardifierData::default(),
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
            interval_entries: Vec::new(),
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
            interval_entries: Vec::new(),
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
            interval_entries: Vec::new(),
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
            interval_entries: Vec::new(),
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
    /// Beardifier is the one node driven by per-chunk structure data rather than the
    /// seed. Checked against the real CPU implementation with actual structures and
    /// junctions in range — an empty-input test would pass on a no-op shader.
    #[test]
    fn gpu_beardifier_matches_cpu_with_real_structures() {
        use crate::graph::{BeardifierData, CompiledGraph, Instruction, OpCode, SamplerPool};
        use pumpkin_util::math::{block_box::BlockBox, vector3::Vector3};
        use pumpkin_world::generation::noise::router::density_function::{
            StaticIndependentChunkNoiseFunctionComponentImpl,
            beardifier::{Beardifier, BeardifierJunction, BeardifierStructure, TerrainAdaptation},
        };

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        // One structure per adaptation mode, so every branch of the shader is exercised.
        let structures = vec![
            BeardifierStructure {
                bounding_box: BlockBox::new(0, 60, 0, 8, 70, 8),
                terrain_adaptation: TerrainAdaptation::BeardThin,
                ground_level_delta: 2,
            },
            BeardifierStructure {
                bounding_box: BlockBox::new(12, 58, 4, 20, 66, 12),
                terrain_adaptation: TerrainAdaptation::BeardBox,
                ground_level_delta: 1,
            },
            BeardifierStructure {
                bounding_box: BlockBox::new(-6, 62, -4, 2, 68, 4),
                terrain_adaptation: TerrainAdaptation::Bury,
                ground_level_delta: 3,
            },
            BeardifierStructure {
                bounding_box: BlockBox::new(6, 55, 10, 14, 64, 18),
                terrain_adaptation: TerrainAdaptation::Encapsulate,
                ground_level_delta: 0,
            },
            BeardifierStructure {
                bounding_box: BlockBox::new(-10, 50, -10, -4, 56, -4),
                terrain_adaptation: TerrainAdaptation::None,
                ground_level_delta: 0,
            },
        ];
        let junctions = vec![
            BeardifierJunction {
                x: 4,
                ground_y: 64,
                z: 4,
            },
            BeardifierJunction {
                x: 10,
                ground_y: 62,
                z: 8,
            },
        ];
        let affected_box = BlockBox::new(-16, 40, -16, 32, 80, 32);

        let cpu = Beardifier::new(structures.clone(), junctions.clone(), Some(affected_box));
        let data = BeardifierData::from_cpu(&structures, &junctions, Some(&affected_box));

        let compiled = CompiledGraph {
            instructions: vec![Instruction::new_for_test(OpCode::Beardifier, 0)],
            samplers: SamplerPool::default(),
            spline_points: Vec::new(),
            interval_entries: Vec::new(),
        };

        // Sweep through and around the structures, including points outside the box.
        let mut points = Vec::new();
        for x in -20..36 {
            for y in [45, 55, 60, 63, 65, 70, 78] {
                points.push([x as f32, y as f32, ((x * 3) % 40 - 18) as f32]);
            }
        }

        let gpu_results = ctx.evaluate_graph_with(&compiled, &points, &data);

        let mut max_diff = 0.0f64;
        let mut nonzero = 0usize;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let pos = Vector3::new(point[0] as i32, point[1] as i32, point[2] as i32);
            let expected = cpu.sample(&pos);
            if expected != 0.0 {
                nonzero += 1;
            }
            max_diff = max_diff.max((expected - f64::from(gpu_value)).abs());
        }

        // Guards against the test passing because everything was zero anyway.
        assert!(
            nonzero > 50,
            "expected many non-zero samples, got {nonzero}; the test data misses the structures"
        );
        assert!(
            max_diff < 1e-4,
            "GPU beardifier diverged from the CPU implementation by {max_diff}"
        );
    }
    /// The strongest check available: the whole overworld router, 217 nodes deep,
    /// evaluated on the GPU against the CPU reference interpreter. Any opcode whose
    /// semantics are subtly wrong shows up here even if its own unit test passed.
    #[test]
    fn gpu_matches_cpu_on_the_full_overworld_router() {
        use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
        use pumpkin_world::generation::GlobalRandomConfig;

        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };

        let config = GlobalRandomConfig::new(1234, false);
        let stack = OVERWORLD_BASE_NOISE_ROUTER.noise.full_component_stack;
        let compiled = crate::graph::compile(stack, &config).expect("overworld lowers");
        let beardifier = crate::graph::BeardifierData::default();

        // Realistic block coordinates spread over a few chunks and the full height range.
        let points: Vec<[f32; 3]> = (0..500)
            .map(|i| {
                let f = i as f32;
                [f * 2.0 - 200.0, (f % 96.0) * 4.0 - 64.0, f * 1.5 - 150.0]
            })
            .collect();

        let gpu_results = ctx.evaluate_graph_with(&compiled, &points, &beardifier);

        let mut max_diff = 0.0f32;
        let mut nonzero = 0usize;
        for (point, &gpu_value) in points.iter().zip(&gpu_results) {
            let cpu_value =
                crate::graph::evaluate_cpu(&compiled, &beardifier, point[0], point[1], point[2]);
            if cpu_value != 0.0 {
                nonzero += 1;
            }
            max_diff = max_diff.max((cpu_value - gpu_value).abs());
        }

        assert!(
            nonzero > 400,
            "expected the router to produce varied output, only {nonzero} non-zero"
        );
        // Both sides run the same f32 arithmetic, so they should agree closely; the
        // slack covers reassociation the shader compiler is free to do.
        assert!(
            max_diff < 1e-2,
            "GPU and CPU disagree on the overworld router by {max_diff}"
        );
    }
}
