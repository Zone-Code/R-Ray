use bevy::color::palettes::tailwind;
use bevy::ecs::observer::TriggerTargets;
use bevy::prelude::*;
use bevy_egui::{EguiPostUpdateSet};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::bevy_inspector::{self};
use picking::GizmoPickingPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use camera::{camera_movement, SdkCamera};
use crate::domain::SdkEntityIcon;
use crate::editor_commands::{handle_input, HistoryManager};
use crate::gizmo::draw_gizmo;
use crate::ui::camera_viewport::set_camera_viewport;
use crate::ui::{show_ui_system, UiState};
use bevy_render::camera::{CameraProjection};
use bevy_window::{Window};
use egui_lucide_icons::icons;
use transform_gizmo_bevy::{
    GizmoCamera, GizmoHotkeys, GizmoOptions, GizmoTarget, TransformGizmoPlugin,
};

/// Placeholder type if gizmo is disabled.
#[cfg(not(egui_dock_gizmo))]
#[derive(Clone, Copy)]
struct GizmoMode;

mod camera;
mod domain;
mod editor_commands;
mod gizmo;
mod ui;
mod utils;
mod picking;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RRay SDK".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(GizmoPickingPlugin)
        .insert_resource(UiState::new())
        .insert_resource(HistoryManager::new())
        .add_systems(Startup, (init_window, setup).chain())
        .add_systems(
            PostUpdate,
            show_ui_system
                .before(EguiPostUpdateSet::ProcessOutput)
                .before(bevy_egui::end_pass_system)
                .before(TransformSystem::TransformPropagate),
        )
        .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system))
        .add_systems(
            Update,
            (
                draw_gizmo,
                camera_movement,
                handle_input,
                //pick_system,
                update_icons
            ),
        )
        .insert_resource(GizmoOptions {
            hotkeys: Some(GizmoHotkeys {
                enable_snapping: Some(KeyCode::ControlLeft),
                enable_accurate_mode: None,
                toggle_rotate: None,
                toggle_translate: None,
                toggle_scale: None,
                toggle_x: None,
                toggle_y: None,
                toggle_z: None,
                deactivate_gizmo: None,
                ..default()
            }),
            // snapping: true,
            ..default()
        })
        .register_type::<SdkCamera>()
        .register_type::<SdkEntityIcon>()
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .run();
}

#[derive(Component)]
struct MainCamera;

fn init_window(mut query: Query<(Entity, &mut Window)>, mut commands: Commands) {
    for (e, mut window) in query.iter_mut() {
        window.set_maximized(true);

        commands.entity(e).insert(PickingBehavior::IGNORE);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle::default());

    let box_size = 2.0;
    let box_thickness = 0.15;
    let box_offset = (box_size + box_thickness) / 2.0;

    // left - red
    let mut transform = Transform::from_xyz(-box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.63, 0.065, 0.05),
            ..Default::default()
        })),
        transform,
    ));
    // right - green
    let mut transform = Transform::from_xyz(box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.45, 0.091),
            ..Default::default()
        })),
    ));
    // bottom - white
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));
    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * box_offset, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));
    // back - white
    let mut transform = Transform::from_xyz(0.0, box_offset, -box_offset);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size + 2.0 * box_thickness,
        ))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
    // top light
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.4, 0.4))),
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, box_size + 0.5 * box_thickness, 0.0),
            )),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::WHITE * 100.0,
                ..Default::default()
            })),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    PointLight {
                        color: Color::WHITE,
                        intensity: 25000.0,
                        ..Default::default()
                    },
                    SdkEntityIcon::new(
                        icons::lucide::LIGHTBULB,
                        utils::SdkColor::Bevy(Color::from(tailwind::YELLOW_500)).into(),
                    ),
                    Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        PointLight {
                            color: Color::WHITE,
                            intensity: 25000.0,
                            ..Default::default()
                        },
                        Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
                    ));
                });
        });

    // directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
    ));

    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, box_offset, 4.0)
            .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
        MainCamera,
        SdkCamera::default(),
        GizmoCamera,
        RayCastPickable,
    ));
}


fn update_icons(
    light_query: Query<Entity, (With<PointLight>, Without<SdkEntityIcon>)>,
    camera_query: Query<Entity, (With<Camera3d>, Without<SdkEntityIcon>)>,
    mesh_query: Query<Entity, (With<Mesh3d>, Without<SdkEntityIcon>)>,
    dir_light_query: Query<Entity, (With<DirectionalLight>, Without<SdkEntityIcon>)>,
    mut commands: Commands
){
    for e in light_query.iter() {
        commands.entity(e).insert(SdkEntityIcon::new(
            icons::lucide::LIGHTBULB,
            utils::SdkColor::Bevy(Color::from(tailwind::YELLOW_500)).into(),
        ));
        println!("inserted icon for {} ", e);
        return;
    }
    for e in camera_query.iter() {
        commands.entity(e).insert(SdkEntityIcon::new(
            icons::lucide::VIDEO,
            utils::SdkColor::Bevy(Color::from(tailwind::BLUE_500)).into(),
        ));
        println!("inserted icon for {} ", e);
        return;
    }
    for e in mesh_query.iter() {
        commands.entity(e).insert(SdkEntityIcon::new(
            icons::lucide::BOX,
            utils::SdkColor::Bevy(Color::from(tailwind::GREEN_400)).into(),
        ));
        println!("inserted icon for {} ", e);
        return;
    }

    for e in dir_light_query.iter() {
        commands.entity(e).insert(SdkEntityIcon::new(
            icons::lucide::SUN,
            utils::SdkColor::Bevy(Color::from(tailwind::GREEN_400)).into(),
        ));
        println!("inserted icon for {} ", e);
        return;
    }
}