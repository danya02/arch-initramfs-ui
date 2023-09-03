mod keys;

use std::{
    fs::{File, OpenOptions},
    os::{fd::OwnedFd, unix::prelude::OpenOptionsExt},
    path::Path,
    time::Instant,
};

pub trait App {
    fn ui(&mut self, ctx: &egui::Context);
}

use egui::{Event, PointerButton};
use input::LibinputInterface;
use keys::SuperModifiers;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use crate::keys::{keycode_to_egui_key, keycode_to_text};

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(unsafe { File::from(fd) })
    }
}

pub fn run(my_app: impl App) {
    let mut egui_ctx = egui::Context::default();

    let start = Instant::now();
    let mut libinput = input::Libinput::new_with_udev(Interface);
    libinput.udev_assign_seat("seat0");

    let scroll_speed = 1.0;

    let mut pointer_x = 0.0;
    let mut pointer_y = 0.0;

    let mut modifiers = SuperModifiers::default();

    use framebuffer::{Framebuffer, KdMode};
    let mut fb = Framebuffer::new("/dev/fb0").unwrap();

    let width = fb.var_screen_info.xres;
    let height = fb.var_screen_info.yres;
    let line_length = fb.fix_screen_info.line_length;
    let bytespp = fb.var_screen_info.bits_per_pixel / 8;

    // Enable graphics mode
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    // Game loop:
    loop {
        // Gather input (mouse, touches, keyboard, screen size, etc):
        let mut egui_events = vec![];

        loop {
            let new_event = libinput.next();
            if let Some(event) = new_event {
                match event {
                    input::Event::Device(_) => (),
                    input::Event::Keyboard(e) => match e {
                        input::event::KeyboardEvent::Key(k) => {
                            use input::event::keyboard::KeyboardEventTrait;
                            let keycode = k.key();
                            let is_down = match k.key_state() {
                                input::event::tablet_pad::KeyState::Pressed => true,
                                input::event::tablet_pad::KeyState::Released => false,
                            };
                            // Parse this key as an egui::Key or a change to a modifier
                            if let Some(key) = keycode_to_egui_key(keycode, modifiers) {
                                match key {
                                    keys::KeyOrModifier::Key(egui_key) => {
                                        egui_events.push(Event::Key {
                                            key: egui_key,
                                            pressed: is_down,
                                            repeat: false,
                                            modifiers: modifiers.into(),
                                        })
                                    }
                                    keys::KeyOrModifier::Mod(super_mod) => {
                                        // This returns a SuperModifiers with only one value set.

                                        // Normal modifier keys: if the event involves them,
                                        // then the corresponding modifier is set to whether the key is pressed.
                                        if super_mod.left_alt {
                                            modifiers.left_alt = is_down;
                                        } else if super_mod.right_alt {
                                            modifiers.right_alt = is_down;
                                        } else if super_mod.left_ctrl {
                                            modifiers.left_ctrl = is_down;
                                        } else if super_mod.right_ctrl {
                                            modifiers.right_ctrl = is_down;
                                        } else if super_mod.left_shift {
                                            modifiers.left_shift = is_down;
                                        } else if super_mod.right_shift {
                                            modifiers.right_shift = is_down;
                                        }
                                        // Lock keys: when pressing down, their value is toggled;
                                        // when releasing, nothing happens.
                                        else if is_down {
                                            if super_mod.caps_lock {
                                                modifiers.caps_lock = !modifiers.caps_lock;
                                            } else if super_mod.num_lock {
                                                modifiers.num_lock = !modifiers.num_lock;
                                            }
                                        }
                                    }
                                }
                            }

                            // Parse this key as text
                            if let Some(text) = keycode_to_text(keycode, modifiers) {
                                egui_events.push(Event::Text(String::from(text)));
                            }
                        }
                        _ => (),
                    },
                    input::Event::Pointer(p) => match p {
                        input::event::PointerEvent::Motion(m) => {
                            pointer_x += m.dx();
                            pointer_y += m.dy();
                            egui_events.push(Event::PointerMoved(egui::pos2(
                                pointer_x as f32,
                                pointer_y as f32,
                            )))
                        }
                        input::event::PointerEvent::MotionAbsolute(m) => {
                            pointer_x = m.absolute_x_transformed(width);
                            pointer_y = m.absolute_y_transformed(height);
                            egui_events.push(Event::PointerMoved(egui::pos2(
                                pointer_x as f32,
                                pointer_y as f32,
                            )))
                        }
                        input::event::PointerEvent::Button(b) => {
                            let btn = match b.button() {
                                272 => PointerButton::Primary,   // BTN_LEFT
                                273 => PointerButton::Secondary, // BTN_RIGHT
                                274 => PointerButton::Middle,    // BTN_MIDDLE
                                276 => PointerButton::Extra1,    // BTN_EXTRA (top side button)
                                275 => PointerButton::Extra2,    // BTN_SIDE (bottom side button)
                                _ => continue,
                            };
                            let pressed = match b.button_state() {
                                input::event::tablet_pad::ButtonState::Pressed => true,
                                input::event::tablet_pad::ButtonState::Released => false,
                            };
                            egui_events.push(Event::PointerButton {
                                pos: egui::pos2(pointer_x as f32, pointer_y as f32),
                                button: btn,
                                pressed,
                                modifiers: modifiers.into(),
                            })
                        }
                        input::event::PointerEvent::ScrollWheel(s) => {
                            use input::event::pointer::PointerScrollEvent;
                            let vert = s.scroll_value(input::event::pointer::Axis::Vertical) as f32
                                * scroll_speed;
                            let hor = s.scroll_value(input::event::pointer::Axis::Horizontal)
                                as f32
                                * scroll_speed;
                            egui_events.push(Event::Scroll(egui::Vec2 { x: hor, y: vert }));
                        }
                        input::event::PointerEvent::ScrollFinger(s) => {
                            use input::event::pointer::PointerScrollEvent;
                            let vert = s.scroll_value(input::event::pointer::Axis::Vertical) as f32
                                * scroll_speed;
                            let hor = s.scroll_value(input::event::pointer::Axis::Horizontal)
                                as f32
                                * scroll_speed;
                            egui_events.push(Event::Scroll(egui::Vec2 { x: hor, y: vert }));
                        }
                        input::event::PointerEvent::ScrollContinuous(s) => {
                            use input::event::pointer::PointerScrollEvent;

                            let vert = s.scroll_value(input::event::pointer::Axis::Vertical) as f32
                                * scroll_speed;
                            let hor = s.scroll_value(input::event::pointer::Axis::Horizontal)
                                as f32
                                * scroll_speed;
                            egui_events.push(Event::Scroll(egui::Vec2 { x: hor, y: vert }));
                        }
                        _ => todo!(),
                    },
                    input::Event::Touch(_) => (), // Unimplemented; touch not expected on this computer
                    input::Event::Tablet(_) => (), // Unimplemented; tablet not expected in pre-boot
                    input::Event::TabletPad(_) => (), // Unimplemented; tablet pad not expected in pre-boot
                    input::Event::Gesture(_) => (),   // Unimplemented; not important in GUI
                    input::Event::Switch(_) => (),    // Unimplemented; not important in GUI
                    _ => (),
                }
            } else {
                break;
            }
        }

        let raw_input: egui::RawInput = egui::RawInput {
            screen_rect: Some(egui::Rect {
                min: egui::pos2(0.0, 0.0),
                max: egui::pos2(width as f32, height as f32),
            }),
            pixels_per_point: None,
            max_texture_side: None,
            time: Some(start.elapsed().as_secs_f64()),
            predicted_dt: 1f32 / 60f32,
            modifiers: (),
            events: egui_events,
            hovered_files: vec![],
            dropped_files: vec![],
            focused: true,
        };
        let full_output = egui_ctx.run(raw_input, |egui_ctx| {
            my_app.ui(egui_ctx); // add panels, windows and widgets to `egui_ctx` here
        });
        let clipped_primitives = egui_ctx.tessellate(full_output.shapes); // creates triangles to paint

        paint(
            dfb.as_mut_slice(),
            fb.get_pixel_layout(),
            &full_output.textures_delta,
            clipped_primitives,
        );

        let platform_output = full_output.platform_output;
        set_cursor_icon(platform_output.cursor_icon);
        // if !platform_output.copied_text.is_empty() {
        //     set_clipboard_text(platform_output.copied_text);
        // }
        // See `egui::FullOutput` and `egui::PlatformOutput` for more
    }

    // At exit, disable graphics mode
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

// TODO: how does egui_glow work???
struct PainterContext {}

impl PainterContext {
    pub fn new() -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");

                glutin_window_context.get_proc_address(&s)
            })
        };
        Self {}
    }
}

fn paint(
    painer_ctx: &mut PainterContext,
    textures_delta: &egui::TexturesDelta,
    clipped_primitives: Vec<egui::ClippedPrimitive>,
) {
}
