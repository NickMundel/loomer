use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use cgmath::Vector2;
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    controller::{Controller, ControllerTrait},
    time::Time,
};

pub struct Spotlight {
    pub radius_mult: f32,
    pub radius_mult_delta: f32,
    pub enabled: bool,
    pub scroll_delta: f32,
    pub resize: bool,
    pub mouse_position: Vector2<f32>,

    controller: SpotlightController,
    uniform: SpotlightUniform,
}

impl Spotlight {
    pub fn process_mouse_events(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Button { button, state } => false,
            DeviceEvent::MouseMotion { delta } => false,
            DeviceEvent::MouseWheel { delta } => {
                self.scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(_) => 0.0,
                };
                true
            }
            _ => false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::ControlLeft),
                    ..
                } => {
                    println!("Enabled Spotlight");
                    self.enabled = !self.enabled;
                    true
                }
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::ShiftLeft),
                    ..
                } => {
                    self.resize = !self.resize;
                    true
                }
                _ => false,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vector2::new(position.x as f32, position.y as f32);
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, time: &Time) {
        if self.enabled {
            self.radius_mult = 5.0;
            self.radius_mult_delta = -15.0;
        }
        if self.scroll_delta != 0.0 && self.enabled && self.resize {
            self.radius_mult_delta -= self.scroll_delta as f32;
        }

        self.radius_mult = (self.radius_mult
            + self.radius_mult_delta * time.last_frame_time.elapsed().as_secs_f32())
        .clamp(0.3, 10.) as f32;
        self.radius_mult_delta -=
            self.radius_mult_delta * time.last_frame_time.elapsed().as_secs_f32() * 4.0;
    }
}

pub struct SpotlightController {
    pub delta_scale: f32,
    pub scale_pivot: Vector2<f32>,
    pub velocity: Vector2<f32>,
    parent: Option<Weak<RefCell<Controller>>>,
}

impl SpotlightController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SpotlightController {
            delta_scale: 0.0,
            scale_pivot: Vector2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            parent: None,
        }))
    }
}

impl ControllerTrait for SpotlightController {
    fn process_mouse_events(&mut self, event: &DeviceEvent) -> bool {
        true
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        true
    }

    fn set_parent(&mut self, parent: Weak<RefCell<Controller>>) {
        self.parent = Some(parent);
    }

    fn update(&mut self) {
        todo!()
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotlightUniform {
    tint: [f32; 4],
    position: [f32; 2],
    radius_mult: f32,
    _padding: [u32; 6],
}

impl SpotlightUniform {
    pub fn new() -> Self {
        SpotlightUniform {
            tint: [0.0, 0.0, 0.0, 0.0],
            position: [0.0, 0.0],
            radius_mult: 1.0,
            _padding: [0, 0, 0, 0, 0, 0],
        }
    }
    pub fn convert_into(&mut self, spotlight: &Spotlight, screen: Vector2<f32>) {
        self.tint = [0.0, 0.0, 0.0, 0.5];
        self.position = [
            spotlight.mouse_position.x / screen.x,
            spotlight.mouse_position.y / screen.y,
        ];
        self.radius_mult = spotlight.radius_mult;

        println!(
            "tint: {:?}, position: {:?}, radius: {:?}",
            self.tint, self.position, self.radius_mult
        );
    }
}
