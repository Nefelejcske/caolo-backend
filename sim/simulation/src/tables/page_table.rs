mod pt_iter;

use crate::prelude::EntityId;

use self::pt_iter::PTIter;
use std::{mem::MaybeUninit, ptr::drop_in_place};

const PAGE_SIZE: usize = 512;
const PAGE_FLAG_SIZE: usize = PAGE_SIZE / 64;
const PAGE_MASK: usize = PAGE_SIZE - 1; // assumes that page size is power of two

type PageEntry<T> = Option<Box<Page<T>>>;

pub struct PageTable<T> {
    num_entities: usize,
    pages: Vec<PageEntry<T>>,
}

impl<T> Default for PageTable<T> {
    fn default() -> Self {
        Self::new(30_000)
    }
}

impl<T> PageTable<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            num_entities: 0,
            pages: Vec::with_capacity(capacity / PAGE_SIZE),
        }
    }

    pub fn get(&self, index: EntityId) -> Option<&T> {
        let index = index.index();
        let page_index = index as usize / PAGE_SIZE;
        self.pages
            .get(page_index)
            .and_then(|p| p.as_ref())
            .and_then(|page| page.get(index as usize & PAGE_MASK))
    }

    pub fn get_mut(&mut self, index: EntityId) -> Option<&mut T> {
        let index = index.index();
        let page_index = index as usize / PAGE_SIZE;
        self.pages
            .get_mut(page_index)
            .and_then(|p| p.as_mut())
            .and_then(|page| page.get_mut(index as usize & PAGE_MASK))
    }

    pub fn remove(&mut self, index: EntityId) -> Option<T> {
        let index = index.index();
        let page_index = index as usize / PAGE_SIZE;
        self.pages
            .get_mut(page_index)
            .and_then(|p| p.as_mut())
            .and_then(|page| page.remove(index as usize & PAGE_MASK))
            .map(|page| {
                // if removal succeeded
                // TODO if page is empty now, delete the page
                self.num_entities -= 1;
                page
            })
    }

    /// Returns the previous value, if any
    pub fn insert(&mut self, index: EntityId, value: T) -> Option<T> {
        if let Some(existing) = self.get_mut(index) {
            Some(std::mem::replace(existing, value))
        } else {
            self.num_entities += 1;
            let page_ind = index.index() as usize / PAGE_SIZE;
            if page_ind >= self.pages.len() {
                self.pages.resize_with(page_ind + 1, Default::default);
            }
            if self.pages[page_ind].is_none() {
                self.pages[page_ind] = Some(Box::new(Page::new()));
            }
            self.pages[page_ind].as_mut().unwrap().insert(
                index.index() as usize & PAGE_MASK,
                index.gen(),
                value,
            );
            None
        }
    }

    pub fn len(&self) -> usize {
        self.num_entities
    }

    pub fn is_empty(&self) -> bool {
        self.num_entities == 0
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (EntityId, &T)> {
        let it = self
            .pages
            .iter()
            .enumerate()
            .filter_map(|(page_id, page)| page.as_ref().map(|page| (page_id, page)))
            .flat_map(|(page_id, page)| {
                let offset = page_id * PAGE_SIZE;
                page.iter().map(move |(mut id, item)| {
                    id.index += offset as u32;
                    (id, item)
                })
            });
        PTIter::new(it, self.num_entities)
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (EntityId, &mut T)> {
        let it = self
            .pages
            .iter_mut()
            .enumerate()
            .filter_map(|(page_id, page)| page.as_mut().map(|page| (page_id, page)))
            .flat_map(|(page_id, page)| {
                let offset = page_id * PAGE_SIZE;
                page.iter_mut().map(move |(mut id, item)| {
                    id.index += offset as u32;
                    (id, item)
                })
            });
        PTIter::new(it, self.num_entities)
    }

    pub fn clear(&mut self) {
        self.num_entities = 0;
        self.pages.clear();
    }

    pub fn contains(&self, index: EntityId) -> bool {
        let index = index.index();
        let page_index = index as usize / PAGE_SIZE;
        self.pages
            .get(page_index)
            .and_then(|p| p.as_ref())
            .map(|page| page.contains(index as usize & PAGE_MASK))
            .unwrap_or_default()
    }
}

#[repr(align(16))]
struct Page<T> {
    filled: [u64; PAGE_FLAG_SIZE],
    // TODO:
    // pack existing entities tightly, also store their ids
    // iterate on tight arrays...
    // eliminate the existance check during iteration
    data: [MaybeUninit<T>; PAGE_SIZE],
    /// generation of the current component
    gens: [u32; PAGE_SIZE],
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Page<T> {
    const D: MaybeUninit<T> = MaybeUninit::uninit();
    pub fn new() -> Self {
        Self {
            filled: [0; PAGE_FLAG_SIZE],
            data: [Self::D; PAGE_SIZE],
            gens: [0; PAGE_SIZE],
        }
    }

    pub fn contains(&self, id: usize) -> bool {
        self.filled
            .get(id / 64)
            .copied()
            .map(|flags| (flags >> (id & 63)) & 1 == 1)
            .unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        (0..self.data.len()).filter_map(move |i| {
            let flag_idx = i / 64;
            unsafe {
                let flags = self.filled.get_unchecked(flag_idx);
                if (*flags & (1 << (i as u64 & 63))) != 0 {
                    Some((
                        EntityId {
                            index: i as u32,
                            gen: *self.gens.get_unchecked(i),
                        },
                        &*self.data.get_unchecked(i).as_ptr(),
                    ))
                } else {
                    None
                }
            }
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> + '_ {
        (0..self.data.len()).filter_map(move |i| {
            let flag_idx = i / 64;
            unsafe {
                let flags = self.filled.get_unchecked(flag_idx);
                if (*flags & (1 << (i as u64 & 63))) != 0 {
                    Some((
                        EntityId {
                            index: i as u32,
                            gen: *self.gens.get_unchecked(i),
                        },
                        &mut *self.data.get_unchecked_mut(i).as_mut_ptr(),
                    ))
                } else {
                    None
                }
            }
        })
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        debug_assert!(i / 64 < self.filled.len());
        unsafe {
            let flags = self.filled.get_unchecked(i / 64);
            ((flags >> (i & 63)) & 1 == 1).then(|| &*self.data.get_unchecked(i).as_ptr())
        }
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        debug_assert!(i / 64 < self.filled.len());
        unsafe {
            let flags = self.filled.get_unchecked(i / 64);
            ((flags >> (i & 63)) & 1 == 1)
                .then(|| &mut *self.data.get_unchecked_mut(i).as_mut_ptr())
        }
    }

    pub fn insert(&mut self, i: usize, gen: u32, value: T) {
        assert!(
            self.filled
                .get(i / 64)
                .copied()
                .map(|flags| flags >> (i & 63) & 1 == 0)
                .unwrap(),
            "EntityId {} is invalid or occupied",
            i
        );
        let flags = self.filled[i / 64];
        self.filled[i / 64] = flags | (1 << (i & 63));
        unsafe {
            std::ptr::write(self.data[i].as_mut_ptr(), value);
            self.gens[i] = gen;
        }
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        if let Some(flags) = self
            .filled
            .get_mut(i / 64)
            .filter(|flags| (**flags >> (i as u64 & 63)) & 1 == 1)
        {
            *flags ^= 1 << (i & 63);
            let res = unsafe { std::ptr::read(self.data.get_unchecked_mut(i).as_mut_ptr()) };
            Some(res)
        } else {
            None
        }
    }
}

impl<T> Drop for Page<T> {
    fn drop(&mut self) {
        for (i, flags) in self.filled.iter().enumerate() {
            for j in 0..64 {
                if (flags >> j) & 1 == 1 {
                    unsafe {
                        drop_in_place(self.data[i * 64 + j].as_mut_ptr());
                    }
                }
            }
        }
    }
}

impl<T: super::TableRow> crate::tables::Table for PageTable<T> {
    type Id = EntityId;

    type Row = T;

    fn delete(&mut self, id: Self::Id) -> Option<Self::Row> {
        self.remove(id)
    }

    fn get(&self, id: Self::Id) -> Option<&Self::Row> {
        self.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_retrieve() {
        let mut table = PageTable::<i64>::new(1024);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        let id = EntityId { index: 512, gen: 0 };
        table.insert(id, 42);

        assert!(!table.is_empty());
        assert_eq!(table.len(), 1);

        // get and get_mut should be consistent
        assert_eq!(table.contains(id), true);
        assert_eq!(table.get(id).copied(), Some(42));
        assert_eq!(table.get_mut(id).copied(), Some(42));

        assert_eq!(table.get(EntityId { index: 128, gen: 0 }), None);
        assert_eq!(table.contains(EntityId { index: 128, gen: 0 }), false);
    }

    #[test]
    fn test_remove() {
        let mut table = PageTable::<i64>::new(1024);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        let id = EntityId {
            index: 21,
            gen: 523,
        };
        table.insert(id, 42);
        assert_eq!(table.get(id).copied(), Some(42));
        let removed = table.remove(id);
        assert_eq!(removed, Some(42));
        assert_eq!(table.get(id).copied(), None);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_iter() {
        let mut table = PageTable::<i64>::new(0);

        table.insert(EntityId { index: 12, gen: 12 }, 12);
        table.insert(
            EntityId {
                index: 521,
                gen: 12,
            },
            521,
        );
        table.insert(
            EntityId {
                index: 333,
                gen: 12,
            },
            333,
        );
        table.insert(
            EntityId {
                index: 666,
                gen: 12,
            },
            666,
        );

        let it = table.iter();
        assert_eq!(it.len(), table.len());
        assert_eq!(it.len(), 4);
        let actual: Vec<_> = it.collect();

        dbg!(&actual);

        let expected = [12, 333, 521, 666];

        assert_eq!(expected.len(), actual.len());

        for (exp, actual) in expected.iter().copied().zip(actual.iter()) {
            assert_eq!(exp, actual.0.index());
            assert_eq!(exp as i64, *actual.1);
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut table = PageTable::<i64>::new(0);

        table.insert(EntityId { index: 12, gen: 12 }, 12);
        table.insert(
            EntityId {
                index: 521,
                gen: 12,
            },
            521,
        );
        table.insert(
            EntityId {
                index: 333,
                gen: 12,
            },
            333,
        );
        table.insert(
            EntityId {
                index: 666,
                gen: 12,
            },
            666,
        );

        let iter_res: Vec<_> = table.iter().map(|(_x, y)| *y).collect();

        let iter_mut: Vec<_> = table.iter_mut().map(|(_x, y)| *y).collect();
        dbg!(&iter_res, &iter_mut);

        assert_eq!(iter_res.len(), iter_mut.len());

        for (exp, actual) in iter_res.iter().zip(iter_mut.iter()) {
            assert_eq!(exp, actual);
        }
    }
}
