use bevy::{ecs::reflect, input::mouse::{MouseMotion, MouseWheel}, prelude::*};
use smart_default::SmartDefault;
use crate::ui::{EguiWindow, UiState};

#[derive(Component, SmartDefault, Reflect)]
#[reflect(Component)]
pub struct SdkCamera {
    #[default(2.0)]
    pub speed: f32,
    #[default(0.002)]
    pub sensitivity: f32,
}

pub fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut ui_state: ResMut<UiState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut SdkCamera, &mut Transform)>,
) {
    if let Some((_, window)) = ui_state.state.find_active_focused() {
        match window {
            EguiWindow::GameView => {}
            _ => {return;}
        }
    }
    
    for (mut camera, mut transform) in query.iter_mut() {
        let mut speed = camera.speed;

        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            speed *= 4f32;
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            speed *= 0.5f32;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            let forward = transform.forward();
            transform.translation += forward * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let back = transform.back();
            transform.translation += back * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            let left = transform.left();
            transform.translation += left * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            let right = transform.right();

            transform.translation += right * speed * time.delta_secs();
        }

        for event in mouse_wheel_events.read() {
            camera.speed += event.y * 0.1; // Adjust the multiplier as needed
            if camera.speed < 0.0 {
                camera.speed = 0.0; // Prevent negative speed
            }
        }

         // Обробка обертання камери (yaw і pitch) через Transform.rotation
         if mouse_button_input.pressed(MouseButton::Right) {
            let mut rotation = transform.rotation.to_euler(EulerRot::YXZ);

            for event in mouse_motion_events.read() {
                let sensitivity = camera.sensitivity;
                rotation.0 -= event.delta.x * sensitivity; // yaw (Y-axis)
                rotation.1 = (rotation.1 - event.delta.y * sensitivity).clamp(
                    -std::f32::consts::FRAC_PI_2 + sensitivity,
                    std::f32::consts::FRAC_PI_2 - sensitivity,
                ); // pitch (X-axis)
            }

            transform.rotation = Quat::from_euler(EulerRot::YXZ, rotation.0, rotation.1, rotation.2);
        }

        if mouse_button_input.pressed(MouseButton::Middle) {
            for event in mouse_motion_events.read() {
                let delta = event.delta;

                // Розрахунок векторів для руху в площині екрана
                let right = transform.rotation * Vec3::X; // Вектор "праворуч" у світових координатах
                let up = transform.rotation * Vec3::Y;    // Вектор "вгору" у світових координатах

                let sensitivity = camera.sensitivity;

                // Рух камери: "праворуч" і "вгору"
                transform.translation -= (right * delta.x + up * delta.y) * sensitivity;
            }
        }
    }
}