#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct OutlineParams {
    threshold: f32,
    color: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: OutlineParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var source_sampler: sampler;

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(source_tex));
    let texel = vec2<f32>(1.0 / dims.x, 1.0 / dims.y);
    let uv = in.uv;

    let tl = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(-texel.x, -texel.y)).rgb);
    let tm = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>( 0.0,     -texel.y)).rgb);
    let tr = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>( texel.x, -texel.y)).rgb);
    let ml = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(-texel.x,  0.0)).rgb);
    let mr = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>( texel.x,  0.0)).rgb);
    let bl = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(-texel.x,  texel.y)).rgb);
    let bm = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>( 0.0,      texel.y)).rgb);
    let br = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>( texel.x,  texel.y)).rgb);

    let gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
    let gy = -tl - 2.0 * tm - tr + bl + 2.0 * bm + br;
    let edge = sqrt(gx * gx + gy * gy);

    let src = textureSample(source_tex, source_sampler, uv);
    if (edge > params.threshold) {
        return params.color;
    }
    return src;
}
