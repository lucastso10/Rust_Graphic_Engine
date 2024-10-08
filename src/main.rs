mod shaders;
mod device;
mod renderer;
mod prerender;

use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::device::{
    Device, DeviceExtensions,
};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{self, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, Surface, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

// ORDEM DA CRIAÇÃO DE OBJETOS
    // instance

    // surface

    // GPU { physical device, logical device, queue creation }

    // Renderer { swapchain, RenderPass, Framebuffers, viewport, command buffers}
    // vertex buffer
    // shaders
    // pipeline

    // event loop

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[name("inColor")]
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

fn main() {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL. Did you install Vulkan?");
    let event_loop = EventLoop::new();

    // instância da vulkan
    //
    // instance
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: Surface::required_extensions(&event_loop),
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let window = Arc::new(WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .expect("Failed to create window")
    );

    // superficie que o vulkan leva em cosideração para desenhar a imagem
    // surface
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    // GPU { physical device, logical device, queue creation }
    let device = device::GPU::new(
        DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        }, 
        &instance, 
        &surface
    );

    // Renderer { swapchain, RenderPass, Framebuffers, viewport, command buffers}
    let renderer = renderer::Renderer::new(
        &device, 
        surface.clone(), 
        window.inner_size(),
    );

    let vertex1 = MyVertex {
        position: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    };
    let vertex2 = MyVertex {
        position: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    };
    let vertex3 = MyVertex {
        position: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    };

    let prerender = prerender::PreRenderer::new(
        &device, 
        vec![vertex1, vertex2, vertex3],
        &renderer.render_pass,
        &renderer.viewport,
    );

    // declara a pipeline final do programa
    //
    // pipeline

    let frames_in_flight = usize::try_from(renderer.swapchain.image_count()).unwrap();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            let constants = shaders::vs::PushConstants {
                transform: [[2.0, 0.0], [0.0, 2.0]],
                position_offset: Into::into([0.0, 0.0]),
                color_offset: [0.0, 0.0, 0.0],
            };

            let command_buffer = renderer.create_command_buffer(
                &device.graphics_queue,
                &prerender.pipeline,
                &prerender.layout,
                &prerender.vertex_buffer,
                &constants,
            );

            // aqui começamos a renderizar a próxima imagem
            let (image_i, _suboptimal, acquire_future) =
                match swapchain::acquire_next_image(renderer.swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

            // wait for the fence related to this image to finish (normally this would be the oldest fence)
            // se não sober o que é fence: https://vulkano.rs/07-windowing/04-event-handling.html?#frames-in-flight-executing-instructions-parallel-to-the-gpu
            if let Some(image_fence) = &fences[image_i as usize] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i as usize].clone() {
                // Create a NowFuture
                None => {
                    let mut now = sync::now(device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(device.graphics_queue.clone(), command_buffer[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    device.graphics_queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(renderer.swapchain.clone(), image_i),
                )
                .then_signal_fence_and_flush();

            fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                Ok(value) => Some(Arc::new(value)),
                Err(VulkanError::OutOfDate) => {
                    None
                }
                Err(e) => {
                    println!("failed to flush future: {e}");
                    None
                }
            };

            previous_fence_i = image_i;
        }
        _ => (),
    });
}

