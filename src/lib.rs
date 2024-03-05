use bytemuck::{cast_slice, Pod, Zeroable};
use std::{collections::HashMap, hash::BuildHasherDefault, iter::once, mem::size_of};
use wgpu::{
    naga::ShaderStage,
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

// lib.rs
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x3];

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
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    indices_length: u32,
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
                shader: include_str!("shader.vert").into(),
                stage: ShaderStage::Vertex,
                defines: HashMap::with_hasher(BuildHasherDefault::default()),
            },
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Glsl {
                shader: include_str!("shader.frag").into(),
                stage: ShaderStage::Fragment,
                defines: HashMap::with_hasher(BuildHasherDefault::default()),
            },
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            // Define vertex pass
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
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
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // Define what to use for depth stenciling
            depth_stencil: None,
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
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            indices_length,
        }
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices_length, 0, 0..1);

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
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => context.resize(size.width, size.height),
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
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
                _ => {}
            },
            _ => {}
        })
        .unwrap()
}
