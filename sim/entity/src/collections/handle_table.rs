use std::{
    alloc::{AllocError, Allocator, Layout},
    mem::{align_of, size_of},
    ptr::NonNull,
};

use crate::EntityId;

pub struct HandleTable {
    entries: *mut Entry,
    cap: u32,
    free_list: u32,
    alloc: Box<dyn Allocator>,
}

const SENTINEL: u32 = !0;

impl HandleTable {
    pub fn new(cap: u32, alloc: Box<dyn Allocator>) -> Result<Self, AllocError> {
        let entries = unsafe {
            let entries = alloc.allocate(Layout::from_size_align_unchecked(
                cap as usize * size_of::<Entry>() + 1,
                align_of::<Entry>(),
            ))?;
            let entries = entries.as_ptr() as *mut Entry;
            {
                let mut entries = entries;
                for i in 0..cap {
                    let entry = &mut *entries;
                    entry.data = i as u32 + 1;
                    entry.gen = 0;
                    entries = entries.add(1);
                }
                (*entries).data = SENTINEL;
                (*entries).gen = SENTINEL;
            }
            entries
        };
        Ok(Self {
            entries,
            cap,
            free_list: 0,
            alloc,
        })
    }

    pub fn alloc(&mut self, data: u32) -> EntityId {
        unsafe {
            let entries = self.entries;

            // pop element off the free list
            assert!(self.free_list != SENTINEL); // TODO: return result?
            let index = self.free_list;
            self.free_list = (*entries.add(self.free_list as usize)).data;

            // create handle
            let entry = &mut *entries.add(index as usize);
            entry.data = data;
            EntityId {
                index,
                gen: entry.gen,
            }
        }
    }

    pub fn free(&mut self, id: EntityId) {
        unsafe {
            let entries = self.entries;

            let index = id.index;
            let entry = &mut *entries.add(index as usize);
            entry.data = self.free_list;
            entry.gen += 1;
            self.free_list = index;
        }
    }

    pub fn look_up(&self, id: EntityId) -> u32 {
        let index = id.index as usize;
        let count = id.gen;
        // TODO: return result?
        assert!(self.entries()[index].gen == count);
        return self.entries()[index].data;
    }

    pub fn update(&mut self, id: EntityId, data: u32) {
        unsafe {
            let index = id.index as usize;
            let gen = id.gen;
            // TODO: return result?
            assert!(self.entries()[index].gen == gen);
            let entries = self.entries;
            (*entries.add(index)).data = data;
        }
    }

    pub fn is_valid(&self, id: EntityId) -> bool {
        let index = id.index as usize;
        let gen = id.gen;
        if index >= self.cap as usize {
            return false;
        }
        self.entries()[index].gen == gen
    }

    fn entries(&self) -> &[Entry] {
        unsafe { std::slice::from_raw_parts(self.entries, self.cap as usize) }
    }
}

impl Drop for HandleTable {
    fn drop(&mut self) {
        unsafe {
            self.alloc.deallocate(
                NonNull::new_unchecked(self.entries as *mut u8),
                Layout::from_size_align_unchecked(
                    self.cap as usize * size_of::<Entry>() + 1,
                    align_of::<Entry>(),
                ),
            );
        }
    }
}

#[derive(Clone, Copy)]
struct Entry {
    data: u32,
    gen: u32,
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_with_cao_arena() {
        let alloc = cao_alloc::linear::LinearAllocator::new(1024 * 8);
        let arena =
            cao_alloc::arena::ArenaAllocator::new(Rc::new(RefCell::new(alloc)), 1024 * 8).unwrap();

        let mut table = HandleTable::new(512, Box::new(arena)).unwrap();

        for i in 0..4 {
            let _e = table.alloc(42);
            dbg!(i, _e);
        }
        for i in 0..4 {
            let e = EntityId { gen: 0, index: i };
            table.free(e);
        }
        for _ in 0..512 {
            let _e = table.alloc(42);
        }
    }
}
