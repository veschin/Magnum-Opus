#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> base_color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let l = normalize(vec3<f32>(0.4, 1.0, 0.3));
    let ndl = max(dot(n, l), 0.0);
    let band = 0.2 + step(0.25, ndl) * 0.4 + step(0.55, ndl) * 0.4;
    return vec4<f32>(base_color.rgb * band, 1.0);
}
