use core::ptr;

use crate::PAGE_SIZE;

static mut HEAD: *mut u8 = (&raw mut crate::heap_start).cast();

pub fn alloc() -> Option<*mut u8> {
    unsafe {
        let head = HEAD;
        HEAD = head.add(PAGE_SIZE);
        Some(head)
    }
}

pub unsafe fn dealloc(_: *mut u8) {}
