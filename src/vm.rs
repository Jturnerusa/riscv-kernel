use core::alloc::{AllocError, Allocator, Layout};

use crate::{kalloc, PAGE_SIZE};
use alloc::boxed::Box;
use bitflags::bitflags;

trait Address {
    fn addr(self) -> usize;
}

#[derive(Clone, Copy, Debug)]
pub struct Page<T>(T);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PhysicalAddress(usize);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VirtualAddress(usize);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct Entry(u64);

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PageEntryFlag: u64 {
        const VALID = 1;
        const READ = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC = 1 << 3;
        const ACCESSED = 1 << 6;
        const DIRTY = 1 << 7;
    }
}

#[repr(C, align(4096))]
#[derive(Clone, Copy, Debug)]
pub struct PageTable {
    entries: [Entry; 512],
}

impl<T: Clone + Copy + Address> Page<T> {
    pub fn new(addr: T) -> Option<Self> {
        if addr.addr() % PAGE_SIZE == 0 {
            Some(Self(addr))
        } else {
            None
        }
    }
}

impl<T: Clone + Copy> Page<T> {
    pub fn get(self) -> T {
        self.0
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self {
            entries: [Entry(0); 512],
        }
    }
}

impl Address for PhysicalAddress {
    fn addr(self) -> usize {
        self.0
    }
}

impl Address for VirtualAddress {
    fn addr(self) -> usize {
        self.0
    }
}

impl VirtualAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get(self) -> usize {
        self.0
    }

    fn vpn(self) -> [usize; 3] {
        let vpn0 = self.0 >> 12 & 0x1ff;
        let vpn1 = self.0 >> 21 & 0x1ff;
        let vpn2 = self.0 >> 30 & 0x1ff;

        [vpn0, vpn1, vpn2]
    }

    fn offset(self) -> u64 {
        (self.0 & 0xfff) as u64
    }
}

impl PhysicalAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get(self) -> usize {
        self.0
    }

    fn ppn(self) -> usize {
        let ppn0 = self.0 >> 12 & 0x1ff;
        let ppn1 = self.0 >> 21 & 0x1ff;
        let ppn2 = self.0 >> 30 & 0x3ff_ffff;

        ppn2 << 18 | ppn1 << 9 | ppn0
    }
}

impl Entry {
    fn new_branch(ppn: usize) -> Self {
        Self((ppn as u64) << 10 | PageEntryFlag::VALID.bits())
    }

    fn new_leaf(ppn: usize, flags: PageEntryFlag) -> Self {
        Self(
            (ppn as u64) << 10
                | (PageEntryFlag::DIRTY | PageEntryFlag::ACCESSED | flags | PageEntryFlag::VALID)
                    .bits(),
        )
    }

    fn ppn(self) -> usize {
        let ppn0 = self.0 >> 10 & 0x1ff;
        let ppn1 = self.0 >> 19 & 0x1ff;
        let ppn2 = self.0 >> 28 & 0x3ff_ffff;

        (ppn2 << 18 | ppn1 << 9 | ppn0) as usize
    }

    fn is_valid(self) -> bool {
        self.0 & PageEntryFlag::VALID.bits() != 0
    }

    fn is_read(self) -> bool {
        self.0 & PageEntryFlag::READ.bits() != 0
    }

    fn is_write(self) -> bool {
        self.0 & PageEntryFlag::WRITE.bits() != 0
    }

    fn is_exec(self) -> bool {
        self.0 & PageEntryFlag::EXEC.bits() != 0
    }

    fn is_leaf(self) -> bool {
        self.0
            & (PageEntryFlag::READ
                | PageEntryFlag::WRITE
                | PageEntryFlag::EXEC
                | PageEntryFlag::VALID)
                .bits()
            != 0
    }
}

impl PageTable {
    pub unsafe fn map(
        &mut self,
        allocator: impl Allocator,
        vaddr: Page<VirtualAddress>,
        paddr: Page<PhysicalAddress>,
        flags: PageEntryFlag,
    ) -> Result<(), AllocError> {
        map(self, allocator, vaddr, paddr, flags, 2)
    }
}

unsafe fn map(
    table: &mut PageTable,
    allocator: impl Allocator,
    vaddr: Page<VirtualAddress>,
    paddr: Page<PhysicalAddress>,
    flags: PageEntryFlag,
    level: usize,
) -> Result<(), AllocError> {
    let vpn = vaddr.get().vpn()[level];
    let entry = table.entries[vpn];

    if !entry.is_valid() && level > 0 {
        let page = Box::into_raw(Box::try_new_in(PageTable::default(), &allocator)?);

        table.entries[vpn] = Entry::new_branch(page.addr() >> 12);
    }

    // a new entry may have been created above, so we need to re-read the entry
    let entry = table.entries[vpn];

    if level == 0 {
        table.entries[vpn] = Entry::new_leaf(paddr.get().ppn(), flags);

        Ok(())
    } else {
        let next = ((entry.ppn() << 12) as *mut PageTable).as_mut().unwrap();

        map(next, allocator, vaddr, paddr, flags, level - 1)
    }
}
