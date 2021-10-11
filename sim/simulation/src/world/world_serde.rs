use std::cell::RefCell;

use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

use crate::prelude::*;

/// serialize only once, will consume the iterator
struct SerializeIter<T: Serialize, It: Iterator<Item = T>> {
    it: RefCell<It>,
    _m: std::marker::PhantomData<T>,
}

impl<T: Serialize, It: Iterator<Item = T>> SerializeIter<T, It> {
    fn new(it: It) -> Self {
        Self {
            it: RefCell::new(it),
            _m: Default::default(),
        }
    }
}

impl<T: Serialize, It: Iterator<Item = T>> Serialize for SerializeIter<T, It> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut it = self.it.borrow_mut();

        let mut state = serializer.serialize_seq(it.size_hint().1)?;

        for item in &mut *it {
            state.serialize_element(&item)?;
        }

        state.end()
    }
}

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CaoloWorld", 9)?;
        state.serialize_field("time", &self.time())?;
        // ser entities
        state.serialize_field("bots", &SerializeIter::new(self.entities.iterby_bot()))?;
        state.serialize_field(
            "structures",
            &SerializeIter::new(self.entities.iterby_structure()),
        )?;
        state.serialize_field(
            "resources",
            &SerializeIter::new(self.entities.iterby_resource()),
        )?;
        // ser users
        state.serialize_field("users", &SerializeIter::new(self.user.iterby_user()))?;
        // ser scripts
        state.serialize_field("scripts", &SerializeIter::new(self.scripts.iterby_cao_ir()))?;
        // ser rooms
        state.serialize_field("rooms", &SerializeIter::new(self.room.iterby_rooms()))?;
        // ser config
        state.serialize_field(
            "room_properties",
            &self.view::<ConfigKey, RoomProperties>().value,
        )?;
        state.serialize_field("game_config", &self.view::<ConfigKey, GameConfig>().value)?;
        state.end()
    }
}
