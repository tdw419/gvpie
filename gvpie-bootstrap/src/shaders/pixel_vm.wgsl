// Pixel VM WGSL Kernel

@group(0) @binding(0) var<storage, read_write> canvas: array<vec4<u32>>;

const OP_SUB: u32 = 0x11u;
const OP_MUL: u32 = 0x12u;

fn bounds_check_pixel(addr: u32) -> bool {
    return addr < arrayLength(&canvas);
}

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    canvas[0] = vec4<u32>(255u, 0u, 255u, 255u); // Write magenta to the first pixel
    let ip = global_id.x;
    let instruction = canvas[ip];
    let opcode = instruction.r;

    switch (opcode) {
        case OP_SUB: {
            let src1 = instruction.g;
            let src2 = instruction.b;
            let dest = instruction.a;
            if (bounds_check_pixel(src1) && bounds_check_pixel(src2) && bounds_check_pixel(dest)) {
                let val1 = canvas[src1].r;
                let val2 = canvas[src2].r;
                let result = val1 - min(val1, val2); // Saturating subtract
                canvas[dest] = vec4<u32>(result, result, result, 255u);
            }
        }
        case OP_MUL: {
            let src1 = instruction.g;
            let src2 = instruction.b;
            let dest = instruction.a;
            if (bounds_check_pixel(src1) && bounds_check_pixel(src2) && bounds_check_pixel(dest)) {
                let val1 = canvas[src1].r;
                let val2 = canvas[src2].r;
                let result = min(val1 * val2, 255u); // Saturating multiply
                canvas[dest] = vec4<u32>(result, result, result, 255u);
            }
        }
        default: {
            // No-op for unknown opcodes
        }
    }
}
