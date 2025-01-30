use crate::PAGE_SIZE;

static mut HEAD: Option<*const u8> = None;

struct FreeList {
    next: Option<*const u8>,
}

pub unsafe fn init() {
    let heap_start = (&crate::heap_start as *const ()).addr();
    let heap_end = (&crate::heap_end as *const ()).addr();

    let heap_len = heap_end - heap_start;

    let pages = heap_len / PAGE_SIZE;

    let last_page = (pages * PAGE_SIZE + heap_start) as *mut u8;

    last_page.cast::<FreeList>().write(FreeList { next: None });

    let mut head = last_page;

    for p in (0..pages).rev() {
        let page = (p * PAGE_SIZE + heap_start) as *mut u8;

        page.cast::<FreeList>().write(FreeList { next: Some(head) });

        head = page;
    }

    HEAD = Some(head);
}

pub fn alloc() -> Option<*mut u8> {
    unsafe {
        match HEAD {
            Some(page) => {
                let free_list = page.cast::<FreeList>();
                HEAD = (*free_list).next;
                Some(page.cast_mut())
            }
            None => None,
        }
    }
}

pub unsafe fn dealloc(page: *mut u8) {
    unsafe {
        let free_list = page.cast::<FreeList>();
        (*free_list).next = HEAD;
        HEAD = Some(page);
    }
}
