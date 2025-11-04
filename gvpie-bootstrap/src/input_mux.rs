use memmap2::{Mmap, MmapMut};
use std::fs::OpenOptions;

pub struct IPC {
    pub machine_mmap: Mmap,
    pub human_mmap: MmapMut,
    pub control_mmap: MmapMut,
}

impl IPC {
    pub fn new() -> Self {
        let machine_file = OpenOptions::new().read(true).write(true).create(true).open("/tmp/gvpie/machine.bin").unwrap();
        let human_file = OpenOptions::new().read(true).write(true).create(true).open("/tmp/gvpie/human.bin").unwrap();
        let control_file = OpenOptions::new().read(true).write(true).create(true).open("/tmp/gvpie/control.bin").unwrap();

        machine_file.set_len(128 * 64).unwrap();
        human_file.set_len(128 * 64 * 4).unwrap();
        control_file.set_len(2).unwrap();

        let machine_mmap = unsafe { Mmap::map(&machine_file).unwrap() };
        let human_mmap = unsafe { MmapMut::map_mut(&human_file).unwrap() };
        let control_mmap = unsafe { MmapMut::map_mut(&control_file).unwrap() };

        Self {
            machine_mmap,
            human_mmap,
            control_mmap,
        }
    }
}
