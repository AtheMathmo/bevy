use bevy_ecs::{
    prelude::Component,
    query::QueryItem,
    reflect::ReflectComponent,
};
use bevy_reflect::{Reflect, std_traits::ReflectDefault};
use bevy_render::{extract_component::ExtractComponent, camera::Camera};

use super::pipeline::DepthOfFieldUniforms;

#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct DepthOfFieldSettings {
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
