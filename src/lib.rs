mod camera;
mod controller;
mod spotlight;
mod state;
mod texture;
mod time;
mod vertex;

use enigo::{Enigo, Mouse, Settings};
use image::RgbaImage;
use libwayshot::WayshotConnection;
use state::State;
use std::env::var_os;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    monitor::MonitorHandle,
    window::{Fullscreen, WindowBuilder},
};
use xcap::Monitor;

fn get_cursor_monitor(monitors: Vec<(usize, MonitorHandle)>) -> Option<MonitorHandle> {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let mouse_location = enigo.location().unwrap();

    for monitor in monitors {
        if monitor.1.position().x <= mouse_location.0
            && mouse_location.0 <= monitor.1.position().x + (monitor.1.size().width as i32)
            && monitor.1.position().y <= mouse_location.1
            && mouse_location.1 <= monitor.1.position().y + (monitor.1.size().height as i32)
        {
            return Some(monitor.clone().1);
        }
    }

    None
}

fn wayland_detect() -> bool {
    let xdg_session_type = var_os("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let wayland_display = var_os("WAYLAND_DISPLAY")
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    xdg_session_type.eq("wayland") || wayland_display.to_lowercase().contains("wayland")
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let monitor = get_cursor_monitor(
        event_loop
            .available_monitors()
            .enumerate()
            .collect::<Vec<_>>(),
    );

    let temp_mon = monitor.clone().unwrap();
    let screenshot: Vec<u8>;
    let (width, height): (u32, u32);

    if wayland_detect() {
        let wayshot_connection =
            WayshotConnection::new().expect("failed to connect to the wayland display server");
        let temp_screenshot = wayshot_connection
            .screenshot_single_output(
                wayshot_connection
                    .get_all_outputs()
                    .iter()
                    .find(|x| x.name == monitor.clone().unwrap().name().unwrap())
                    .unwrap(),
                false,
            )
            .expect("failed to take a screenshot");
        (width, height) = temp_screenshot.dimensions();
        screenshot = temp_screenshot.as_raw().to_owned();
    } else {
        let screen = Monitor::from_point(temp_mon.position().x, temp_mon.position().y).unwrap();
        let temp_screenshot = screen.capture_image().unwrap();
        screenshot = temp_screenshot.as_raw().to_owned();
        (width, height) = (temp_screenshot.width(), temp_screenshot.height());
        temp_screenshot.save("target/test.png").unwrap();
    }

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_fullscreen(Some(Fullscreen::Borderless(monitor)))
        .with_transparent(true)
        .with_resizable(false)
        .with_blur(true)
        .build(&event_loop)
        .unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(
        &window,
        RgbaImage::from_raw(width, height, screenshot).unwrap(),
    )
    .await;
    let mut surface_configured = false;

    let _ = event_loop
        .run(move |event, control_flow| {
            match event {
                Event::DeviceEvent { ref event, .. } => {
                    !state.device_input(event);
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => {
                    if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::KeyboardInput { event, .. } => match event {
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                } => control_flow.exit(),
                                _ => {}
                            },
                            WindowEvent::MouseInput { button, .. } => match button {
                                MouseButton::Right => {
                                    control_flow.exit();
                                }
                                _ => {}
                            },
                            WindowEvent::Resized(physical_size) => {
                                log::info!("physical_size: {physical_size:?}");
                                surface_configured = true;
                                state.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                state.window().request_redraw();

                                if !surface_configured {
                                    return;
                                }

                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                                state.time.update();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
