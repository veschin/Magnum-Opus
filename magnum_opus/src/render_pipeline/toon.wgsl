#import bevy_pbr::forward_io::VertexOutput

struct ToonParams {
    base_color: vec4<f32>,
    ambient: vec4<f32>,
    sun_dir: vec3<f32>,
    bands: u32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: ToonParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let l = normalize(-params.sun_dir);
    let n_dot_l = max(dot(n, l), 0.0);

    let bands = max(f32(params.bands), 1.0);
    let banded = floor(n_dot_l * bands + 0.5) / bands;

    let ambient = params.ambient.rgb;
    let lit = ambient + banded * (vec3<f32>(1.0) - ambient);

    return vec4<f32>(params.base_color.rgb * lit, 1.0);
}
