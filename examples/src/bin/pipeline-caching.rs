// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

// This example demonstrates how to use pipeline caching.
//
// Using a `PipelineCache` can improve performance significantly, by checking if the requested
// pipeline exists in the cache and if so, return that pipeline directly or insert that new
// pipeline into the cache.
//
// You can retrieve the data in the cache as a `Vec<u8>` and save that to a binary file. Later you
// can load that file and build a PipelineCache with the given data. Be aware that the Vulkan
// implementation does not check if the data is valid and vulkano currently does not either.
// Invalid data can lead to driver crashes or worse. Using the same cache data with a different GPU
// probably won't work, a simple driver update can lead to invalid data as well. To check if your
// data is valid you can find inspiration here:
// https://zeux.io/2019/07/17/serializing-pipeline-cache/
//
// In the future, vulkano might implement those safety checks, but for now, you would have to do
// that yourself or trust the data and the user.

use std::{
    fs::{remove_file, rename, File},
    io::{Read, Write},
};
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    pipeline::{cache::PipelineCache, ComputePipeline},
    VulkanLibrary,
};

fn main() {
    // As with other examples, the first step is to create an instance.
    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap();

    // Choose which physical device to use.
    let device_extensions = DeviceExtensions {
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.intersects(QueueFlags::COMPUTE))
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    // Now initializing the device.
    let (device, _) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    // We are creating an empty PipelineCache to start somewhere.
    let pipeline_cache = PipelineCache::empty(device.clone()).unwrap();

    // We need to create the compute pipeline that describes our operation. We are using the shader
    // from the basic-compute-shader example.
    //
    // If you are familiar with graphics pipeline, the principle is the same except that compute
    // pipelines are much simpler to create.
    //
    // Pass the `PipelineCache` as an optional parameter to the `ComputePipeline` constructor. For
    // `GraphicPipeline`s you can use the `GraphicPipelineBuilder` that has a method
    // `build_with_cache(cache: Arc<PipelineCache>)`.
    let _pipeline = {
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                src: r"
                    #version 450

                    layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

                    layout(set = 0, binding = 0) buffer Data {
                        uint data[];
                    };

                    void main() {
                        uint idx = gl_GlobalInvocationID.x;
                        data[idx] *= 12;
                    }
                ",
            }
        }
        let shader = cs::load(device.clone()).unwrap();
        ComputePipeline::new(
            device.clone(),
            shader.entry_point("main").unwrap(),
            &(),
            Some(pipeline_cache.clone()),
            |_| {},
        )
        .unwrap()
    };

    // Normally you would use your pipeline for computing, but we just want to focus on the cache
    // functionality. The cache works the same for a `GraphicsPipeline`, a `ComputePipeline` is
    // just simpler to build.
    //
    // We are now going to retrieve the cache data into a Vec<u8> and save that to a file on our
    // disk.

    if let Ok(data) = pipeline_cache.get_data() {
        if let Ok(mut file) = File::create("pipeline_cache.bin.tmp") {
            if file.write_all(&data).is_ok() {
                let _ = rename("pipeline_cache.bin.tmp", "pipeline_cache.bin");
            } else {
                let _ = remove_file("pipeline_cache.bin.tmp");
            }
        }
    }

    // The `PipelineCache` is now saved to disk and can be loaded the next time the application is
    // started. This way, the pipelines do not have to be rebuild and pipelines that might exist in
    // the cache can be build far quicker.
    //
    // To load the cache from the file, we just need to load the data into a Vec<u8> and build the
    // `PipelineCache` from that. Note that this function is currently unsafe as there are no
    // checks, as it was mentioned at the start of this example.
    let data = {
        if let Ok(mut file) = File::open("pipeline_cache.bin") {
            let mut data = Vec::new();
            if file.read_to_end(&mut data).is_ok() {
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    };

    let second_cache = if let Some(data) = data {
        // This is unsafe because there is no way to be sure that the file contains valid data.
        unsafe { PipelineCache::with_data(device, &data).unwrap() }
    } else {
        PipelineCache::empty(device).unwrap()
    };

    // As the `PipelineCache` of the Vulkan implementation saves an opaque blob of data, there is
    // no real way to know if the data is correct. There might be differences in the byte blob
    // here, but it should still work. If it doesn't, please check if there is an issue describing
    // this problem, and if not open a new one, on the GitHub page.
    assert_eq!(
        pipeline_cache.get_data().unwrap(),
        second_cache.get_data().unwrap(),
    );
    println!("Success");
}
