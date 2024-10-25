mod camera;
mod device;
mod keyboard;
mod object;
mod prerender;
mod renderer;
mod shaders;

use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use vulkano::buffer::BufferContents;
use vulkano::device::DeviceExtensions;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::swapchain::{self, Surface, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
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

#[derive(BufferContents, Vertex, Clone, PartialEq)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[name("color")]
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    normal: [f32; 3],
    #[format(R32G32_SFLOAT)]
    texcoord: [f32; 2],
}

fn main() {
    let library = vulkano::VulkanLibrary::new()
        .expect("no local Vulkan library/DLL. Did you install Vulkan?");
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

    let window = Arc::new(
        WindowBuilder::new()
            .with_resizable(false)
            .build(&event_loop)
            .expect("Failed to create window"),
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
        &surface,
    );

    // Renderer { swapchain, RenderPass, Framebuffers, viewport, command buffers}
    let renderer = renderer::Renderer::new(&device, surface.clone(), window.inner_size());

    let mut object = object::Object::new("obj/vase.obj");

    object.translation = Vec3::from_array([0.0, 0.5, 0.0]);
    object.scale = Vec3::from_array([1.5, 1.5, 1.5]);

    let prerender = prerender::PreRenderer::new(
        &device,
        &object,
        &renderer.render_pass,
        &renderer.viewport,
    );

    let frames_in_flight = usize::try_from(renderer.swapchain.image_count()).unwrap();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    let mut inputs = keyboard::Keyboard::default();
    let mut camera = camera::Camera::new(
            renderer.get_aspect_ratio(),
            Vec3::from_array([0.0, 0.0, -3.0]),
            Vec3::from_array([0.0, 0.0, 0.0])
        );

    let mut delta_time = 0.0;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = winit::event_loop::ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => inputs.keyboard_events(input),
            _ => (),
        },
        Event::MainEventsCleared => {
            let frame_time = Instant::now();

            if !inputs.active.is_empty() {
                camera.move_camera(delta_time, &inputs);
            }

            let uniform = shaders::vs::Data {
                camera: (camera.projection * camera.view).to_cols_array_2d(),
                modelMatrix: object.calculate_matrix(),
            };

            let command_buffer = renderer.create_command_buffer(
                &device.graphics_queue,
                &prerender,
                &uniform,
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
                .then_execute(
                    device.graphics_queue.clone(),
                    command_buffer[image_i as usize].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    device.graphics_queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(
                        renderer.swapchain.clone(),
                        image_i,
                    ),
                )
                .then_signal_fence_and_flush();

            fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                Ok(value) => Some(Arc::new(value)),
                Err(VulkanError::OutOfDate) => None,
                Err(e) => {
                    println!("failed to flush future: {e}");
                    None
                }
            };

            previous_fence_i = image_i;

            delta_time = frame_time.elapsed().as_secs_f32();
        }
        _ => (),
    });
}
