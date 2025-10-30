use super::uart16550::Uart16550;
use crate::hybrid_canvas::HybridCanvasBackend;

#[derive(Clone, Default)]
pub struct IoPorts {
    pub com1: Uart16550,
}

impl IoPorts {
    pub fn new() -> Self {
        Self {
            com1: Uart16550::new(),
        }
    }

    pub fn out(&mut self, port: u16, val: u8, canvas: &mut dyn HybridCanvasBackend) {
        match port {
            0x3F8..=0x3FF => self.com1.out(port, val, canvas),
            _ => {
                // Ignore writes to other ports for now
            }
        }
    }

    pub fn inn(&self, port: u16) -> u8 {
        match port {
            0x3F8..=0x3FF => self.com1.inn(port),
            _ => 0x00,
        }
    }
}
