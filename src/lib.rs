use bui::{ttf::CachedFace};
use bui_basic::{signal::{ResizedSignal, CursorMovedSignal, MouseLeftDownSignal, MouseLeftUpSignal, CharacterInputSignal, SignalReciever, RedrawCallback}, containers::Init};
use winit::{window::{Window, Fullscreen}, event_loop::ControlFlow, event::{Event, WindowEvent, MouseButton, ElementState, VirtualKeyCode, TouchPhase}};
use log::info;
use std::sync::{Arc, Mutex};

pub mod setup;
pub mod game;
pub mod bui_view;
use bui_view::BuiView;

fn resume(window: &Window, resumed: &mut bool, bui_view: &mut BuiView) {
    bui_view.resume(window);
    *resumed = true;
}

pub struct ResumeCallback {
    resume: bool
}

impl ResumeCallback {
    pub fn new(resume: bool) -> Self {
        Self {
            resume,
        }
    }

    pub fn or(&mut self, resume_callback: ResumeCallback) {
        if resume_callback.get_resume() {
            self.resume = true;
        }
    }

    pub fn get_resume(&self) -> bool {
        self.resume
    }
}

#[cfg_attr(target_os = "android", ndk_glue::main(logger(level = "debug", filter="language_tutor_bui,bui_basic,bui_file_browser", tag = "language-tutor")))]
pub fn main() {
    #[cfg(target_arch="wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).unwrap();
        info!("Word Search Solver logger initialized");
    }
    #[cfg(not(any(target_os="android", target_arch="wasm32")))]
    {
        env_logger::init();
        info!("Word Search Solver logger initialized");
    }
    info!("Starting Word Search Solver...");
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Word Search Solver")
        // .with_inner_size(winit::dpi::PhysicalSize {
        //     width: 640_f32,
        //     height: 640_f32,
        // })
        .build(&event_loop).unwrap();
    
    let mut resx: f32;
    let mut resy: f32;

    #[cfg(target_arch="wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();
        let dom_window = web_sys::window().unwrap();
        let document = dom_window.document().unwrap();
        let body = document.body().unwrap();

        resx = canvas.width() as f32;
        resy = canvas.height() as f32;

        body.append_child(&canvas).unwrap();
    }
    
    #[cfg(not(target_arch="wasm32"))]
    {
        resx = 640.0;
        resy = 640.0;
    }

    let face = Arc::from(Mutex::from(CachedFace::from_vec(include_bytes!("NotoSansJP-Regular.otf").to_vec(), 0).unwrap()));

    let mut bui_view = BuiView::new(face, resx, resy);
    bui_view.init();

    let take_redraw_callback = |redraw_callback: RedrawCallback, window: &Window| {
        if redraw_callback.get_redraw() {
            window.request_redraw()
        }
    };

    let mut resumed = false;
    #[cfg(target_os="android")]
    let mut capitalized = false;

    #[cfg(not(target_os="android"))]
    {
        resume(&window, &mut resumed, &mut bui_view);
    }
    info!("Starting event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Exit requested. Exiting Language Tutor...");
                *control_flow = ControlFlow::Exit;
            },
            Event::Resumed => {
                info!("Resuming Language Tutor...");
                resume(&window, &mut resumed, &mut bui_view);
                info!("Resumed Language Tutor!");
            },
            Event::Suspended => {
                info!("Suspending Language Tutor...");
                if resumed {
                    resumed = false;
                }
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        info!("Language Tutor resized");
                        resx = physical_size.width as f32;
                        resy = physical_size.height as f32;
                        take_redraw_callback(
                            bui_view.take_signal(&mut ResizedSignal {
                                resxp: physical_size.width,
                                resyp: physical_size.height,
                                resx,
                                resy,
                            }),
                            &window,
                        );
                    },
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    } => {
                        info!("Language Tutor scale factor changed");
                        resx = new_inner_size.width as f32;
                        resy = new_inner_size.height as f32;
                        if !resumed {
                            return;
                        }
                        take_redraw_callback(
                            bui_view.take_signal(&mut ResizedSignal {
                                resxp: new_inner_size.width,
                                resyp: new_inner_size.height,
                                resx,
                                resy,
                            }),
                            &window,
                        );
                    },
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        take_redraw_callback(
                            bui_view.take_signal(&mut CursorMovedSignal {
                                pixel_posx: position.x as f32,
                                pixel_posy: position.y as f32,
                                norm_posx: position.x as f32/resx*2.0-1.0,
                                norm_posy: position.y as f32/resy*-2.0+1.0,
                            }),
                            &window,
                        );
                    },
                    WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        match button {
                            MouseButton::Left if *state == ElementState::Pressed => take_redraw_callback(
                                bui_view.take_signal(&mut MouseLeftDownSignal()),
                                &window,
                            ),
                            MouseButton::Left if *state == ElementState::Released => {
                                let (redraw_callback, resume_callback) = bui_view.take_signal(&mut MouseLeftUpSignal());
                                take_redraw_callback(
                                    redraw_callback,
                                    &window,
                                );
                                if resume_callback.get_resume() {
                                    resume(&window, &mut resumed, &mut bui_view)
                                }
                            },
                            _ => {}
                        }
                    },
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        if let Some(virtual_keycode) = input.virtual_keycode {
                            match virtual_keycode {
                                VirtualKeyCode::F11 if input.state == ElementState::Pressed  => if let Some(_) = window.fullscreen() {
                                    window.set_fullscreen(None);
                                } else {
                                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                },
                                _ => {}
                            }
                        }

                        #[cfg(target_os="android")]
                        {
                            info!("Got keyboard input: {:#?}", input);
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                if input.state == ElementState::Pressed {
                                    match virtual_keycode {
                                        VirtualKeyCode::LShift => capitalized = true,
                                        _ => {}
                                    }
                                } else {
                                    let renderer = renderer.as_mut().unwrap();
                                    let mut line_renderer = text_renderer.as_mut().unwrap();
                                    let c = match virtual_keycode {
                                        VirtualKeyCode::LShift => {
                                            capitalized = false;
                                            return;
                                        },
                                        VirtualKeyCode::Key0 if !capitalized => '0',
                                        VirtualKeyCode::Key1 if !capitalized => '1',
                                        VirtualKeyCode::Key2 if !capitalized => '2',
                                        VirtualKeyCode::Key3 if !capitalized => '3',
                                        VirtualKeyCode::Key4 if !capitalized => '4',
                                        VirtualKeyCode::Key5 if !capitalized => '5',
                                        VirtualKeyCode::Key6 if !capitalized => '6',
                                        VirtualKeyCode::Key7 if !capitalized => '7',
                                        VirtualKeyCode::Key8 if !capitalized => '8',
                                        VirtualKeyCode::Key9 if !capitalized => '9',
                                        VirtualKeyCode::A if capitalized => 'A',
                                        VirtualKeyCode::A if !capitalized => 'a',
                                        VirtualKeyCode::B if capitalized => 'B',
                                        VirtualKeyCode::B if !capitalized => 'b',
                                        VirtualKeyCode::C if capitalized => 'C',
                                        VirtualKeyCode::C if !capitalized => 'c',
                                        VirtualKeyCode::D if capitalized => 'D',
                                        VirtualKeyCode::D if !capitalized => 'd',
                                        VirtualKeyCode::E if capitalized => 'E',
                                        VirtualKeyCode::E if !capitalized => 'e',
                                        VirtualKeyCode::F if capitalized => 'F',
                                        VirtualKeyCode::F if !capitalized => 'f',
                                        VirtualKeyCode::G if capitalized => 'G',
                                        VirtualKeyCode::G if !capitalized => 'g',
                                        VirtualKeyCode::H if capitalized => 'H',
                                        VirtualKeyCode::H if !capitalized => 'h',
                                        VirtualKeyCode::I if capitalized => 'I',
                                        VirtualKeyCode::I if !capitalized => 'i',
                                        VirtualKeyCode::J if capitalized => 'J',
                                        VirtualKeyCode::J if !capitalized => 'j',
                                        VirtualKeyCode::K if capitalized => 'K',
                                        VirtualKeyCode::K if !capitalized => 'k',
                                        VirtualKeyCode::L if capitalized => 'L',
                                        VirtualKeyCode::L if !capitalized => 'l',
                                        VirtualKeyCode::M if capitalized => 'M',
                                        VirtualKeyCode::M if !capitalized => 'm',
                                        VirtualKeyCode::N if capitalized => 'N',
                                        VirtualKeyCode::N if !capitalized => 'n',
                                        VirtualKeyCode::O if capitalized => 'O',
                                        VirtualKeyCode::O if !capitalized => 'o',
                                        VirtualKeyCode::P if capitalized => 'P',
                                        VirtualKeyCode::P if !capitalized => 'p',
                                        VirtualKeyCode::Q if capitalized => 'Q',
                                        VirtualKeyCode::Q if !capitalized => 'q',
                                        VirtualKeyCode::R if capitalized => 'R',
                                        VirtualKeyCode::R if !capitalized => 'r',
                                        VirtualKeyCode::S if capitalized => 'S',
                                        VirtualKeyCode::S if !capitalized => 's',
                                        VirtualKeyCode::T if capitalized => 'T',
                                        VirtualKeyCode::T if !capitalized => 't',
                                        VirtualKeyCode::U if capitalized => 'U',
                                        VirtualKeyCode::U if !capitalized => 'u',
                                        VirtualKeyCode::V if capitalized => 'V',
                                        VirtualKeyCode::V if !capitalized => 'v',
                                        VirtualKeyCode::W if capitalized => 'W',
                                        VirtualKeyCode::W if !capitalized => 'w',
                                        VirtualKeyCode::X if capitalized => 'X',
                                        VirtualKeyCode::X if !capitalized => 'x',
                                        VirtualKeyCode::Y if capitalized => 'Y',
                                        VirtualKeyCode::Y if !capitalized => 'y',
                                        VirtualKeyCode::Z if capitalized => 'Z',
                                        VirtualKeyCode::Z if !capitalized => 'z',
                                        VirtualKeyCode::Semicolon if capitalized => ':',
                                        VirtualKeyCode::Semicolon if !capitalized => ';',
                                        VirtualKeyCode::Slash if !capitalized => '/',
                                        VirtualKeyCode::Period if !capitalized => '.',
                                        VirtualKeyCode::Minus if capitalized => '_',
                                        VirtualKeyCode::Minus if !capitalized => '-',
                                        _ => return
                                    };
                                    take_reconstruct_callback(
                                        bui_view.take_signal(&mut CharacterInputSignal {
                                            input: c,
                                        }),
                                        &bui_view,
                                        &mut line_renderer,
                                        renderer,
                                        &window,
                                    );
                                }
                            }
                        }
                    },
                    WindowEvent::ReceivedCharacter(c) => {
                        #[cfg(target_os="android")]
                        info!("Got character: {}", c);

                        #[cfg(not(target_os="android"))]
                        {
                            take_redraw_callback(
                                bui_view.take_signal(&mut CharacterInputSignal {
                                    input: *c,
                                }),
                                &window,
                            );
                        }
                    }
                    WindowEvent::Touch(touch) => {
                        if touch.id == 0 {
                            take_redraw_callback(
                                bui_view.take_signal(&mut CursorMovedSignal {
                                    pixel_posx: touch.location.x as f32,
                                    pixel_posy: touch.location.y as f32,
                                    norm_posx: touch.location.x as f32/resx*2.0-1.0,
                                    norm_posy: touch.location.y as f32/resy*-2.0+1.0,
                                }),
                                &window,
                            );
                            match touch.phase {
                                TouchPhase::Started => take_redraw_callback(
                                    bui_view.take_signal(&mut MouseLeftDownSignal()),
                                    &window,
                                ),
                                TouchPhase::Ended | TouchPhase::Cancelled => {
                                    let (redraw_callback, resume_callback) = bui_view.take_signal(&mut MouseLeftUpSignal());
                                    take_redraw_callback(
                                        redraw_callback,
                                        &window,
                                    );
                                    if resume_callback.get_resume() {
                                        resume(&window, &mut resumed, &mut bui_view)
                                    }
                                },
                                TouchPhase::Moved => {}
                            }
                        }
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(_) => {
                bui_view.render();
            }
            _ => {}
        }
    });
}