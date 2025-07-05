#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> time: f32;
@group(2) @binding(1) var<uniform> mouse_pos: vec2<f32>;
@group(2) @binding(2) var<uniform> instability_heat: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple test - purple gradient with time animation
    let uv = in.uv;
    
    // Animated gradient
    let color = vec3<f32>(
        0.1 + sin(time) * 0.05,
        0.05,
        0.2 + cos(time * 0.7) * 0.1
    );
    
    // Add UV gradient
    let gradient = (uv.x + uv.y) * 0.5;
    
    return vec4<f32>(color + gradient * 0.1, 1.0);
}