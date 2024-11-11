use std::cell::RefCell;
use std::rc::{Rc, Weak};

use winit::event::{DeviceEvent, WindowEvent};

pub struct Controller {
    scrolled_amount: f32,
    children: Vec<Rc<RefCell<dyn ControllerTrait>>>,
}

impl Controller {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Controller {
            scrolled_amount: 0.0,
            children: Vec::new(),
        }))
    }

    pub fn add_child(&mut self, child: Rc<RefCell<dyn ControllerTrait>>) {
        self.children.push(child);
    }

    pub fn process_device_events(&mut self, event: &DeviceEvent) -> bool {
        for child in &self.children {
            if child.borrow_mut().process_mouse_events(event) {
                return true;
            }
        }
        false
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        for child in &self.children {
            if child.borrow_mut().process_events(event) {
                return true;
            }
        }
        false
    }

    pub fn scrolled_amount(&self) -> f32 {
        self.scrolled_amount
    }
}

pub trait ControllerTrait {
    fn process_mouse_events(&mut self, event: &DeviceEvent) -> bool;
    fn process_events(&mut self, event: &WindowEvent) -> bool;
    fn set_parent(&mut self, parent: Weak<RefCell<Controller>>);
    fn update(&mut self);
}
