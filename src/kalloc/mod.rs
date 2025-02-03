use core::{alloc::GlobalAlloc, ptr};

pub mod bump;

#[global_allocator]
pub static DUMMY: Dummy = Dummy;

struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        panic!()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        panic!()
    }
}
