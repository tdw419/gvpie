use std::fs::File;
use memmap2::Mmap;

pub enum InputSource {
    Keyboard(char),
    FileCommand(Vec<u8>),
}

pub struct InputMultiplexer {
    file_path: String,
    mmap: Option<Mmap>,
}

impl InputMultiplexer {
    pub fn new(file_path: &str) -> Self {
        let file = File::open(file_path).ok();
        let mmap = file.and_then(|f| unsafe { Mmap::map(&f).ok() });

        Self {
            file_path: file_path.to_string(),
            mmap,
        }
    }

    pub fn poll_file(&mut self) -> Option<Vec<u8>> {
        if let Some(mmap) = &self.mmap {
            // A simple protocol: if the first byte is non-zero, there's a command.
            if !mmap.is_empty() && mmap[0] != 0 {
                // Find the end of the command (null-terminated string).
                let end = mmap.iter().position(|&b| b == 0).unwrap_or(mmap.len());
                let cmd = mmap[1..end].to_vec();

                // Clear the command by setting the first byte to 0.
                // This requires a mutable map.
                if let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(&self.file_path) {
                    if let Ok(mut mmap_mut) = unsafe { memmap2::MmapMut::map_mut(&file) } {
                        mmap_mut[0] = 0;
                        mmap_mut.flush().ok();
                    }
                }

                return Some(cmd);
            }
        }
        None
    }

    pub fn next_command(&mut self, keyboard_input: Option<char>) -> Option<InputSource> {
        if let Some(cmd) = self.poll_file() {
            return Some(InputSource::FileCommand(cmd));
        }

        if let Some(ch) = keyboard_input {
            return Some(InputSource::Keyboard(ch));
        }

        None
    }
}
