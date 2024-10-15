use crate::shaders;
use crate::MyVertex;
use std::sync::Arc;

use vulkano::buffer::allocator::SubbufferAllocator;
use vulkano::buffer::allocator::SubbufferAllocatorCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, PipelineLayout},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{ColorSpace, Surface, SurfaceCapabilities, Swapchain, SwapchainCreateInfo},
};
use winit::dpi::PhysicalSize;

use crate::device::GPU;

pub struct Renderer {
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub viewport: Viewport,
    //pub images: Vec<Arc<Image>>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    uniform_buffer_allocator: SubbufferAllocator,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl Renderer {
    pub fn new(device: &GPU, surface: Arc<Surface>, dimensions: PhysicalSize<u32>) -> Self {
        // ao inves de utilizarmos direto a superficíe para desenhar as imagens,
        // que não é ideal, pois pode causar efeitos estranhos já que renderizamos
        // a imagem em tempo real, utilizamos um swapchain que garante que a imagen
        // exibida foi renderizada.
        //
        // swapchain
        let (swapchain, images) = Self::create_swapchain(
            device.clone(),
            surface.clone(),
            device
                .physical_device
                .surface_capabilities(&surface, Default::default())
                .expect("failed to get surface capabilities"),
            dimensions,
            device
                .physical_device
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
        let framebuffers = Self::create_framebuffers(&images, render_pass.clone(), device);

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let uniform_buffer_allocator = SubbufferAllocator::new(
            device.memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        Self {
            swapchain,
            render_pass,
            framebuffers,
            //images,
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                device.clone(),
                Default::default(),
            ),
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
            uniform_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        let aspect = self.swapchain.image_extent();
        aspect[0] as f32 / aspect[1] as f32
    }

    fn create_swapchain(
        logical_device: Arc<Device>,
        surface: Arc<Surface>,
        capabilities: SurfaceCapabilities,
        dimensions: PhysicalSize<u32>,
        formats: Vec<(Format, ColorSpace)>,
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
        // Essas duas variáveis podem trocar mas agora não vejo porque
        let composite_alpha = capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .unwrap();
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
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap()
    }

    fn create_framebuffers(
        images: &[Arc<Image>],
        render_pass: Arc<RenderPass>,
        device: &GPU,
    ) -> Vec<Arc<Framebuffer>> {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let depth_buffer = ImageView::new_default(
            Image::new(
                memory_allocator,
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: images[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();
        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
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
        uniforms: &shaders::vs::Data,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
        self.framebuffers
            .iter()
            .map(|framebuffer| {
                let buffer: Subbuffer<shaders::vs::Data> =
                    self.uniform_buffer_allocator.allocate_sized().unwrap();
                *buffer.write().unwrap() = *uniforms;

                let descriptor_set = {
                    let descriptor_set_layouts = pipeline_layout.set_layouts();
                    let descriptor_set_layout = descriptor_set_layouts.get(0).unwrap();
                    PersistentDescriptorSet::new(
                        &self.descriptor_set_allocator,
                        descriptor_set_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer.clone())], // 0 is the binding
                        [],
                    )
                    .unwrap()
                };

                let mut builder = AutoCommandBufferBuilder::primary(
                    &self.command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.22, 0.22, 0.22, 1.0].into()),
                                Some(1f32.into()),
                            ],
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
                    .bind_descriptor_sets(
                        vulkano::pipeline::PipelineBindPoint::Graphics,
                        pipeline_layout.clone(),
                        0,
                        descriptor_set.clone(),
                    )
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
