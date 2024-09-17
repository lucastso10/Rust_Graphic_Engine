mod shaders;
mod device;
mod renderer;

use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::device::{
    Device, DeviceExtensions, Queue,
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
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
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

    // Renderer { swapchain, RenderPass, Framebuffers, viewport }
    // vertex buffer
    // shaders
    // pipeline
    // command buffers

    // event loop

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
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

    // Renderer { swapchain, RenderPass, Framebuffers, viewport }
    let renderer = renderer::Renderer::new(
        &device, 
        surface.clone(), 
        window.inner_size(),
    );

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let vertex1 = MyVertex {
        position: [0.5, 0.5],
    };
    let vertex2 = MyVertex {
        position: [-0.5, -0.5],
    };
    let vertex3 = MyVertex {
        position: [-0.5, 0.5],
    };
    let vertex4 = MyVertex {
        position: [0.5, -0.5],
    };


    // cria os vertex que serão exibidos na tela
    //
    // vertex buffer
    let vertex_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vec![vertex1, vertex2, vertex3, vertex4],
    )
    .unwrap();

    // carrega os shaders que serão utilizados na renderização
    //
    // shaders
    let vs = shaders::vs::load(device.clone()).expect("failed to create shader module");
    let fs = shaders::fs::load(device.clone()).expect("failed to create shader module");

    // declara a pipeline final do programa
    //
    // pipeline
    let pipeline = get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        renderer.render_pass.clone(),
        renderer.viewport.clone(),
    );

    // command buffers são buffers de comandos que serão executados
    // pela GPU, como o envio de comandos para a GPU pode ser lento
    // vale a pena juntarmos esses comandos em um buffer e enviar
    // um pacote só.
    // 
    // command buffers
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let command_buffers = get_command_buffers(
        &command_buffer_allocator,
        &device.graphics_queue,
        &pipeline,
        &renderer.framebuffers,
        &vertex_buffer,
    );

    // aqui começa o loop de execução do programa
    //
    // event loop

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
                .then_execute(device.graphics_queue.clone(), command_buffers[image_i as usize].clone())
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

fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state = MyVertex::per_vertex()
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &[Arc<Framebuffer>],
    vertex_buffer: &Subbuffer<[MyVertex]>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass(Default::default())
                .unwrap();

            builder.build().unwrap()
        })
        .collect()
}