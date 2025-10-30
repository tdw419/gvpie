use rand::random;

#[repr(C)]
pub struct OSConfig {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct OSHandle {
    instance_id: u64,
    framebuffer: *mut u8,
    framebuffer_size: usize,
    width: u32,
    height: u32,
}

#[no_mangle]
pub extern "C" fn pixelos_create(config: OSConfig) -> *mut OSHandle {
    let framebuffer_size = (config.width * config.height * 4) as usize;
    let framebuffer = vec![0u8; framebuffer_size].into_boxed_slice();

    let handle = Box::new(OSHandle {
        instance_id: random(),
        framebuffer: Box::into_raw(framebuffer) as *mut u8,
        framebuffer_size,
        width: config.width,
        height: config.height,
    });

    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn pixelos_step(handle: *mut OSHandle) {
    let os = unsafe { &mut *handle };
    let framebuffer = unsafe { std::slice::from_raw_parts_mut(os.framebuffer, os.framebuffer_size) };

    for (i, pixel) in framebuffer.chunks_exact_mut(4).enumerate() {
        let x = i % os.width as usize;
        let y = i / os.width as usize;
        pixel[0] = (x % 255) as u8; // R
        pixel[1] = (y % 255) as u8; // G
        pixel[2] = 0;             // B
        pixel[3] = 255;           // A
    }
}

#[no_mangle]
pub extern "C" fn pixelos_send_key(_handle: *mut OSHandle, _keycode: u32, _modifiers: u32) {
    // In a real implementation, you would handle the key press here.
}

#[no_mangle]
pub extern "C" fn pixelos_get_framebuffer(handle: *mut OSHandle) -> *const u8 {
    let os = unsafe { &*handle };
    os.framebuffer
}

#[no_mangle]
pub extern "C" fn pixelos_destroy(handle: *mut OSHandle) {
    if !handle.is_null() {
        unsafe {
            let os = Box::from_raw(handle);
            let framebuffer = Box::from_raw(os.framebuffer);
            drop(framebuffer);
            drop(os);
        }
    }
}
