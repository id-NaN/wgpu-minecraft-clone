mod camera;
mod camera_controller;
mod game_render_data;
mod mesh;
pub mod model;
mod shader;
mod texture;
pub mod texture_atlas;
mod world;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{self};

use camera::Camera;
use camera_controller::CameraController;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
pub use game_render_data::GameRenderData;
use mesh::{Mesh, Vertex};
use shader::load_shader_module;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use self::camera::{
    ObliqueOrthographicCamera,
    OrthographicCamera,
    PerspectiveCamera,
};
pub use self::texture_atlas::TextureAtlas;
use self::world::{ChunkMeshEvent, RenderWorld};
use crate::settings::SETTINGS;
use crate::world::{ChunkEvent, World};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: glm::Mat4,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_projection: glm::identity(),
        }
    }

    fn update_view_projection(&mut self, camera: &dyn Camera) {
        self.view_projection = camera.get_view_projection();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    direction: nalgebra::UnitVector3<f32>,
    _pad_0: u32,
    color: glm::Vec3,
    _pad_1: u32,
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: Option<wgpu::TextureFormat>,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: &wgpu::ShaderModule,
    vertex_entry: &str,
    fragment_entry: &str,
) -> wgpu::RenderPipeline {
    let fragment_targets = match color_format {
        Some(format) => vec![Some(wgpu::ColorTargetState {
            format: format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::all(),
        })],
        None => vec![],
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: vertex_entry,
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: fragment_entry,
            targets: &fragment_targets,
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format: format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn calc_light_direction(frame: u64) -> na::UnitVector3<f32> {
    na::Unit::new_normalize(glm::rotate_vec3(
        &glm::vec3(0.0, 1.0, 0.0),
        10000 as f32 / 500.0,
        &glm::vec3(1.0, 0.5, 0.0).normalize(),
    ))
}
pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    depth_texture: Texture,
    pipeline: wgpu::RenderPipeline,
    camera: Box<dyn Camera>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    texture_atlas_image: Texture,
    texture_atlas_bind_group: wgpu::BindGroup,
    chunk_queue: Arc<Mutex<VecDeque<ChunkMeshEvent>>>,
    chunk_meshes: HashMap<glm::IVec2, Mesh>,
    frame_count: u64,
}

impl<'a> Renderer<'a> {
    fn new(
        window: &'a Window,
        render_data: GameRenderData,
        world: Arc<Mutex<World>>,
    ) -> Result<(Self, RenderWorld)> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window) }?;

        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        ))
        .wrap_err("Failure requesting adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features:
                    wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            desired_maximum_frame_latency: 2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut camera = Box::new(PerspectiveCamera::new(
            (size.width as f32) / (size.height as f32),
            0.1,
            1000.0,
        ));
        // let mut camera = Box::new(ObliqueOrthographicCamera::new(
        //     100.0,
        //     100.0,
        //     (size.width as f32) / (size.height as f32),
        //     glm::vec3(0.0, 0.0, 1.0),
        // ));
        let mut camera_controller = CameraController::new();
        camera_controller.set_position(glm::vec3(0.0, 80.0, 0.0));
        camera_controller.update(camera.as_mut());
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(camera.as_ref());
        let camera_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera uniform"),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[camera_uniform]),
            });

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );
        let camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let light_uniform = LightUniform {
            direction: nalgebra::Unit::new_normalize(glm::vec3(0.4, 0.7, 0.3)),
            color: glm::vec3(1.5, 1.5, 1.5),
            _pad_0: 0,
            _pad_1: 0,
        };
        let light_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light uniform"),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[light_uniform]),
            });
        let light_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Light bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );
        let light_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Light bind group"),
                layout: &light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                }],
            });

        let texture_atlas_image = render_data.texture_atlas().texture(
            &device,
            &queue,
            Some("Block Atlas"),
        );

        let atlas_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            },
        );

        let texture_atlas_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture atlas bind group"),
                layout: &atlas_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            texture_atlas_image.view(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            texture_atlas_image.sampler(),
                        ),
                    },
                ],
            });

        let depth_texture = Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            Some("Depth Texture"),
        );

        let shader = load_shader_module(&device, "main", &[])?;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &atlas_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some(config.format),
            Some(Texture::DEPTH_FORMAT),
            &[Vertex::desc()],
            &shader,
            "vs_main",
            "fs_main",
        );

        let (render_world, chunk_queue) = RenderWorld::new(world, render_data);

        Ok((
            Self {
                window,
                size,
                surface,
                device,
                queue,
                config,
                pipeline,
                camera,
                camera_uniform,
                camera_buffer,
                camera_bind_group,
                camera_controller,
                light_uniform,
                light_buffer,
                light_bind_group,
                depth_texture,
                texture_atlas_image,
                texture_atlas_bind_group,
                chunk_queue,
                chunk_meshes: HashMap::new(),
                frame_count: 0,
            },
            render_world,
        ))
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                self.config.width,
                self.config.height,
                Some("Depth Texture"),
            );
            self.camera.set_aspect_ratio(
                (new_size.width as f32) / (new_size.height as f32),
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.handle_event(event)
    }

    fn mouse_event(&mut self, delta: glm::Vec2) -> Result<()> {
        let position = winit::dpi::PhysicalPosition {
            x: self.size.width / 2,
            y: self.size.height / 2,
        };
        self.window.set_cursor_position(position)?;
        self.camera_controller.handle_mouse_move(delta);
        Ok(())
    }

    fn update(&mut self) {
        self.camera_controller.update(self.camera.as_mut());
        self.camera_uniform
            .update_view_projection(self.camera.as_ref());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        let mut light_direction = calc_light_direction(self.frame_count);
        if light_direction.y < 0.0 {
            light_direction = -light_direction
        }
        self.light_uniform.direction = light_direction;
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    fn render(&mut self) -> Result<()> {
        self.frame_count += 1;
        match self.surface.get_current_texture() {
            Ok(output) => {
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Render encoder"),
                    },
                );
                {
                    {
                        let mut render_pass = encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {
                                label: Some("Render pass"),
                                color_attachments: &[Some(
                                    wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(
                                                wgpu::Color {
                                                    r: 0.0,
                                                    g: 0.0,
                                                    b: 0.0,
                                                    a: 1.0,
                                                },
                                            ),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    },
                                )],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: self.depth_texture.view(),
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: wgpu::StoreOp::Store,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                ..Default::default()
                            },
                        );
                        render_pass.set_pipeline(&self.pipeline);
                        render_pass.set_bind_group(
                            0,
                            &self.camera_bind_group,
                            &[],
                        );
                        render_pass.set_bind_group(
                            1,
                            &self.texture_atlas_bind_group,
                            &[],
                        );
                        render_pass.set_bind_group(
                            2,
                            &self.light_bind_group,
                            &[],
                        );
                        for mesh in self.chunk_meshes.values() {
                            render_pass.set_vertex_buffer(
                                0,
                                mesh.vertex_buffer()?.slice(..),
                            );
                            render_pass.set_index_buffer(
                                mesh.index_buffer()?.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            render_pass.draw_indexed(
                                0..mesh.index_count(),
                                0,
                                0..1,
                            );
                        }
                    }
                }
                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            Err(wgpu::SurfaceError::Outdated) => {
                log::info!("Outdated surface texture");
                self.surface.configure(&self.device, &self.config)
            }
            Err(wgpu::SurfaceError::Lost) => {
                log::info!("Swapchain lost");
                self.resize(self.window.inner_size())
            }
            Err(e) => {
                log::error!("Error: {e}")
            }
        };
        while let Some(event) = self.chunk_queue.lock().unwrap().pop_front() {
            match event {
                ChunkMeshEvent::Update { position, mut mesh } => {
                    mesh.update_buffers(&self.device, &self.queue);
                    self.chunk_meshes.insert(position, mesh)
                }
                ChunkMeshEvent::Unload(position) => {
                    self.chunk_meshes.remove(&position)
                }
            };
        }
        Ok(())
    }

    fn run_mesh_thread(
        render_world: RenderWorld,
        chunk_receiver: Receiver<ChunkEvent>,
    ) {
        let mut render_world = render_world;
        let mut chunk_receiver = chunk_receiver;
        while let Ok(event) = chunk_receiver.recv() {
            render_world.handle_chunk_event(event);
        }
    }
}

pub fn run(
    render_data: GameRenderData,
    chunk_receiver: Receiver<ChunkEvent>,
    world: Arc<Mutex<World>>,
) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;
    match SETTINGS.graphics.window {
        crate::settings::WindowMode::Default { width, height } => {
            window.request_inner_size(winit::dpi::PhysicalSize {
                width,
                height,
            });
        }
        crate::settings::WindowMode::Maximized => {
            window.set_maximized(true);
        }
        crate::settings::WindowMode::Fullscreen => {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(
                window.current_monitor(),
            )))
        }
    }
    let (mut renderer, render_world) =
        Renderer::new(&window, render_data, world)?;
    window.set_cursor_visible(false);

    thread::scope(move |s| {
        s.spawn(move || {
            Renderer::run_mesh_thread(render_world, chunk_receiver)
        });
        event_loop.run(move |event, control_flow| match event {
        winit::event::Event::WindowEvent {
            window_id,
            ref event,
        } => {
            if window_id == renderer.window.id() && !renderer.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size)
                    }
                    WindowEvent::ScaleFactorChanged {
                        ..
                    } => renderer.resize(renderer.window.inner_size()),
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.update();
                        renderer.render().unwrap();
                        renderer.window.request_redraw();
                    }
                    _ => {}
                }
            }
        }
        winit::event::Event::DeviceEvent {
            event: winit::event::DeviceEvent::MouseMotion { delta: (x, y) },
            ..
        } => renderer.mouse_event(glm::vec2(x as f32, y as f32)).unwrap(),
        _ => {}
    })
    })?;
    Ok(())
}
