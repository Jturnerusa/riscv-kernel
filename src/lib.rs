#![no_std]
#![allow(dead_code)]

use bitflags::bitflags;
use core::arch::asm;
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

bitflags! {
    pub struct SatpFlag: u64 {
        const SV39 = 8 << 60;
    }
}

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    kprintln!("starting kernel");

    let mut table = PageTable::default();
    map_kernel_memory(&mut table);
    kprintln!("mapped kernel memory");

    asm!(
        "csrw satp, {}",
        in(reg) SatpFlag::SV39.bits() | PhysicalAddress::new((&raw const table).addr()).ppn().get() as u64
    );
    kprintln!("enabled virtual memory");

    loop {}
}

unsafe fn map_kernel_memory(root: &mut PageTable) {
    kprintln!("mapping text section");
    identity_map_range(
        root,
        (&raw const text_start).addr(),
        (&raw const text_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::EXEC,
    );

    kprintln!("mapping rodata section");
    identity_map_range(
        root,
        (&raw const rodata_start).addr(),
        (&raw const rodata_end).addr(),
        PageEntryFlag::READ,
    );

    kprintln!("mapping data section");
    identity_map_range(
        root,
        (&raw const data_start).addr(),
        (&raw const data_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    );

    kprintln!("mapping bss section");
    identity_map_range(
        root,
        (&raw const bss_start).addr(),
        (&raw const bss_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    );
}

unsafe fn identity_map_range(root: &mut PageTable, start: usize, end: usize, flags: PageEntryFlag) {
    for p in (start..end).step_by(PAGE_SIZE) {
        let vaddr = Page::new(VirtualAddress::new(p)).unwrap();
        let paddr = Page::new(PhysicalAddress::new(p)).unwrap();

        root.map(vaddr, paddr, flags);
    }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}
