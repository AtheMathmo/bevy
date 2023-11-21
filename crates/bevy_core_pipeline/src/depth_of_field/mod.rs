use bevy_app::{App, Plugin};
use bevy_asset::{AssetServer, Handle};
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
    extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
    globals::{GlobalsBuffer, GlobalsUniform},
    prelude::Camera,
    render_graph::{NodeRunError, RenderGraphApp, RenderGraphContext, ViewNode, ViewNodeRunner},
    render_resource::*,
    renderer::{RenderAdapter, RenderContext, RenderDevice, RenderQueue},
    texture::{CachedTexture, TextureCache, BevyDefault},
    view::{Msaa, ViewUniform, ViewUniformOffset, ViewUniforms},
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy_utils::{
    prelude::default,
    tracing::{error, warn},
};

mod pipeline;
mod settings;

const DOF_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(925529476923908);

pub struct DepthOfFieldPlugin;

impl Plugin for DepthOfFieldPlugin {

    fn build(&self, app: &mut App) {
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
struct DoFNode {}

impl ViewNode for DoFNode {
    type ViewQuery = &'static ExtractedCamera;

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        camera: QueryItem<Self::ViewQuery>,
        world: &World
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}