// Void shader - the pregnant nothingness before differentiation

#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> time: f32;
@group(2) @binding(1) var<uniform> mouse_pos: vec2<f32>;
@group(2) @binding(2) var<uniform> instability_heat: f32;

// Random function
fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// 2D noise function
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = rand(i);
    let b = rand(i + vec2<f32>(1.0, 0.0));
    let c = rand(i + vec2<f32>(0.0, 1.0));
    let d = rand(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// FBM (Fractal Brownian Motion)
fn fbm(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var value = 0.0;
    var amplitude = 0.5;
    
    for (var i = 0; i < 6; i++) {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get UV coordinates from the mesh
    let uv = in.uv;
    
    // Sub-black floor
    var base_color = vec3<f32>(0.0);
    
    // Quantum noise shimmer
    let noise_scale = 100.0;
    let noise_speed = vec3<f32>(0.01, 0.007, -0.013);
    let noise_offset = vec3<f32>(
        time * noise_speed.x,
        time * noise_speed.y,
        time * noise_speed.z
    );
    
    let shimmer1 = fbm(uv * noise_scale + noise_offset.xy);
    let shimmer2 = fbm(uv * noise_scale * 1.3 + noise_offset.yz);
    let shimmer3 = fbm(uv * noise_scale * 0.7 + noise_offset.xz);
    
    let shimmer = (shimmer1 + shimmer2 + shimmer3) / 3.0;
    
    // Deep blue-purple color for the shimmer
    let shimmer_color = vec3<f32>(0.05, 0.02, 0.1) * shimmer * 0.05;
    
    // Cosmic specks (vacuum fluctuations)
    let speck_threshold = 0.9999;
    let speck_value = step(speck_threshold, rand(uv + vec2<f32>(time * 0.1)));
    let speck_flicker = sin(time * 10.0 + rand(floor(uv * 1000.0)) * 6.28) * 0.5 + 0.5;
    let specks = speck_value * speck_flicker * 0.5;
    
    // Lens-to-black vignette
    let center_dist = length(uv - vec2<f32>(0.5));
    let vignette = 1.0 - smoothstep(0.3, 0.8, center_dist);
    let over_vignette = 1.0 - smoothstep(0.5, 1.0, center_dist * 1.2);
    
    // Mouse interaction - local gamma lift
    let mouse_dist = length(uv - mouse_pos);
    let mouse_influence = smoothstep(0.2, 0.0, mouse_dist) * 0.01;
    
    // Combine all layers
    var final_color = base_color + shimmer_color;
    final_color += vec3<f32>(specks);
    final_color *= vignette;
    final_color *= over_vignette;
    final_color += mouse_influence * instability_heat;
    
    // HDR sub-black trick (will be tone-mapped)
    final_color -= 0.02 * (1.0 - vignette);
    
    return vec4<f32>(final_color, 1.0);
}