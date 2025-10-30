use hybrid_canvas::{HybridCanvasBackend, TextRunOperation};

/// Extremely small emulation of a 16550A UART (COM1) sufficient for earlyprintk.
pub struct Uart16550 {
    lcr: u8,
    dll: u8,
    dlm: u8,
    ier: u8,
    mcr: u8,
    scr: u8,
    transmit_buffer: Vec<u8>,
    next_line_y: f32,
}

impl Uart16550 {
    pub fn new() -> Self {
        Self {
            lcr: 0,
            dll: 1,
            dlm: 0,
            ier: 0,
            mcr: 0,
            scr: 0,
            transmit_buffer: Vec::with_capacity(256),
            next_line_y: 24.0,
        }
    }

    pub fn in8(&self, port: u16) -> u8 {
        match port {
            0x3f8 => {
                if self.lcr & 0x80 != 0 {
                    self.dll
                } else {
                    0
                }
            }
            0x3f9 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm
                } else {
                    self.ier
                }
            }
            0x3fa => 0x01, // IIR: no interrupts pending
            0x3fb => self.lcr,
            0x3fc => self.mcr,
            0x3fd => 0x60, // LSR: THRE | TEMT set
            0x3fe => 0,    // MSR
            0x3ff => self.scr,
            _ => 0xff,
        }
    }

    pub fn out8<C: HybridCanvasBackend>(&mut self, port: u16, value: u8, canvas: &mut C) {
        match port {
            0x3f8 => {
                if self.lcr & 0x80 != 0 {
                    self.dll = value;
                } else {
                    self.transmit_buffer.push(value);
                    if value == b'\n' || self.transmit_buffer.len() >= 160 {
                        if let Ok(line) = String::from_utf8(self.transmit_buffer.clone()) {
                            canvas.execute_text_run(TextRunOperation {
                                text: line.clone(),
                                x: 50.0,
                                y: self.next_line_y,
                                px_size: 12.0,
                            });
                        }
                        self.transmit_buffer.clear();
                        self.next_line_y += 16.0;
                    }
                }
            }
            0x3f9 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm = value;
                } else {
                    self.ier = value;
                }
            }
            0x3fa => {} // FIFO control ignored
            0x3fb => self.lcr = value,
            0x3fc => self.mcr = value,
            0x3fe => {} // Modem status ignored
            0x3ff => self.scr = value,
            _ => {}
        }
    }
}
