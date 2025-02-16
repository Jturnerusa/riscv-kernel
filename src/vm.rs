use core::alloc::{AllocError, Allocator};

use alloc::boxed::Box;
use bitflags::bitflags;

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PhysicalAddress(usize);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VirtualAddress(usize);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Ppn(usize);

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

impl VirtualAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn vpn(self) -> [usize; 3] {
        let vpn0 = self.0 >> 12 & 0x1ff;
        let vpn1 = self.0 >> 21 & 0x1ff;
        let vpn2 = self.0 >> 30 & 0x1ff;

        [vpn0, vpn1, vpn2]
    }

    pub fn offset(self) -> usize {
        self.0 & 0xfff
    }
}

impl PhysicalAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn ppn(self) -> Ppn {
        let ppn0 = self.0 >> 12 & 0x1ff;
        let ppn1 = self.0 >> 21 & 0x1ff;
        let ppn2 = self.0 >> 30 & 0x3ff_ffff;

        Ppn(ppn2 << 18 | ppn1 << 9 | ppn0)
    }
}

impl Entry {
    fn new_branch(ppn: Ppn) -> Self {
        Self((ppn.get() as u64) << 10 | PageEntryFlag::VALID.bits())
    }

    fn new_leaf(ppn: Ppn, flags: PageEntryFlag) -> Self {
        Self(
            (ppn.get() as u64) << 10
                | (PageEntryFlag::DIRTY | PageEntryFlag::ACCESSED | flags | PageEntryFlag::VALID)
                    .bits(),
        )
    }

    fn ppn(self) -> Ppn {
        let ppn0 = self.0 >> 10 & 0x1ff;
        let ppn1 = self.0 >> 19 & 0x1ff;
        let ppn2 = self.0 >> 28 & 0x3ff_ffff;

        Ppn((ppn2 << 18 | ppn1 << 9 | ppn0) as usize)
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
        self.0 & (PageEntryFlag::READ | PageEntryFlag::WRITE | PageEntryFlag::EXEC).bits() != 0
    }
}

impl Ppn {
    pub fn new(ppn: usize) -> Self {
        Self(ppn)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }

    pub fn into_addr(self) -> usize {
        self.0 << 12
    }
}

impl PageTable {
    pub unsafe fn map(
        &mut self,
        allocator: &impl Allocator,
        vaddr: Ppn,
        paddr: Ppn,
        flags: PageEntryFlag,
    ) -> Result<(), AllocError> {
        map(self, allocator, vaddr, paddr, flags, 2)
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self {
            entries: [Entry(0); 512],
        }
    }
}

unsafe fn map(
    table: &mut PageTable,
    allocator: &impl Allocator,
    vaddr: Ppn,
    paddr: Ppn,
    flags: PageEntryFlag,
    level: usize,
) -> Result<(), AllocError> {
    let vpn = VirtualAddress::new(vaddr.into_addr()).vpn()[level];
    let entry = table.entries[vpn];

    if !entry.is_valid() && level > 0 {
        let page = Box::into_raw(Box::try_new_in(PageTable::default(), allocator)?);
        let page_paddr = PhysicalAddress::new(page.addr());

        table.entries[vpn] = Entry::new_branch(page_paddr.ppn());
    }

    // a new entry may have been created above, so we need to re-read the entry
    let entry = table.entries[vpn];

    if level == 0 {
        table.entries[vpn] = Entry::new_leaf(paddr, flags);

        Ok(())
    } else {
        let next = (entry.ppn().into_addr() as *mut PageTable)
            .as_mut()
            .unwrap();

        map(next, allocator, vaddr, paddr, flags, level - 1)
    }
}
