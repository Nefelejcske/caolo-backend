/*
 * Memory layout of objects: [Finalizer][Object]...
 */
use std::mem::size_of;
use std::ptr::{drop_in_place, NonNull};

use crate::{
    linear::{aligned_size, LinearAllocator},
    AllocError,
};

struct Finalizer {
    finna: unsafe fn(NonNull<u8>),
    next: Option<NonNull<Finalizer>>,
}

pub struct ScopeStack {
    allocator: NonNull<LinearAllocator>,
    rewind_point: NonNull<u8>,
    fin_stack: Option<NonNull<Finalizer>>,
}

impl ScopeStack {
    pub fn new(allocator: &mut LinearAllocator) -> Self {
        Self {
            rewind_point: allocator.current(),
            allocator: NonNull::new(allocator as *mut _).unwrap(),
            fin_stack: None,
        }
    }

    /// Allocates a new Plain Old Data object
    ///
    /// PODs are assumed not to implement `Drop` thus the `Copy` requirement
    pub fn alloc_pod<T>(&mut self) -> Result<NonNull<T>, AllocError>
    where
        T: Copy,
    {
        unsafe {
            (*self.allocator.as_ptr())
                .allocate(std::mem::size_of::<T>())
                .map(|ptr| NonNull::new_unchecked(ptr.as_ptr() as *mut _))
        }
    }

    /// consumers must initialize the given object
    pub fn alloc_obj<T>(&mut self) -> Result<NonNull<T>, AllocError>
    where
        T: Drop,
    {
        unsafe {
            let result_ptr = (*self.allocator.as_ptr())
                .allocate(aligned_size(size_of::<Finalizer>()) + size_of::<T>())?;
            let result_ptr = result_ptr.as_ptr();
            let fin = result_ptr as *mut Finalizer;
            {
                std::ptr::write(
                    fin,
                    Finalizer {
                        finna: finalizer::<T>,
                        next: self.fin_stack,
                    },
                )
            }
            self.fin_stack = Some(NonNull::new_unchecked(fin));
            let o = result_ptr.add(aligned_size(size_of::<Finalizer>())) as *mut T;
            Ok(NonNull::new_unchecked(o))
        }
    }
}

unsafe fn finalizer<T>(ptr: NonNull<u8>) {
    if std::mem::needs_drop::<T>() {
        let ptr = ptr.as_ptr() as *mut T;
        drop_in_place(ptr);
    }
}

impl Drop for ScopeStack {
    fn drop(&mut self) {
        unsafe {
            let mut f = self.fin_stack;
            while let Some(mut fin) = f {
                let obj =
                    (fin.as_ptr() as *mut u8).add(aligned_size(std::mem::size_of::<Finalizer>()));
                let fin = fin.as_mut();
                let obj = NonNull::new(obj).unwrap();
                (fin.finna)(obj);

                f = fin.next;
            }
            self.allocator.as_mut().rewind(self.rewind_point);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct TestPod {
        _data: [i32; 512],
    }

    struct TestObj {
        res: NonNull<i32>,
    }

    impl Drop for TestObj {
        fn drop(&mut self) {
            unsafe {
                *self.res.as_mut() += 1;
            }
        }
    }

    #[test]
    fn test_pod_alloc() {
        let mut lin = LinearAllocator::new(10_000);
        let mut sc = ScopeStack::new(&mut lin);

        let _a = sc.alloc_pod::<TestPod>().unwrap();
        let _b = sc.alloc_pod::<TestPod>().unwrap();
        let _c = sc.alloc_pod::<TestPod>().unwrap();
        let _d = sc.alloc_pod::<TestPod>().unwrap();
    }

    #[test]
    fn test_obj_alloc() {
        let mut lin = LinearAllocator::new(500);
        let mut sc = ScopeStack::new(&mut lin);

        let mut count = 0;

        for _ in 0..12 {
            unsafe {
                let a = sc.alloc_obj::<TestObj>().unwrap();
                std::ptr::write(
                    a.as_ptr(),
                    TestObj {
                        res: NonNull::new_unchecked((&mut count) as *mut _),
                    },
                );
            }
        }
        assert_eq!(0, count, "Expected count to be untouched");

        drop(sc);

        assert_eq!(12, count);
    }

    #[test]
    fn test_mixed_alloc() {
        let mut lin = LinearAllocator::new(100_000);
        let mut sc = ScopeStack::new(&mut lin);

        let mut count = 0;

        for _ in 0..6 {
            unsafe {
                let a = sc.alloc_obj::<TestObj>().unwrap();
                std::ptr::write(
                    a.as_ptr(),
                    TestObj {
                        res: NonNull::new_unchecked((&mut count) as *mut _),
                    },
                );
                let _b = sc.alloc_pod::<TestPod>().unwrap();
            }
        }
        assert_eq!(0, count, "Expected count to be untouched");

        drop(sc);

        assert_eq!(6, count);
    }
}
