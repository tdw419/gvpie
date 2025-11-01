@group(0) @binding(0) var input_texture: texture_2d<u32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

fn random(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn energy_function(p: vec2<i32>) -> f32 {
    if (p.x % 2 == 0 && p.y % 2 == 0) {
        return 1.0;
    }
    if (p.x % 2 == 1 && p.y % 2 == 1) {
        return 1.0;
    }
    return 0.0;
}

fn gibbs_sample(p: vec2<i32>, seed: f32) -> f32 {
    let e = energy_function(p);
    let r = random(vec2<f32>(f32(p.x) + seed, f32(p.y) + seed));
    if (r < e) {
        return 1.0;
    }
    return 0.0;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let p = vec2<i32>(i32(id.x), i32(id.y));
    let seed = f32(textureLoad(input_texture, p, 0).r);
    let sample = gibbs_sample(p, seed);
    let color = vec4<f32>(sample, sample, sample, 1.0);
    textureStore(output_texture, p, color);
}
