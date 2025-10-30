#import "contract.wgsl"

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_coord: vec2<f32>,
}

@group(BINDING_GROUP) @binding(BINDING_STATE)
var<storage, read> state: EditorState;

@group(BINDING_GROUP) @binding(BINDING_UNIFORMS)
var<uniform> uniforms: RenderUniforms;

@group(BINDING_GROUP) @binding(BINDING_FONT_TEXTURE)
var font_texture: texture_2d<f32>;

@group(BINDING_GROUP) @binding(BINDING_FONT_SAMPLER)
var font_sampler: sampler;

fn march_terrain(ray_origin: vec3<f32>, ray_dir: vec3<f32>, time: f32) -> vec4<f32> {
    var t: f32 = 0.0;
    for (var i: u32 = 0u; i < 80u; i = i + 1u) {
        let pos = ray_origin + ray_dir * t;
        let terrain =
            sin(pos.x * 0.2) * cos(pos.z * 0.2) * 3.0 +
            sin(pos.x * 0.5) * cos(pos.z * 0.5) * 1.0 +
            sin(pos.x * 1.0 + time * 0.2) * cos(pos.z * 1.0 - time * 0.2) * 0.5;
        let dist = pos.y - terrain;

        if (abs(dist) < 0.05) {
            let eps = 0.01;
            let normal = normalize(vec3<f32>(
                terrain - (
                    sin((pos.x - eps) * 0.2) * cos(pos.z * 0.2) * 3.0 +
                    sin((pos.x - eps) * 0.5) * cos(pos.z * 0.5) * 1.0 +
                    sin((pos.x - eps) * 1.0 + time * 0.2) * cos(pos.z * 1.0 - time * 0.2) * 0.5
                ),
                1.0,
                terrain - (
                    sin(pos.x * 0.2) * cos((pos.z - eps) * 0.2) * 3.0 +
                    sin(pos.x * 0.5) * cos((pos.z - eps) * 0.5) * 1.0 +
                    sin(pos.x * 1.0 + time * 0.2) * cos((pos.z - eps) * 1.0 - time * 0.2) * 0.5
                )
            ));

            let light_dir = normalize(vec3<f32>(0.8, 0.6, 0.4));
            let diffuse = clamp(dot(normal, light_dir), 0.3, 1.0);
            let specular = pow(max(0.0, dot(reflect(-light_dir, normal), -ray_dir)), 8.0) * 0.5;

            let slope = 1.0 - normal.y;
            var base_color = vec3<f32>(0.3, 0.6, 0.3);
            if (slope > 0.35) {
                base_color = vec3<f32>(0.5, 0.5, 0.5);
            }
            if (terrain < -2.0) {
                base_color = vec3<f32>(0.2, 0.3, 0.8);
            }

            return vec4<f32>(base_color * (diffuse + specular), 1.0);
        }

        t = t + max(0.05, dist * 0.5);
        if (t > 120.0) {
            break;
        }
    }

    let sun_dir = normalize(vec3<f32>(sin(time * 0.1), 0.5 + sin(time * 0.05) * 0.3, cos(time * 0.1)));
    let sun_intensity = pow(max(0.0, dot(ray_dir, sun_dir)), 32.0);
    let base_sky = vec3<f32>(0.3, 0.45, 0.75);
    let horizon = vec3<f32>(0.9, 0.7, 0.5);
    let sky_t = clamp(0.5 * (ray_dir.y + 1.0), 0.0, 1.0);

    let sky_color = mix(horizon, base_sky, sky_t) + vec3<f32>(sun_intensity);
    return vec4<f32>(sky_color, 1.0);
}

fn render_gaussian_splats(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec4<f32> {
    if (!cards_initialized()) {
        return vec4<f32>(0.0);
    }

    let count = get_cards_count();
    if (count == 0u) {
        return vec4<f32>(0.0);
    }

    var accumulated = vec4<f32>(0.0);

    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let splat = read_splat(i);
        let radius = splat.scale * 1.2;
        let oc = ray_origin - splat.position;
        let a = dot(ray_dir, ray_dir);
        let b = 2.0 * dot(oc, ray_dir);
        let c = dot(oc, oc) - radius * radius;
        let disc = b * b - 4.0 * a * c;

        if (disc > 0.0) {
            let t = (-b - sqrt(disc)) / (2.0 * a);
            if (t > 0.0) {
                let hit = ray_origin + ray_dir * t;
                let intensity = evaluate_gaussian_splat(splat, hit);
                if (intensity > 0.001) {
                    let sample = vec4<f32>(splat.color * intensity, intensity);
                    accumulated = accumulated + vec4<f32>(
                        sample.rgb * (1.0 - accumulated.a),
                        sample.a * (1.0 - accumulated.a)
                    );

                    if (accumulated.a > 0.98) {
                        break;
                    }
                }
            }
        }
    }

    return accumulated;
}

fn render_3d_scene(coord: vec2<f32>, time: f32) -> vec4<f32> {
    let viewport = vec2<f32>(uniforms.viewport_width, uniforms.viewport_height);
    var camera = default_camera_3d();
    if (camera_is_initialised()) {
        camera = load_camera_3d();
    }
    let ray_origin = camera.position;
    let ray_dir = get_camera_ray(camera, coord, viewport);

    let terrain = march_terrain(ray_origin, ray_dir, time);
    let splats = render_gaussian_splats(ray_origin, ray_dir);

    return vec4<f32>(
        mix(terrain.rgb, splats.rgb, clamp(splats.a, 0.0, 1.0)),
        1.0
    );
}

fn render_text_layer(coord: vec2<f32>) -> vec4<f32> {
    let uv = coord / vec2<f32>(uniforms.viewport_width, uniforms.viewport_height);
    let glyph = textureSample(font_texture, font_sampler, uv);
    return vec4<f32>(glyph.rgb, glyph.a);
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vid], 0.0, 1.0);
    output.frag_coord = (positions[vid] * 0.5 + vec2<f32>(0.5)) *
        vec2<f32>(uniforms.viewport_width, uniforms.viewport_height);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (is_3d_mode(state.scroll_offset)) {
        return render_3d_scene(input.frag_coord, uniforms.time);
    }
    return render_text_layer(input.frag_coord);
}
