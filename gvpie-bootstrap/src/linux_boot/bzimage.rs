use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use bytemuck::{Pod, Zeroable};

use super::params::SetupHeader;

#[derive(Clone)]
pub struct BzImage {
    pub kernel: Vec<u8>,
    pub header: SetupHeader,
}

impl BzImage {
    pub fn load(path: &str) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut kernel = Vec::new();
        file.read_to_end(&mut kernel)
            .map_err(|e| format!("read vmlinuz: {e}"))?;

        // setup_header resides at offset 0x1f1 from start of file.
        let mut header_bytes = [0u8; std::mem::size_of::<SetupHeader>()];
        file.seek(SeekFrom::Start(0x1f1))
            .map_err(|e| format!("seek header: {e}"))?;
        file.read_exact(&mut header_bytes)
            .map_err(|e| format!("read header: {e}"))?;
        let header = *bytemuck::from_bytes::<SetupHeader>(&header_bytes);

        if header.header != 0x5372_6448 {
            return Err("bzImage missing magic HdrS".into());
        }
        if header.version < 0x020b {
            return Err(format!(
                "Linux boot protocol version too old: 0x{:x}",
                header.version
            ));
        }

        Ok(Self { kernel, header })
    }
}
