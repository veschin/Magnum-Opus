#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct PostProcessParams {
    outline_color: vec4<f32>,
    outline_threshold: f32,
    posterize_levels: f32,
    outline_enabled: f32,
    _pad: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: PostProcessParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var source_sampler: sampler;

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

fn posterize(c: vec3<f32>, levels: f32) -> vec3<f32> {
    if (levels < 2.0) {
        return c;
    }
    return floor(c * levels + 0.5) / levels;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(source_tex));
    let texel = vec2<f32>(1.0 / dims.x, 1.0 / dims.y);
    let uv = in.uv;

    let src = textureSample(source_tex, source_sampler, uv);

    // Roberts cross 2×2 - samples two diagonal pairs around the current
    // pixel. Produces a 1-texel-thick outline (where 3×3 Sobel spills into
    // 2-3 texels on either side of the boundary), which nearest-neighbour
    // preserves as a 1-scale-factor line on screen.
    var edge: f32 = 0.0;
    if (params.outline_enabled > 0.5) {
        let c00 = luminance(textureSample(source_tex, source_sampler, uv).rgb);
        let c10 = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(texel.x, 0.0)).rgb);
        let c01 = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(0.0, texel.y)).rgb);
        let c11 = luminance(textureSample(source_tex, source_sampler, uv + vec2<f32>(texel.x, texel.y)).rgb);

        let g1 = c00 - c11;
        let g2 = c10 - c01;
        edge = sqrt(g1 * g1 + g2 * g2);
    }

    if (edge > params.outline_threshold) {
        return params.outline_color;
    }

    let posterized = posterize(src.rgb, params.posterize_levels);
    return vec4<f32>(posterized, src.a);
}
