use crate::hybrid_canvas::{HybridCanvasBackend, TextRunOperation}; // Assuming these types exist

#[derive(Clone, Default)]
pub struct Uart16550 {
    thr_buf: Vec<u8>,
    next_line_y: f32,
}

impl Uart16550 {
    pub fn new() -> Self {
        Self {
            thr_buf: Vec::new(),
            next_line_y: 20.0, // Initial y-position for rendering text
        }
    }

    /// Handle a write to a UART register.
    pub fn out(&mut self, port: u16, val: u8, canvas: &mut dyn HybridCanvasBackend) {
        match port {
            0x3F8 => { // Transmit Holding Register (THR)
                self.thr_buf.push(val);
                // If we see a newline or the buffer is full, flush it to the canvas.
                if val == b'\n' || self.thr_buf.len() > 160 {
                    let line = String::from_utf8_lossy(&self.thr_buf).to_string();

                    // This is where the kernel's printk output gets rendered!
                    canvas.execute_text_run(TextRunOperation::at(8.0, self.next_line_y, &line));

                    self.next_line_y += 14.0; // Advance to the next line
                    self.thr_buf.clear();
                }
            }
            _ => {
                // For early boot, we can safely ignore writes to other UART registers
                // like LCR, DLL, DLM, IER, etc.
            }
        }
    }

    /// Handle a read from a UART register.
    pub fn inn(&self, port: u16) -> u8 {
        match port {
            0x3FD => 0x60, // Line Status Register (LSR): TX empty and ready.
            _ => 0x00,     // For other registers, returning 0 is safe for early boot.
        }
    }
}
