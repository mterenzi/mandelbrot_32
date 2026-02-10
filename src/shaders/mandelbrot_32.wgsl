struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};

struct Uniforms {
    center: vec2<f32>, // The X,Y coordinate we are looking at
    zoom: f32,
    aspect: f32,       // Screen width / height
};

// @group(0) matches set_bind_group(0, ...) in Rust
// @binding(0) matches the layout we created
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Pass the vertex position through to the fragment shader as a coordinate.
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.coord = model.position.xy;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Map Screen to Fractal Coordinates
    var uv = in.coord;

    // Fix aspect ratio
    uv.x *= uniforms.aspect;

    // Map to Complex Plane
    let c = uniforms.center + (uv * (1.0 / uniforms.zoom));

    // --- The Mandelbrot Loop ---
    // z = z^2 + c

    var z = vec2<f32>(0.0, 0.0);
    var iterations = 0.0;
    let max_iterations = 512.0;
    var escaped  = false;

    for (var i = 0.0; i < max_iterations; i += 1.0) {
        // Complex squaring: (x+yi)^2 = (x^2 - y^2) + (2xy)i
        let x = (z.x * z.x) - (z.y * z.y) + c.x;
        let y = (2.0 * z.x * z.y) + c.y;
        
        z = vec2<f32>(x, y);

        // Escape condition: If distance > 2.0 (so dist squared > 4.0)
        if (dot(z, z) > 4.0) {
            escaped = true;
            iterations = i;
            break;
        }
    }

    if (escaped) {
        // Color it based on how fast it escaped.
        
        // let t = iterations / max_iterations;
        // // A simple "Fire" palette (Red/Yellowish)
        // return vec4<f32>(t * 4.0, t * 2.0, t * 0.5, 1.0);

        // Rainbow palette using sine waves for smooth coloring
        let freq = 0.1; 
        let r = 0.5 + 0.5 * sin(freq * iterations + 0.0);
        let g = 0.5 + 0.5 * sin(freq * iterations + 2.09); // +120 degrees
        let b = 0.5 + 0.5 * sin(freq * iterations + 4.18); // +240 degrees

        return vec4<f32>(r, g, b, 1.0);
    } else {
        // Paint it Black.
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}