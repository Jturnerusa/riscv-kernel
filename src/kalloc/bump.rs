use core::{alloc::AllocError, cell::Cell, marker::PhantomData, ptr::NonNull};

pub struct Allocator<'a> {
    ptr: *mut u8,
    len: usize,
    bp: Cell<usize>,
    allocations: Cell<usize>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Allocator<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            ptr: buf.as_mut_ptr(),
            len: buf.len(),
            bp: Cell::new(0),
            allocations: Cell::new(0),
            phantom: PhantomData,
        }
    }
}

unsafe impl<'a> core::alloc::Allocator for Allocator<'a> {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() == 0 {
            return Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0));
        }

        let p = unsafe { self.ptr.add(self.bp.get()) };
        let left = self.len - self.bp.get();
        let needed = p.align_offset(layout.align()) + layout.size();

        if needed > left {
            return Err(AllocError);
        }

        let aligned_ptr = unsafe { NonNull::new(p.add(p.align_offset(layout.align()))).unwrap() };

        debug_assert!(aligned_ptr.is_aligned_to(layout.align()));

        let slice = NonNull::slice_from_raw_parts(aligned_ptr, layout.size());

        self.bp.set(self.bp.get().checked_add(needed).unwrap());
        self.allocations
            .set(self.allocations.get().checked_add(1).unwrap());

        Ok(slice)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: core::alloc::Layout) {
        self.allocations
            .set(self.allocations.get().checked_sub(1).unwrap());

        if self.allocations.get() == 0 {
            self.bp.set(0);
        }
    }
}
