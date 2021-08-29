use std::{
    alloc::{self, AllocError},
    cell::RefCell,
    ptr::NonNull,
    rc::Rc,
};

use crate::linear::{aligned_size, LinearAllocator};

/// Linear allocator wrapper for containers.
///
/// Arenas reserve a chunk of memory and allocate on demand. They rewind the underlying linear
/// allocator when destroyed.
///
/// When an Arena is cloned, both will share the same underlying memory.
/// This memory is returned to the LinearAllocator when all arenas are freed
#[derive(Clone)]
pub struct ArenaAllocator {
    alloc: Rc<RefCell<LinearAllocator>>,
    current: Rc<RefCell<*mut u8>>,
    begin: *mut u8,
    end: *mut u8,
}

impl ArenaAllocator {
    pub fn new(alloc: Rc<RefCell<LinearAllocator>>, cap: usize) -> Result<Self, AllocError> {
        unsafe {
            let begin = alloc.borrow_mut().allocate(cap)?;
            let begin = (*begin.as_ptr()).as_mut_ptr();
            let current = Rc::new(RefCell::new(begin));
            let end = begin.add(cap);
            Ok(Self {
                alloc,
                begin,
                current,
                end,
            })
        }
    }
}

impl Drop for ArenaAllocator {
    fn drop(&mut self) {
        // this is the last arena
        if Rc::strong_count(&self.current) == 1 {
            self.alloc
                .borrow_mut()
                .rewind(NonNull::new(self.begin).unwrap());
        }
    }
}

unsafe impl alloc::Allocator for ArenaAllocator {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = aligned_size(layout.size(), layout.align());
        let mut current = self.current.borrow_mut();
        unsafe {
            if self.end.offset_from(*current) < size as isize {
                return Err(AllocError);
            }
            let result = *current;
            *current = current.add(size);

            Ok(NonNull::new(std::slice::from_raw_parts_mut(result, size) as *mut _).unwrap())
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) {
        // noop
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linalloc_in_vec() {
        let alloc = Rc::new(RefCell::new(LinearAllocator::new(10_000)));
        let alloc = ArenaAllocator::new(alloc, 5000).unwrap();

        let mut v = Vec::with_capacity_in(128, alloc);

        for i in 0..128 {
            v.push(i);
        }
    }
}
