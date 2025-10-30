use bytemuck::{Pod, Zeroable};

/// The `LinuxBootParams` struct, also known as `boot_params` in the kernel.
/// This structure is filled in by the boot loader and passed to the kernel.
/// The layout MUST match the Linux boot protocol.
/// see https://www.kernel.org/doc/Documentation/x86/boot.txt
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct LinuxBootParams {
    // ... fields from 0x000 to 0x1f0 ...
    pub screen_info: [u8; 0x1f1 - 0x000],
    // The setup_header struct starts at offset 0x1F1
    pub setup_header: SetupHeader,
    // ... other fields ...
    pub boot_params_end: [u8; 2992 - (0x1f1 + std::mem::size_of::<SetupHeader>())],
}

unsafe impl Zeroable for LinuxBootParams {}
unsafe impl Pod for LinuxBootParams {}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SetupHeader {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_dev: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,       // Magic: 'HdrS' (0x53726448)
    pub version: u16,      // Boot protocol version
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
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub min_alignment: u8,
    pub xloadflags: u16,
    pub cmdline_size: u32,
    pub hardware_subarch: u32,
    pub hardware_subarch_data: u64,
    pub payload_offset: u32,
    pub payload_length: u32,
    pub setup_data: u64,
    pub pref_address: u64,
    pub init_size: u32,
    pub handover_offset: u32,
}
