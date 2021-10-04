//! Structs intended to be used as table indices.
//!

pub mod entity_id;

use crate::empty_key;
use crate::geometry::Axial;
use serde::{Deserialize, Serialize};
use std::ops::Add;

pub use entity_id::EntityId;

#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct EntityTime(pub EntityId, pub u64);

#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct IntentId(pub u32);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct ScriptId(pub uuid::Uuid);

#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct UserId(pub uuid::Uuid);

#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct WorldPosition {
    pub room: Axial,
    #[serde(rename = "roomPos")]
    pub pos: Axial,
}

/// Newtype wrapper around Axial point for positions that are inside a room.
#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct RoomPosition(pub Axial);

/// Newtype wrapper around Axial point for room ids.
#[derive(
    Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize, Deserialize,
)]
pub struct Room(pub Axial);

impl Room {
    pub fn as_array(self) -> [i32; 2] {
        self.0.as_array()
    }

    pub fn new(x: i32, y: i32) -> Self {
        Self(Axial::new(x, y))
    }

    pub fn dist(self, Room(other): Self) -> u32 {
        self.0.dist(other)
    }
}

impl Add for Room {
    type Output = Self;

    fn add(self, Room(b): Self) -> Self {
        Self(self.0.add(b))
    }
}

unsafe impl Send for Room {}

unsafe impl Send for UserId {}
unsafe impl Send for EntityId {}
unsafe impl Send for ScriptId {}

// Identify config tables
empty_key!(ConfigKey);

// Storage key for unindexed tables.
empty_key!(EmptyKey);
