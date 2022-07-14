use std::sync::{Arc, Mutex};

use bui::{ttf::CachedFace, rect::{SizeAndCenter, FillAspect, Points}, text::TextRenderer, renderer::Renderer};
use bui_basic::{text::Text, text_input::TextInput, button::{Button, ClickedCallback}, containers::{Init, Fill}, signal::{SignalReciever, ResizedSignal, RedrawCallback, CursorMovedSignal, MouseLeftDownSignal, MouseLeftUpSignal, CharacterInputSignal}, construct::{LineTarget, StandardConstructTarget, Construct}};
use winit::window::Window;

pub struct Setup {
    title: Text,
    sizex_label: Text,
    sizex_input: TextInput,
    sizey_label: Text,
    sizey_input: TextInput,
    go_button: Button<Text>,
    error: Text,
    setup_area: SizeAndCenter,
    resx: f32,
    resy: f32,
    renderer: Option<Renderer>,
    text_renderer: Option<TextRenderer>,
}

impl Setup {
    pub fn new(face: Arc<Mutex<CachedFace>>, resx: f32, resy: f32) -> Self {
        // let initial_size = (resx.max(resy) as u32/4).to_string();
        let initial_size = "30".to_string();
        Self {
            title: Text::new_with_res("WGPU Game of Life", face.clone(), resx, resy),
            sizex_label: Text::new_with_res("Size X:", face.clone(), resx, resy),
            sizex_input: TextInput::new_with_res(initial_size.clone(), face.clone(), resx, resy),
            sizey_label: Text::new_with_res("Size Y:", face.clone(), resx, resy),
            sizey_input: TextInput::new_with_res(initial_size, face.clone(), resx, resy),
            go_button: Button::new(Text::new_with_res("Go", face.clone(), resx, resy), SizeAndCenter::ZERO),
            error: Text::new_with_res("", face, resx, resy),
            setup_area: SizeAndCenter::ZERO,
            resx,
            resy,
            renderer: None,
            text_renderer: None,
        }
    }

    pub fn resume(&mut self, window: &Window) {
        let renderer = futures::executor::block_on(Renderer::new(window));
        self.text_renderer = Some(TextRenderer::new(renderer.device(), renderer.config().format, 5000, renderer.config().width, renderer.config().height));
        self.renderer = Some(renderer);
        self.construct();
    }

    pub fn construct(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        let text_renderer = self.text_renderer.as_mut().unwrap();
        let mut text_lines = LineTarget(Vec::new());
        text_lines.append(self.title.construct());
        text_lines.append(self.sizex_label.construct());
        text_lines.append(self.sizex_input.construct());
        text_lines.append(self.sizey_label.construct());
        text_lines.append(self.sizey_input.construct());
        text_lines.append(self.go_button.construct());
        text_lines.append(self.error.construct());
        text_renderer.set_line_buffer(renderer.queue(), text_lines.0.as_slice());
    }

    pub fn render(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        let text_renderer = self.text_renderer.as_ref().unwrap();
        match renderer.surface().get_current_texture() {
            Ok(surface_texture) => {
                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

                text_renderer.render_all(&mut encoder, &view, wgpu::LoadOp::Clear(wgpu::Color::WHITE));

                renderer.queue().submit(std::iter::once(encoder.finish()));
                surface_texture.present();
            },
            Err(wgpu::SurfaceError::Lost) => {
                eprintln!("Surface lost!");
                renderer.reconfigure();
            },
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory!");
                // *control_flow = ControlFlow::Exit;
                panic!()
            },
            Err(e) => {
                eprintln!("Surface error: {:?}", e);
            },
        }
    }

    pub fn get_dimensions(&self) -> Result<(u32, u32), &str> {
        let width = match self.sizex_input.get_text().parse() {
            Ok(width) => if width > 0 {
                if width > 8192 {
                    return Err("width exceeds the maximum of 8192")
                } else {
                    width
                }
            } else {
                return Err("width must be greater than 0")
            },
            Err(_) => return Err("failed to parse width as whole number")
        };

        let height = match self.sizey_input.get_text().parse() {
            Ok(height) => if height > 0 {
                if height > 8192 {
                    return Err("height exceeds the maximum of 8192")
                } else {
                    height
                }
            } else {
                return Err("height must be greater than 0")
            },
            Err(_) => return Err("failed to parse height as whole number")
        };

        Ok((width, height))
    }

    pub fn set_error(&mut self, err: String) {
        self.error.set_text(err);
    }
}

impl Init for Setup {
    fn init(&mut self) {
        self.fill(SizeAndCenter::FULL);
        self.go_button.init();
    }
}

impl Fill for Setup {
    fn fill(&mut self, fill_target: SizeAndCenter) {
        self.setup_area = FillAspect {
            placement_area: fill_target,
            centerx: 0.0,
            centery: 0.0,
            resx: self.resx,
            resy: self.resy,
            aspect: 3.0/4.0
        }.into();

        self.title.fill(self.setup_area.get_relative(Points {
            p1x: -0.9,
            p1y: 0.9,
            p2x: 0.9,
            p2y: 0.8
        }.into()));

        self.sizex_label.fill(self.setup_area.get_relative(Points {
            p1x: -0.9,
            p1y: 0.75,
            p2x: -0.025,
            p2y: 0.65
        }.into()));

        self.sizex_input.fill(self.setup_area.get_relative(Points {
            p1x: 0.025,
            p1y: 0.75,
            p2x: 0.9,
            p2y: 0.65
        }.into()));

        self.sizey_label.fill(self.setup_area.get_relative(Points {
            p1x: -0.9,
            p1y: 0.6,
            p2x: -0.025,
            p2y: 0.5
        }.into()));

        self.sizey_input.fill(self.setup_area.get_relative(Points {
            p1x: 0.025,
            p1y: 0.6,
            p2x: 0.9,
            p2y: 0.5
        }.into()));

        self.go_button.fill(self.setup_area.get_relative(Points {
            p1x: -0.6,
            p1y: 0.45,
            p2x: 0.6,
            p2y: 0.35
        }.into()));

        self.error.fill(self.setup_area.get_relative(Points {
            p1x: -0.9,
            p1y: 0.4,
            p2x: 0.9,
            p2y: -0.9,
        }.into()));
    }
}

impl SignalReciever<ResizedSignal, RedrawCallback> for Setup {
    fn take_signal(&mut self, signal: &mut ResizedSignal) -> RedrawCallback {
        self.resx = signal.resx;
        self.resy = signal.resy;

        let renderer = self.renderer.as_mut().unwrap();
        renderer.resize(signal.resxp, signal.resyp);
        let text_renderer = self.text_renderer.as_mut().unwrap();
        text_renderer.on_resize(renderer.device(), signal.resxp, signal.resyp);

        self.title.take_signal(signal);
        self.sizex_label.take_signal(signal);
        self.sizex_input.take_signal(signal);
        self.sizey_label.take_signal(signal);
        self.sizey_input.take_signal(signal);
        self.go_button.take_signal(signal);
        self.error.take_signal(signal);

        self.construct();

        RedrawCallback::new(true)
    }
}

impl SignalReciever<CursorMovedSignal, RedrawCallback> for Setup {
    fn take_signal(&mut self, signal: &mut CursorMovedSignal) -> RedrawCallback {
        let mut reconstruct_signal = self.title.take_signal(signal).0;
        reconstruct_signal.or(self.sizex_label.take_signal(signal).0);
        reconstruct_signal.or(self.sizey_label.take_signal(signal).0);
        reconstruct_signal.or(self.error.take_signal(signal).0);
        self.sizex_input.take_signal(signal);
        self.sizey_input.take_signal(signal);
        self.go_button.take_signal(signal);

        if reconstruct_signal.get_reconstruct() {
            self.construct();
            RedrawCallback::new(true)
        } else {
            RedrawCallback::new(false)
        }
    }
}

impl SignalReciever<MouseLeftDownSignal, RedrawCallback> for Setup {
    fn take_signal(&mut self, signal: &mut MouseLeftDownSignal) -> RedrawCallback {
        let mut reconstruct_signal = self.title.take_signal(signal).0;
        reconstruct_signal.or(self.sizex_label.take_signal(signal).0);
        reconstruct_signal.or(self.sizey_label.take_signal(signal).0);
        reconstruct_signal.or(self.error.take_signal(signal).0);
        self.go_button.take_signal(signal);

        if reconstruct_signal.get_reconstruct() {
            self.construct();
            RedrawCallback::new(true)
        } else {
            RedrawCallback::new(false)
        }
    }
}

impl SignalReciever<MouseLeftUpSignal, SetupLeftUp> for Setup {
    fn take_signal(&mut self, signal: &mut MouseLeftUpSignal) -> SetupLeftUp {
        self.title.take_signal(signal);
        self.sizex_label.take_signal(signal);
        self.sizex_input.take_signal(signal);
        self.sizey_label.take_signal(signal);
        self.sizey_input.take_signal(signal);
        self.error.take_signal(signal);
        if self.go_button.take_signal(signal) == ClickedCallback::Clicked {
            SetupLeftUp::Go
        } else {
            SetupLeftUp::DoNothing
        }
    }
}

pub enum SetupLeftUp {
    DoNothing,
    Go,
}

impl SignalReciever<CharacterInputSignal, RedrawCallback> for Setup {
    fn take_signal(&mut self, signal: &mut CharacterInputSignal) -> RedrawCallback {
        let mut reconstruct_signal = self.sizex_input.take_signal(signal);
        reconstruct_signal.or(self.sizey_input.take_signal(signal));

        if reconstruct_signal.get_reconstruct() {
            self.construct();
            RedrawCallback::new(true)
        } else {
            RedrawCallback::new(false)
        }
    }
}