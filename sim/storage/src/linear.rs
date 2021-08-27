use std::{alloc, ptr::NonNull};

use crate::AllocError;

pub struct LinearAllocator {
    begin: *mut u8,
    current: *mut u8,
    end: *mut u8,
}

impl LinearAllocator {
    const ALIGNMENT: usize = 16;

    pub fn new(cap: usize) -> Self {
        unsafe {
            let cap = aligned_size(cap);
            let begin = alloc::alloc(alloc::Layout::from_size_align(cap, Self::ALIGNMENT).unwrap());
            let end = begin.add(cap);
            Self {
                begin,
                end,
                current: begin,
            }
        }
    }

    pub fn allocate(&mut self, size: usize) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            let size = aligned_size(size);

            let result = self.current;
            let offset = self.end.offset_from(result);
            if offset < size as isize {
                return Err(AllocError::OutOfMemory);
            }
            self.current = self.current.add(size);

            Ok(NonNull::new(result).unwrap())
        }
    }

    pub fn rewind(&mut self, ptr: NonNull<u8>) {
        self.current = ptr.as_ptr();
    }

    pub fn current(&self) -> NonNull<u8> {
        NonNull::new(self.current).unwrap()
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

pub const fn aligned_size(size: usize) -> usize {
    (size + (LinearAllocator::ALIGNMENT - 1)) & !(LinearAllocator::ALIGNMENT - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_alloc_linear() {
        let mut alloc = LinearAllocator::new(1024);

        let a = alloc.allocate(1024).unwrap();
        alloc.rewind(a);
    }

    #[test]
    fn test_can_alloc_linear_twice() {
        let mut alloc = LinearAllocator::new(2048);

        let a = alloc.allocate(512).unwrap();
        let b = alloc.allocate(512).unwrap();
        alloc.rewind(b);
        alloc.rewind(a);
    }
}
