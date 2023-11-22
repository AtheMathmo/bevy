// Depth of field shader

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// Define here, for now
const TWOPI: f32 = 6.28318530718;
const znear: f32 = 0.1;
const zfar: f32 = 1000.0;

const NUM_RINGS: f32 = 3.0;
const RING_TAP_MULTIPLE: f32 = 8.0;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var texture_sampler: sampler;

struct DepthOfFieldUniforms {
    focal_length: f32,
    aperture_diameter: f32,
    focal_distance: f32
}
@group(0) @binding(3) var<uniform> settings: DepthOfFieldUniforms;

fn compute_coc(depth: f32) -> f32 {
    // Physically realistic circle of confusion, scaled by film height
    // Convert to linear depth
    let z = znear * zfar / (znear + depth * (zfar - znear));
    return abs(
        settings.aperture_diameter * settings.focal_length * (settings.focal_distance - z) /
        (z * (settings.focal_distance - settings.focal_length))
    ) / 0.024;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Center samples
    var col = textureSample(screen_texture, texture_sampler, in.uv);
    let depth = textureSample(depth_texture, texture_sampler, in.uv);
    
    let max_coc = 0.01;//settings.aperture_diameter * settings.focal_length / (settings.focal_distance - settings.focal_length);
    var coc = compute_coc(depth);
    coc = clamp(coc, 0.0, max_coc);

    var weight = coc;
    for (var ring_idx = NUM_RINGS; ring_idx >= 1.0; ring_idx -= 1.0) {
        let max_taps = RING_TAP_MULTIPLE * ring_idx;
        for (var tap_idx = 0.0; tap_idx < max_taps; tap_idx += 1.0) {
            // TODO: Divide by the screen resolution
            let ring_radius = ring_idx * coc / NUM_RINGS;
            let theta = TWOPI * tap_idx / max_taps;
            let pos = in.uv + vec2<f32>(ring_radius * cos(theta), ring_radius * sin(theta));
            // pos = (r cos t, r sin t), r = ring_idx * max_rad / NUM_RINGS, t = 2 * pi * tap_idx / RING_TAP_MULTIPLE  * ring_idx
            
            let r_depth = textureSample(depth_texture, texture_sampler, pos);
            let r_coc = clamp(compute_coc(r_depth), 0.0, max_coc);

            var r_weight = select(1.0, r_coc, r_depth > depth);
            r_weight = select(1.0, r_weight, coc > r_coc + 0.002);
            r_weight = saturate(r_weight);

            col += r_weight * textureSample(screen_texture, texture_sampler, pos);
            weight += r_weight;
        }
    }
    // return vec4<f32>(vec3<f32>(coc) / 0.01, 1.0);
    return col / weight;
}

// d = a/z + b => z = a / (d - b)
// d = 0 => z = far => a = -far * b
// d = 1 => z = near => near * (1 - b) = a

// near * (1 - b) + far * b = 0 => b = - near / (far - near)
// => a = far * near / (far - near)