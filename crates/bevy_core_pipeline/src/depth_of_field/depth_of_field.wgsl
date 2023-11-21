// Depth of field shader

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var texture_sampler: sampler;

struct DepthOfFieldUniforms {
    focal_length: f32,
    aperture_diameter: f32,
    focus_distance: f32
}
@group(0) @binding(3) var<uniform> settings: DepthOfFieldUniforms;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Output color
    let col = textureSample(screen_texture, texture_sampler, in.uv);
    let depth = textureSample(depth_texture, texture_sampler, in.uv);
    // // Compute blur factor based on the CoC size scaled and
    // // normalized to the [0..1] range
    // let pixCoC = abs(settings.aperture_diameter * settings.focal_length * (settings.focus_distance - depth) /
    // (settings.focus_distance * (depth - settings.focal_length)));
    // Depth/blurriness value scaled to the [0..1] range
    // let blur = saturate(pixCoC * scale / maxCoC);
    return vec4<f32>(vec3<f32>(depth), 1.0);
}

