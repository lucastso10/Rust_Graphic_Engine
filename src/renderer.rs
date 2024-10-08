
use crate::shaders;
use crate::MyVertex;
use std::sync::Arc;

use vulkano::{
    device::{Queue, Device},
    format::Format, 
    image::{view::ImageView, Image, ImageUsage}, 
    pipeline::{PipelineLayout, GraphicsPipeline, graphics::viewport::Viewport},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, 
    swapchain::{ColorSpace, Surface, SurfaceCapabilities, Swapchain, SwapchainCreateInfo},
    command_buffer::{PrimaryAutoCommandBuffer, SubpassContents, SubpassBeginInfo, RenderPassBeginInfo, CommandBufferUsage, AutoCommandBufferBuilder, allocator::StandardCommandBufferAllocator},
    buffer::Subbuffer,
};
use winit::dpi::PhysicalSize;

use crate::device::GPU;

pub struct Renderer {
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub viewport: Viewport,
    pub images: Vec<Arc<Image>>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
}

impl Renderer {
    pub fn new(
        device: &GPU,
        surface: Arc<Surface>,
        dimensions: PhysicalSize<u32>, 
    ) -> Self{
        // ao inves de utilizarmos direto a superficíe para desenhar as imagens,
        // que não é ideal, pois pode causar efeitos estranhos já que renderizamos
        // a imagem em tempo real, utilizamos um swapchain que garante que a imagen
        // exibida foi renderizada.
        //
        // swapchain
        let (swapchain, images) = Self::create_swapchain(
            device.clone(), 
            surface.clone(),
            device.physical_device
                .surface_capabilities(&surface, Default::default())
                .expect("failed to get surface capabilities"), 
            dimensions, 
            device.physical_device
                .surface_formats(&surface, Default::default())
                .unwrap(),
        );

        // cria o caminho para o vulkan saber onde que os precisa mostrar
        // as imagens, especificando cores e saturação (veja a função
        // get_render_pass)
        //
        // render pass
        let render_pass = Self::create_render_pass(device.clone(), swapchain.clone());

        // Cria o buffer onde as imagens serão renderizadas antes de serem
        // exibidas na tela
        //
        // framebuffers
        let framebuffers = Self::create_framebuffers(&images, render_pass.clone());

        Self {
            swapchain,
            render_pass,
            framebuffers,
            images,
            command_buffer_allocator: StandardCommandBufferAllocator::new(device.clone(), Default::default()),
            // A viewport basically describes the region of 
            // the framebuffer that the output will be rendered to. 
            // This will almost always be (0, 0) to (width, height)
            //
            // viewport
            viewport: Viewport {
                offset: [0.0, 0.0],
                extent: dimensions.into(),
                depth_range: 0.0..=1.0,
            },
        }
    }

    fn create_swapchain(
        logical_device: Arc<Device>, 
        surface: Arc<Surface>,
        capabilities: SurfaceCapabilities, 
        dimensions: PhysicalSize<u32>, 
        formats: Vec<(Format, ColorSpace)>
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>) {

        // Essas duas variáveis podem trocar mas agora não vejo porque
        let composite_alpha = capabilities.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = formats[0].0;

        Swapchain::new(
            logical_device,
            surface,
            SwapchainCreateInfo {
                min_image_count: capabilities.min_image_count,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        // Isso provavelmente vai mudar drasticamente
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: swapchain.image_format(), // set the format the same as the swapchain
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap()
    }

    fn create_framebuffers(images: &[Arc<Image>], render_pass: Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    }

    pub fn create_command_buffer(
        &self,
        queue: &Arc<Queue>,
        pipeline: &Arc<GraphicsPipeline>,
        pipeline_layout: &Arc<PipelineLayout>,
        vertex_buffer: &Subbuffer<[MyVertex]>,
        push_constants: &shaders::vs::PushConstants,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
        self.framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    &self.command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                )
                .unwrap();



                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
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
                    //.bind_descriptor_sets(
                    //    PipelineBindPoint::Graphics,
                    //    pipeline.layout().clone(),
                    //    0,
                    //    description_set.clone(),
                    //)
                    //.unwrap()
                    .push_constants(pipeline_layout.clone(), 0, *push_constants)
                    .unwrap()
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();

                builder.build().unwrap()
            })
            .collect()
    }
}
