use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderModule,
};

use crate::{device::GPU, shaders, MyVertex};

pub struct PreRenderer {
    //memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: Subbuffer<[MyVertex]>,
    //vs: Arc<ShaderModule>,
    //fs: Arc<ShaderModule>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub layout: Arc<PipelineLayout>,
}

impl PreRenderer {
    pub fn new(
        device: &GPU,
        objects: Vec<MyVertex>,
        render_pass: &Arc<RenderPass>,
        viewport: &Viewport,
    ) -> Self {
        let vertex_buffer = Buffer::from_iter(
            device.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            objects,
        )
        .unwrap();

        let vs = shaders::vs::load(device.clone()).expect("failed to create shader module");
        let fs = shaders::fs::load(device.clone()).expect("failed to create shader module");

        let (pipeline, layout) = Self::get_pipeline(
            device,
            vs.clone(),
            fs.clone(),
            render_pass.clone(),
            viewport.clone(),
        );
        Self {
            //memory_allocator,
            vertex_buffer,
            //vs,
            //fs,
            pipeline,
            layout,
        }
    }

    fn get_pipeline(
        device: &GPU,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
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

        let graphics_pipeline = GraphicsPipeline::new(
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
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout.clone())
            },
        )
        .unwrap();
        (graphics_pipeline, layout)
    }
}
