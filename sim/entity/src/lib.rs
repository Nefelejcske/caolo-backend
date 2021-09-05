#![feature(allocator_api)]

pub mod collections;
pub mod world;

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

impl From<u64> for EntityId {
    fn from(i: u64) -> Self {
        Self {
            gen: (i >> 32) as u32,
            index: i as u32,
        }
    }
}

impl From<EntityId> for u64 {
    fn from(id: EntityId) -> Self {
        (id.gen as u64) << 32 | id.index as u64
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self { gen: !0, index: !0 }
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}:{}", self.gen, self.index)
    }
}

#[cfg(test)]
mod tests {
    use crate::EntityId;

    #[test]
    fn entity_id_cast_u64_consistent() {
        let a = EntityId {
            gen: 42,
            index: 696969,
        };

        let id: u64 = a.into();
        let b: EntityId = id.into();

        assert_eq!(a, b);
    }
}
