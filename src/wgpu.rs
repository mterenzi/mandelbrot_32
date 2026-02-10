use std::sync::Arc;

use wgpu::{BindingType, BufferBindingType, ShaderStages, util::DeviceExt};
use winit::{dpi::PhysicalSize, window::Window};

use crate::primitives::{QUAD_VERTICES, Uniforms, Vertex};

pub struct WgpuState<'a> {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    pub uniform_data: UniformData,
}

pub struct UniformData {
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub center: [f32; 2],
    pub zoom: f32,
}

impl WgpuState<'_> {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        println!("Created WGPU instance: {:?}", instance);

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        println!("Created surface: {:?}", surface);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");
        println!("Found adapter: {:?}", adapter);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await
            .expect("Failed to create device");
        println!("Device and Queue created successfully!");

        // Select the first format that supports sRGB, or fallback to the first available
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0], // usually Fifo (Vsync)
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // IMPORTANT: This creates the swap chain. Without this, nothing renders.
        surface.configure(&device, &config);
        println!("Surface configured.");

        let bind_group_layout = Self::create_bind_group_layout(&device);
        println!("Bind group layout created.");

        let uniform_data = Self::create_uniform_data(&device, &bind_group_layout);
        println!("Uniform data created.");

        let render_pipeline = Self::create_render_pipeline(&device, &config, &bind_group_layout);
        println!("Render pipeline created.");

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        println!("Vertex buffer created.");

        println!("WGPU setup complete.");

        WgpuState {
            instance,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            uniform_data,
        }
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,                                                // Slot 0
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT, // Accessible in both vertex and fragment shaders
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn create_uniform_data(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> UniformData {
        let uniforms = Uniforms::new();

        // usage: UNIFORM (for shader) | COPY_DST (so we can update it every frame)
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Connects the 'uniform_buffer' to 'binding: 0'
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        UniformData {
            uniforms,
            uniform_buffer,
            bind_group,
            center: [0.0, 0.0],
            zoom: 1.0,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mandelbrot_32.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),

            // A. Vertex Stage (The Architect)
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // The function name in WGSL
                buffers: &[Vertex::desc()], // We are hardcoding vertices in the shader, so no buffers needed yet
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            // B. Fragment Stage (The Painter)
            // It's wrapped in Option because some pipelines (like depth-only) don't have color.
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"), // The function name in WGSL
                targets: &[Some(wgpu::ColorTargetState {
                    // IMPORTANT: This must match the surface configuration!
                    format: config.format,
                    // Replace pixels, don't blend them (for now)
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            // C. Primitive Topology (How to interpret the points)
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 3 vertices = 1 triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Counter-Clockwise is the standard "front"
                cull_mode: Some(wgpu::Face::Back), // Don't draw the back of the triangle
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            depth_stencil: None, // We aren't checking depth (z-buffer) yet
            multisample: wgpu::MultisampleState {
                count: 1, // No anti-aliasing (MSAA) yet
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        render_pipeline
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            // We must reconfigure the surface every time the window size changes
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_data.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.draw(0..QUAD_VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {
        let aspect = self.config.width as f32 / self.config.height as f32;

        // Update the Uniforms struct with our current state
        self.uniform_data.uniforms.center = self.uniform_data.center;
        self.uniform_data.uniforms.zoom = self.uniform_data.zoom;
        self.uniform_data.uniforms.aspect = aspect;

        // Write the new data to the GPU
        self.queue.write_buffer(
            &self.uniform_data.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_data.uniforms]),
        );
    }
}
