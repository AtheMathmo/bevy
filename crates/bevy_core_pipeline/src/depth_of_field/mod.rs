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

const DOF_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(925529476923908);

pub struct DepthOfFieldPlugin;

impl Plugin for DepthOfFieldPlugin {

    fn build(&self, app: &mut App) {
        app.register_type::<DepthOfFieldSettings>();
        app.add_plugins((
            ExtractComponentPlugin::<DepthOfFieldSettings>::default(),
            UniformComponentPlugin::<DepthOfFieldUniforms>::default()
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
            .init_resource::<DoFPipeline>();

        
    }
}

#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct DepthOfFieldSettings {
    pub focal_length: f32,
    pub aperture_diameter: f32,
    pub focus_distance: f32
}

#[derive(Component, ShaderType, Clone, Copy)]
pub struct DepthOfFieldUniforms {
    pub focal_length: f32,
    pub aperture_diameter: f32,
    pub focus_distance: f32
}

impl ExtractComponent for DepthOfFieldSettings {
    type Query = (&'static Self, &'static Camera);

    type Filter = ();
    type Out = (Self, DepthOfFieldUniforms);

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some((
            settings.clone(),
            DepthOfFieldUniforms {
                focal_length: settings.focal_length,
                aperture_diameter: settings.aperture_diameter,
                focus_distance: settings.focus_distance
            }
        ))
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

#[derive(Resource)]
struct DoFPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for DoFPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // We need to define the bind group layout used for our pipeline
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("post_process_bind_group_layout"),
            entries: &[
                // The screen texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // The settings uniform that will control the effect
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(DepthOfFieldUniforms::min_size()),
                    },
                    count: None,
                },
            ],
        });

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader: DOF_SHADER_HANDLE,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
