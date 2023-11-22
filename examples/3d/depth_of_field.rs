//! A scene showcasing screen space ambient occlusion.

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    prelude::*,
};
use bevy_internal::core_pipeline::depth_of_field::{DepthOfFieldBundle, DepthOfFieldPlugin, DepthOfFieldSettings};
use std::f32::consts::PI;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 5.0,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..Default::default()
        }))
        // TAA is highly recommended when applying screen-space Depth of field effects!
        .add_plugins((TemporalAntiAliasPlugin, DepthOfFieldPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(TemporalAntiAliasBundle::default())
        .insert(DepthOfFieldBundle {
            settings: DepthOfFieldSettings {
                focal_distance: 0.35,
                aperture_diameter: 0.01,
                focal_length: 0.01
            },
            ..default()
        });

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.5),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: material.clone(),
        transform: Transform::from_xyz(-0.3, -0.5, -0.2),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material,
        transform: Transform::from_xyz(1.0, 0.0, 0.0),
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.3,
                sectors: 72,
                stacks: 36,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.4, 0.4, 0.4),
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                ..default()
            }),
            transform: Transform::from_xyz(-2.0, 1.5, -1.85),
            ..default()
        },
        SphereMarker,
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI * -0.15,
            PI * -0.15,
        )),
        ..default()
    });

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 26.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn update(
    camera: Query<
        (
            Entity,
            Option<&DepthOfFieldSettings>,
        ),
        With<Camera>,
    >,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
   

    let (camera_entity, dof_settings) = camera.single();

    let mut commands = commands.entity(camera_entity);
    if keycode.just_pressed(KeyCode::Space) {
        if dof_settings.is_some() {
            commands.remove::<DepthOfFieldSettings>();
        } else {
            commands.insert(DepthOfFieldSettings {
                    focal_distance: 0.33,
                    ..default()
                }
            );
        }
    } else if dof_settings.is_some() {
        let settings = dof_settings.unwrap();

        let mut new_settings = settings.clone();
        if keycode.pressed(KeyCode::Q) {
            new_settings.aperture_diameter = (new_settings.aperture_diameter + 0.05 * time.delta_seconds()).min(0.2);
        }
        if keycode.pressed(KeyCode::A) {
            new_settings.aperture_diameter = (new_settings.aperture_diameter - 0.05 * time.delta_seconds()).max(0.01);
        }

        if keycode.pressed(KeyCode::W) {
            new_settings.focal_distance = (new_settings.focal_distance + 0.5 * time.delta_seconds()).min(5.0);
        }
        if keycode.pressed(KeyCode::S) {
            new_settings.focal_distance = (new_settings.focal_distance - 0.5 * time.delta_seconds()).max(1e-4);
        }

        if keycode.pressed(KeyCode::E) {
            new_settings.focal_length = (new_settings.focal_length + 0.05 * time.delta_seconds()).min(new_settings.focal_distance - 1e-5);
        }
        if keycode.pressed(KeyCode::D) {
            new_settings.focal_length = (new_settings.focal_length - 0.05 * time.delta_seconds()).max(0.01);
        }
        commands.insert(new_settings);
    }
    

    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;
    text.clear();

    text.push_str("Depth of Field:\n");
    text.push_str(match dof_settings {
        Some(_) => "(Space) Enabled\n",
        None => "(Space) Disabled\n",
    });
    if let Some(settings) = dof_settings {
        text.push_str("Q/A: Change aperture diameter\n");
        text.push_str(&format!("Aperture diameter: {:.2}\n", settings.aperture_diameter));

        text.push_str("W/S: Change focal distance\n");
        text.push_str(&format!("Focal distance: {:.2}\n", settings.focal_distance));

        text.push_str("E/D: Change focal length\n");
        text.push_str(&format!("Focal length: {:.2}\n", settings.focal_length));
    }
    
}

#[derive(Component)]
struct SphereMarker;
