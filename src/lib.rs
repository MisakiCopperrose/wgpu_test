mod camera;
mod instance;
mod texture;

use bytemuck::{cast_slice, Pod, Zeroable};
use glam::{vec2, vec3, vec4, Quat, Vec2, Vec3};
use rand::Rng;
use std::{collections::HashMap, hash::BuildHasherDefault, iter::once, mem::size_of};
use wgpu::{
    naga::ShaderStage,
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use crate::{camera::Camera, instance::InstanceData};
// lib.rs
const VERTICES: &[Vertex] = &[
    Vertex {
        position: vec3(-0.0868241, 0.49240386, 0.0),
        tex_coords: vec2(0.4131759, 0.00759614),
    }, // A
    Vertex {
        position: vec3(-0.49513406, 0.06958647, 0.0),
        tex_coords: vec2(0.0048659444, 0.43041354),
    }, // B
    Vertex {
        position: vec3(-0.21918549, -0.44939706, 0.0),
        tex_coords: vec2(0.28081453, 0.949397),
    }, // C
    Vertex {
        position: vec3(0.35966998, -0.3473291, 0.0),
        tex_coords: vec2(0.85967, 0.84732914),
    }, // D
    Vertex {
        position: vec3(0.44147372, 0.2347359, 0.0),
        tex_coords: vec2(0.9414737, 0.2652641),
    }, // E
];

const CAMERA_SPEED: f32 = 5.0;
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, 0];
const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vec3 = vec3(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: Vec3,
    tex_coords: Vec2,
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Context<'a> {
    device: Device,
    queue: Queue,
    surface: Surface<'a>,
    depth_texture: texture::Texture,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    indices_length: u32,
    texture_bind_group: BindGroup,
    texture: texture::Texture,
    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    instances: Vec<instance::Instance>,
    instance_buffer: Buffer,
}

impl<'a> Context<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .unwrap();

        println!("info: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        println!("caps: {:?}", caps);

        let config = surface
            .get_default_config(
                &adapter,
                window.inner_size().width.max(1),
                window.inner_size().height.max(1),
            )
            .unwrap();

        surface.configure(&device, &config);

        println!("format: {:?}", format);

        let texture = texture::Texture::from_file(
            &device,
            &queue,
            "src/resources/textures/happy-tree.png",
            "happy-tree.png",
        )
        .unwrap();
        // Describes a set of resources and how they can be accessed by a shader.
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let camera = Camera {
            eye: vec3(0f32, 1f32, 2f32),
            target: Vec3::ZERO,
            up: vec3(0f32, 1f32, 0f32),
            aspect: config.width as f32 / config.height as f32,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        };

        let camera_matrix = camera.build_view_projection();

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: cast_slice(&[camera_matrix]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = vec3(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT;

                    let rotation = if position == vec3(0.0, 0.0, 0.0) {
                        Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), f32::to_radians(0.0))
                    } else {
                        Quat::from_axis_angle(position.normalize(), f32::to_radians(45.0))
                    };

                    let colour_value = rand::thread_rng().gen();
                    let colour = vec4(colour_value, colour_value, colour_value, 1.0);

                    instance::Instance {
                        position,
                        rotation,
                        colour,
                    }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(instance::Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let indices_length = INDICES.len() as u32;

        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Glsl {
                shader: include_str!("resources/shaders/shader.vert").into(),
                stage: ShaderStage::Vertex,
                defines: HashMap::with_hasher(BuildHasherDefault::default()),
            },
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Glsl {
                shader: include_str!("resources/shaders/shader.frag").into(),
                stage: ShaderStage::Fragment,
                defines: HashMap::with_hasher(BuildHasherDefault::default()),
            },
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            // Define vertex pass
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[Vertex::desc(), InstanceData::descriptor()],
            },
            // Define fragment pass
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Define how to handle meshes/topolgy
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // Define what to use for depth stenciling
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                // LESS means pixels will be drawn front to back
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            // MSAA
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            device,
            queue,
            surface,
            depth_texture,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            indices_length,
            texture_bind_group,
            texture,
            camera,
            camera_buffer,
            camera_bind_group,
            instances,
            instance_buffer,
        }
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera.input_move_camera(event, CAMERA_SPEED)
    }

    fn update(&mut self) {
        let view_proj = self.camera.build_view_projection();

        std::println!("View: {}", view_proj);

        self.queue
            .write_buffer(&self.camera_buffer, 0, cast_slice(&[view_proj]))
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        // Get surface texture
        let output = self.surface.get_current_texture()?;
        // Texture descriptor (metadata etc)
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        // Encoder builds the command buffers
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // Creates the clear pass (render pass == bucket o' drawing calls)
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices_length, 0, 0..self.instances.len() as _);
        // Release the mutable borrow of the render pass
        drop(render_pass);
        // Submit the clear pass
        self.queue.submit(once(encoder.finish()));
        // Show rendertarget on the surface
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut context = Context::new(&window).await;

    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent { event, .. } => {
                if !context.input(&event) {
                    match event {
                        WindowEvent::Resized(size) => context.resize(size.width, size.height),
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::RedrawRequested => {
                            std::println!("Update");

                            context.update();

                            match context.render() {
                                Ok(_) => {}
                                Err(SurfaceError::Lost) => {
                                    context.resize(context.config.width, context.config.height)
                                }
                                Err(SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        _ => {}
                    }
                } else {
                    window.request_redraw();
                }
            }
            _ => {}
        })
        .unwrap()
}
