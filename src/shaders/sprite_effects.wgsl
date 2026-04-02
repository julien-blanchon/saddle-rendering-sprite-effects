#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SpriteEffectsUniform {
    base_color: vec4<f32>,
    flash_color: vec4<f32>,
    edge_color: vec4<f32>,
    uv_rect: vec4<f32>,
    flash: vec4<f32>,
    dissolve: vec4<f32>,
    dissolve_aux: vec4<f32>,
    palette: vec4<f32>,
    flags: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: SpriteEffectsUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var source_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var palette_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var palette_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var mask_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var mask_sampler: sampler;

fn effect_uv(uv: vec2<f32>) -> vec2<f32> {
    var x = uv.x;
    var y = uv.y;
    if material.flags.x > 0.5 {
        x = 1.0 - x;
    }
    if material.flags.y > 0.5 {
        y = 1.0 - y;
    }

    return vec2<f32>(x, y);
}

fn remap_uv(local_uv: vec2<f32>) -> vec2<f32> {
    let min_uv = material.uv_rect.xy;
    let max_uv = material.uv_rect.zw;
    let uv_rect = vec2<f32>(
        mix(min_uv.x, max_uv.x, local_uv.x),
        mix(min_uv.y, max_uv.y, local_uv.y),
    );
    let texel = 0.5 / vec2<f32>(textureDimensions(source_texture));
    return clamp(uv_rect, min_uv + texel, max_uv - texel);
}

fn hash21(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(
        dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3)),
    );
    return fract(sin(q.x + q.y) * 43758.5453);
}

fn palette_match(base_color: vec4<f32>) -> vec4<f32> {
    if material.flags.z < 0.5 {
        return base_color;
    }

    let palette_size = vec2<f32>(textureDimensions(palette_texture));
    let columns = u32(max(material.palette.z, 1.0));
    let source_row = material.palette.x;
    let target_row = material.palette.y;
    let epsilon = material.palette.w;

    var matched = false;
    var recolored = base_color;
    for (var i: u32 = 0u; i < 32u; i = i + 1u) {
        if i >= columns {
            break;
        }
        let u = (f32(i) + 0.5) / palette_size.x;
        let source_uv = vec2<f32>(u, (source_row + 0.5) / palette_size.y);
        let target_uv = vec2<f32>(u, (target_row + 0.5) / palette_size.y);
        let source_color = textureSampleLevel(palette_texture, palette_sampler, source_uv, 0.0);
        if distance(base_color.rgb, source_color.rgb) <= epsilon {
            recolored = textureSampleLevel(palette_texture, palette_sampler, target_uv, 0.0);
            matched = true;
            break;
        }
    }

    if matched {
        if material.flags.w > 0.5 {
            recolored.a = base_color.a;
        }
        return recolored;
    }
    return base_color;
}

fn dissolve_value(uv: vec2<f32>) -> f32 {
    let pattern = material.dissolve.z;
    if pattern < 0.5 {
        return hash21(uv * material.dissolve_aux.xy);
    }
    if pattern < 1.5 {
        return uv.x;
    }
    if pattern < 2.5 {
        return 1.0 - uv.x;
    }
    if pattern < 3.5 {
        return uv.y;
    }
    if pattern < 4.5 {
        return 1.0 - uv.y;
    }
    if pattern < 5.5 {
        let centered = uv * 2.0 - vec2<f32>(1.0, 1.0);
        return clamp(length(centered), 0.0, 1.0);
    }
    if pattern < 6.5 {
        let centered = uv * 2.0 - vec2<f32>(1.0, 1.0);
        return 1.0 - clamp(length(centered), 0.0, 1.0);
    }

    let sampled = textureSampleLevel(mask_texture, mask_sampler, uv, 0.0);
    return dot(sampled.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let local_uv = effect_uv(mesh.uv);
    let source_uv = remap_uv(local_uv);
    var color = textureSample(source_texture, source_sampler, source_uv);
    if color.a <= 0.001 {
        discard;
    }

    color = palette_match(color);
    color = color * material.base_color;

    let threshold = material.dissolve.x;
    if material.dissolve.w > 0.5 {
        let dissolve = dissolve_value(local_uv);
        if dissolve < threshold {
            discard;
        }

        let edge_width = max(material.dissolve.y, 0.0001);
        let edge_delta = dissolve - threshold;
        if edge_delta <= edge_width {
            let edge_mix = 1.0 - clamp(edge_delta / edge_width, 0.0, 1.0);
            color = vec4(
                mix(color.rgb, material.edge_color.rgb, edge_mix * material.edge_color.a),
                color.a,
            );
        }
    }

    if material.flash.z > 0.5 {
        let intensity = clamp(material.flash.x, 0.0, 1.0);
        if material.flash.y > 0.5 {
            color = vec4(
                color.rgb + material.flash_color.rgb * intensity * (1.0 - color.rgb),
                color.a,
            );
        } else {
            color = vec4(
                mix(color.rgb, material.flash_color.rgb, intensity),
                color.a,
            );
        }
    }

    return color;
}
