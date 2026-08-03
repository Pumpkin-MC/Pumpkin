//! EXPERIMENTAL GPU terrain density noise.
//!
//! Off unless you enable both the `gpu-experimental-noise` Cargo feature and
//! `gpu_experimental_terrain_noise` in the level config, and it is not wired into
//! the live generator.
//!
//! What it is good for: generating a lot of terrain at once. Offline pregeneration,
//! bulk worldgen, background generation of a large area. This workload uploads 32
//! bytes of parameters and downloads one byte per voxel, so unlike block light there
//! is no input transfer to lose to, and it is pure arithmetic per voxel.
//!
//! What it is bad for: generating a chunk. There is a fixed cost of roughly 1.5-3 ms
//! per dispatch - buffer allocation, descriptor setup, submit, fence - that does not
//! shrink with the problem, while the CPU finishes an entire chunk column in 0.7-3.0
//! ms. A single 16x16x16 section is 13-20x SLOWER on the discrete GPU than just doing
//! it on the CPU. The crossover is around 0.9M voxels, roughly nine chunks, so a
//! server generating chunks one at a time gains nothing; work has to be batched.
//!
//! Measured against rayon across all CPU cores, not a single thread: 8-11x on an RTX
//! 5070 and 2-3x on Arrow Lake-S integrated graphics at 9x9 chunks by 6-12 octaves.
//! This is the only workload here where integrated graphics beat the CPU at all.
//!
//! Two caveats. This is representative fractal value noise, not Pumpkin's
//! data-driven noise router, so it shows the shape of terrain generation suits a GPU,
//! not that porting the real router is cheap. And buffers are allocated per call
//! rather than kept resident the way `lighting::gpu` does it, which is a large part
//! of that fixed per-dispatch cost.
//!
//! CPU and GPU floats round differently, so agreement is a near-threshold tolerance
//! rather than bit equality; in practice zero voxels differed at every size tested.

use std::ffi::CStr;
use std::time::{Duration, Instant};

use ash::vk;
use rayon::prelude::*;

const WORKGROUP_SIZE: u64 = 64;
const MAX_WORKGROUPS_PER_DIM: u64 = 65_535;
const PARAMS_WORDS: usize = 8;
const PARAMS_BYTES: u64 = (PARAMS_WORDS * size_of::<u32>()) as u64;

const NOISE_SPV: &[u8] = include_bytes!("noise.spv");

#[derive(Debug, thiserror::Error)]
pub enum GpuNoiseError {
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
    #[error("field of {0} voxels exceeds this device's storage buffer limit")]
    FieldTooLarge(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoiseAdapter {
    Integrated,
    Discrete,
    Any,
}

#[derive(Clone, Copy, Debug)]
pub struct NoiseParams {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    pub octaves: u32,
    pub seed: u32,
    pub frequency: f32,
    pub threshold: f32,
}

impl NoiseParams {
    #[must_use]
    pub const fn total(&self) -> usize {
        (self.size_x as usize) * (self.size_y as usize) * (self.size_z as usize)
    }

    const fn words(&self) -> [u32; PARAMS_WORDS] {
        [
            self.size_x,
            self.size_y,
            self.size_z,
            self.total() as u32,
            self.octaves,
            self.seed,
            self.frequency.to_bits(),
            self.threshold.to_bits(),
        ]
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct NoiseTimings {
    pub gpu: Duration,
    pub readback: Duration,
    pub total: Duration,
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

const fn hash3(x: u32, y: u32, z: u32, seed: u32) -> u32 {
    let mut h = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(z.wrapping_mul(2_246_822_519))
        .wrapping_add(seed.wrapping_mul(3_266_489_917));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

fn value_at(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    f32::from(((hash3(x as u32, y as u32, z as u32, seed)) & 0xFFFF) as u16) * (2.0 / 65535.0) - 1.0
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    t.mul_add(b - a, a)
}

fn value_noise(px: f32, py: f32, pz: f32, seed: u32) -> f32 {
    let (fx, fy, fz) = (px.floor(), py.floor(), pz.floor());
    let (cx, cy, cz) = (fx as i32, fy as i32, fz as i32);
    let (ux, uy, uz) = (fade(px - fx), fade(py - fy), fade(pz - fz));

    let c000 = value_at(cx, cy, cz, seed);
    let c100 = value_at(cx + 1, cy, cz, seed);
    let c010 = value_at(cx, cy + 1, cz, seed);
    let c110 = value_at(cx + 1, cy + 1, cz, seed);
    let c001 = value_at(cx, cy, cz + 1, seed);
    let c101 = value_at(cx + 1, cy, cz + 1, seed);
    let c011 = value_at(cx, cy + 1, cz + 1, seed);
    let c111 = value_at(cx + 1, cy + 1, cz + 1, seed);

    let x00 = mix(c000, c100, ux);
    let x10 = mix(c010, c110, ux);
    let x01 = mix(c001, c101, ux);
    let x11 = mix(c011, c111, ux);
    let y0 = mix(x00, x10, uy);
    let y1 = mix(x01, x11, uy);
    mix(y0, y1, uz)
}

fn density(x: u32, y: u32, z: u32, p: &NoiseParams) -> f32 {
    let mut px = x as f32 * p.frequency;
    let mut py = y as f32 * p.frequency;
    let mut pz = z as f32 * p.frequency;
    let mut sum = 0.0f32;
    let mut amp = 1.0f32;
    let mut norm = 0.0f32;
    for _ in 0..p.octaves {
        sum += value_noise(px, py, pz, p.seed) * amp;
        norm += amp;
        px *= 2.0;
        py *= 2.0;
        pz *= 2.0;
        amp *= 0.5;
    }
    let n = sum / norm;
    let gradient = (y as f32).mul_add(-(2.0 / p.size_y as f32), 1.0);
    n + gradient
}

#[must_use]
pub fn density_field_cpu(p: &NoiseParams) -> Vec<u8> {
    let total = p.total();
    let mut out = vec![0u8; total];
    let plane = (p.size_x as usize) * (p.size_z as usize);
    for (idx, slot) in out.iter_mut().enumerate() {
        let x = (idx % p.size_x as usize) as u32;
        let z = ((idx / p.size_x as usize) % p.size_z as usize) as u32;
        let y = (idx / plane) as u32;
        *slot = u8::from(density(x, y, z, p) > p.threshold);
    }
    out
}

#[must_use]
pub fn density_field_cpu_parallel(p: &NoiseParams) -> Vec<u8> {
    let total = p.total();
    let mut out = vec![0u8; total];
    let plane = (p.size_x as usize) * (p.size_z as usize);
    out.par_iter_mut().enumerate().for_each(|(idx, slot)| {
        let x = (idx % p.size_x as usize) as u32;
        let z = ((idx / p.size_x as usize) % p.size_z as usize) as u32;
        let y = (idx / plane) as u32;
        *slot = u8::from(density(x, y, z, p) > p.threshold);
    });
    out
}

struct Buffer {
    handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
}

pub struct GpuNoiseEngine {
    _entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    memory_props: vk::PhysicalDeviceMemoryProperties,
    max_storage_buffer: u64,
    name: String,
}

impl GpuNoiseEngine {
    #[expect(clippy::too_many_lines)]
    pub fn new(selector: NoiseAdapter) -> Result<Self, GpuNoiseError> {
        unsafe {
            let entry = ash::Entry::load()?;
            let instance = entry.create_instance(
                &vk::InstanceCreateInfo::default().application_info(
                    &vk::ApplicationInfo::default()
                        .application_name(c"pumpkin-noise-gpu")
                        .api_version(vk::make_api_version(0, 1, 2, 0)),
                ),
                None,
            )?;

            let wanted = match selector {
                NoiseAdapter::Integrated => Some(vk::PhysicalDeviceType::INTEGRATED_GPU),
                NoiseAdapter::Discrete => Some(vk::PhysicalDeviceType::DISCRETE_GPU),
                NoiseAdapter::Any => None,
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
                return Err(GpuNoiseError::NoDevice);
            };

            let props = instance.get_physical_device_properties(physical);
            let memory_props = instance.get_physical_device_memory_properties(physical);
            let name = CStr::from_ptr(props.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();

            let family = instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE));
            let Some(family) = family else {
                instance.destroy_instance(None);
                return Err(GpuNoiseError::NoComputeQueue);
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

            let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..2)
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

            let words: Vec<u32> = NOISE_SPV
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let module = device
                .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)?;
            let stage_info = vk::ComputePipelineCreateInfo::default()
                .stage(
                    vk::PipelineShaderStageCreateInfo::default()
                        .stage(vk::ShaderStageFlags::COMPUTE)
                        .module(module)
                        .name(c"main"),
                )
                .layout(pipeline_layout);
            let pipeline = device
                .create_compute_pipelines(vk::PipelineCache::null(), &[stage_info], None)
                .map_err(|(_, e)| e)?[0];
            device.destroy_shader_module(module, None);

            let command_pool = device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;

            Ok(Self {
                max_storage_buffer: u64::from(props.limits.max_storage_buffer_range),
                memory_props,
                name,
                _entry: entry,
                instance,
                device,
                queue,
                command_pool,
                descriptor_layout,
                pipeline_layout,
                pipeline,
            })
        }
    }

    #[must_use]
    pub fn device_name(&self) -> &str {
        &self.name
    }

    fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        required: vk::MemoryPropertyFlags,
        preferred: vk::MemoryPropertyFlags,
    ) -> Result<Buffer, GpuNoiseError> {
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
            .ok_or(GpuNoiseError::NoMemoryType(required))?;
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
        unsafe {
            self.device.destroy_buffer(buffer.handle, None);
            self.device.free_memory(buffer.memory, None);
        }
    }

    #[expect(clippy::too_many_lines)]
    pub fn density_field(
        &self,
        p: &NoiseParams,
        out: &mut [u8],
    ) -> Result<NoiseTimings, GpuNoiseError> {
        let total = p.total();
        assert_eq!(out.len(), total, "output length must match the field size");
        let packed_len = (total as u64).div_ceil(4) * 4;
        if packed_len > self.max_storage_buffer {
            return Err(GpuNoiseError::FieldTooLarge(total));
        }

        let started = Instant::now();
        let device_local = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let host_visible =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let none = vk::MemoryPropertyFlags::empty();
        let storage = vk::BufferUsageFlags::STORAGE_BUFFER;

        let params_buf = self.create_buffer(PARAMS_BYTES, storage, host_visible, none)?;
        let out_buf = self.create_buffer(
            packed_len,
            storage | vk::BufferUsageFlags::TRANSFER_SRC,
            device_local,
            none,
        )?;
        let readback = self.create_buffer(
            packed_len,
            vk::BufferUsageFlags::TRANSFER_DST,
            host_visible,
            vk::MemoryPropertyFlags::HOST_CACHED,
        )?;

        let result = (|| -> Result<NoiseTimings, GpuNoiseError> {
            let mut timings = NoiseTimings::default();
            unsafe {
                let base = self
                    .device
                    .map_memory(
                        params_buf.memory,
                        0,
                        params_buf.size,
                        vk::MemoryMapFlags::empty(),
                    )?
                    .cast::<u32>();
                std::ptr::copy_nonoverlapping(p.words().as_ptr(), base, PARAMS_WORDS);
                self.device.unmap_memory(params_buf.memory);
            };

            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(2)];
            let (pool, set) = unsafe {
                let pool = self.device.create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .max_sets(1)
                        .pool_sizes(&sizes),
                    None,
                )?;
                let layouts = [self.descriptor_layout];
                let set = self.device.allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )?[0];
                let infos = [
                    [vk::DescriptorBufferInfo::default()
                        .buffer(params_buf.handle)
                        .range(params_buf.size)],
                    [vk::DescriptorBufferInfo::default()
                        .buffer(out_buf.handle)
                        .range(out_buf.size)],
                ];
                let writes: Vec<vk::WriteDescriptorSet> = infos
                    .iter()
                    .enumerate()
                    .map(|(i, info)| {
                        vk::WriteDescriptorSet::default()
                            .dst_set(set)
                            .dst_binding(i as u32)
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .buffer_info(info)
                    })
                    .collect();
                self.device.update_descriptor_sets(&writes, &[]);
                (pool, set)
            };

            let gpu_start = Instant::now();
            let (gx, gy) = dispatch_dims((total as u64).div_ceil(4));
            self.submit_blocking(|cmd| unsafe {
                self.device
                    .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.pipeline);
                self.device.cmd_bind_descriptor_sets(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_dispatch(cmd, gx, gy, 1);
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
                    out_buf.handle,
                    readback.handle,
                    &[vk::BufferCopy::default().size(packed_len)],
                );
            })?;
            timings.gpu = gpu_start.elapsed();

            let read_start = Instant::now();
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
                std::ptr::copy_nonoverlapping(base, out.as_mut_ptr(), total);
                self.device.unmap_memory(readback.memory);
            };
            timings.readback = read_start.elapsed();

            unsafe {
                self.device.destroy_descriptor_pool(pool, None);
            };
            Ok(timings)
        })();

        for b in [&params_buf, &out_buf, &readback] {
            self.destroy_buffer(b);
        }
        let mut timings = result?;
        timings.total = started.elapsed();
        Ok(timings)
    }

    fn submit_blocking(&self, f: impl FnOnce(vk::CommandBuffer)) -> Result<(), GpuNoiseError> {
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

impl Drop for GpuNoiseEngine {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_pipeline(self.pipeline, None);
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
    use super::{GpuNoiseEngine, NoiseAdapter, NoiseParams, density_field_cpu};

    #[test]
    fn gpu_density_field_agrees_with_the_cpu() {
        let Ok(engine) = GpuNoiseEngine::new(NoiseAdapter::Any) else {
            return;
        };
        let p = NoiseParams {
            size_x: 32,
            size_y: 64,
            size_z: 32,
            octaves: 6,
            seed: 1234,
            frequency: 0.02,
            threshold: 0.0,
        };
        let cpu = density_field_cpu(&p);
        let mut gpu = vec![0u8; p.total()];
        engine.density_field(&p, &mut gpu).expect("dispatch failed");

        let mismatches = cpu.iter().zip(&gpu).filter(|(a, b)| a != b).count();
        let fraction = mismatches as f64 / cpu.len() as f64;
        assert!(
            fraction < 0.001,
            "GPU and CPU classification differ on {mismatches} of {} voxels ({:.4}%); only \
             voxels within float rounding of the threshold should disagree",
            cpu.len(),
            fraction * 100.0
        );
    }
}
