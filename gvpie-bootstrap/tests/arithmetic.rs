use gvpie_bootstrap::pixel_language::ops::PixelOp;
use gvpie_bootstrap::pixel_language::executor::{PixelInstruction, PixelMachine};
use gvpie_bootstrap::pixel_language::assembler::PixelAssembler;


#[test]
fn test_assembler_arithmetic_execution() {
    let assembler = PixelAssembler::new(10, 10);
    let program = assembler.assemble_from_text("
        SUB 0 1 2
        MUL 3 4 5
    ");

    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 10, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 4, g: 0, b: 0, a: 0 };
    machine.canvas[3] = PixelInstruction { r: 5, g: 0, b: 0, a: 0 };
    machine.canvas[4] = PixelInstruction { r: 10, g: 0, b: 0, a: 0 };

    machine.execute_instruction(&program[0]);
    machine.execute_instruction(&program[1]);

    assert_eq!(machine.canvas[2].r, 6);
    assert_eq!(machine.canvas[5].r, 50);
}

#[test]
fn test_sub_operation() {
    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 10, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 4, g: 0, b: 0, a: 0 };

    let instruction = PixelInstruction {
        r: PixelOp::Sub as u8,
        g: 0, // src1
        b: 1, // src2
        a: 2, // dest
    };
    machine.execute_instruction(&instruction);

    assert_eq!(machine.canvas[2].r, 6);
}

#[test]
fn test_sub_underflow() {
    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 4, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 10, g: 0, b: 0, a: 0 };

    let instruction = PixelInstruction {
        r: PixelOp::Sub as u8,
        g: 0, // src1
        b: 1, // src2
        a: 2, // dest
    };
    machine.execute_instruction(&instruction);

    assert_eq!(machine.canvas[2].r, 0);
}

#[test]
fn test_mul_operation() {
    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 5, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 10, g: 0, b: 0, a: 0 };

    let instruction = PixelInstruction {
        r: PixelOp::Mul as u8,
        g: 0, // src1
        b: 1, // src2
        a: 2, // dest
    };
    machine.execute_instruction(&instruction);

    assert_eq!(machine.canvas[2].r, 50);
}

#[test]
fn test_mul_overflow() {
    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 20, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 20, g: 0, b: 0, a: 0 };

    let instruction = PixelInstruction {
        r: PixelOp::Mul as u8,
        g: 0, // src1
        b: 1, // src2
        a: 2, // dest
    };
    machine.execute_instruction(&instruction);

    assert_eq!(machine.canvas[2].r, 255);
}

#[test]
fn test_mul_by_zero() {
    let mut machine = PixelMachine::new(10, 10);
    machine.canvas[0] = PixelInstruction { r: 100, g: 0, b: 0, a: 0 };
    machine.canvas[1] = PixelInstruction { r: 0, g: 0, b: 0, a: 0 };

    let instruction = PixelInstruction {
        r: PixelOp::Mul as u8,
        g: 0, // src1
        b: 1, // src2
        a: 2, // dest
    };
    machine.execute_instruction(&instruction);

    assert_eq!(machine.canvas[2].r, 0);
}
