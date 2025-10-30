mod bzimage;
mod params;

use std::io::Read;

use bzimage::BzImage;
use flate2::read::GzDecoder;

use crate::memory::{AddressSpaceId, Layer4Memory};

pub use params::{BootParams, SetupHeader};

const BOOT_PARAMS_ADDR: u64 = 0x0009_0000;
const CMDLINE_ADDR: u64 = 0x0009_A000;
const KERNEL_LOAD_ADDR: u64 = 0x0010_0000;
const INITRD_LOAD_ADDR: u64 = 0x0200_0000;

pub struct LinuxBootLoader {
    bz: BzImage,
    initrd: Vec<u8>,
    cmdline: String,
}

impl LinuxBootLoader {
    pub fn load_tinycore() -> Result<Self, String> {
        let bz = BzImage::load("assets/tinycore/vmlinuz64")?;
        let initrd_gz = std::fs::read("assets/tinycore/corepure64.gz")
            .map_err(|e| format!("read corepure64.gz: {e}"))?;
        let mut decoder = GzDecoder::new(&initrd_gz[..]);
        let mut initrd = Vec::new();
        decoder
            .read_to_end(&mut initrd)
            .map_err(|e| format!("inflate corepure64.gz: {e}"))?;

        let cmdline = "console=ttyS0,115200 earlyprintk=serial,ttyS0,115200 earlycon=uart,io,0x3f8,115200n8 loglevel=8\0";

        Ok(Self {
            bz,
            initrd,
            cmdline: cmdline.to_string(),
        })
    }

    pub fn install(&self, mem: &mut Layer4Memory) -> Result<(AddressSpaceId, SetupHeader), String> {
        let pid = mem.create_address_space();

        mem.map_and_write(pid, KERNEL_LOAD_ADDR, &self.bz.kernel)?;
        mem.map_and_write(pid, INITRD_LOAD_ADDR, &self.initrd)?;
        mem.map_and_write(pid, CMDLINE_ADDR, self.cmdline.as_bytes())?;

        let mut params = BootParams::zeroed();
        params.hdr = self.bz.header;
        params.hdr.boot_flag = 0xAA55;
        params.hdr.header = 0x5372_6448;
        params.hdr.version = 0x020b;
        params.hdr.type_of_loader = 0xff;
        params.hdr.loadflags |= 0x01;
        params.hdr.initrd_addr_max = 0x37ff_ffff;

        params.ext_cmd_line_ptr = CMDLINE_ADDR as u32;
        params.ext_ramdisk_image = INITRD_LOAD_ADDR as u32;
        params.ext_ramdisk_size = self.initrd.len() as u32;
        params.hdr.ramdisk_image = params.ext_ramdisk_image;
        params.hdr.ramdisk_size = params.ext_ramdisk_size;

        mem.map_and_write(pid, BOOT_PARAMS_ADDR, bytemuck::bytes_of(&params))?;
        mem.lineage()
            .push(format!(
                "Boot env mapped: kernel=0x{KERNEL_LOAD_ADDR:08x}, initrd=0x{INITRD_LOAD_ADDR:08x}, cmdline=0x{CMDLINE_ADDR:08x}, params=0x{BOOT_PARAMS_ADDR:08x}"
            ));

        Ok((pid, self.bz.header))
    }
}
