use bevy_app::{App, Plugin};
use bevy_asset::{AssetServer, Handle, load_internal_asset};
use crate::{
    core_3d,
    core_3d::CORE_3D,
    prelude::Camera3d,
    prepass::DepthPrepass, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity},
    query::{QueryItem, With},
    reflect::ReflectComponent,
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
};
use bevy_reflect::{Reflect, std_traits::ReflectDefault};
use bevy_render::{
    camera::{ExtractedCamera, TemporalJitter},
    extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin, ComponentUniforms},
    globals::{GlobalsBuffer, GlobalsUniform},
    prelude::Camera,
    render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, ViewNode, ViewNodeRunner},
    render_resource::*,
    renderer::{RenderAdapter, RenderContext, RenderDevice, RenderQueue},
    texture::{CachedTexture, TextureCache, BevyDefault},
    view::{Msaa, ViewUniform, ViewUniformOffset, ViewUniforms, ViewTarget, ViewDepthTexture},
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};

mod pipeline;
mod settings;

pub use settings::DepthOfFieldSettings;

use self::pipeline::{DepthOfFieldUniforms, DoFPipeline};

const DOF_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2518633835294359600);

pub struct DepthOfFieldPlugin;

impl Plugin for DepthOfFieldPlugin {

    fn build(&self, app: &mut App) {
        load_internal_asset!(app, DOF_SHADER_HANDLE, "depth_of_field.wgsl", Shader::from_wgsl);

        app.register_type::<settings::DepthOfFieldSettings>();
        app.add_plugins((
            ExtractComponentPlugin::<settings::DepthOfFieldSettings>::default(),
            UniformComponentPlugin::<pipeline::DepthOfFieldUniforms>::default()
        ));

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<DoFNode>>(
                CORE_3D,
                core_3d::graph::node::DEPTH_OF_FIELD
            )
            .add_render_graph_edges(
                CORE_3D,
                &[
                    core_3d::graph::node::END_MAIN_PASS,
                    core_3d::graph::node::DEPTH_OF_FIELD,
                    super::taa::draw_3d_graph::node::TAA
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<pipeline::DoFPipeline>();

        
    }
}

#[derive(Default)]
struct DoFNode;

impl ViewNode for DoFNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static DepthOfFieldSettings
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, view_target, view_depth, settings): QueryItem<Self::ViewQuery>,
        world: &World
    ) -> Result<(), NodeRunError> {
        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let dof_pipeline = world.resource::<DoFPipeline>();
        let uniforms = world.resource::<ComponentUniforms<DepthOfFieldUniforms>>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(pipeline) = pipeline_cache.get_render_pipeline(dof_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        // Get the settings uniform binding
        let Some(settings_binding) = uniforms.uniforms().binding() else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "dof_bind_group_layout",
            &dof_pipeline.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                post_process.source,
                // The depth of view
                &view_depth.view,
                // Use the sampler created for the pipeline
                &dof_pipeline.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("dof_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: Some(
                RenderPassDepthStencilAttachment {
                    view: &view_depth.view,
                    depth_ops: Some(Operations { load: LoadOp::Load, store: false }),
                    stencil_ops: None
                }
            ),
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}