use bytemuck::{Pod, Zeroable};

/// setup_header as defined in Documentation/x86/boot.rst
#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SetupHeader {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_dev: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,
    pub version: u16,
    pub realmode_swtch: u32,
    pub start_sys_seg: u16,
    pub kernel_version: u16,
    pub type_of_loader: u8,
    pub loadflags: u8,
    pub setup_move_size: u16,
    pub code32_start: u32,
    pub ramdisk_image: u32,
    pub ramdisk_size: u32,
    pub bootsect_kludge: u32,
    pub heap_end_ptr: u16,
    pub ext_loader_ver: u8,
    pub ext_loader_type: u8,
    pub cmd_line_ptr: u32,
    pub initrd_addr_max: u32,
}

/// boot_params structure passed to the kernel.
#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BootParams {
    pub screen_info: [u8; 0x40],
    pub apm_bios_info: [u8; 0x14],
    pub _pad1: [u8; 0x20],
    pub tboot_addr: u64,
    pub ist_info: [u8; 0x10],
    pub _pad2: [u8; 0x30],
    pub hd0_info: [u8; 0x10],
    pub hd1_info: [u8; 0x10],
    pub sys_desc_table: [u8; 0x10],
    pub olpc_ofw_header: [u8; 0x10],
    pub ext_ramdisk_image: u32,
    pub ext_ramdisk_size: u32,
    pub ext_cmd_line_ptr: u32,
    pub _pad3: [u8; 0x68],
    pub hdr: SetupHeader,
    pub _pad4: [u8; 0x2d0],
}
