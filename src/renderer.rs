mod camera;
mod camera_controller;
mod game_render_data;
mod mesh;
pub mod model;
mod shader;
mod texture;
pub mod texture_atlas;
mod world;

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

use self::camera::{OrthographicCamera, PerspectiveCamera};
pub use self::texture_atlas::TextureAtlas;
use self::world::mesh_world;
use crate::settings::SETTINGS;
use crate::world::World;

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
) -> wgpu::RenderPipeline {
    let fragment_targets =
        [color_format.map(|format| wgpu::ColorTargetState {
            format: format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::all(),
        })];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: color_format.map(|_| wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
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

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: Texture,
    pipeline: wgpu::RenderPipeline,
    depth_camera: Box<dyn Camera>,
    depth_camera_uniform: CameraUniform,
    depth_camera_buffer: wgpu::Buffer,
    depth_camera_bind_group: wgpu::BindGroup,
    shadow_depth_texture: Texture,
    shadow_pipeline: wgpu::RenderPipeline,
    camera: Box<dyn Camera>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    world_meshes: Vec<Mesh>,
    texture_atlas_image: Texture,
    texture_atlas_bind_group: wgpu::BindGroup,
}

impl Renderer {
    async fn new(
        window: Window,
        world: World,
        render_data: GameRenderData,
    ) -> Result<Self> {
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

        let mut world_meshes = mesh_world(&world, &render_data);
        for mesh in world_meshes.iter_mut() {
            mesh.update_buffers(&device, &queue);
        }

        // let mut camera = Box::new(PerspectiveCamera::new(
        //     size.width as f32,
        //     size.height as f32,
        //     0.1,
        //     1000.0,
        // ));
        let mut camera =
            Box::new(OrthographicCamera::new(1000.0, 500.0, 500.0));
        let mut camera_controller = CameraController::new();
        let depth_position = glm::vec3(200.0, 350.0, 150.0);
        // camera_controller.set_position(glm::vec3(0.0, 100.0, 0.0));
        camera_controller.set_position(depth_position);
        camera_controller.set_rotation(na::UnitQuaternion::new_normalize(
            glm::quat_look_at_lh(&depth_position, &glm::vec3(0.0, 1.0, 0.0)),
        ));
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

        let mut depth_camera = Box::new(PerspectiveCamera::new(
            1.0, 1.0, 1.0,
            1000.0,
            // SETTINGS.graphics.shadow.shadow_distance as f32 * 2.0,
            // SETTINGS.graphics.shadow.shadow_distance as f32 * 2.0,
        ));
        let depth_position = glm::vec3(400.0, 700.0, 300.0);
        depth_camera.set_position(depth_position);
        depth_camera.set_rotation(na::UnitQuaternion::new_normalize(
            glm::quat_look_at(&-depth_position, &glm::vec3(0.0, 1.0, 0.0)),
        ));
        let mut depth_camera_uniform = CameraUniform::new();
        depth_camera_uniform.update_view_projection(depth_camera.as_ref());
        let depth_camera_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Depth Camera uniform"),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[depth_camera_uniform]),
            });

        let depth_camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Depth Camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: depth_camera_buffer.as_entire_binding(),
                }],
            });

        let light_uniform = LightUniform {
            direction: nalgebra::Unit::new_normalize(glm::vec3(0.4, 0.7, 0.3)),
            color: glm::vec3(1.0, 1.0, 1.0),
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

        let texture_atlas_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture atlas bind group layout"),
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
                layout: &texture_atlas_bind_group_layout,
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

        let shader = load_shader_module(&device, "main")?;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_atlas_bind_group_layout,
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
        );

        let shadow_depth_texture = Texture::create_depth_texture(
            &device,
            SETTINGS.graphics.shadow.shadow_map_resolution,
            SETTINGS.graphics.shadow.shadow_map_resolution,
            Some("Shadow Depth Texture"),
        );

        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow pipeline layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let shadow_pipeline = create_render_pipeline(
            &device,
            &shadow_pipeline_layout,
            None,
            Some(Texture::DEPTH_FORMAT),
            &[Vertex::desc()],
            &shader,
        );

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
            light_uniform,
            light_buffer,
            light_bind_group,
            depth_texture,
            world_meshes,
            texture_atlas_image,
            texture_atlas_bind_group,
            shadow_depth_texture,
            shadow_pipeline,
            depth_camera,
            depth_camera_bind_group,
            depth_camera_buffer,
            depth_camera_uniform,
        })
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
            // self.camera
            //     .set_size(new_size.width as f32, new_size.height as f32);
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
    }

    fn render(&mut self) -> Result<()> {
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
                        let mut shadow_pass = encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {
                                label: Some("Shadow pass"),
                                color_attachments: &[],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: self.shadow_depth_texture.view(),
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: true,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                            },
                        );
                        shadow_pass.set_pipeline(&self.shadow_pipeline);
                        shadow_pass.set_bind_group(
                            0,
                            &self.depth_camera_bind_group,
                            &[],
                        );
                        for mesh in &self.world_meshes {
                            shadow_pass.set_vertex_buffer(
                                0,
                                mesh.vertex_buffer()?.slice(..),
                            );
                            shadow_pass.set_index_buffer(
                                mesh.index_buffer()?.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            shadow_pass.draw_indexed(
                                0..mesh.index_count(),
                                0,
                                0..1,
                            );
                        }
                    }
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
                                            store: true,
                                        },
                                    },
                                )],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: self.depth_texture.view(),
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: true,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
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
                        for mesh in &self.world_meshes {
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
        Ok(())
    }
}

pub fn run(world: World, render_data: GameRenderData) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    match SETTINGS.graphics.window {
        crate::settings::WindowMode::Default { width, height } => {
            window.set_inner_size(winit::dpi::PhysicalSize { width, height });
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
    let mut renderer =
        pollster::block_on(Renderer::new(window, world, render_data))?;
    renderer.window.set_cursor_visible(false);
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(window_id)
            if window_id == renderer.window.id() =>
        {
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
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size)
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => renderer.resize(**new_inner_size),
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
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
    });
}
