use anyhow::{Context, Result};
use notify::{Event, RecursiveMode, Watcher};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

mod pixel_vm;
mod glyph_rom;

use pixel_vm::PixelVM;
use glyph_rom::GLYPH_ROM;

const GVPIE_DIR: &str = "/tmp/gvpie";
const CMD_PATH: &str = "/tmp/gvpie/cmd.json";
const OUT_PATH: &str = "/tmp/gvpie/out.raw";

#[derive(Debug, Deserialize)]
struct Command {
    op: String,
    #[serde(default)]
    code: String,
    #[serde(default = "default_width")]
    width: u32,
    #[serde(default = "default_height")]
    height: u32,
    #[serde(default)]
    format: String,
}

fn default_width() -> u32 { 128 }
fn default_height() -> u32 { 64 }

struct GVPIEDaemon {
    device: wgpu::Device,
    queue: wgpu::Queue,
    canvas_width: u32,
    canvas_height: u32,
}

impl GVPIEDaemon {
    async fn new() -> Result<Self> {
        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Request adapter (headless)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find GPU adapter")?;

        println!("✅ GPU Adapter: {:?}", adapter.get_info().name);

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GVPIE Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .context("Failed to create GPU device")?;

        println!("✅ GPU Device created");

        Ok(Self {
            device,
            queue,
            canvas_width: 128,
            canvas_height: 64,
        })
    }

    fn handle_command(&mut self, cmd: Command) -> Result<()> {
        match cmd.op.as_str() {
            "render_program" => {
                println!("🎨 Rendering pixel program ({} bytes)", cmd.code.len());
                let rgba = self.render_pixel_program(&cmd.code, cmd.width, cmd.height)?;
                fs::write(OUT_PATH, rgba)?;
                println!("✅ Wrote {} bytes to {}", cmd.width * cmd.height * 4, OUT_PATH);
            }
            "ping" => {
                println!("🏓 Ping received");
                fs::write(OUT_PATH, b"pong")?;
            }
            "read_canvas" => {
                println!("📖 Reading canvas");
                // Return current canvas state (if we cached it)
                let empty = vec![0u8; (cmd.width * cmd.height * 4) as usize];
                fs::write(OUT_PATH, empty)?;
            }
            _ => {
                println!("❌ Unknown op: {}", cmd.op);
            }
        }
        Ok(())
    }

    fn render_pixel_program(&self, code: &str, width: u32, height: u32) -> Result<Vec<u8>> {
        // Parse pixel program
        let vm = PixelVM::new();
        let instructions = vm.parse(code)?;

        println!("📝 Parsed {} instructions", instructions.len());

        // Create RGBA buffer (CPU-side rendering for now)
        let mut rgba = vec![0u8; (width * height * 4) as usize];

        // Execute instructions
        for inst in instructions {
            match inst {
                pixel_vm::Instruction::Txt { x, y, text } => {
                    self.render_text(&mut rgba, width, height, x, y, &text);
                }
                pixel_vm::Instruction::Rect { x, y, w, h } => {
                    self.render_rect(&mut rgba, width, height, x, y, w, h);
                }
                pixel_vm::Instruction::Halt => break,
            }
        }

        Ok(rgba)
    }

    fn render_text(&self, buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, text: &str) {
        let mut cursor_x = x;
        for ch in text.chars() {
            let glyph = glyph_rom::get_glyph(ch);

            for gy in 0..glyph_rom::GLYPH_HEIGHT {
                for gx in 0..glyph_rom::GLYPH_WIDTH {
                    if glyph.is_pixel_set(gx, gy) {
                        let px = cursor_x + gx as u32;
                        let py = y + gy as u32;

                        if px < width && py < height {
                            let idx = ((py * width + px) * 4) as usize;
                            buffer[idx] = 255;     // R
                            buffer[idx + 1] = 255; // G
                            buffer[idx + 2] = 255; // B
                            buffer[idx + 3] = 255; // A
                        }
                    }
                }
            }

            cursor_x += (glyph_rom::GLYPH_WIDTH + 1) as u32;
        }
    }

    fn render_rect(&self, buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, w: u32, h: u32) {
        for py in y..(y + h).min(height) {
            for px in x..(x + w).min(width) {
                let idx = ((py * width + px) * 4) as usize;
                buffer[idx] = 255;     // R
                buffer[idx + 1] = 255; // G
                buffer[idx + 2] = 255; // B
                buffer[idx + 3] = 255; // A
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        println!("👀 Watching {}", CMD_PATH);

        // Create GVPIE directory
        fs::create_dir_all(GVPIE_DIR)?;

        // Watch for file changes
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(tx)?;

        watcher.watch(Path::new(GVPIE_DIR), RecursiveMode::NonRecursive)?;

        // Also poll manually since file watching can be unreliable
        loop {
            // Check for file system events
            if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
                if let Ok(Event { paths, .. }) = event {
                    if paths.iter().any(|p| p.ends_with("cmd.json")) {
                        self.process_command()?;
                    }
                }
            }

            // Also check manually
            if Path::new(CMD_PATH).exists() {
                self.process_command()?;
            }
        }
    }

    fn process_command(&mut self) -> Result<()> {
        let cmd_path = Path::new(CMD_PATH);
        if !cmd_path.exists() {
            return Ok(());
        }

        let json = fs::read_to_string(cmd_path)?;
        fs::remove_file(cmd_path)?; // Consume the command

        let cmd: Command = serde_json::from_str(&json)?;
        self.handle_command(cmd)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    println!(r#"
╔════════════════════════════════════════════════════════════════╗
║                🎨 GVPIE DAEMON - GPU RENDERER 🎨               ║
╚════════════════════════════════════════════════════════════════╝

File-socket IPC:
  Command: /tmp/gvpie/cmd.json
  Output:  /tmp/gvpie/out.raw

Press Ctrl+C to stop.
"#);

    let mut daemon = pollster::block_on(GVPIEDaemon::new())?;
    daemon.run()?;

    Ok(())
}
