pub struct CpuTextSurface {
    w: u32,
    h: u32,
    buf: Vec<u8>, // RGBA8
}

impl CpuTextSurface {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            buf: vec![0; (w * h * 4) as usize],
        }
    }

    pub fn clear_rgba(&mut self, rgba: [u8; 4]) {
        for px in self.buf.chunks_exact_mut(4) {
            px.copy_from_slice(&rgba);
        }
    }

    pub fn draw_text(&mut self, s: &str, x: i32, baseline_y: i32, px: f32) {
        let scale = (px / GLYPH_HEIGHT as f32).clamp(1.0, 64.0);
        let glyph_w = (GLYPH_WIDTH as f32 * scale).ceil() as i32;
        let glyph_h = (GLYPH_HEIGHT as f32 * scale).ceil() as i32;
        let advance = (glyph_w + scale.ceil() as i32).max(1);
        let mut pen_x = x as f32;
        for ch in s.chars() {
            if ch == '\n' {
                // crude newline: move to next row block
                pen_x = x as f32;
                continue;
            }
            let pattern = glyph_pattern(ch);
            let origin_x = pen_x.round() as i32;
            let origin_y = (baseline_y as f32 - glyph_h as f32).round() as i32;

            for (row_idx, row_bits) in pattern.iter().enumerate() {
                for col in 0..GLYPH_WIDTH {
                    if (row_bits >> (GLYPH_WIDTH - 1 - col)) & 1 == 1 {
                        self.fill_block(
                            origin_x + (col as i32 * scale as i32),
                            origin_y + (row_idx as i32 * scale as i32),
                            scale as i32,
                            scale as i32,
                        );
                    }
                }
            }

            pen_x += advance as f32;
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    fn fill_block(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && py >= 0 && (px as u32) < self.w && (py as u32) < self.h {
                    let idx = ((py as u32 * self.w + px as u32) * 4) as usize;
                    self.buf[idx + 0] = 0xF8;
                    self.buf[idx + 1] = 0xF8;
                    self.buf[idx + 2] = 0xF8;
                    self.buf[idx + 3] = 0xFF;
                }
            }
        }
    }
}

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;

fn glyph_pattern(ch: char) -> [u8; GLYPH_HEIGHT] {
    match ch {
        'd' => [0b01110, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01111],
        'i' => [0b00100, 0, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'r' => [0b11110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        ' ' => [0; GLYPH_HEIGHT],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}
