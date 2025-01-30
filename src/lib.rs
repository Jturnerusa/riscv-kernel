#![no_std]
#![allow(dead_code)]

#[macro_use]
mod kprint;
mod kalloc;
mod uart;

const PAGE_SIZE: usize = 4096;

extern "C" {
    static heap_start: ();
    static heap_end: ();
}

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    loop {}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}
