use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;

pub struct LinuxBootLoader {
    pub kernel_data: Vec<u8>,
    pub initrd_data: Vec<u8>,
    pub cmdline: String,
}

impl LinuxBootLoader {
    pub fn load_tinycore() -> Result<Self> {
        let kernel_data = fs::read("tinycore/vmlinuz")
            .context("Failed to read kernel file tinycore/vmlinuz")?;

        let core_gz = fs::read("tinycore/core.gz")
            .context("Failed to read initrd file tinycore/core.gz")?;

        let mut decoder = GzDecoder::new(&core_gz[..]);
        let mut initrd_data = Vec::new();
        decoder.read_to_end(&mut initrd_data)?;

        Ok(Self {
            kernel_data,
            initrd_data,
            cmdline: "console=ttyS0 root=/dev/ram0".to_string(),
        })
    }
}
