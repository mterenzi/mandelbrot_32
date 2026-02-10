#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3], // [x, y, z]
}

// Counterclockwise winding order for front face
pub const QUAD_VERTICES: &[Vertex] = &[
    // --- TRIANGLE 1 ---
    Vertex {
        position: [-1.0, 1.0, 0.0],
    }, // Top Left
    Vertex {
        position: [-1.0, -1.0, 0.0],
    }, // Bottom Left
    Vertex {
        position: [1.0, -1.0, 0.0],
    }, // Bottom Right
    // --- TRIANGLE 2 ---
    Vertex {
        position: [-1.0, 1.0, 0.0],
    }, // Top Left
    Vertex {
        position: [1.0, -1.0, 0.0],
    }, // Bottom Right
    Vertex {
        position: [1.0, 1.0, 0.0],
    }, // Top Right
];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // (Should be 12 bytes: 3 floats)
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Attribute 0: Position
                wgpu::VertexAttribute {
                    offset: 0,                             // Start at byte 0
                    shader_location: 0,                    // Send to @location(0) in shader
                    format: wgpu::VertexFormat::Float32x3, // It's 3 floats (x, y, z)
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub center: [f32; 2], // Offset 0  (8 bytes)
    pub zoom: f32,        // Offset 8  (4 bytes)
    pub aspect: f32,      // Offset 12 (4 bytes)
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            center: [0.0, 0.0], // Start at the center of the universe
            zoom: 1.0,          // Default zoom
            aspect: 1.0,        // Default square aspect
        }
    }
}
