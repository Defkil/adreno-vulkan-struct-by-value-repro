//! The same dispatch through wgpu from WGSL, for the DX12 control.
use std::{path::Path, sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

pub async fn run(path: &Path, dx12: bool) -> Vec<f32> {
    let source = std::fs::read_to_string(path).unwrap();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: if dx12 {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::VULKAN
        },
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = instance
        .request_adapter(&Default::default())
        .await
        .expect("GPU adapter");
    let info = adapter.get_info();
    println!(
        "wgpu adapter: {} / {:?} / driver {:?} / {:?}",
        info.name, info.backend, info.driver, info.driver_info
    );
    let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &crate::uniform_bytes(),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let sentinel: Vec<u8> = std::iter::repeat_n(-1.0f32, crate::OUTPUT_LEN)
        .flat_map(f32::to_le_bytes)
        .collect();
    let size = sentinel.len() as u64;
    let output = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &sentinel,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &staging, 0, size);
    queue.submit([encoder.finish()]);
    let (send, recv) = mpsc::channel();
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| send.send(r).unwrap());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .unwrap();
    recv.recv_timeout(Duration::from_secs(30)).unwrap().unwrap();
    staging
        .slice(..)
        .get_mapped_range()
        .unwrap()
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect()
}
