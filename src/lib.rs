#![no_std]
#![allow(dead_code)]
#![feature(allocator_api, alloc_layout_extra, pointer_is_aligned_to)]

mod kalloc;
#[macro_use]
mod kprint;
mod sbi;

const PAGE_SIZE: usize = 4096;

extern "C" {
    static heap_start: ();
    static heap_end: ();
}

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    kprintln!("starting kernel");
    loop {}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}
