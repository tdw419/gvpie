use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use anyhow::{Context, Result};

pub use super::params::SetupHeader;

pub struct BzImage {
    pub kernel_data: Vec<u8>,
    pub setup_header: SetupHeader,
}

impl BzImage {
    pub fn load(path: &str) -> Result<Self> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open bzImage file: {}", path))?;

        let mut kernel_data = Vec::new();
        file.read_to_end(&mut kernel_data)
            .context("Failed to read kernel data")?;

        // The setup_header starts at a fixed offset of 0x1f1 in the bzImage file.
        file.seek(SeekFrom::Start(0x1f1))
            .context("Failed to seek to setup_header")?;

        let mut header_data = [0u8; std::mem::size_of::<SetupHeader>()];
        file.read_exact(&mut header_data)
            .context("Failed to read setup_header")?;

        let setup_header: SetupHeader = bytemuck::pod_read_unaligned(&header_data);

        // Validate the header magic number 'HdrS' and protocol version.
        if setup_header.header != 0x53726448 {
            anyhow::bail!("Invalid bzImage header magic. Expected 'HdrS'.");
        }
        let version = setup_header.version;
        if version < 0x0207 { // Protocol 2.07+ is required for 64-bit kernels
            anyhow::bail!("Unsupported boot protocol version: {:#x}", version);
        }

        Ok(Self {
            kernel_data,
            setup_header,
        })
    }
}
