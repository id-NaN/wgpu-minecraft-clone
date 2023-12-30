use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use super::camera::PerspectiveCamera;
use super::Camera;

pub struct CameraController {
    position: glm::Vec3,
    yaw: f32,
    pitch: f32,

    forward_pressed: bool,
    backward_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
    control_pressed: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            forward_pressed: false,
            backward_pressed: false,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            control_pressed: false,
        }
    }

    pub fn set_position(&mut self, postion: glm::Vec3) {
        self.position = postion
    }

    pub fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>) {
        let angles = glm::quat_euler_angles(&rotation);
        self.pitch = angles.z;
        self.yaw = angles.x;
    }

    pub fn update(&mut self, camera: &mut dyn Camera) {
        let rotation =
            na::UnitQuaternion::from_euler_angles(self.pitch, self.yaw, 0.0);
        let forward = rotation
            * na::UnitVector3::new_normalize(glm::vec3(0.0, 0.0, 1.0));
        let up = rotation
            * na::UnitVector3::new_normalize(glm::vec3(0.0, 1.0, 0.0));
        let right = rotation
            * na::UnitVector3::new_normalize(glm::vec3(1.0, 0.0, 0.0));

        let velocity = if self.control_pressed { 1.0 } else { 0.1 };

        if self.forward_pressed {
            self.position += forward.scale(velocity);
        }
        if self.backward_pressed {
            self.position -= forward.scale(velocity);
        }
        if self.up_pressed {
            self.position += up.scale(velocity);
        }
        if self.down_pressed {
            self.position -= up.scale(velocity);
        }
        if self.right_pressed {
            self.position += right.scale(velocity);
        }
        if self.left_pressed {
            self.position -= right.scale(velocity);
        }

        camera.set_position(self.position);
        camera.set_rotation(rotation);
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.forward_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.backward_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.up_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.down_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.left_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.right_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::LControl => {
                        self.control_pressed = pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn handle_mouse_move(&mut self, delta: glm::Vec2) {
        self.pitch += delta.y / 200.0;
        self.yaw += delta.x / 200.0;

        self.pitch = self
            .pitch
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
}
