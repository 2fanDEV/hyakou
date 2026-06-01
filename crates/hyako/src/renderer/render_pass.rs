use std::rc::Rc;

use bytemuck::bytes_of;
use hyakou_core::types::ModelMatrixBindingMode;
use shared::SharedAccess;
use wgpu::{
    BindGroup, Color, CommandEncoder, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, StoreOp, TextureView,
};

use crate::{
    gpu::{buffers::model_matrix::ModelMatrixUniform, render_mesh::RenderMesh},
    renderer::{SceneRenderInput, SceneRendererInner, frame::FrameTarget},
};

impl SceneRendererInner {
    pub(super) fn render_scene(
        &mut self,
        target: &mut FrameTarget<'_>,
        input: SceneRenderInput<'_>,
    ) {
        {
            let _render_pass = begin_render_pass(
                target.encoder,
                target.color_view,
                target.depth_view,
                "Main Command Buffer",
                LoadOp::Clear(Color {
                    r: 0.3,
                    g: 0.2,
                    b: 0.8,
                    a: 1.0,
                }),
                LoadOp::Clear(1.0),
            );
        }

        if let Some(light_bind_group) = self.gpu_resources.light_bind_group() {
            let camera_bind_group = self.gpu_resources.camera_bind_group();
            let model_binding_mode = self.ctx.model_binding_mode;
            let light_render_pipeline = &self.ctx.light_render_pipeline;
            let no_light_render_pipeline = &self.ctx.no_light_render_pipeline;

            for elem in input.normal_meshes {
                Self::record_scene_pass_command_encoder(
                    target,
                    elem,
                    light_render_pipeline,
                    model_binding_mode,
                    camera_bind_group,
                    light_bind_group,
                );
            }

            for elem in input.light_meshes {
                Self::record_scene_pass_command_encoder(
                    target,
                    elem,
                    no_light_render_pipeline,
                    model_binding_mode,
                    camera_bind_group,
                    light_bind_group,
                );
            }
        }

        self.render_outlined_meshes(target, input.outlined_meshes);
    }

    fn render_outlined_meshes(
        &mut self,
        target: &mut FrameTarget<'_>,
        outlined_meshes: &[Rc<RenderMesh>],
    ) {
        if outlined_meshes.is_empty() {
            return;
        }

        let Some(outline_bind_group) = self.gpu_resources.outline_bind_group() else {
            return;
        };

        let mut render_pass = begin_render_pass(
            target.encoder,
            target.color_view,
            target.depth_view,
            "Outline Command Buffer",
            LoadOp::Load,
            LoadOp::Load,
        );

        render_pass.set_pipeline(&self.ctx.outline_render_pipeline);
        render_pass.set_bind_group(0, self.gpu_resources.camera_bind_group(), &[]);
        render_pass.set_bind_group(
            Self::outline_bind_group_index(self.ctx.model_binding_mode),
            outline_bind_group,
            &[],
        );

        for render_mesh in outlined_meshes {
            Self::record_outline_draw_commands(
                &mut render_pass,
                render_mesh,
                target.queue,
                self.ctx.model_binding_mode,
            );
        }
    }

    fn record_scene_pass_command_encoder(
        target: &mut FrameTarget<'_>,
        render_mesh: &RenderMesh,
        render_pipeline: &RenderPipeline,
        model_binding_mode: ModelMatrixBindingMode,
        camera_bind_group: &BindGroup,
        light_bind_group: &BindGroup,
    ) {
        let mut render_pass = begin_render_pass(
            target.encoder,
            target.color_view,
            target.depth_view,
            "Main Command Buffer",
            LoadOp::Load,
            LoadOp::Load,
        );

        render_pass.set_pipeline(render_pipeline);
        Self::apply_model_matrix(
            &mut render_pass,
            render_mesh,
            target.queue,
            model_binding_mode,
            2,
        );
        render_pass.set_vertex_buffer(0, render_mesh.vertex_buffer.slice(..));
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(
            Self::material_bind_group_index(model_binding_mode),
            &render_mesh.material.bind_group,
            &[],
        );
        render_pass.set_index_buffer(
            render_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..render_mesh.index_count, 0, 0..1);
    }

    fn record_outline_draw_commands(
        render_pass: &mut wgpu::RenderPass<'_>,
        render_mesh: &RenderMesh,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
    ) {
        Self::apply_model_matrix(render_pass, render_mesh, queue, model_binding_mode, 1);
        render_pass.set_vertex_buffer(0, render_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            render_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..render_mesh.index_count, 0, 0..1);
    }

    fn apply_model_matrix(
        render_pass: &mut wgpu::RenderPass<'_>,
        render_mesh: &RenderMesh,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
        uniform_bind_group_index: u32,
    ) {
        let model_matrix = render_mesh.transform.read_shared(|t| t.get_matrix());
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => {
                render_pass.set_immediates(0, bytes_of(&model_matrix));
            }
            ModelMatrixBindingMode::Uniform => {
                let model_uniform = ModelMatrixUniform::new(model_matrix);
                let model_uniform_buffer = render_mesh.model_uniform_buffer.as_ref().expect(
                    "Uniform model binding mode requires a model uniform buffer on RenderMesh",
                );
                let model_bind_group = render_mesh
                    .model_bind_group
                    .as_ref()
                    .expect("Uniform model binding mode requires a model bind group on RenderMesh");
                queue.write_buffer(model_uniform_buffer, 0, bytes_of(&model_uniform));
                render_pass.set_bind_group(uniform_bind_group_index, model_bind_group, &[]);
            }
        }
    }

    fn outline_bind_group_index(model_binding_mode: ModelMatrixBindingMode) -> u32 {
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => 1,
            ModelMatrixBindingMode::Uniform => 2,
        }
    }

    fn material_bind_group_index(model_binding_mode: ModelMatrixBindingMode) -> u32 {
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => 2,
            ModelMatrixBindingMode::Uniform => 3,
        }
    }
}

fn begin_render_pass<'a>(
    encoder: &'a mut CommandEncoder,
    color_view: &'a TextureView,
    depth_view: &'a TextureView,
    label: &'static str,
    color_load: LoadOp<Color>,
    depth_load: LoadOp<f32>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: color_view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: color_load,
                store: StoreOp::Store,
            },
        })],
        multiview_mask: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(Operations {
                load: depth_load,
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        }),
    })
}
