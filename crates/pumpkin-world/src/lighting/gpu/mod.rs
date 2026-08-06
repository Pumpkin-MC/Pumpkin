//! EXPERIMENTAL GPU block-light propagation.
//!
//! Off unless you enable both the `gpu-experimental-lighting` Cargo feature and
//! `gpu_experimental_lighting` in the level config, and it is not wired into the
//! live `LightEngine`.
//!
//! What it is good for: bulk relighting of one large region that stays resident on
//! the device across many updates, on a discrete GPU. A GPU cannot cheaply maintain
//! the engine's BFS frontier, so this runs the equivalent fixed-point relaxation
//! over the whole affected volume instead.
//!
//! What it is bad for, which is most things:
//!
//! - Integrated graphics. The iGPU lost to the CPU at every size measured, because
//!   reading the result back costs more than the CPU spends solving.
//! - Small or one-shot work. Below ~0.9M voxels a single fence round trip costs more
//!   than the entire solve.
//! - Localised updates in a realistically sized region. Dispatch is restricted to
//!   the Y slab containing the affected box, which is contiguous but spans the full
//!   region in X and Z, so cost scales with region area rather than with what
//!   actually changed. The bundled benchmark uses scattered updates that inflate the
//!   box for the CPU too; one torch in a view-distance-sized region is the case this
//!   handles worst, and fixing it needs a true 3D box dispatch.
//!
//! Measured against an equivalent incremental CPU solve, not against a full rebuild:
//! 3.8-4.5x faster on an RTX 5070 from 0.9M voxels up, and 44-65x faster than the
//! earlier full-reupload version of this same path. Residency costs VRAM, about 99 MB
//! for a 9x9 chunk region, which is its own reason not to hold many regions at once.
//!
//! [`GpuLightEngine::propagate`] is the original one-shot form: allocate, upload the
//! whole grid, relax, download, free. Transfer dominates it. [`ResidentVolume`] is
//! the incremental form that keeps the grid on the device and uploads only changed
//! voxels.
//!
//! Shaders are `propagate.comp`, `pack.comp`, `patch.comp` and `clear.comp`;
//! `build_shaders.sh` regenerates the checked-in SPIR-V.

use std::ffi::CStr;
use std::time::{Duration, Instant};

use ash::vk;

use crate::lighting::volume::{LightVolume, PropDelta};

const PASSES: u32 = 15;
const RESIDENT_PASSES: u32 = PASSES + 1;
const WORKGROUP_SIZE: u64 = 64;
const MAX_WORKGROUPS_PER_DIM: u64 = 65_535;

const DIMS_WORDS: usize = 16;
const DIMS_BYTES: u64 = (DIMS_WORDS * size_of::<u32>()) as u64;
const SETS: usize = 4;
const BINDINGS: usize = 5;

const PROPAGATE_SPV: &[u8] = include_bytes!("propagate.spv");
const PACK_SPV: &[u8] = include_bytes!("pack.spv");
const PATCH_SPV: &[u8] = include_bytes!("patch.spv");
const CLEAR_SPV: &[u8] = include_bytes!("clear.spv");

#[derive(Debug, thiserror::Error)]
pub enum GpuLightError {
    #[error("failed to load the Vulkan loader: {0}")]
    Loader(#[from] ash::LoadingError),
    #[error("Vulkan call failed: {0}")]
    Vulkan(#[from] vk::Result),
    #[error("no Vulkan physical device matched the requested selector")]
    NoDevice,
    #[error("physical device exposes no compute queue family")]
    NoComputeQueue,
    #[error("no memory type satisfies {0:?}")]
    NoMemoryType(vk::MemoryPropertyFlags),
    #[error("volume of {0} voxels exceeds this device's storage buffer limit")]
    VolumeTooLarge(usize),
    #[error("{0} deltas exceeds the {1} this resident volume was built for")]
    TooManyDeltas(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdapterSelector {
    Integrated,
    Discrete,
    Any,
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub readback_memory: vk::MemoryPropertyFlags,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PhaseTimings {
    pub upload: Duration,
    pub compute: Duration,
    pub readback: Duration,
    pub total: Duration,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ResidentTimings {
    pub host: Duration,
    pub gpu: Duration,
    pub readback: Duration,
    pub total: Duration,
    pub dispatch_voxels: usize,
    pub box_voxels: usize,
    pub readback_bytes: usize,
}

const fn dispatch_dims(items: u64) -> (u32, u32) {
    let groups = items.div_ceil(WORKGROUP_SIZE);
    if groups <= MAX_WORKGROUPS_PER_DIM {
        (groups as u32, 1)
    } else {
        (
            MAX_WORKGROUPS_PER_DIM as u32,
            groups.div_ceil(MAX_WORKGROUPS_PER_DIM) as u32,
        )
    }
}

const fn dims_words(
    size: [u32; 3],
    total: u32,
    base: u32,
    count: u32,
    delta_count: u32,
    bounds: ([u32; 3], [u32; 3]),
) -> [u32; DIMS_WORDS] {
    let (lo, hi) = bounds;
    [
        size[0],
        size[1],
        size[2],
        total,
        base,
        count,
        delta_count,
        0,
        lo[0],
        lo[1],
        lo[2],
        0,
        hi[0],
        hi[1],
        hi[2],
        0,
    ]
}

const fn whole_grid(size: [u32; 3]) -> ([u32; 3], [u32; 3]) {
    ([0, 0, 0], [size[0] - 1, size[1] - 1, size[2] - 1])
}

fn spv_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

struct Buffer {
    handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
}

struct Descriptors {
    pool: vk::DescriptorPool,
    sets: [vk::DescriptorSet; SETS],
}

fn device_name(props: &vk::PhysicalDeviceProperties) -> String {
    // SAFETY: `device_name` is a `[c_char; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE]` that the
    // driver fills with a NUL-terminated string, so a terminator exists inside the array
    // and `CStr::from_ptr` cannot read past it. `props` is borrowed for the whole call, so
    // the pointer stays valid, and `into_owned` copies the bytes before the borrow ends.
    unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

pub fn list_devices() -> Result<Vec<(String, vk::PhysicalDeviceType)>, GpuLightError> {
    // SAFETY: `entry` is a freshly loaded Vulkan loader and stays alive for the whole
    // block. The `InstanceCreateInfo` and the `ApplicationInfo` it borrows are temporaries
    // that live until `create_instance` returns. Every physical device queried comes from
    // this instance and is only read before `destroy_instance`, which runs exactly once
    // and after the last use of any handle. The returned `Vec` holds only owned `String`s
    // and plain enum values, so nothing that outlives the instance refers to it.
    unsafe {
        let entry = ash::Entry::load()?;
        let instance = entry.create_instance(
            &vk::InstanceCreateInfo::default().application_info(
                &vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 2, 0)),
            ),
            None,
        )?;
        let devices = instance
            .enumerate_physical_devices()?
            .into_iter()
            .map(|pd| {
                let props = instance.get_physical_device_properties(pd);
                (device_name(&props), props.device_type)
            })
            .collect();
        instance.destroy_instance(None);
        Ok(devices)
    }
}

pub struct GpuLightEngine {
    _entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    propagate_pipeline: vk::Pipeline,
    pack_pipeline: vk::Pipeline,
    patch_pipeline: vk::Pipeline,
    clear_pipeline: vk::Pipeline,
    memory_props: vk::PhysicalDeviceMemoryProperties,
    max_storage_buffer: u64,
    info: DeviceInfo,
}

impl GpuLightEngine {
    #[expect(clippy::too_many_lines)]
    pub fn new(selector: AdapterSelector) -> Result<Self, GpuLightError> {
        // SAFETY: every handle passed to a Vulkan call here was produced by the matching
        // create call earlier in this same block and is still live: `physical` comes from
        // this `instance`, `device` from this `physical`, and the descriptor layout,
        // pipeline layout, shader modules and command pool from this `device`. Each
        // `*CreateInfo` borrows locals (`priorities`, `queue_info`, `bindings`,
        // `set_layouts`, and inside `build` the SPIR-V word vector and `info`) that outlive
        // the call reading them. The `build` closure destroys each shader module only after
        // `create_compute_pipelines` has consumed it, which the spec permits, and is run
        // four times against the four checked-in SPIR-V blobs. On both early-return paths
        // the instance is destroyed exactly once and no handle derived from it is used
        // afterwards.
        unsafe {
            let entry = ash::Entry::load()?;
            let instance = entry.create_instance(
                &vk::InstanceCreateInfo::default().application_info(
                    &vk::ApplicationInfo::default()
                        .application_name(c"pumpkin-light-gpu")
                        .api_version(vk::make_api_version(0, 1, 2, 0)),
                ),
                None,
            )?;

            let wanted = match selector {
                AdapterSelector::Integrated => Some(vk::PhysicalDeviceType::INTEGRATED_GPU),
                AdapterSelector::Discrete => Some(vk::PhysicalDeviceType::DISCRETE_GPU),
                AdapterSelector::Any => None,
            };
            let physical = instance
                .enumerate_physical_devices()?
                .into_iter()
                .find(|pd| {
                    wanted.is_none_or(|w| {
                        instance.get_physical_device_properties(*pd).device_type == w
                    })
                });
            let Some(physical) = physical else {
                instance.destroy_instance(None);
                return Err(GpuLightError::NoDevice);
            };

            let props = instance.get_physical_device_properties(physical);
            let memory_props = instance.get_physical_device_memory_properties(physical);

            let family = instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE));
            let Some(family) = family else {
                instance.destroy_instance(None);
                return Err(GpuLightError::NoComputeQueue);
            };
            let family = family as u32;

            let priorities = [1.0f32];
            let queue_info = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&priorities)];
            let device = instance.create_device(
                physical,
                &vk::DeviceCreateInfo::default().queue_create_infos(&queue_info),
                None,
            )?;
            let queue = device.get_device_queue(family, 0);

            let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..BINDINGS as u32)
                .map(|i| {
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(i)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .descriptor_count(1)
                        .stage_flags(vk::ShaderStageFlags::COMPUTE)
                })
                .collect();
            let descriptor_layout = device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )?;
            let set_layouts = [descriptor_layout];
            let pipeline_layout = device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts),
                None,
            )?;

            let build = |spv: &[u8]| -> Result<vk::Pipeline, vk::Result> {
                let module = device.create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&spv_words(spv)),
                    None,
                )?;
                let info = vk::ComputePipelineCreateInfo::default()
                    .stage(
                        vk::PipelineShaderStageCreateInfo::default()
                            .stage(vk::ShaderStageFlags::COMPUTE)
                            .module(module)
                            .name(c"main"),
                    )
                    .layout(pipeline_layout);
                let pipelines = device
                    .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
                    .map_err(|(_, e)| e)?;
                device.destroy_shader_module(module, None);
                Ok(pipelines[0])
            };
            let propagate_pipeline = build(PROPAGATE_SPV)?;
            let pack_pipeline = build(PACK_SPV)?;
            let patch_pipeline = build(PATCH_SPV)?;
            let clear_pipeline = build(CLEAR_SPV)?;

            let command_pool = device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;

            let readback_memory = resolve_memory_flags(
                &memory_props,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vk::MemoryPropertyFlags::HOST_CACHED,
            );

            Ok(Self {
                info: DeviceInfo {
                    name: device_name(&props),
                    device_type: props.device_type,
                    readback_memory,
                },
                max_storage_buffer: u64::from(props.limits.max_storage_buffer_range),
                memory_props,
                _entry: entry,
                instance,
                device,
                queue,
                command_pool,
                descriptor_layout,
                pipeline_layout,
                propagate_pipeline,
                pack_pipeline,
                patch_pipeline,
                clear_pipeline,
            })
        }
    }

    #[must_use]
    pub const fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        required: vk::MemoryPropertyFlags,
        preferred: vk::MemoryPropertyFlags,
    ) -> Result<Buffer, GpuLightError> {
        // SAFETY: `self.device` is the logical device created in `new` and destroyed only
        // in `Drop`, so it is live for all of `&self`. `buffer` is created here and is
        // still alive when its memory requirements are queried and when memory is bound;
        // nothing destroys it in between. The memory type index comes from
        // `find_memory_type` over this same device's `memory_props` filtered by
        // `reqs.memory_type_bits`, so it is a valid index for this device and compatible
        // with the buffer, and the allocation is `reqs.size`, the driver-reported minimum.
        // The buffer is freshly created with no memory bound, so `bind_buffer_memory` is
        // called exactly once on it as Vulkan requires.
        unsafe {
            let buffer = self.device.create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(size)
                    .usage(usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )?;
            let reqs = self.device.get_buffer_memory_requirements(buffer);
            let index = find_memory_type(
                &self.memory_props,
                reqs.memory_type_bits,
                required,
                preferred,
            )
            .ok_or(GpuLightError::NoMemoryType(required))?;
            let memory = self.device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(reqs.size)
                    .memory_type_index(index),
                None,
            )?;
            self.device.bind_buffer_memory(buffer, memory, 0)?;
            Ok(Buffer {
                handle: buffer,
                memory,
                size,
            })
        }
    }

    fn destroy_buffer(&self, buffer: &Buffer) {
        // SAFETY: `buffer` was produced by `create_buffer` on this same `self.device` and
        // is destroyed exactly once. Every caller has already retired the GPU work that
        // referenced it: `propagate` and `upload_props` run this after `submit_blocking`
        // has waited on the fence of the last submission touching these buffers (or, on
        // the descriptor-creation failure path, before anything was ever submitted), and
        // `ResidentVolume::drop` runs it after `device_wait_idle`. The `Buffer` is never
        // used again afterwards. The buffer handle is destroyed before the memory backing
        // it is freed, which is the order Vulkan requires.
        unsafe {
            self.device.destroy_buffer(buffer.handle, None);
            self.device.free_memory(buffer.memory, None);
        }
    }

    fn submit_blocking(&self, f: impl FnOnce(vk::CommandBuffer)) -> Result<(), GpuLightError> {
        // SAFETY: the command buffer is allocated from `self.command_pool`, recorded
        // between a matched `begin`/`end` pair, submitted to `self.queue` (which belongs
        // to the same queue family the pool was created for), and freed only after
        // `wait_for_fences` reports the submission complete, so it is never freed or
        // re-recorded while pending. The fence is created here and destroyed after that
        // same wait. `buffers` and the `SubmitInfo` array outlive `queue_submit`.
        // Caller requirement: Vulkan requires external synchronisation of a command pool
        // and of a queue, and this type derives `Sync` automatically, so callers must not
        // drive one engine from two threads concurrently.
        unsafe {
            let cmd = self.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )?[0];
            self.device.begin_command_buffer(
                cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
            f(cmd);
            self.device.end_command_buffer(cmd)?;

            let fence = self
                .device
                .create_fence(&vk::FenceCreateInfo::default(), None)?;
            let buffers = [cmd];
            self.device.queue_submit(
                self.queue,
                &[vk::SubmitInfo::default().command_buffers(&buffers)],
                fence,
            )?;
            self.device.wait_for_fences(&[fence], true, u64::MAX)?;

            self.device.destroy_fence(fence, None);
            self.device.free_command_buffers(self.command_pool, &[cmd]);
            Ok(())
        }
    }

    #[expect(clippy::too_many_lines)]
    pub fn propagate(&self, volume: &mut LightVolume) -> Result<PhaseTimings, GpuLightError> {
        let mut timings = PhaseTimings::default();
        let started = Instant::now();
        let mut mark = started;

        let total = volume.voxel_count();
        let byte_len = (total * size_of::<u32>()) as u64;
        if byte_len > self.max_storage_buffer {
            return Err(GpuLightError::VolumeTooLarge(total));
        }
        let packed_len = (total as u64).div_ceil(4) * 4;

        let device_local = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let host_visible =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let none = vk::MemoryPropertyFlags::empty();
        let storage = vk::BufferUsageFlags::STORAGE_BUFFER;
        let dst = vk::BufferUsageFlags::TRANSFER_DST;
        let src = vk::BufferUsageFlags::TRANSFER_SRC;

        let dims_buf = self.create_buffer(DIMS_BYTES, storage | dst, device_local, none)?;
        let props_buf = self.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_a = self.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_b = self.create_buffer(byte_len, storage, device_local, none)?;
        let packed_buf = self.create_buffer(packed_len, storage | src, device_local, none)?;
        let staging = self.create_buffer(DIMS_BYTES + byte_len * 2, src, host_visible, none)?;
        let readback = self.create_buffer(
            packed_len,
            dst,
            host_visible,
            vk::MemoryPropertyFlags::HOST_CACHED,
        )?;
        let owned = [
            &dims_buf,
            &props_buf,
            &buf_a,
            &buf_b,
            &packed_buf,
            &staging,
            &readback,
        ];

        let descriptors = match self.build_descriptors(
            &dims_buf,
            &props_buf,
            &buf_a,
            &buf_b,
            &packed_buf,
            &dims_buf,
        ) {
            Ok(d) => d,
            Err(e) => {
                for b in owned {
                    self.destroy_buffer(b);
                }
                return Err(e);
            }
        };

        let result = (|| -> Result<(), GpuLightError> {
            let size = [volume.size_x, volume.size_y, volume.size_z];
            let dims = dims_words(
                size,
                total as u32,
                0,
                total as u32,
                0,
                ([0, 0, 0], [0, 0, 0]),
            );

            // SAFETY: `staging` is HOST_VISIBLE | HOST_COHERENT and has not been mapped
            // yet, so mapping its whole range is legal and needs no explicit flush. It was
            // created with `DIMS_BYTES + byte_len * 2` bytes and the three copies fill
            // exactly that: `[0, DIMS_BYTES)` from the `dims` array of `DIMS_WORDS` u32s,
            // `[DIMS_BYTES, +byte_len)` from `volume.props`, and
            // `[DIMS_BYTES + byte_len, +byte_len)` from `seed`. `volume.props` is a
            // `Vec<u32>` of `volume.voxel_count() == total` elements and `seed` is built
            // with one u32 per `volume.light` entry, also `total`, so each source really
            // holds `byte_len == total * size_of::<u32>()` bytes. The destinations are
            // disjoint sub-ranges of a device allocation distinct from every source, so no
            // copy overlaps. The range is unmapped before the block ends.
            unsafe {
                let base = self
                    .device
                    .map_memory(staging.memory, 0, staging.size, vk::MemoryMapFlags::empty())?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(
                    dims.as_ptr().cast::<u8>(),
                    base,
                    DIMS_BYTES as usize,
                );
                std::ptr::copy_nonoverlapping(
                    volume.props.as_ptr().cast::<u8>(),
                    base.add(DIMS_BYTES as usize),
                    byte_len as usize,
                );
                let seed: Vec<u32> = volume.light.iter().map(|&v| u32::from(v)).collect();
                std::ptr::copy_nonoverlapping(
                    seed.as_ptr().cast::<u8>(),
                    base.add(DIMS_BYTES as usize + byte_len as usize),
                    byte_len as usize,
                );
                self.device.unmap_memory(staging.memory);
            };

            // SAFETY: `submit_blocking` invokes this closure with a command buffer in the
            // recording state. Each copy reads the sub-range of `staging` that was just
            // written above and writes a destination created with at least that many
            // bytes: `DIMS_BYTES` into `dims_buf` (created with `DIMS_BYTES`), and
            // `byte_len` from offsets `DIMS_BYTES` and `DIMS_BYTES + byte_len` into
            // `props_buf` and `buf_a` (both created with `byte_len`). All four buffers were
            // created with the matching TRANSFER_SRC/TRANSFER_DST usage and stay alive
            // until after the fence wait inside `submit_blocking`.
            self.submit_blocking(|cmd| unsafe {
                self.device.cmd_copy_buffer(
                    cmd,
                    staging.handle,
                    dims_buf.handle,
                    &[vk::BufferCopy::default().size(DIMS_BYTES)],
                );
                self.device.cmd_copy_buffer(
                    cmd,
                    staging.handle,
                    props_buf.handle,
                    &[vk::BufferCopy::default()
                        .src_offset(DIMS_BYTES)
                        .size(byte_len)],
                );
                self.device.cmd_copy_buffer(
                    cmd,
                    staging.handle,
                    buf_a.handle,
                    &[vk::BufferCopy::default()
                        .src_offset(DIMS_BYTES + byte_len)
                        .size(byte_len)],
                );
            })?;
            timings.upload = mark.elapsed();
            mark = Instant::now();

            let (gx, gy) = dispatch_dims(total as u64);
            let (pgx, pgy) = dispatch_dims((total as u64).div_ceil(4));
            // SAFETY: `submit_blocking` invokes this closure with a command buffer in the
            // recording state. The leading barrier makes the uploads visible to the
            // compute stage before the first dispatch. Every descriptor set indexed here
            // (`sets[i % 2]` and `sets[3]`) exists: `build_descriptors` always allocates
            // `SETS == 4` of them, and each was allocated from `self.descriptor_layout`,
            // the only set layout in `self.pipeline_layout`, which both pipelines bound
            // here were created with. `dispatch_dims` clamps every group count to
            // `MAX_WORKGROUPS_PER_DIM`, within `maxComputeWorkGroupCount`.
            // `compute_barrier` separates consecutive relaxation passes so each reads what
            // the previous wrote. All handles outlive the fence wait in `submit_blocking`.
            self.submit_blocking(|cmd| unsafe {
                self.device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::DependencyFlags::empty(),
                    &[vk::MemoryBarrier::default()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)],
                    &[],
                    &[],
                );
                self.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.propagate_pipeline,
                );
                for i in 0..PASSES {
                    self.device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::COMPUTE,
                        self.pipeline_layout,
                        0,
                        &[descriptors.sets[(i % 2) as usize]],
                        &[],
                    );
                    self.device.cmd_dispatch(cmd, gx, gy, 1);
                    compute_barrier(&self.device, cmd);
                }
                self.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pack_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pipeline_layout,
                    0,
                    &[descriptors.sets[3]],
                    &[],
                );
                self.device.cmd_dispatch(cmd, pgx, pgy, 1);
            })?;
            timings.compute = mark.elapsed();
            mark = Instant::now();

            // SAFETY: `submit_blocking` invokes this closure with a command buffer in the
            // recording state. The barrier makes the pack shader's stores to `packed_buf`
            // visible to the following transfer read. The copy moves `packed_len` bytes,
            // exactly the size both `packed_buf` (TRANSFER_SRC) and `readback`
            // (TRANSFER_DST) were created with, so it is in bounds at both ends, and both
            // buffers outlive the fence wait inside `submit_blocking`.
            self.submit_blocking(|cmd| unsafe {
                self.device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[vk::MemoryBarrier::default()
                        .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                        .dst_access_mask(vk::AccessFlags::TRANSFER_READ)],
                    &[],
                    &[],
                );
                self.device.cmd_copy_buffer(
                    cmd,
                    packed_buf.handle,
                    readback.handle,
                    &[vk::BufferCopy::default().size(packed_len)],
                );
            })?;

            // SAFETY: `readback` is HOST_VISIBLE | HOST_COHERENT and has not been mapped
            // yet, and the `submit_blocking` above already waited on the fence for the
            // copy into it, so the transfer has completed and coherent memory makes the
            // result visible without an explicit invalidate. The mapping covers
            // `readback.size == packed_len`, which is `total` rounded up to 4, so reading
            // `total` bytes is in bounds; `volume.light` is a `Vec<u8>` of
            // `volume.voxel_count() == total` elements, so the write is in bounds too.
            // Source and destination are distinct allocations, and the range is unmapped
            // before the block ends.
            unsafe {
                let base = self
                    .device
                    .map_memory(
                        readback.memory,
                        0,
                        readback.size,
                        vk::MemoryMapFlags::empty(),
                    )?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(base, volume.light.as_mut_ptr(), total);
                self.device.unmap_memory(readback.memory);
            };
            timings.readback = mark.elapsed();
            Ok(())
        })();

        // SAFETY: `descriptors.pool` was created by `build_descriptors` from `self.device`
        // and is destroyed exactly once here. Its four sets are only ever bound in command
        // buffers recorded by `submit_blocking`, which does not return `Ok` until
        // `wait_for_fences` reports that submission complete, so on the success path
        // nothing pending references them. On the error paths the closure aborts either
        // before any set was bound or because the submit itself failed, so again no
        // queued work holds them. Destroying the pool implicitly frees its sets, and
        // `descriptors` is not used again.
        unsafe {
            self.device.destroy_descriptor_pool(descriptors.pool, None);
        };
        for b in owned {
            self.destroy_buffer(b);
        }
        result?;

        timings.total = started.elapsed();
        Ok(timings)
    }

    fn build_descriptors(
        &self,
        dims: &Buffer,
        props: &Buffer,
        a: &Buffer,
        b: &Buffer,
        packed: &Buffer,
        deltas: &Buffer,
    ) -> Result<Descriptors, GpuLightError> {
        // SAFETY: the pool is created with `max_sets(SETS)` and `BINDINGS * SETS`
        // STORAGE_BUFFER descriptors, exactly what allocating `SETS` copies of
        // `self.descriptor_layout` (which declares `BINDINGS` storage-buffer bindings)
        // consumes, so the allocation cannot exhaust the pool and `sets` comes back with
        // `SETS == 4` entries, making the `sets[0..=3]` indexing in bounds.
        // Load-bearing: `infos` is created with `with_capacity(count)` and receives exactly
        // `count` pushes (four pairs times `BINDINGS` buffers), so it never reallocates
        // while `writes` holds pointers into it, and both outlive
        // `update_descriptor_sets`. Every buffer named is owned by the caller and alive,
        // and the sets are not yet bound in any pending command buffer.
        unsafe {
            let count = (BINDINGS * SETS) as u32;
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(count)];
            let pool = self.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(SETS as u32)
                    .pool_sizes(&sizes),
                None,
            )?;
            let layouts = [self.descriptor_layout; SETS];
            let sets = self.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&layouts),
            )?;

            let mut infos = Vec::with_capacity(count as usize);
            for (from, to) in [(a, b), (b, a), (a, packed), (b, packed)] {
                for buf in [dims, props, from, to, deltas] {
                    infos.push([vk::DescriptorBufferInfo::default()
                        .buffer(buf.handle)
                        .offset(0)
                        .range(buf.size)]);
                }
            }
            let writes: Vec<vk::WriteDescriptorSet> = infos
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    vk::WriteDescriptorSet::default()
                        .dst_set(sets[i / BINDINGS])
                        .dst_binding((i % BINDINGS) as u32)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(info)
                })
                .collect();
            self.device.update_descriptor_sets(&writes, &[]);

            Ok(Descriptors {
                pool,
                sets: [sets[0], sets[1], sets[2], sets[3]],
            })
        }
    }

    pub fn resident<'e>(
        &'e self,
        volume: &mut LightVolume,
        max_deltas: usize,
    ) -> Result<ResidentVolume<'e>, GpuLightError> {
        ResidentVolume::new(self, volume, max_deltas)
    }
}

pub struct ResidentVolume<'e> {
    engine: &'e GpuLightEngine,
    size: [u32; 3],
    total: usize,
    dims_buf: Buffer,
    props_buf: Buffer,
    buf_a: Buffer,
    buf_b: Buffer,
    packed_buf: Buffer,
    deltas_buf: Buffer,
    dims_staging: Buffer,
    readback: Buffer,
    dims_ptr: *mut u32,
    deltas_ptr: *mut u32,
    readback_ptr: *const u8,
    descriptors: Descriptors,
    cmd: vk::CommandBuffer,
    fence: vk::Fence,
    max_deltas: usize,
}

impl<'e> ResidentVolume<'e> {
    fn new(
        engine: &'e GpuLightEngine,
        volume: &mut LightVolume,
        max_deltas: usize,
    ) -> Result<Self, GpuLightError> {
        let total = volume.voxel_count();
        let byte_len = (total * size_of::<u32>()) as u64;
        if byte_len > engine.max_storage_buffer {
            return Err(GpuLightError::VolumeTooLarge(total));
        }
        let packed_len = (total as u64).div_ceil(4) * 4;
        let delta_bytes = (max_deltas.max(1) * 2 * size_of::<u32>()) as u64;

        let device_local = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let host_visible =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let none = vk::MemoryPropertyFlags::empty();
        let storage = vk::BufferUsageFlags::STORAGE_BUFFER;
        let dst = vk::BufferUsageFlags::TRANSFER_DST;
        let src = vk::BufferUsageFlags::TRANSFER_SRC;

        let dims_buf = engine.create_buffer(DIMS_BYTES, storage | dst, device_local, none)?;
        let props_buf = engine.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_a = engine.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_b = engine.create_buffer(byte_len, storage | dst, device_local, none)?;
        let packed_buf = engine.create_buffer(packed_len, storage | src, device_local, none)?;
        let deltas_buf = engine.create_buffer(delta_bytes, storage, host_visible, none)?;
        let dims_staging = engine.create_buffer(DIMS_BYTES, src, host_visible, none)?;
        let readback = engine.create_buffer(
            packed_len,
            dst,
            host_visible,
            vk::MemoryPropertyFlags::HOST_CACHED,
        )?;

        let descriptors = engine.build_descriptors(
            &dims_buf,
            &props_buf,
            &buf_a,
            &buf_b,
            &packed_buf,
            &deltas_buf,
        )?;

        // SAFETY: `dims_staging`, `deltas_buf` and `readback` were all just created from
        // HOST_VISIBLE | HOST_COHERENT memory types and none has been mapped yet, so
        // mapping each whole range once is legal; Vulkan guarantees a mapping is aligned
        // to at least `minMemoryMapAlignment` (>= 4), so the `u32` casts are aligned.
        // These three mappings deliberately persist: the buffers and their memory are
        // moved into the returned `ResidentVolume`, which owns them for its whole
        // lifetime, and `ResidentVolume::drop` unmaps each exactly once (after
        // `device_wait_idle`) before freeing them, so the stored pointers never dangle
        // while the struct is alive. `cmd` is allocated from `engine.command_pool` and
        // `fence` from `engine.device`; the `'e` lifetime keeps `engine` alive for longer
        // than the `ResidentVolume` that frees them in `drop`.
        let (dims_ptr, deltas_ptr, readback_ptr, cmd, fence) = unsafe {
            let dims_ptr = engine
                .device
                .map_memory(
                    dims_staging.memory,
                    0,
                    dims_staging.size,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<u32>();
            let deltas_ptr = engine
                .device
                .map_memory(
                    deltas_buf.memory,
                    0,
                    deltas_buf.size,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<u32>();
            let readback_ptr = engine
                .device
                .map_memory(
                    readback.memory,
                    0,
                    readback.size,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<u8>()
                .cast_const();
            let cmd = engine.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(engine.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )?[0];
            let fence = engine
                .device
                .create_fence(&vk::FenceCreateInfo::default(), None)?;
            (dims_ptr, deltas_ptr, readback_ptr, cmd, fence)
        };

        let resident = Self {
            engine,
            size: [volume.size_x, volume.size_y, volume.size_z],
            total,
            dims_buf,
            props_buf,
            buf_a,
            buf_b,
            packed_buf,
            deltas_buf,
            dims_staging,
            readback,
            dims_ptr,
            deltas_ptr,
            readback_ptr,
            descriptors,
            cmd,
            fence,
            max_deltas,
        };

        resident.upload_props(volume)?;
        let full = resident.aligned_total();
        resident.solve(0, full, 0, whole_grid(resident.size))?;
        resident.copy_back(volume, 0, total);
        Ok(resident)
    }

    const fn aligned_total(&self) -> u32 {
        (self.total as u32).div_ceil(4) * 4
    }

    fn upload_props(&self, volume: &LightVolume) -> Result<(), GpuLightError> {
        let byte_len = (self.total * size_of::<u32>()) as u64;
        let staging = self.engine.create_buffer(
            byte_len,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::empty(),
        )?;
        let result = (|| -> Result<(), GpuLightError> {
            // SAFETY: `staging` was created immediately above from a HOST_VISIBLE |
            // HOST_COHERENT memory type with exactly `byte_len` bytes and has not been
            // mapped yet, so mapping its whole range is legal and needs no explicit flush.
            // `volume.props` is a `Vec<u32>` of `self.total` elements, so it holds exactly
            // `byte_len == self.total * size_of::<u32>()` bytes and the copy is in bounds
            // at both ends. The source is a heap allocation distinct from the mapped
            // device memory, so the ranges cannot overlap, and the range is unmapped
            // before the block ends.
            unsafe {
                let base = self
                    .engine
                    .device
                    .map_memory(staging.memory, 0, staging.size, vk::MemoryMapFlags::empty())?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(
                    volume.props.as_ptr().cast::<u8>(),
                    base,
                    byte_len as usize,
                );
                self.engine.device.unmap_memory(staging.memory);
            };
            // SAFETY: `submit_blocking` invokes this closure with a command buffer in the
            // recording state. The copy moves `byte_len` bytes from `staging`, which was
            // created with exactly that size and TRANSFER_SRC usage, into `props_buf`,
            // created with the same size and TRANSFER_DST. `buf_a` and `buf_b` were also
            // created with `byte_len` bytes, so zero-filling `[0, byte_len)` covers each
            // exactly, and `byte_len` is `total * size_of::<u32>()`, hence the multiple of
            // four that `cmd_fill_buffer` requires for its offset and size. `staging`
            // outlives the fence wait inside `submit_blocking` because it is destroyed
            // only after this closure's call returns.
            self.engine.submit_blocking(|cmd| unsafe {
                let device = &self.engine.device;
                device.cmd_copy_buffer(
                    cmd,
                    staging.handle,
                    self.props_buf.handle,
                    &[vk::BufferCopy::default().size(byte_len)],
                );
                device.cmd_fill_buffer(cmd, self.buf_a.handle, 0, byte_len, 0);
                device.cmd_fill_buffer(cmd, self.buf_b.handle, 0, byte_len, 0);
            })
        })();
        self.engine.destroy_buffer(&staging);
        result
    }

    #[expect(clippy::too_many_lines)]
    fn solve(
        &self,
        base: u32,
        count: u32,
        delta_count: u32,
        bounds: ([u32; 3], [u32; 3]),
    ) -> Result<(), GpuLightError> {
        let words = dims_words(
            self.size,
            self.total as u32,
            base,
            count,
            delta_count,
            bounds,
        );
        // SAFETY: `self.dims_ptr` is the persistent mapping of `dims_staging`, valid for
        // the whole lifetime of this `ResidentVolume` (see `new`), suitably aligned for
        // `u32`, and covers `DIMS_BYTES == DIMS_WORDS * size_of::<u32>()` bytes, so
        // writing `DIMS_WORDS` u32s is in bounds. `words` is a fresh stack array of that
        // exact length living in host memory distinct from the mapping, so the ranges do
        // not overlap. `ResidentVolume` holds raw pointers and is therefore neither `Send`
        // nor `Sync`, so no other thread can be writing this staging buffer, and the GPU
        // is not reading it: the previous `solve` ended with `wait_for_fences`, and the
        // copy into `dims_buf` is recorded below in the submission this write feeds. The
        // memory is HOST_COHERENT, so no explicit flush is needed.
        unsafe { std::ptr::copy_nonoverlapping(words.as_ptr(), self.dims_ptr, DIMS_WORDS) };

        let (gx, gy) = dispatch_dims(u64::from(count));
        let (pgx, pgy) = dispatch_dims(u64::from(count).div_ceil(4));
        let (dgx, dgy) = dispatch_dims(u64::from(delta_count));
        let device = &self.engine.device;
        let layout = self.engine.pipeline_layout;

        // SAFETY: `self.cmd` is owned by this `ResidentVolume` and is not pending when it
        // is reset: it was freshly allocated in `new`, and every later `solve` ends with
        // `wait_for_fences(&[self.fence])`. `ResidentVolume` holds raw pointers so it is
        // neither `Send` nor `Sync`, which rules out a second thread re-recording the same
        // command buffer through `&self`. Recording sits between a matched
        // `begin`/`end_command_buffer` pair, `reset_fences` precedes the resubmission of
        // `self.fence`, and every pipeline and descriptor set belongs to `self.engine` and
        // to `self.descriptors` (which always has `SETS == 4` entries, so `sets[0]`,
        // `sets[2]` and `sets[i % 2]` are in bounds); all were created against `layout`.
        // `dispatch_dims` clamps every group count to `MAX_WORKGROUPS_PER_DIM`, and the
        // patch dispatch is skipped entirely when `delta_count` is zero. The final copy
        // uses `copy_len = count.min(self.total - base)`, so `base + copy_len <= total <=
        // packed_len`, keeping it inside both `packed_buf` and `readback`. Barriers order
        // each stage's writes before the next stage's reads.
        unsafe {
            device.reset_command_buffer(self.cmd, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(
                self.cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;

            device.cmd_copy_buffer(
                self.cmd,
                self.dims_staging.handle,
                self.dims_buf.handle,
                &[vk::BufferCopy::default().size(DIMS_BYTES)],
            );
            device.cmd_pipeline_barrier(
                self.cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)],
                &[],
                &[],
            );

            if delta_count > 0 {
                device.cmd_bind_pipeline(
                    self.cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.engine.patch_pipeline,
                );
                device.cmd_bind_descriptor_sets(
                    self.cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    layout,
                    0,
                    &[self.descriptors.sets[0]],
                    &[],
                );
                device.cmd_dispatch(self.cmd, dgx, dgy, 1);
                compute_barrier(device, self.cmd);
            }

            device.cmd_bind_pipeline(
                self.cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.engine.clear_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                self.cmd,
                vk::PipelineBindPoint::COMPUTE,
                layout,
                0,
                &[self.descriptors.sets[0]],
                &[],
            );
            device.cmd_dispatch(self.cmd, gx, gy, 1);
            compute_barrier(device, self.cmd);

            device.cmd_bind_pipeline(
                self.cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.engine.propagate_pipeline,
            );
            for i in 0..RESIDENT_PASSES {
                device.cmd_bind_descriptor_sets(
                    self.cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    layout,
                    0,
                    &[self.descriptors.sets[(i % 2) as usize]],
                    &[],
                );
                device.cmd_dispatch(self.cmd, gx, gy, 1);
                compute_barrier(device, self.cmd);
            }

            device.cmd_bind_pipeline(
                self.cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.engine.pack_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                self.cmd,
                vk::PipelineBindPoint::COMPUTE,
                layout,
                0,
                &[self.descriptors.sets[2]],
                &[],
            );
            device.cmd_dispatch(self.cmd, pgx, pgy, 1);

            device.cmd_pipeline_barrier(
                self.cmd,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)],
                &[],
                &[],
            );
            let copy_len = u64::from(count).min(self.total as u64 - u64::from(base));
            device.cmd_copy_buffer(
                self.cmd,
                self.packed_buf.handle,
                self.readback.handle,
                &[vk::BufferCopy::default()
                    .src_offset(u64::from(base))
                    .dst_offset(u64::from(base))
                    .size(copy_len)],
            );

            device.end_command_buffer(self.cmd)?;
            device.reset_fences(&[self.fence])?;
            let buffers = [self.cmd];
            device.queue_submit(
                self.engine.queue,
                &[vk::SubmitInfo::default().command_buffers(&buffers)],
                self.fence,
            )?;
            device.wait_for_fences(&[self.fence], true, u64::MAX)?;
        };
        Ok(())
    }

    const fn copy_back(&self, volume: &mut LightVolume, base: usize, len: usize) {
        // SAFETY: `self.readback_ptr` is the persistent mapping of `readback`, valid for
        // the whole lifetime of this `ResidentVolume` (see `new`) and covering
        // `packed_len` bytes. Every caller clamps the range: `new` and `download` pass
        // `base == 0, len == self.total`, and `apply_deltas` passes `copy_len =
        // count.min(self.total - base)`, so `base + len <= self.total <= packed_len` and
        // the read is in bounds. `volume.light` is a `Vec<u8>` with `voxel_count() ==
        // self.total` elements, so the write is in bounds too, and it is a heap allocation
        // distinct from the mapping. Each caller runs after a `solve` or `submit_blocking`
        // that waited on its fence, so the device transfer into `readback` has completed,
        // and the memory is HOST_COHERENT so no invalidate is needed to see it.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.readback_ptr.add(base),
                volume.light.as_mut_ptr().add(base),
                len,
            );
        }
    }

    pub fn apply_deltas(
        &self,
        volume: &mut LightVolume,
        deltas: &[PropDelta],
    ) -> Result<ResidentTimings, GpuLightError> {
        let mut timings = ResidentTimings::default();
        if deltas.is_empty() {
            return Ok(timings);
        }
        if deltas.len() > self.max_deltas {
            return Err(GpuLightError::TooManyDeltas(deltas.len(), self.max_deltas));
        }
        let started = Instant::now();

        volume.apply_prop_deltas(deltas);
        let Some(bounds) = volume.affected_box(deltas) else {
            return Ok(timings);
        };

        let plane = (self.size[0] as usize) * (self.size[2] as usize);
        let raw_base = bounds.min[1] as usize * plane;
        let raw_end = ((bounds.max[1] as usize + 1) * plane).min(self.total);
        let base = raw_base & !3usize;
        let end = raw_end.div_ceil(4) * 4;
        let count = end - base;

        // SAFETY: `self.deltas_ptr` is the persistent mapping of `deltas_buf`, valid for
        // the whole lifetime of this `ResidentVolume` (see `new`) and aligned for `u32`.
        // `deltas.len() <= self.max_deltas` was checked above, and `deltas_buf` was created
        // with `max_deltas.max(1) * 2 * size_of::<u32>()` bytes, so the largest offset
        // written, `i * 2 + 1` for `i < deltas.len()`, stays below the buffer's u32 count.
        // `ResidentVolume` is neither `Send` nor `Sync`, so no other thread is writing the
        // same mapping, and the GPU is not reading it: the previous `solve` was awaited on
        // its fence and the patch dispatch that consumes these words is only submitted by
        // the `solve` call below. The memory is HOST_COHERENT, so no explicit flush is
        // needed for the shader to observe the writes.
        unsafe {
            for (i, &(idx, p)) in deltas.iter().enumerate() {
                let packed = u32::from(p.opacity.min(15)) | (u32::from(p.luminance.min(15)) << 4);
                self.deltas_ptr.add(i * 2).write(idx);
                self.deltas_ptr.add(i * 2 + 1).write(packed);
            }
        }
        timings.host = started.elapsed();

        let gpu_start = Instant::now();
        self.solve(
            base as u32,
            count as u32,
            deltas.len() as u32,
            (bounds.min, bounds.max),
        )?;
        timings.gpu = gpu_start.elapsed();

        let read_start = Instant::now();
        let copy_len = count.min(self.total - base);
        self.copy_back(volume, base, copy_len);
        timings.readback = read_start.elapsed();

        timings.total = started.elapsed();
        timings.dispatch_voxels = count;
        timings.box_voxels = bounds.voxels();
        timings.readback_bytes = copy_len;
        Ok(timings)
    }

    pub fn download(&self, volume: &mut LightVolume) -> Result<(), GpuLightError> {
        let words = dims_words(
            self.size,
            self.total as u32,
            0,
            self.aligned_total(),
            0,
            ([0, 0, 0], [0, 0, 0]),
        );
        // SAFETY: same mapping as in `solve` - `self.dims_ptr` is the persistent, u32
        // aligned mapping of `dims_staging`, which covers exactly `DIMS_WORDS` u32s and
        // stays valid for this `ResidentVolume`'s lifetime, and `words` is a distinct
        // stack array of that length, so the copy is in bounds and non-overlapping. No
        // GPU work is reading `dims_staging` here either: the last `solve` waited on its
        // fence, and the copy into `dims_buf` is recorded in the submission below.
        // HOST_COHERENT memory needs no explicit flush.
        unsafe { std::ptr::copy_nonoverlapping(words.as_ptr(), self.dims_ptr, DIMS_WORDS) };
        let (pgx, pgy) = dispatch_dims((self.total as u64).div_ceil(4));
        let device = &self.engine.device;
        let layout = self.engine.pipeline_layout;
        // SAFETY: `submit_blocking` invokes this closure with a command buffer in the
        // recording state, and it is a different command buffer from `self.cmd`, so this
        // does not disturb the resident one. Every handle belongs to this volume or to
        // `self.engine`: `dims_staging` and `dims_buf` were both created with `DIMS_BYTES`
        // bytes and the matching TRANSFER usage; `self.descriptors.sets[2]` exists because
        // `build_descriptors` always allocates `SETS == 4` sets from
        // `engine.descriptor_layout`, the only set layout in `layout`, which
        // `pack_pipeline` was created with. `dispatch_dims` clamps the group counts. The
        // barriers order the dims upload before the pack shader reads it and the pack
        // shader's stores before the readback transfer. The final copy is `packed_buf.size`
        // bytes into `readback`, which was created with the same `packed_len`.
        self.engine.submit_blocking(|cmd| unsafe {
            device.cmd_copy_buffer(
                cmd,
                self.dims_staging.handle,
                self.dims_buf.handle,
                &[vk::BufferCopy::default().size(DIMS_BYTES)],
            );
            device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)],
                &[],
                &[],
            );
            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.engine.pack_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::COMPUTE,
                layout,
                0,
                &[self.descriptors.sets[2]],
                &[],
            );
            device.cmd_dispatch(cmd, pgx, pgy, 1);
            device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)],
                &[],
                &[],
            );
            device.cmd_copy_buffer(
                cmd,
                self.packed_buf.handle,
                self.readback.handle,
                &[vk::BufferCopy::default().size(self.packed_buf.size)],
            );
        })?;
        self.copy_back(volume, 0, self.total);
        Ok(())
    }

    #[must_use]
    pub const fn resident_bytes(&self) -> u64 {
        self.props_buf.size + self.buf_a.size + self.buf_b.size + self.packed_buf.size
    }
}

impl Drop for ResidentVolume<'_> {
    fn drop(&mut self) {
        let device = &self.engine.device;
        // SAFETY: `device_wait_idle` runs first, so no submission still references the
        // command buffer, fence, descriptor sets or buffers below. Each of the three
        // memories was mapped exactly once in `new` and has stayed mapped since, so a
        // single `unmap_memory` each is correct, and the raw pointers into them are not
        // used again. `fence`, `cmd` and the descriptor pool were all created in `new`
        // from this `engine`, whose `'e` lifetime outlives `self`, and are destroyed
        // exactly once here; freeing the pool implicitly frees its sets. All of this
        // precedes the `destroy_buffer` loop, so every memory is unmapped before it is
        // freed. `&mut self` rules out concurrent use.
        unsafe {
            let _ = device.device_wait_idle();
            device.unmap_memory(self.dims_staging.memory);
            device.unmap_memory(self.deltas_buf.memory);
            device.unmap_memory(self.readback.memory);
            device.destroy_fence(self.fence, None);
            device.free_command_buffers(self.engine.command_pool, &[self.cmd]);
            device.destroy_descriptor_pool(self.descriptors.pool, None);
        };
        for b in [
            &self.dims_buf,
            &self.props_buf,
            &self.buf_a,
            &self.buf_b,
            &self.packed_buf,
            &self.deltas_buf,
            &self.dims_staging,
            &self.readback,
        ] {
            self.engine.destroy_buffer(b);
        }
    }
}

fn compute_barrier(device: &ash::Device, cmd: vk::CommandBuffer) {
    // SAFETY: every caller passes a command buffer that is currently recording, inside a
    // `begin_command_buffer`/`end_command_buffer` pair, and that was allocated from a pool
    // on `device`; a pipeline barrier is legal outside a render pass in a compute-only
    // command buffer. The `MemoryBarrier` array is a temporary that lives until
    // `cmd_pipeline_barrier` returns, and the empty slices are valid zero-length slices.
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)],
            &[],
            &[],
        );
    }
}

fn find_memory_type(
    props: &vk::PhysicalDeviceMemoryProperties,
    type_bits: u32,
    required: vk::MemoryPropertyFlags,
    preferred: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let search = |extra: vk::MemoryPropertyFlags| {
        (0..props.memory_type_count).find(|&i| {
            type_bits & (1 << i) != 0
                && props.memory_types[i as usize]
                    .property_flags
                    .contains(required | extra)
        })
    };
    search(preferred).or_else(|| search(vk::MemoryPropertyFlags::empty()))
}

fn resolve_memory_flags(
    props: &vk::PhysicalDeviceMemoryProperties,
    required: vk::MemoryPropertyFlags,
    preferred: vk::MemoryPropertyFlags,
) -> vk::MemoryPropertyFlags {
    find_memory_type(props, u32::MAX, required, preferred)
        .map_or_else(vk::MemoryPropertyFlags::empty, |i| {
            props.memory_types[i as usize].property_flags
        })
}

impl Drop for GpuLightEngine {
    fn drop(&mut self) {
        // SAFETY: `device_wait_idle` first, so no submission still references any of these
        // objects. Each handle was created once in `new`, has not been destroyed since,
        // and is destroyed exactly once here; `&mut self` guarantees no concurrent use.
        // Children are destroyed before their parents: all four compute pipelines
        // (propagate, pack, patch, clear), the pipeline layout, the descriptor set layout
        // and the command pool belong to `device` and go first, then the device, then the
        // instance that owns it. Any `ResidentVolume` borrowing this engine has already
        // been dropped, since it holds a `&'e GpuLightEngine`. Nothing is used after its
        // destroy call.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_pipeline(self.propagate_pipeline, None);
            self.device.destroy_pipeline(self.pack_pipeline, None);
            self.device.destroy_pipeline(self.patch_pipeline, None);
            self.device.destroy_pipeline(self.clear_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AdapterSelector, GpuLightEngine};
    use crate::lighting::volume::{LightVolume, PropDelta, VoxelProps};

    const SX: u32 = 48;
    const SY: u32 = 40;
    const SZ: u32 = 32;

    fn idx(x: u32, y: u32, z: u32) -> usize {
        (((y * SZ) + z) * SX + x) as usize
    }

    fn scene() -> Vec<VoxelProps> {
        let mut props = vec![VoxelProps::default(); (SX * SY * SZ) as usize];
        for y in 0..8 {
            for z in 0..SZ {
                for x in 0..SX {
                    props[idx(x, y, z)].opacity = 15;
                }
            }
        }
        for z in 10..14 {
            for y in 8..20 {
                props[idx(24, y, z)].opacity = 15;
            }
        }
        props[idx(6, 14, 16)].luminance = 14;
        props[idx(40, 12, 8)].luminance = 9;
        props
    }

    fn reference_light(volume: &LightVolume) -> Vec<u8> {
        let mut copy = LightVolume {
            size_x: volume.size_x,
            size_y: volume.size_y,
            size_z: volume.size_z,
            props: volume.props.clone(),
            light: vec![0; volume.voxel_count()],
        };
        copy.propagate_reference();
        copy.light
    }

    fn ticks() -> Vec<Vec<PropDelta>> {
        let lit = |l: u8| VoxelProps {
            opacity: 0,
            luminance: l,
        };
        let solid = VoxelProps {
            opacity: 15,
            luminance: 0,
        };
        vec![
            vec![(idx(20, 16, 16) as u32, lit(15))],
            vec![(idx(6, 14, 16) as u32, lit(0))],
            vec![(idx(18, 16, 16) as u32, solid)],
            vec![
                (idx(20, 16, 16) as u32, lit(0)),
                (idx(34, 18, 20) as u32, lit(13)),
                (idx(18, 16, 16) as u32, lit(0)),
            ],
        ]
    }

    #[test]
    fn resident_delta_sequence_matches_reference_and_full_reupload() {
        let Ok(engine) = GpuLightEngine::new(AdapterSelector::Any) else {
            return;
        };

        let props = scene();
        let mut volume = LightVolume::new(SX, SY, SZ, &props);
        let resident = engine
            .resident(&mut volume, 64)
            .expect("failed to build resident volume");

        assert_eq!(
            volume.light,
            reference_light(&volume),
            "initial resident solve disagrees with the fixed point"
        );

        for (n, tick) in ticks().iter().enumerate() {
            resident
                .apply_deltas(&mut volume, tick)
                .unwrap_or_else(|e| panic!("tick {n} failed: {e}"));

            let expected = reference_light(&volume);
            assert_eq!(
                volume.light, expected,
                "tick {n} diverged from the fixed point"
            );

            let mut downloaded = LightVolume::new(SX, SY, SZ, &props);
            downloaded.props.clone_from(&volume.props);
            resident
                .download(&mut downloaded)
                .expect("full download failed");
            assert_eq!(
                downloaded.light, expected,
                "tick {n}: resident device state diverged outside the slab that was copied back"
            );

            let mut one_shot = LightVolume::new(SX, SY, SZ, &props);
            one_shot.props.clone_from(&volume.props);
            engine
                .propagate(&mut one_shot)
                .expect("full-reupload path failed");
            assert_eq!(
                one_shot.light, expected,
                "tick {n}: full-reupload GPU path diverged from the fixed point"
            );
        }
    }
}
