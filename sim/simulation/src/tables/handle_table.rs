use std::{
    alloc::{alloc, dealloc, Layout},
    mem::{align_of, size_of},
    ptr,
};

use crate::indices::EntityId;

pub struct HandleTable {
    entries: *mut Entry,
    cap: u32,
    free_list: u32,
}

unsafe impl Send for HandleTable {}
unsafe impl Sync for HandleTable {}

const SENTINEL: u32 = !0;

impl HandleTable {
    pub fn new(cap: u32) -> Self {
        let entries;
        unsafe {
            entries = alloc(Layout::from_size_align_unchecked(
                size_of::<Entry>() * (cap as usize + 1),
                align_of::<Entry>(),
            )) as *mut Entry;
            assert!(!entries.is_null());
            for i in 0..cap {
                ptr::write(
                    entries.add(i as usize),
                    Entry {
                        data: i + 1,
                        gen: 0,
                    },
                );
            }
            ptr::write(
                entries.add(cap as usize),
                Entry {
                    data: SENTINEL,
                    gen: SENTINEL,
                },
            );
        };
        Self {
            entries,
            cap,
            free_list: 0,
        }
    }

    pub fn alloc(&mut self) -> EntityId {
        unsafe {
            let entries = self.entries;

            // pop element off the free list
            assert!(self.free_list != SENTINEL); // TODO: return result?
            let index = self.free_list;
            self.free_list = (*entries.add(self.free_list as usize)).data;

            // create handle
            let entry = &mut *entries.add(index as usize);
            entry.data = 0;
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

    pub fn get_at_index(&self, ind: u32) -> EntityId {
        let entry = self.entries()[ind as usize];
        EntityId {
            gen: entry.gen,
            index: ind,
        }
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
            dealloc(
                self.entries as *mut u8,
                Layout::from_size_align_unchecked(
                    size_of::<Entry>() * (self.cap as usize + 1),
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
    use super::*;

    #[test]
    fn test_with_cao_arena() {
        let mut table = HandleTable::new(512);

        for i in 0..4 {
            let _e = table.alloc();
            dbg!(i, _e);
        }
        for i in 0..4 {
            let e = EntityId { gen: 0, index: i };
            table.free(e);
        }
        for _ in 0..512 {
            let _e = table.alloc();
        }
    }
}
