#![feature(allocator_api)]

pub mod collections;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct EntityId {
    pub(crate) gen: u32,
    pub(crate) index: u32,
}

impl EntityId {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn gen(self) -> u32 {
        self.gen
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self { gen: !0, index: !0 }
    }
}

#[cfg(test)]
mod tests {}
