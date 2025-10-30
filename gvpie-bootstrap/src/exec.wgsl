struct Regs {
    // Core command queue state
    head : u32,
    tail : u32,
    cap : u32,
    err_op : u32,   // REPURPOSE: cppm_cost (total cost this frame)

    // Error and performance state
    err_code : u32, // REPURPOSE: cppm_pixels (pixels modified)
    err_addr : u32, // REPURPOSE: cppm_instrs (instructions executed)
    frame : u32,    // Keep as frame counter
    _pad : u32,     // REPURPOSE: cppm_budget (0 = unlimited)
};

@group(0) @binding(0) var<storage, read_write> regs : Regs;
@group(0) @binding(1) var<storage, read> cmdq : array<u32>;
@group(0) @binding(2) var workTex : texture_storage_2d<rgba8uint, read_write>;
@group(0) @binding(3) var viewTex : texture_storage_2d<rgba8unorm, write>;

fn u8_at(w : u32, byte_index : u32) -> u32 {
    return (w >> (8u * byte_index)) & 0xFFu;
}

// Calculate computational cost for an instruction
fn cppm_calc_cost(op: u32, a: u32, b: u32, c: u32, d: u32) -> u32 {
    switch op {
        case 0x0u: { return 1u; }                    // PLOT - single pixel
        case 0x1u: { return c * d + 2u; }            // RECT - area + setup
        case 0x2u: { return c * d * 2u + 5u; }       // BLIT - area * 2 + copy overhead
        case 0xFu: { return 0u; }                    // SYNC - free
        default: { return 1u; }                      // Unknown - minimal cost
    }
}

fn write_pixel(coord : vec2<i32>, val_byte : u32) {
    textureStore(workTex, coord, vec4<u32>(val_byte, regs.frame, 0u, 255u));

    let val_norm = f32(val_byte) / 255.0;
    textureStore(viewTex, coord, vec4<f32>(val_norm, val_norm, val_norm, 1.0));
}

@compute @workgroup_size(1)
fn exec() {
    var steps : u32 = 0u;
    let maxSteps : u32 = 4096u;
    let cap_mask : u32 = regs.cap - 1u;

    loop {
        // Check tail/head and max steps
        if (regs.tail == regs.head || steps >= maxSteps) {
            break;
        }

        // CPPM throttling check (only if budget > 0)
        if (regs._pad > 0u && regs.err_op > regs._pad) {
            break; // Stop execution if we're over budget
        }

        let idx_cmd = (regs.tail & cap_mask) * 4u;
        let w0 = cmdq[idx_cmd + 0u];
        let w1 = cmdq[idx_cmd + 1u];
        let w2 = cmdq[idx_cmd + 2u];
        let w3 = cmdq[idx_cmd + 3u];
        let op_byte = u8_at(w0, 0u);
        let op = op_byte & 0x0Fu;
        let a : u32 = (w0 >> 16u) & 0xFFFFu;
        let b : u32 = w1 & 0xFFFFu;
        let c : u32 = (w1 >> 16u) & 0xFFFFu;
        let d : u32 = w2 & 0xFFFFu;
        let v0 : u32 = (w2 >> 16u) & 0xFFFFu;
        let v1 : u32 = w3 & 0xFFFFu;
        let val_byte = v0 & 0xFFu;

        switch (op) {
            case 0x0u: {
                // CPPM monitoring
                let cost = cppm_calc_cost(op, a, b, c, d);
                atomicAdd(&regs.err_op, cost);       // cppm_cost
                atomicAdd(&regs.err_code, 1u);      // cppm_pixels
                atomicAdd(&regs.err_addr, 1u);      // cppm_instrs

                // Original PLOT logic
                let coord = vec2<i32>(i32(a), i32(b));
                write_pixel(coord, val_byte);
            }
            case 0x1u: {
                // CPPM monitoring
                let cost = cppm_calc_cost(op, a, b, c, d);
                atomicAdd(&regs.err_op, cost);       // cppm_cost
                atomicAdd(&regs.err_code, c * d);  // cppm_pixels
                atomicAdd(&regs.err_addr, 1u);     // cppm_instrs

                // Original RECT logic
                let x_start = i32(a);
                let y_start = i32(b);
                let w = i32(c);
                let h = i32(d);
                for (var yy : i32 = 0; yy < h; yy = yy + 1) {
                    for (var xx : i32 = 0; xx < w; xx = xx + 1) {
                        let coord = vec2<i32>(x_start + xx, y_start + yy);
                        write_pixel(coord, val_byte);
                    }
                }
            }
            case 0x2u: { // BLIT: Copy block from (a, b) to (c, d) with size (v0, v1)
                // CPPM monitoring
                let cost = cppm_calc_cost(op, a, b, c, d);
                atomicAdd(&regs.err_op, cost);       // cppm_cost
                atomicAdd(&regs.err_code, v0 * v1);  // cppm_pixels
                atomicAdd(&regs.err_addr, 1u);     // cppm_instrs

                // Original BLIT logic
                let src_x = i32(a);
                let src_y = i32(b);
                let dst_x = i32(c);
                let dst_y = i32(d);
                let w = i32(v0);
                let h = i32(v1);

                // Copy loop: Sequential read and write (correct for workgroup_size(1))
                for (var yy:i32 = 0; yy < h; yy++) {
                    for (var xx:i32 = 0; xx < w; xx++) {

                        let src_coord = vec2<i32>(src_x + xx, src_y + yy);
                        let dst_coord = vec2<i32>(dst_x + xx, dst_y + yy);

                        // 1. Read source texel data (includes R channel byte value and G channel frame stamp)
                        let texel = textureLoad(workTex, src_coord);

                        // 2. Write data to destination
                        textureStore(workTex, dst_coord, texel);
                    }
                }
            }
            case 0xFu: {
                // CPPM monitoring
                let cost = cppm_calc_cost(op, a, b, c, d);
                atomicAdd(&regs.err_op, cost);       // cppm_cost
                atomicAdd(&regs.err_addr, 1u);     // cppm_instrs
                // No pixels modified

                // Original SYNC logic
                regs.frame = regs.frame + 1u;

                // Reset CPPM counters for new frame (important!)
                let current_budget = regs._pad;     // Preserve budget
                atomicStore(&regs.err_op, 0u);     // Reset cost
                atomicStore(&regs.err_code, 0u);   // Reset pixels
                atomicStore(&regs.err_addr, 0u);   // Reset instructions
                atomicStore(&regs._pad, current_budget); // Keep budget
            }
            default: {
                // CPPM monitoring (unknown instruction)
                let cost = cppm_calc_cost(op, a, b, c, d);
                atomicAdd(&regs.err_op, cost);       // cppm_cost
                atomicAdd(&regs.err_addr, 1u);     // cppm_instrs

                if (regs.err_op == 0u) {
                    regs.err_op = op;
                }
                break;
            }
        }

        regs.tail = regs.tail + 1u;
        steps = steps + 1u;
    }
}
