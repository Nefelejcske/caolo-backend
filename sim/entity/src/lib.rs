#![feature(allocator_api)]

pub mod collections;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct EntityId(pub(crate) EntityPl);

impl Default for EntityId {
    fn default() -> Self {
        Self(EntityPl { gen: 0, index: 0 })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EntityPl {
    gen: u32,
    index: u32,
}

#[cfg(test)]
mod tests {}
