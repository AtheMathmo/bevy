// Depth of field shader

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// Define here, for now
const TWOPI: f32 = 6.28318530718;

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

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Center depth
    var col = textureSample(screen_texture, texture_sampler, in.uv);
    let depth = textureSample(depth_texture, texture_sampler, in.uv);
    // Compute blur factor based on the CoC size scaled and
    // normalized to the [0..1] range
    let pixCoC = abs(settings.aperture_diameter * settings.focal_length * (settings.focal_distance - depth) /
        (settings.focal_distance * (depth - settings.focal_length)));
    // Depth/blurriness value scaled to the [0..1] range
    let blur = saturate(pixCoC * 5.0);

    var weight = pixCoC;
    for (var ring_idx = NUM_RINGS; ring_idx >= 1.0; ring_idx -= 1.0) {
        let max_taps = RING_TAP_MULTIPLE * ring_idx;
        for (var tap_idx = 0.0; tap_idx < max_taps; tap_idx += 1.0) {
            // TODO: Divide by the screen resolution
            let ring_radius = ring_idx * pixCoC / NUM_RINGS;
            let theta = TWOPI * tap_idx / max_taps;
            let pos = in.uv + vec2<f32>(ring_radius * cos(theta), ring_radius * sin(theta));
            // pos = (r cos t, r sin t), r = ring_idx * max_rad / NUM_RINGS, t = 2 * pi * tap_idx / RING_TAP_MULTIPLE  * ring_idx
            
            let r_depth = textureSample(depth_texture, texture_sampler, pos);
            let r_pixCoC = abs(settings.aperture_diameter * settings.focal_length * (settings.focal_distance - r_depth) /
                (settings.focal_distance * (r_depth - settings.focal_length)));
            weight += r_pixCoC;

            col += r_pixCoC * textureSample(screen_texture, texture_sampler, pos);
        }
    }
    return col / weight;
}

