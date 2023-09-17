mod camera;
mod camera_controller;
mod game_render_data;
mod mesh;
pub mod model;
mod shader;
mod texture;
pub mod texture_atlas;
mod world;

use std::path::PathBuf;

use camera::Camera;
use camera_controller::CameraController;
use color_eyre::{eyre::ContextCompat, Result};
use mesh::{Mesh, Vertex};
use shader::load_shader_module;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::world::Chunk;

pub use self::texture_atlas::TextureAtlas;
use self::world::mesh_chunk;
pub use game_render_data::GameRenderData;

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

    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.get_view_projection();
    }
}

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    depth_texture: Texture,
    world_mesh: Mesh,
    texture_atlas_image: Texture,
    texture_atlas_bind_group: wgpu::BindGroup,
}

impl Renderer {
    async fn new(window: Window, chunk: Chunk, render_data: GameRenderData) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .wrap_err("Failure requesting adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = load_shader_module(&device, "main")?;

        let mut world_mesh = mesh_chunk(&chunk, &render_data);
        world_mesh.update_buffers(&device, &queue);

        let mut camera = Camera::new(size.width as f32, size.height as f32, 0.1, 1000.0);
        let mut camera_controller = CameraController::new();
        camera_controller.update(&mut camera);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera uniform"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // let texture_atlas_image =
        //     render_data
        //         .texture_atlas()
        //         .texture(&device, &queue, Some("Block Atlas"));

        let texture_atlas_image = Texture::new(
            &device,
            &queue,
            Some("Block Atlas"),
            &PathBuf::from("assets/textures/blocks/cobblestone.png"),
        )?;

        let texture_atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture atlas bind group"),
            layout: &texture_atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_atlas_image.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture_atlas_image.sampler()),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_atlas_bind_group_layout],
                push_constant_ranges: &[],
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, Some("Depth Texture"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
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
        });

        Ok(Self {
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
            depth_texture,
            world_mesh,
            texture_atlas_image,
            texture_atlas_bind_group,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .set_size(new_size.width as f32, new_size.height as f32);
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
        self.camera_controller.update(&mut self.camera);
        self.camera_uniform.update_view_projection(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<()> {
        match self.surface.get_current_texture() {
            Ok(output) => {
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render encoder"),
                        });
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: self.depth_texture.view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });
                    render_pass.set_pipeline(&self.pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.texture_atlas_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.world_mesh.vertex_buffer()?.slice(..));
                    render_pass.set_index_buffer(
                        self.world_mesh.index_buffer()?.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..self.world_mesh.index_count(), 0, 0..1);
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
                self.resize(self.size)
            }
            Err(e) => {
                log::error!("Error: {e}")
            }
        };
        Ok(())
    }
}

pub fn run(chunk: Chunk, render_data: GameRenderData) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let mut renderer = pollster::block_on(Renderer::new(window, chunk, render_data))?;
    renderer.window.set_cursor_visible(false);

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(window_id) if window_id == renderer.window.id() => {
            renderer.update();
            renderer.render().unwrap();
        }
        winit::event::Event::MainEventsCleared => {
            renderer.window.request_redraw();
        }
        winit::event::Event::WindowEvent {
            window_id,
            ref event,
        } => {
            if window_id == renderer.window.id() && !renderer.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size)
                    }
                    winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        winit::event::Event::DeviceEvent {
            event: winit::event::DeviceEvent::MouseMotion { delta: (x, y) },
            ..
        } => renderer.mouse_event(glm::vec2(x as f32, y as f32)).unwrap(),
        _ => {}
    });
}
