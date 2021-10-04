use cao_lang::prelude::Value;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, mem::size_of};

pub type Index = u32;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub(crate) index: Index,
    pub(crate) gen: u32,
}

impl EntityId {
    pub fn new(index: Index, gen: u32) -> Self {
        Self { gen, index }
    }

    pub fn index(self) -> Index {
        self.index
    }

    pub fn gen(self) -> u32 {
        self.gen
    }
}

impl From<u64> for EntityId {
    fn from(i: u64) -> Self {
        debug_assert!(size_of::<u32>() == size_of::<Index>());

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

impl TryFrom<Value> for EntityId {
    type Error = Value;
    fn try_from(s: Value) -> Result<EntityId, Value> {
        match s {
            Value::Integer(i) => {
                if i < 0 {
                    return Err(s);
                }
                Ok((i as u64).into())
            }
            _ => Err(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EntityId;

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
