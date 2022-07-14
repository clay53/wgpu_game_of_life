use std::sync::{Arc, Mutex};

use bui::{rect::{SizeAndCenter, Points, FillAspect}, ttf::CachedFace, renderer::Renderer, text::TextRenderer};
use bui_basic::{button::{Button, ClickedCallback}, text::Text, signal::{SignalReciever, ResizedSignal, RedrawCallback, CursorMovedSignal, MouseLeftDownSignal, MouseLeftUpSignal}, construct::{Construct, LineTarget, StandardConstructTarget}, containers::{Init, Fill}};
use winit::window::Window;

pub struct Game {
    exit_button: Button<Text>,
    step_button: Button<Text>,
    renderer: Option<Renderer>,
    text_renderer: Option<TextRenderer>,
    game_of_life: Option<GameOfLife>,
    resx: f32,
    resy: f32,
    width: u32,
    height: u32,
    bottom_bar_sy: f32,
}

impl Game {
    pub fn new(face: Arc<Mutex<CachedFace>>, resx: f32, resy: f32, width: u32, height: u32) -> Self {
        Self {
            exit_button: Button::new(Text::new_with_res("Exit", face.clone(), resx, resy), SizeAndCenter::ZERO),
            step_button: Button::new(Text::new_with_res("Step", face.clone(), resx, resy), SizeAndCenter::ZERO),
            renderer: None,
            text_renderer: None,
            game_of_life: None,
            resx,
            resy,
            width,
            height,
            bottom_bar_sy: 0.0,
        }
    }

    pub fn resume(&mut self, window: &Window) {
        let renderer = futures::executor::block_on(Renderer::new(window));
        self.text_renderer = Some(TextRenderer::new(renderer.device(), renderer.config().format, 1000, renderer.config().width, renderer.config().height));
        let mut game_of_life = GameOfLife::new(renderer.device(), renderer.config().format, self.width, self.height, self.resx, self.resy);
        game_of_life.fill(Self::calculate_game_of_life_space(self.bottom_bar_sy));
        self.game_of_life = Some(game_of_life);
        self.renderer = Some(renderer);
        self.construct();
    }

    fn calculate_game_of_life_space(bottom_bar_sy: f32) -> SizeAndCenter {
        SizeAndCenter {
            sx: 1.0,
            sy: 1.0-bottom_bar_sy,
            cx: 0.0,
            cy: 0.0+bottom_bar_sy,
        }
    }

    pub fn render(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        let text_renderer = self.text_renderer.as_ref().unwrap();
        let game_of_life = self.game_of_life.as_mut().unwrap();
        match renderer.surface().get_current_texture() {
            Ok(surface_texture) => {
                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Game render encoder"),
                });

                text_renderer.render_all(&mut encoder, &view, wgpu::LoadOp::Clear(wgpu::Color::WHITE));
                game_of_life.render(&mut encoder, &view, wgpu::LoadOp::Load);

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
                todo!()
            },
            Err(e) => {
                eprintln!("Surface error: {:?}", e);
            },
        }
    }

    pub fn construct(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        let text_renderer = self.text_renderer.as_mut().unwrap();
        let mut text_lines = LineTarget(Vec::new());
        text_lines.append(self.exit_button.construct());
        text_lines.append(self.step_button.construct());
        text_renderer.set_line_buffer(renderer.queue(), text_lines.0.as_slice());
        
        let game_of_life = self.game_of_life.as_mut().unwrap();
        game_of_life.construct(renderer.queue());
    }
}

impl Init for Game {
    fn init(&mut self) {
        self.fill(SizeAndCenter::FULL);
        self.exit_button.init();
        self.step_button.init();
    }
}

impl Fill for Game {
    fn fill(&mut self, fill_target: SizeAndCenter) {
        let mut bottom_bar_area: SizeAndCenter = FillAspect { // TODO: make this not stupid
            placement_area: fill_target,
            centerx: 0.0,
            centery: 0.0,
            resx: self.resx,
            resy: self.resy,
            aspect: 20.0,
        }.into();
        bottom_bar_area.cy = -1.0+bottom_bar_area.sy;
        self.bottom_bar_sy = bottom_bar_area.sy;

        self.exit_button.fill(bottom_bar_area.get_relative(Points {
            p1x: -0.999,
            p1y: 0.95,
            p2x: -0.7,
            p2y: -0.95,
        }.into()));

        self.step_button.fill(bottom_bar_area.get_relative(Points {
            p1x: -0.6,
            p1y: 0.95,
            p2x: 0.6,
            p2y: -0.95,
        }.into()));

        if let Some(game_of_life) = self.game_of_life.as_mut() {
            game_of_life.fill(Self::calculate_game_of_life_space(self.bottom_bar_sy));
        }
    }
}

impl SignalReciever<ResizedSignal, RedrawCallback> for Game {
    fn take_signal(&mut self, signal: &mut ResizedSignal) -> RedrawCallback {
        self.resx = signal.resx;
        self.resy = signal.resy;

        let renderer = self.renderer.as_mut().unwrap();
        renderer.resize(signal.resxp, signal.resyp);
        let text_renderer = self.text_renderer.as_mut().unwrap();
        text_renderer.on_resize(renderer.device(), signal.resxp, signal.resyp);

        // TODO: does calculations that are redone after this fills. Don't
        self.exit_button.take_signal(signal);
        self.step_button.take_signal(signal);
        
        let game_of_life = self.game_of_life.as_mut().unwrap();
        game_of_life.take_signal(signal);

        self.fill(SizeAndCenter::FULL);
        self.construct();

        RedrawCallback::new(true)
    }
}

impl SignalReciever<CursorMovedSignal, ()> for Game {
    fn take_signal(&mut self, signal: &mut CursorMovedSignal) {
        self.exit_button.take_signal(signal);
        self.step_button.take_signal(signal);
        let game_of_life = self.game_of_life.as_mut().unwrap();
        game_of_life.take_signal(signal);
    }
}

impl SignalReciever<MouseLeftDownSignal, ()> for Game {
    fn take_signal(&mut self, signal: &mut MouseLeftDownSignal) {
        self.exit_button.take_signal(signal);
        self.step_button.take_signal(signal);
    }
}

impl SignalReciever<MouseLeftUpSignal, (GameSetupCallback, RedrawCallback)> for Game {
    fn take_signal(&mut self, signal: &mut MouseLeftUpSignal) -> (GameSetupCallback, RedrawCallback) {
        let mut redraw_callback = RedrawCallback::new(false);
        let mut game_setup_callback = GameSetupCallback::None;
        let renderer = self.renderer.as_mut().unwrap();
        let game_of_life = self.game_of_life.as_mut().unwrap();

        if self.exit_button.take_signal(signal) == ClickedCallback::Clicked {
            game_setup_callback = GameSetupCallback::Setup;
        }
        if self.step_button.take_signal(signal) == ClickedCallback::Clicked {
            game_of_life.compute(&renderer);
            redraw_callback = RedrawCallback::new(true);
        }

        if let Some((x, y)) = game_of_life.take_signal(signal) {
            game_of_life.toggle(x, y, &renderer);
            redraw_callback = RedrawCallback::new(true);
        }

        (game_setup_callback, redraw_callback)
    }
}

pub enum GameSetupCallback {
    None,
    Setup,
}

enum Board {
    A,
    B,
}

bui::typed_uniform!(ToggleCellUniform, [i32; 2], "Toggle Cell Uniform");

pub struct GameOfLife {
    board_a: wgpu::Texture,
    board_b: wgpu::Texture,
    compute_pipeline: wgpu::ComputePipeline,
    board_bind_group_a: wgpu::BindGroup,
    board_bind_group_b: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_a: wgpu::BindGroup,
    render_bind_group_b: wgpu::BindGroup,
    active_board: Board,
    width: u32,
    height: u32,
    aspect: f32,
    vertices: [GameOfLifeVertex; 4],
    vertex_buffer: wgpu::Buffer,
    resx: f32,
    resy: f32,
    area: Points,
    mousex: f32,
    mousey: f32,
    toggle_pipeline: wgpu::ComputePipeline,
    toggle_cell_uniform: ToggleCellUniform,
    toggle_bind_group_a: wgpu::BindGroup,
    toggle_bind_group_b: wgpu::BindGroup,
}

impl GameOfLife {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat, width: u32, height: u32, resx: f32, resy: f32) -> Self {
        let compute_shader = device.create_shader_module(wgpu::include_wgsl!("game_of_life.wgsl"));

        let board_a = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game of Life board A"),
            size: wgpu::Extent3d {
                width,
                height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        });

        let board_a_view = board_a.create_view(&wgpu::TextureViewDescriptor::default());

        let board_b = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game of Life board B"),
            size: wgpu::Extent3d {
                width,
                height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        });

        let board_b_view = board_b.create_view(&wgpu::TextureViewDescriptor::default());

        let board_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game of Life board bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: wgpu::TextureFormat::Rgba16Float,
                    },
                    count: None,
                }
            ]
        });

        let board_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &board_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&board_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&board_b_view)
                }
            ],
            label: Some("Game of Life board bind group A")
        });

        let board_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &board_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&board_b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&board_a_view)
                }
            ],
            label: Some("Game of Life board bind group B")
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Game of Life compute pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Game of Life compute pipeline layout"),
                bind_group_layouts: &[
                    &board_bind_group_layout
                ],
                push_constant_ranges: &[],
            })),
            module: &compute_shader,
            entry_point: "compute_board"
        });

        let render_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Game of Life render sampler"),
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game of Life render bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ]
        });

        let render_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game of Life render bind group A"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&board_a_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_sampler)
                },
            ]
        });

        let render_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game of Life render bind group B"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&board_b_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_sampler)
                },
            ]
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Game of Life vertex buffer"),
            size: GAME_OF_LIFE_VERTEX_SIZE*4,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Game of Life render pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Game of Life render pipeline layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &compute_shader,
                entry_point: "render_vert",
                buffers: &[
                    GameOfLifeVertex::desc(),
                ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &compute_shader,
                entry_point: "render_frag",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let toggle_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Game of Life toggle bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: wgpu::TextureFormat::Rgba16Float,
                    },
                    count: None,
                }
            ]
        });

        let toggle_cell_uniform = ToggleCellUniform::new(device);

        let toggle_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game of Life toggle bind group"),
            layout: &toggle_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: toggle_cell_uniform.binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&board_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&board_b_view)
                }
            ]
        });

        let toggle_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game of Life toggle bind group"),
            layout: &toggle_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: toggle_cell_uniform.binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&board_b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&board_a_view)
                }
            ]
        });

        let toggle_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Game of Life toggle pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Game of Life toggle pipeline descriptor"),
                bind_group_layouts: &[
                    &toggle_bind_group_layout
                ],
                push_constant_ranges: &[]
            })),
            module: &compute_shader,
            entry_point: "toggle"
        });

        Self {
            board_a,
            board_b,
            compute_pipeline,
            board_bind_group_a,
            board_bind_group_b,
            active_board: Board::A,
            render_pipeline,
            render_bind_group_a,
            render_bind_group_b,
            width,
            height,
            aspect: width as f32/height as f32,
            vertices: [
                GameOfLifeVertex {
                    position: [0.0, 0.0],
                    tex_coords: [1.0, 1.0]
                },
                GameOfLifeVertex {
                    position: [0.0, 0.0],
                    tex_coords: [1.0, 0.0]
                },
                GameOfLifeVertex {
                    position: [0.0, 0.0],
                    tex_coords: [0.0, 1.0]
                },
                GameOfLifeVertex {
                    position: [0.0, 0.0],
                    tex_coords: [0.0, 0.0]
                }
            ],
            vertex_buffer,
            resx,
            resy,
            area: SizeAndCenter::ZERO.into(), // TODO: don't be lazy
            mousex: 0.0,
            mousey: 0.0,
            toggle_pipeline,
            toggle_cell_uniform,
            toggle_bind_group_a,
            toggle_bind_group_b,
        }
    }

    pub fn compute(&mut self, renderer: &Renderer) {
        let mut command_encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game of Life compute command encoder")
        });

        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Game of Life compute pass")
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        match self.active_board {
            Board::A => {
                compute_pass.set_bind_group(0, &self.board_bind_group_a, &[]);
                self.active_board = Board::B;
            },
            Board::B => {
                compute_pass.set_bind_group(0, &self.board_bind_group_b, &[]);
                self.active_board = Board::A;
            },
        }
        compute_pass.dispatch_workgroups(self.width, self.height, 1);

        drop(compute_pass);
        renderer.queue().submit(std::iter::once(command_encoder.finish()));
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, loadop: wgpu::LoadOp<wgpu::Color>) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game of Life render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: loadop,
                        store: true,
                    }
                })
            ],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        match self.active_board {
            Board::A => render_pass.set_bind_group(0, &self.render_bind_group_a, &[]),
            Board::B => render_pass.set_bind_group(0, &self.render_bind_group_b, &[]),
        }
        render_pass.draw(0..4, 0..1);
    }

    pub fn construct(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice::<GameOfLifeVertex, u8>(&self.vertices));
    }

    pub fn toggle(&mut self, x: i32, y: i32, renderer: &Renderer) {
        self.toggle_cell_uniform.set(&[x, y], renderer.queue());

        let mut command_encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game of Life compute command encoder")
        });

        match self.active_board {
            Board::A => command_encoder.copy_texture_to_texture(self.board_a.as_image_copy(), self.board_b.as_image_copy(), wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            }),
            Board::B => command_encoder.copy_texture_to_texture(self.board_b.as_image_copy(), self.board_a.as_image_copy(), wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            }),
        }

        renderer.queue().submit(std::iter::once(command_encoder.finish()));

        let mut command_encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game of Life compute command encoder")
        });

        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Game of Life compute pass")
        });

        compute_pass.set_pipeline(&self.toggle_pipeline);
        match self.active_board {
            Board::A => {
                compute_pass.set_bind_group(0, &self.toggle_bind_group_a, &[]);
                self.active_board = Board::B;
            },
            Board::B => {
                compute_pass.set_bind_group(0, &self.toggle_bind_group_b, &[]);
                self.active_board = Board::A;
            },
        }
        compute_pass.dispatch_workgroups(1, 1, 1);

        drop(compute_pass);
        renderer.queue().submit(std::iter::once(command_encoder.finish()));
    }
}

impl Fill for GameOfLife {
    fn fill(&mut self, fill_target: SizeAndCenter) {
        let points: Points = FillAspect {
            placement_area: fill_target,
            centerx: 0.0,
            centery: 0.0,
            resx: self.resx,
            resy: self.resy,
            aspect: self.aspect
        }.into();
        self.area = points;
        self.vertices[0].position = [points.p2x, points.p1y];
        self.vertices[1].position = [points.p2x, points.p2y];
        self.vertices[2].position = [points.p1x, points.p1y];
        self.vertices[3].position = [points.p1x, points.p2y];
    }
}

impl SignalReciever<ResizedSignal, ()> for GameOfLife {
    fn take_signal(&mut self, signal: &mut ResizedSignal) {
        self.resx = signal.resx;
        self.resy = signal.resy;
    }
}

impl SignalReciever<CursorMovedSignal, ()> for GameOfLife {
    fn take_signal(&mut self, signal: &mut CursorMovedSignal) -> () {
        self.mousex = signal.norm_posx;
        self.mousey = signal.norm_posy;
    }
}

impl SignalReciever<MouseLeftUpSignal, Option<(i32, i32)>> for GameOfLife {
    fn take_signal(&mut self, _signal: &mut MouseLeftUpSignal) -> Option<(i32, i32)> {
        if self.mousex >= self.area.p1x && self.mousex < self.area.p2x && self.mousey <= self.area.p1y && self.mousey > self.area.p2y {
            Some((
                ((self.mousex-self.area.p1x)/(self.area.p2x-self.area.p1x)*self.width as f32) as i32,
                ((self.mousey-self.area.p2y)/(self.area.p1y-self.area.p2y)*self.height as f32) as i32
             ))
        } else {
            None
        }
    }
}

const GAME_OF_LIFE_VERTEX_SIZE: u64 = std::mem::size_of::<GameOfLifeVertex>() as u64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GameOfLifeVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl GameOfLifeVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GameOfLifeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ]
        }
    }
}