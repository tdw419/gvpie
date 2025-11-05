// PPL Interpreter WGSL Kernel

@group(0) @binding(0) var<storage, read> analyzer_program: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read> source_program: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read_write> output_buffer: array<u32>;
@group(0) @binding(3) var<storage, read_write> registers: array<u32>;

const OP_LOAD_SOURCE: u32 = 0x01u;
const OP_WRITE_OUTPUT: u32 = 0x02u;
const OP_COUNT_OPCODE: u32 = 0x10u;
const OP_CALCULATE_COMPLEXITY: u32 = 0x11u;
const OP_HALT: u32 = 0xFFu;

fn bounds_check(addr: u32, len: u32) -> bool {
    return addr < len;
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    var ip: u32 = 0u;
    loop {
        if (!bounds_check(ip, arrayLength(&analyzer_program))) {
            break;
        }

        let instruction = analyzer_program[ip];
        let opcode = instruction.r;

        switch (opcode) {
            case OP_LOAD_SOURCE: {
                let reg_idx = instruction.g;
                let source_addr = instruction.b;
                if (bounds_check(reg_idx, arrayLength(&registers)) && bounds_check(source_addr, arrayLength(&source_program))) {
                    registers[reg_idx] = source_program[source_addr].r;
                }
            }
            case OP_WRITE_OUTPUT: {
                let reg_idx = instruction.g;
                let output_addr = instruction.b;
                if (bounds_check(reg_idx, arrayLength(&registers)) && bounds_check(output_addr, arrayLength(&output_buffer))) {
                    output_buffer[output_addr] = registers[reg_idx];
                }
            }
            case OP_COUNT_OPCODE: {
                let opcode_to_match = instruction.g;
                let reg_idx = instruction.b;
                var count: u32 = 0u;
                for (var i: u32 = 0u; i < arrayLength(&source_program); i = i + 1u) {
                    if (source_program[i].r == opcode_to_match) {
                        count = count + 1u;
                    }
                }
                if (bounds_check(reg_idx, arrayLength(&registers))) {
                    registers[reg_idx] = count;
                }
            }
            case OP_CALCULATE_COMPLEXITY: {
                let reg_idx = instruction.g;
                if (bounds_check(reg_idx, arrayLength(&registers))) {
                    registers[reg_idx] = arrayLength(&source_program);
                }
            }
            case OP_HALT: {
                break;
            }
            default: {
                // No-op for unknown opcodes
            }
        }
        ip = ip + 1u;
    }
}
