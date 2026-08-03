//! EXPERIMENTAL Vulkan compute path for block-light propagation. Feature-gated,
//! off by default, and not wired into the live lighting engine.
//!
//! A GPU can't cheaply maintain the engine's BFS frontier, so this runs the
//! equivalent fixed-point relaxation: every voxel recomputes
//! `max(luminance, max_neighbour_light - max(opacity, 1))` in parallel, ping-ponging
//! between two device-local buffers. Light starts at 15 and drops by at least one
//! per step, so [`PASSES`] waves always converge. A final pass packs the result to
//! one byte per voxel to quarter the readback.
//!
//! Shaders are `propagate.comp` / `pack.comp`; each documents the `glslc` line that
//! produced its checked-in `.spv`.

use std::ffi::CStr;

use ash::vk;

use crate::lighting::volume::LightVolume;

/// Enough waves for block light (max 15, falloff >= 1) to converge.
const PASSES: u32 = 15;
const WORKGROUP_SIZE: u64 = 64;
/// `maxComputeWorkGroupCount` floor guaranteed by Vulkan.
const MAX_WORKGROUPS_PER_DIM: u64 = 65_535;

const PROPAGATE_SPV: &[u8] = include_bytes!("propagate.spv");
const PACK_SPV: &[u8] = include_bytes!("pack.spv");

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
}

/// How to choose among the available Vulkan physical devices.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdapterSelector {
    /// First `PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU`.
    Integrated,
    /// First `PHYSICAL_DEVICE_TYPE_DISCRETE_GPU`.
    Discrete,
    /// First device of any type.
    Any,
}

/// Identifying details of the selected physical device.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    /// Flags chosen for the readback buffer. `HOST_CACHED` or not largely
    /// explains readback cost on integrated devices.
    pub readback_memory: vk::MemoryPropertyFlags,
}

/// Wall-clock breakdown of one [`GpuLightEngine::propagate`] call.
#[derive(Clone, Copy, Default, Debug)]
pub struct PhaseTimings {
    /// Allocation plus host-to-device upload of props and seed light.
    pub upload: std::time::Duration,
    /// Submitting and waiting for the relaxation and pack dispatches.
    pub compute: std::time::Duration,
    /// Device-to-host copy and unpacking back into `volume.light`.
    pub readback: std::time::Duration,
    /// Sum of the three phases.
    pub total: std::time::Duration,
}

/// Splits a linear workgroup count across X and Y to fit `maxComputeWorkGroupCount`.
const fn dispatch_dims(items: u64) -> (u32, u32) {
    let groups = items.div_ceil(WORKGROUP_SIZE);
    #[expect(clippy::cast_possible_truncation)]
    if groups <= MAX_WORKGROUPS_PER_DIM {
        (groups as u32, 1)
    } else {
        (
            MAX_WORKGROUPS_PER_DIM as u32,
            groups.div_ceil(MAX_WORKGROUPS_PER_DIM) as u32,
        )
    }
}

fn spv_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// A buffer plus the device memory backing it.
struct Buffer {
    handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
}

struct Descriptors {
    pool: vk::DescriptorPool,
    sets: [vk::DescriptorSet; 3],
}

fn device_name(props: &vk::PhysicalDeviceProperties) -> String {
    // Safety: Vulkan guarantees `device_name` is a NUL-terminated string.
    unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Lists every Vulkan physical device the loader can see.
///
/// # Errors
/// Returns an error if the Vulkan loader or instance cannot be created.
pub fn list_devices() -> Result<Vec<(String, vk::PhysicalDeviceType)>, GpuLightError> {
    // Safety: the instance is used only here and destroyed before returning.
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

/// A Vulkan device plus the compiled relaxation pipelines, reusable across volumes.
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
    memory_props: vk::PhysicalDeviceMemoryProperties,
    max_storage_buffer: u64,
    info: DeviceInfo,
}

impl GpuLightEngine {
    /// Opens a device matching `selector` and builds both compute pipelines.
    ///
    /// # Errors
    /// [`GpuLightError::NoDevice`] if nothing matches, or [`GpuLightError::Vulkan`]
    /// if any Vulkan object cannot be created.
    #[expect(clippy::too_many_lines)]
    pub fn new(selector: AdapterSelector) -> Result<Self, GpuLightError> {
        // Safety: every handle lands in the returned struct and is released in `Drop`.
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
            #[expect(clippy::cast_possible_truncation)]
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

            // dims, props, source light, destination.
            let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..4)
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

            let command_pool = device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;

            // Uncached mappings make host reads roughly 10x slower on iGPUs.
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
            })
        }
    }

    #[must_use]
    pub const fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Allocates a buffer backed by memory with `required`, preferring `preferred` too.
    fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        required: vk::MemoryPropertyFlags,
        preferred: vk::MemoryPropertyFlags,
    ) -> Result<Buffer, GpuLightError> {
        // Safety: caller releases these via `destroy_buffer` before the device dies.
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
        // Safety: ours, and all work referencing it has been fenced.
        unsafe {
            self.device.destroy_buffer(buffer.handle, None);
            self.device.free_memory(buffer.memory, None);
        }
    }

    /// Records `f` into a one-shot command buffer and blocks until the GPU is done.
    fn submit_blocking(&self, f: impl FnOnce(vk::CommandBuffer)) -> Result<(), GpuLightError> {
        // Safety: command buffer and fence live and die inside this call.
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

    /// Propagates block light on the GPU, overwriting `volume.light`. Timing this
    /// measures what a server would really pay: allocation, upload, dispatch, readback.
    ///
    /// # Errors
    /// If the volume exceeds device buffer limits, or any Vulkan operation fails.
    #[expect(clippy::too_many_lines)]
    pub fn propagate(&self, volume: &mut LightVolume) -> Result<PhaseTimings, GpuLightError> {
        let mut timings = PhaseTimings::default();
        let started = std::time::Instant::now();
        let mut mark = started;

        let total = volume.voxel_count();
        let byte_len = (total * size_of::<u32>()) as u64;
        if byte_len > self.max_storage_buffer {
            return Err(GpuLightError::VolumeTooLarge(total));
        }
        // One byte per voxel after packing, rounded up to a whole word.
        let packed_len = (total as u64).div_ceil(4) * 4;

        let device_local = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let host_visible =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let none = vk::MemoryPropertyFlags::empty();
        let storage = vk::BufferUsageFlags::STORAGE_BUFFER;
        let dst = vk::BufferUsageFlags::TRANSFER_DST;
        let src = vk::BufferUsageFlags::TRANSFER_SRC;

        let dims_buf = self.create_buffer(16, storage | dst, device_local, none)?;
        let props_buf = self.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_a = self.create_buffer(byte_len, storage | dst, device_local, none)?;
        let buf_b = self.create_buffer(byte_len, storage, device_local, none)?;
        let packed_buf = self.create_buffer(packed_len, storage | src, device_local, none)?;
        // Staging holds dims, then props, then seed light, contiguously.
        let staging = self.create_buffer(16 + byte_len * 2, src, host_visible, none)?;
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

        let descriptors =
            match self.build_descriptors(&dims_buf, &props_buf, &buf_a, &buf_b, &packed_buf) {
                Ok(d) => d,
                Err(e) => {
                    for b in owned {
                        self.destroy_buffer(b);
                    }
                    return Err(e);
                }
            };

        let result = (|| -> Result<(), GpuLightError> {
            #[expect(clippy::cast_possible_truncation)]
            let dims = [volume.size_x, volume.size_y, volume.size_z, total as u32];

            // Safety: host-visible, coherent, unused, and the writes fit.
            unsafe {
                let base = self
                    .device
                    .map_memory(staging.memory, 0, staging.size, vk::MemoryMapFlags::empty())?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(dims.as_ptr().cast::<u8>(), base, 16);
                std::ptr::copy_nonoverlapping(
                    volume.props.as_ptr().cast::<u8>(),
                    base.add(16),
                    byte_len as usize,
                );
                let seed: Vec<u32> = volume.light.iter().map(|&v| u32::from(v)).collect();
                std::ptr::copy_nonoverlapping(
                    seed.as_ptr().cast::<u8>(),
                    base.add(16 + byte_len as usize),
                    byte_len as usize,
                );
                self.device.unmap_memory(staging.memory);
            };

            self.submit_blocking(|cmd| {
                // Safety: recording, and every buffer outlives the fenced submit.
                unsafe {
                    self.device.cmd_copy_buffer(
                        cmd,
                        staging.handle,
                        dims_buf.handle,
                        &[vk::BufferCopy::default().size(16)],
                    );
                    self.device.cmd_copy_buffer(
                        cmd,
                        staging.handle,
                        props_buf.handle,
                        &[vk::BufferCopy::default().src_offset(16).size(byte_len)],
                    );
                    self.device.cmd_copy_buffer(
                        cmd,
                        staging.handle,
                        buf_a.handle,
                        &[vk::BufferCopy::default()
                            .src_offset(16 + byte_len)
                            .size(byte_len)],
                    );
                }
            })?;
            timings.upload = mark.elapsed();
            mark = std::time::Instant::now();

            let (gx, gy) = dispatch_dims(total as u64);
            let (pgx, pgy) = dispatch_dims((total as u64).div_ceil(4));
            self.submit_blocking(|cmd| {
                // Safety: as above. Barriers order each dispatch after the last one's writes.
                unsafe {
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
                    // PASSES is odd, so the converged result lives in buf_b.
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
                        &[descriptors.sets[2]],
                        &[],
                    );
                    self.device.cmd_dispatch(cmd, pgx, pgy, 1);
                }
            })?;
            timings.compute = mark.elapsed();
            mark = std::time::Instant::now();

            self.submit_blocking(|cmd| {
                // Safety: as above.
                unsafe {
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
                }
            })?;

            // Safety: host-visible, the copy is fenced, and we read exactly `total` bytes.
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
                // Already one byte per voxel, so this is a plain memcpy.
                std::ptr::copy_nonoverlapping(base, volume.light.as_mut_ptr(), total);
                self.device.unmap_memory(readback.memory);
            };
            timings.readback = mark.elapsed();
            Ok(())
        })();

        // Safety: everything above is fenced, so nothing still references these.
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

    /// The three descriptor sets: A -> B, B -> A, B -> packed.
    fn build_descriptors(
        &self,
        dims: &Buffer,
        props: &Buffer,
        a: &Buffer,
        b: &Buffer,
        packed: &Buffer,
    ) -> Result<Descriptors, GpuLightError> {
        // Safety: caller owns and destroys the pool once submissions complete.
        unsafe {
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(12)];
            let pool = self.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(3)
                    .pool_sizes(&sizes),
                None,
            )?;
            let layouts = [self.descriptor_layout; 3];
            let sets = self.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&layouts),
            )?;

            let mut infos = Vec::with_capacity(12);
            for (from, to) in [(a, b), (b, a), (b, packed)] {
                for buf in [dims, props, from, to] {
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
                    #[expect(clippy::cast_possible_truncation)]
                    vk::WriteDescriptorSet::default()
                        .dst_set(sets[i / 4])
                        .dst_binding((i % 4) as u32)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(info)
                })
                .collect();
            self.device.update_descriptor_sets(&writes, &[]);

            Ok(Descriptors {
                pool,
                sets: [sets[0], sets[1], sets[2]],
            })
        }
    }
}

fn compute_barrier(device: &ash::Device, cmd: vk::CommandBuffer) {
    // Safety: `cmd` is in the recording state.
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

/// What flags a host-visible allocation would actually get, for the bench to record.
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
        // Safety: all ours, and every submission in `propagate` was fenced.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_pipeline(self.propagate_pipeline, None);
            self.device.destroy_pipeline(self.pack_pipeline, None);
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
