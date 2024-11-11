use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::controller::{Controller, ControllerTrait};
use cgmath::Vector2;
use winit::event::{DeviceEvent, MouseButton, WindowEvent};

use crate::time::Time;

const VELOCITY_THRESHOLD: f32 = 15.0;

pub struct Camera {
    pub offset: Vector2<f32>,
    pub target: Vector2<f32>,
    pub rotation: f32,
    pub zoom: f32,

    controller: CameraController,
    uniform: CameraUniform,
}

impl Camera {
    fn get_screen_to_world2D(&self, screen: Vector2<f32>) -> Vector2<f32> {
        screen / self.zoom + self.target
    }
}

pub struct CameraController {
    pub delta_scale: f32,
    pub scale_pivot: Vector2<f32>,
    pub velocity: Vector2<f32>,
    parent: Option<Weak<RefCell<Controller>>>,
}

impl CameraController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(CameraController {
            delta_scale: 0.0,
            scale_pivot: Vector2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            parent: None,
        }))
    }
}

impl ControllerTrait for CameraController {
    fn process_mouse_events(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Button { button, state } => false,
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = Vector2::new(delta.0 as f32, delta.1 as f32);
                true
            }
            _ => false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vector2::new(position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                self.scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(_) => 0.0,
                };
                true
            }
            WindowEvent::MouseInput { button, .. } => match button {
                MouseButton::Middle => {
                    self.middle_pressed = !self.middle_pressed;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self) {
        if self.scroll_delta != 0.0 {
            self.delta_scale += self.scroll_delta as f32;
        }

        if self.delta_scale.abs() > 0.5 {
            let p0 = self.mouse_position / self.zoom;
            self.zoom = (self.zoom
                + self.delta_scale * time.last_frame_time.elapsed().as_secs_f32())
            .clamp(1.0, 10.);
            let p1 = self.mouse_position / self.zoom;
            self.target = p0 - p1;
            self.delta_scale -=
                self.delta_scale * time.last_frame_time.elapsed().as_secs_f32() * 4.0
        }

        if self.middle_pressed {
            let delta = self.get_screen_to_world2D(self.mouse_position - self.mouse_delta)
                - self.get_screen_to_world2D(self.mouse_position);
            self.target += delta;
            self.velocity = delta * time.fps as f32;
        } else if (self.velocity.x * self.velocity.x) + (self.velocity.y * self.velocity.y)
            > VELOCITY_THRESHOLD * VELOCITY_THRESHOLD
        {
            self.target += self.velocity * time.last_frame_time.elapsed().as_secs_f32();
            self.velocity -= self.velocity * time.last_frame_time.elapsed().as_secs_f32() * 6.0;
        }
    }

    fn set_parent(&mut self, parent: Weak<RefCell<Controller>>) {
        self.parent = Some(parent);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    offset: [f32; 2],
    scale: f32,
    // shitty padding, read more here:https://stackoverflow.com/questions/75522842/problem-with-aligning-rust-structs-to-send-to-the-gpu-using-bytemuck-and-wgpu
    _padding: u32,
}

impl CameraUniform {
    pub fn new() -> Self {
        CameraUniform {
            offset: [0.0, 0.0],
            scale: 1.0,
            _padding: 0,
        }
    }
    pub fn convert_into(&mut self, camera: &Camera, screen: Vector2<f32>) {
        self.offset = [camera.target.x / screen.x, camera.target.y / screen.y];
        self.scale = camera.zoom;
        println!("offset: {:?}, scale: {:?}", self.offset, camera.zoom);
    }
}
