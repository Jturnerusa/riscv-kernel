#![no_std]
#![allow(dead_code)]
#![feature(allocator_api, alloc_layout_extra, pointer_is_aligned_to)]

extern crate alloc;

use core::{alloc::Allocator, arch::asm, slice};
use vm::{Page, PageEntryFlag, PageTable, PhysicalAddress, VirtualAddress};

mod kalloc;
#[macro_use]
mod kprint;
mod sbi;
mod vm;

const PAGE_SIZE: usize = 4096;

extern "C" {
    static text_start: ();
    static text_end: ();
    static rodata_start: ();
    static rodata_end: ();
    static data_start: ();
    static data_end: ();
    static bss_start: ();
    static stack_top: ();
    static mut heap_start: ();
    static heap_end: ();
    static bss_end: ();
}

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    kprintln!("starting kernel");

    let heap = slice::from_raw_parts_mut(
        (&raw mut heap_start) as *mut u8,
        (&raw const heap_end).addr() - (&raw const heap_start).addr(),
    );

    let allocator = kalloc::bump::Allocator::new(heap);

    let mut table = PageTable::default();
    map_kernel_memory(&mut table, &allocator);
    kprintln!("mapped kernel memory");

    asm!(
        "csrw satp, {}",
        in(reg) (8 << 60) | ((&raw const table).addr() >> 12)
    );
    kprintln!("enabled virtual memory");

    loop {}
}

unsafe fn map_kernel_memory(root: &mut PageTable, allocator: &impl Allocator) {
    kprintln!("mapping text section");
    identity_map_range(
        root,
        allocator,
        (&text_start as *const ()).addr(),
        (&text_end as *const ()).addr(),
        PageEntryFlag::READ | PageEntryFlag::EXEC,
    );

    kprintln!("mapping rodata section");
    identity_map_range(
        root,
        allocator,
        (&rodata_start as *const ()).addr(),
        (&rodata_end as *const ()).addr(),
        PageEntryFlag::READ,
    );

    kprintln!("mapping data section");
    identity_map_range(
        root,
        allocator,
        (&data_start as *const ()).addr(),
        (&data_end as *const ()).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    );

    kprintln!("mapping bss section");
    identity_map_range(
        root,
        allocator,
        (&bss_start as *const ()).addr(),
        (&bss_end as *const ()).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    );
}

unsafe fn identity_map_range(
    root: &mut PageTable,
    allocator: &impl Allocator,
    start: usize,
    end: usize,
    flags: PageEntryFlag,
) {
    for p in (start..end).step_by(PAGE_SIZE) {
        let vaddr = Page::new(VirtualAddress::new(p)).unwrap();
        let paddr = Page::new(PhysicalAddress::new(p)).unwrap();

        root.map(allocator, vaddr, paddr, flags).unwrap();
    }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}
