#![no_std]
#![allow(dead_code)]
#![feature(allocator_api, alloc_layout_extra, pointer_is_aligned_to)]

extern crate alloc;

use alloc::boxed::Box;
use bitflags::bitflags;
use core::{
    alloc::{AllocError, Allocator},
    arch::asm,
    slice,
};
use vm::{PageEntryFlag, PageTable, PhysicalAddress, Ppn, VirtualAddress};

mod kalloc;
#[macro_use]
mod kprint;
mod sbi;
mod vm;

const PAGE_SIZE: usize = 4096;

extern "C" {
    static text_start: *const u8;
    static text_end: *const u8;
    static rodata_start: *const u8;
    static rodata_end: *const u8;
    static data_start: *const u8;
    static data_end: *const u8;
    static bss_start: *const u8;
    static stack_top: *const u8;
    static mut heap_start: *const u8;
    static heap_end: *const u8;
    static bss_end: *const u8;
}

bitflags! {
    pub struct SatpFlag: u64 {
        const SV39 = 8 << 60;
    }
}

#[allow(clippy::empty_loop)]
#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    kprintln!("starting kernel");

    let heap = slice::from_raw_parts_mut(
        (&raw mut heap_start).cast(),
        (&raw const heap_end).addr() - (&raw const heap_start).addr(),
    );
    let allocator = kalloc::bump::Allocator::new(heap);

    let table = Box::into_raw(Box::try_new_in(PageTable::default(), &allocator).unwrap());

    map_kernel_memory(table.as_mut().unwrap(), &allocator).unwrap();

    kprintln!("mapped kernel memory");

    asm!(
        "csrw satp, {}",
        in(reg) SatpFlag::SV39.bits() | PhysicalAddress::new(table.addr()).ppn().get() as u64
    );

    kprintln!("enabled virtual memory");

    loop {}
}

unsafe fn map_kernel_memory(
    root: &mut PageTable,
    allocator: &impl Allocator,
) -> Result<(), AllocError> {
    kprintln!("mapping text section");
    identity_map_range(
        root,
        allocator,
        (&raw const text_start).addr(),
        (&raw const text_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::EXEC,
    )?;

    kprintln!("mapping rodata section");
    identity_map_range(
        root,
        allocator,
        (&raw const rodata_start).addr(),
        (&raw const rodata_end).addr(),
        PageEntryFlag::READ,
    )?;

    kprintln!("mapping data section");
    identity_map_range(
        root,
        allocator,
        (&raw const data_start).addr(),
        (&raw const data_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    )?;

    kprintln!("mapping bss section");
    identity_map_range(
        root,
        allocator,
        (&raw const bss_start).addr(),
        (&raw const heap_end).addr(),
        PageEntryFlag::READ | PageEntryFlag::WRITE,
    )?;

    Ok(())
}

unsafe fn identity_map_range(
    root: &mut PageTable,
    allocator: &impl Allocator,
    start: usize,
    end: usize,
    flags: PageEntryFlag,
) -> Result<(), AllocError> {
    for p in (start..end).step_by(PAGE_SIZE) {
        let vaddr = Ppn::from_addr(p);
        let paddr = Ppn::from_addr(p);

        root.map(allocator, vaddr, paddr, flags)?;
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}
