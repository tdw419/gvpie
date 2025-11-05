// Re-export PixelInstruction from parent module
pub use crate::PixelInstruction;

/// Execution error codes for pixel programs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExecutionErrorCode {
    Success = 0,
    InvalidOpcode = 1,
    OutOfBounds = 2,
    StackOverflow = 3,
    StackUnderflow = 4,
    DivisionByZero = 5,
    Timeout = 6,
    Unknown = 255,
}

impl ExecutionErrorCode {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionErrorCode::Success)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionErrorCode::Success => "success",
            ExecutionErrorCode::InvalidOpcode => "invalid_opcode",
            ExecutionErrorCode::OutOfBounds => "out_of_bounds",
            ExecutionErrorCode::StackOverflow => "stack_overflow",
            ExecutionErrorCode::StackUnderflow => "stack_underflow",
            ExecutionErrorCode::DivisionByZero => "division_by_zero",
            ExecutionErrorCode::Timeout => "timeout",
            ExecutionErrorCode::Unknown => "unknown",
        }
    }
}
