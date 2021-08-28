use std::{
    alloc::{self, AllocError},
    cell::RefCell,
    ptr::NonNull,
    rc::Rc,
};

pub struct LinearAllocator {
    begin: *mut u8,
    current: *mut u8,
    end: *mut u8,
}

impl LinearAllocator {
    const ALIGNMENT: usize = 16;

    pub fn new(cap: usize) -> Self {
        unsafe {
            let cap = aligned_size(cap, 16);
            let begin = alloc::alloc(alloc::Layout::from_size_align(cap, Self::ALIGNMENT).unwrap());
            let end = begin.add(cap);
            Self {
                begin,
                end,
                current: begin,
            }
        }
    }

    /// # Safety
    ///
    /// For each `allocate` a corresponding [rewind](LinearAllocator::rewind) must be called!
    /// Rewind calls must be performed in the opposite order as the `allocate` calls!
    pub unsafe fn allocate(&mut self, size: usize) -> Result<NonNull<[u8]>, AllocError> {
        let size = aligned_size(size, 16);

        let result = self.current;
        let offset = self.end.offset_from(result);
        if offset < size as isize {
            return Err(AllocError);
        }
        self.current = self.current.add(size);

        Ok(NonNull::new(std::slice::from_raw_parts_mut(result, size) as *mut _).unwrap())
    }

    pub fn rewind(&mut self, ptr: NonNull<u8>) {
        unsafe {
            assert!(self.current.offset_from(ptr.as_ptr()) >= 0
                , "Trying to rewind to a point in front of `current`! This indicates that memory was freed in the wrong order!");
            self.current = ptr.as_ptr();
        }
    }

    pub fn current(&self) -> NonNull<u8> {
        NonNull::new(self.current).unwrap()
    }
}

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

impl Drop for LinearAllocator {
    fn drop(&mut self) {
        debug_assert!(
            self.begin == self.current,
            "Some memory has not been returned to the allocator"
        );

        unsafe {
            let size = self.end.offset_from(self.begin);
            alloc::dealloc(
                self.begin,
                alloc::Layout::from_size_align(size as usize, Self::ALIGNMENT).unwrap(),
            );
        }
    }
}

#[inline]
pub fn aligned_size(size: usize, alignment: usize) -> usize {
    debug_assert!(
        (alignment & (alignment - 1)) == 0,
        "Expected powers of two alignment"
    );
    (size + (alignment - 1)) & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_alloc_linear() {
        let mut alloc = LinearAllocator::new(1024);

        unsafe {
            let a = alloc.allocate(1024).unwrap();
            alloc.rewind(NonNull::new_unchecked((*a.as_ptr()).as_mut_ptr()));
        }
    }

    #[test]
    fn test_can_alloc_linear_twice() {
        let mut alloc = LinearAllocator::new(2048);

        unsafe {
            let a = alloc.allocate(512).unwrap();
            let b = alloc.allocate(512).unwrap();
            alloc.rewind(NonNull::new_unchecked((*b.as_ptr()).as_mut_ptr()));
            alloc.rewind(NonNull::new_unchecked((*a.as_ptr()).as_mut_ptr()));
        }
    }

    #[test]
    #[should_panic]
    fn linear_bad_rewind_is_panic() {
        let mut alloc = LinearAllocator::new(2048);

        unsafe {
            let a = alloc.allocate(512).unwrap();
            let b = alloc.allocate(512).unwrap();
            // bad order, b should come first
            alloc.rewind(NonNull::new_unchecked((*a.as_ptr()).as_mut_ptr()));
            alloc.rewind(NonNull::new_unchecked((*b.as_ptr()).as_mut_ptr()));
        }
    }

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
