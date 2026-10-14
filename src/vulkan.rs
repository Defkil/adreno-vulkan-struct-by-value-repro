//! Raw Vulkan (ash) runner: one compute dispatch, one uniform buffer, one
//! storage buffer, host-visible memory, readback of the storage buffer.
use ash::vk;
use std::ffi::CStr;

pub struct Adapter {
    pub device_name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub api_version: String,
    pub driver_version_raw: u32,
    pub driver_name: String,
    pub driver_info: String,
    pub conformance: String,
}

fn version(v: u32) -> String {
    format!(
        "{}.{}.{}",
        vk::api_version_major(v),
        vk::api_version_minor(v),
        vk::api_version_patch(v)
    )
}

fn cstr(raw: &[std::ffi::c_char]) -> String {
    unsafe { CStr::from_ptr(raw.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Runs `spirv` (entry point `main`, workgroup size 1) with `uniform` bound at
/// set 0 binding 0 and a storage buffer of `output_len` floats (prefilled with
/// -1.0) at set 0 binding 1, and returns the storage buffer contents.
pub fn run(spirv: &[u32], uniform: &[u8], output_len: usize) -> (Adapter, Vec<f32>) {
    unsafe {
        let entry = ash::Entry::load().expect("Vulkan loader (vulkan-1.dll) not found");
        let app = vk::ApplicationInfo::default()
            .application_name(c"adreno-struct-by-value-repro")
            .api_version(vk::API_VERSION_1_2);
        let instance = entry
            .create_instance(
                &vk::InstanceCreateInfo::default().application_info(&app),
                None,
            )
            .unwrap();
        let physical = instance.enumerate_physical_devices().unwrap();
        let physical = *physical
            .iter()
            .find(|&&p| {
                instance.get_physical_device_properties(p).device_type
                    != vk::PhysicalDeviceType::CPU
            })
            .unwrap_or(&physical[0]);

        let mut driver = vk::PhysicalDeviceDriverProperties::default();
        let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut driver);
        instance.get_physical_device_properties2(physical, &mut props2);
        let props = props2.properties;
        let c = driver.conformance_version;
        let adapter = Adapter {
            device_name: cstr(&props.device_name),
            vendor_id: props.vendor_id,
            device_id: props.device_id,
            api_version: version(props.api_version),
            driver_version_raw: props.driver_version,
            driver_name: cstr(&driver.driver_name),
            driver_info: cstr(&driver.driver_info).trim_end().to_owned(),
            conformance: format!("{}.{}.{}.{}", c.major, c.minor, c.subminor, c.patch),
        };

        let family = instance
            .get_physical_device_queue_family_properties(physical)
            .iter()
            .position(|f| f.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .unwrap() as u32;
        let priorities = [1.0];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(family)
            .queue_priorities(&priorities)];
        let device = instance
            .create_device(
                physical,
                &vk::DeviceCreateInfo::default().queue_create_infos(&queue_info),
                None,
            )
            .unwrap();
        let queue = device.get_device_queue(family, 0);
        let memory_props = instance.get_physical_device_memory_properties(physical);

        let make_buffer = |size: usize, usage: vk::BufferUsageFlags, data: &[u8]| {
            let buffer = device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size as u64)
                        .usage(usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .unwrap();
            let req = device.get_buffer_memory_requirements(buffer);
            let wanted =
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let type_index = (0..memory_props.memory_type_count)
                .find(|&i| {
                    req.memory_type_bits & (1 << i) != 0
                        && memory_props.memory_types[i as usize]
                            .property_flags
                            .contains(wanted)
                })
                .unwrap();
            let memory = device
                .allocate_memory(
                    &vk::MemoryAllocateInfo::default()
                        .allocation_size(req.size)
                        .memory_type_index(type_index),
                    None,
                )
                .unwrap();
            device.bind_buffer_memory(buffer, memory, 0).unwrap();
            let ptr = device
                .map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())
                .unwrap() as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            (buffer, memory, ptr)
        };

        let (ubo, ubo_mem, _) =
            make_buffer(uniform.len(), vk::BufferUsageFlags::UNIFORM_BUFFER, uniform);
        let sentinel: Vec<u8> = std::iter::repeat_n(-1.0f32, output_len)
            .flat_map(f32::to_le_bytes)
            .collect();
        let (ssbo, ssbo_mem, ssbo_ptr) = make_buffer(
            sentinel.len(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            &sentinel,
        );

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let set_layout = device
            .create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
            .unwrap();
        let set_layouts = [set_layout];
        let pipeline_layout = device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts),
                None,
            )
            .unwrap();
        let module = device
            .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(spirv), None)
            .unwrap();
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module)
            .name(c"main");
        let pipeline = device
            .create_compute_pipelines(
                vk::PipelineCache::null(),
                &[vk::ComputePipelineCreateInfo::default()
                    .stage(stage)
                    .layout(pipeline_layout)],
                None,
            )
            .unwrap()[0];

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1),
        ];
        let pool = device
            .create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&pool_sizes),
                None,
            )
            .unwrap();
        let set = device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&set_layouts),
            )
            .unwrap()[0];
        let ubo_info = [vk::DescriptorBufferInfo::default()
            .buffer(ubo)
            .range(vk::WHOLE_SIZE)];
        let ssbo_info = [vk::DescriptorBufferInfo::default()
            .buffer(ssbo)
            .range(vk::WHOLE_SIZE)];
        device.update_descriptor_sets(
            &[
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&ubo_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(&ssbo_info),
            ],
            &[],
        );

        let cmd_pool = device
            .create_command_pool(
                &vk::CommandPoolCreateInfo::default().queue_family_index(family),
                None,
            )
            .unwrap();
        let cmd = device
            .allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(cmd_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )
            .unwrap()[0];
        device
            .begin_command_buffer(
                cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .unwrap();
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline);
        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::COMPUTE,
            pipeline_layout,
            0,
            &[set],
            &[],
        );
        device.cmd_dispatch(cmd, 1, 1, 1);
        let barrier = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::HOST_READ)];
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            &barrier,
            &[],
            &[],
        );
        device.end_command_buffer(cmd).unwrap();
        let fence = device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .unwrap();
        let cmds = [cmd];
        device
            .queue_submit(
                queue,
                &[vk::SubmitInfo::default().command_buffers(&cmds)],
                fence,
            )
            .unwrap();
        device
            .wait_for_fences(&[fence], true, 10_000_000_000)
            .unwrap();

        let values = std::slice::from_raw_parts(ssbo_ptr as *const f32, output_len).to_vec();

        device.destroy_fence(fence, None);
        device.destroy_command_pool(cmd_pool, None);
        device.destroy_descriptor_pool(pool, None);
        device.destroy_pipeline(pipeline, None);
        device.destroy_shader_module(module, None);
        device.destroy_pipeline_layout(pipeline_layout, None);
        device.destroy_descriptor_set_layout(set_layout, None);
        for (buffer, memory) in [(ubo, ubo_mem), (ssbo, ssbo_mem)] {
            device.unmap_memory(memory);
            device.destroy_buffer(buffer, None);
            device.free_memory(memory, None);
        }
        device.destroy_device(None);
        instance.destroy_instance(None);
        (adapter, values)
    }
}
