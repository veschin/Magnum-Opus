#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var normal_texture: texture_2d<f32>;
@group(0) @binding(3) var nearest_sampler: sampler;

// --- Tuning variables ---
// 0 = depth silhouettes only, 1 = depth + normal (all edges)
const OUTLINE_MODE: u32 = 1u;
// Blending: 0.0 = invisible, 1.0 = fully opaque
const OUTLINE_OPACITY: f32 = 0.5;
// Silhouette outlines: darken the base color by this factor
const SILHOUETTE_DARKEN: f32 = 0.3;
// Convex edge highlights: lighten the base color by this factor
const EDGE_BRIGHTEN: f32 = 1.6;

const DEPTH_THRESHOLD: f32 = 0.0008;
const NORMAL_THRESHOLD: f32 = 0.2;

fn decode_normal(coord: vec2<i32>) -> vec3<f32> {
    return normalize(textureLoad(normal_texture, coord, 0).xyz * 2.0 - 1.0);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let coord = vec2<i32>(in.position.xy);
    let color = textureLoad(screen_texture, coord, 0);

    let dc = textureLoad(depth_texture, coord, 0);
    let du = textureLoad(depth_texture, coord + vec2(0, -1), 0);
    let dd = textureLoad(depth_texture, coord + vec2(0, 1), 0);
    let dl = textureLoad(depth_texture, coord + vec2(-1, 0), 0);
    let dr = textureLoad(depth_texture, coord + vec2(1, 0), 0);
    let depth_edge = max(max(abs(dc - du), abs(dc - dd)), max(abs(dc - dl), abs(dc - dr)));

    let is_silhouette = depth_edge > DEPTH_THRESHOLD;

    var is_convex_edge = false;
    if OUTLINE_MODE == 1u {
        let nc = decode_normal(coord);
        let nu = decode_normal(coord + vec2(0, -1));
        let nd = decode_normal(coord + vec2(0, 1));
        let nl = decode_normal(coord + vec2(-1, 0));
        let nr = decode_normal(coord + vec2(1, 0));
        let normal_edge = max(
            max(1.0 - dot(nc, nu), 1.0 - dot(nc, nd)),
            max(1.0 - dot(nc, nl), 1.0 - dot(nc, nr))
        );
        is_convex_edge = normal_edge > NORMAL_THRESHOLD;
    }

    if is_silhouette {
        let outline = color.rgb * SILHOUETTE_DARKEN;
        return vec4<f32>(mix(color.rgb, outline, OUTLINE_OPACITY), 1.0);
    }

    if is_convex_edge {
        let highlight = min(color.rgb * EDGE_BRIGHTEN, vec3(1.0));
        return vec4<f32>(mix(color.rgb, highlight, OUTLINE_OPACITY), 1.0);
    }

    return color;
}
