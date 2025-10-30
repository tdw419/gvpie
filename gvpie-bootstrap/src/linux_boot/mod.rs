use flate2::read::GzDecoder;
use std::io::Read;
use anyhow::{Context, Result};

use crate::gpu_memory_manager::GPUMemoryManager; // Assuming GPUMemoryManager will be in scope
use crate::linux_boot::params::LinuxBootParams;
use crate::linux_boot::bzimage::BzImage;

pub mod bzimage;
pub mod params;

pub struct LinuxBootLoader {
    pub kernel_data: Vec<u8>,
    pub initrd_data: Vec<u8>,
    pub cmdline: String,
    pub setup_header: bzimage::SetupHeader,
}

impl LinuxBootLoader {
    pub fn load_tinycore() -> Result<Self> {
        let bzimage = BzImage::load("third_party/tinycore/vmlinuz64")?;

        let core_gz = std::fs::read("third_party/tinycore/corepure64.gz")
            .context("Failed to read initrd file")?;
        let mut decoder = GzDecoder::new(&core_gz[..]);
        let mut initrd_data = Vec::new();
        decoder.read_to_end(&mut initrd_data)
            .context("Failed to decompress initrd")?;

        let cmdline = "console=ttyS0,115200 earlyprintk=serial,ttyS0,115200 printk.devkmsg=on loglevel=8".to_string();

        Ok(Self {
            kernel_data: bzimage.kernel_data,
            setup_header: bzimage.setup_header,
            initrd_data,
            cmdline,
        })
    }

    pub fn setup_boot_environment(
        &self,
        memory_mgr: &mut GPUMemoryManager,
    ) -> Result<u32> {
        // Define memory layout
        const KERNEL_ADDR: u64 = 0x100000;
        const INITRD_ADDR: u64 = 0x2000000;
        const BOOT_PARAMS_ADDR: u64 = 0x90000;
        const CMDLINE_ADDR: u64 = 0x91000;

        // Map and write data to emulated memory
        memory_mgr.map_memory(KERNEL_ADDR, self.kernel_data.len());
        memory_mgr.write_memory(KERNEL_ADDR, &self.kernel_data);

        memory_mgr.map_memory(INITRD_ADDR, self.initrd_data.len());
        memory_mgr.write_memory(INITRD_ADDR, &self.initrd_data);

        memory_mgr.map_memory(CMDLINE_ADDR, self.cmdline.len() + 1);
        memory_mgr.write_memory(CMDLINE_ADDR, self.cmdline.as_bytes());

        // Create and write boot_params struct
        let mut boot_params: LinuxBootParams = unsafe { std::mem::zeroed() };
        boot_params.setup_header = self.setup_header;
        boot_params.setup_header.type_of_loader = 0xff; // Undefined loader
        boot_params.setup_header.loadflags |= 0x01; // LOADED_HIGH
        boot_params.setup_header.ramdisk_image = INITRD_ADDR as u32;
        boot_params.setup_header.ramdisk_size = self.initrd_data.len() as u32;
        boot_params.setup_header.cmd_line_ptr = CMDLINE_ADDR as u32;
        boot_params.setup_header.cmdline_size = self.cmdline.len() as u32;
        boot_params.setup_header.initrd_addr_max = 0x37FFFFFF; // Max address for initrd

        memory_mgr.map_memory(BOOT_PARAMS_ADDR, std::mem::size_of::<LinuxBootParams>());
        memory_mgr.write_memory(BOOT_PARAMS_ADDR, bytemuck::bytes_of(&boot_params));

        log::info!("Linux boot environment created in emulated memory.");

        Ok(self.setup_header.code32_start)
    }
}
