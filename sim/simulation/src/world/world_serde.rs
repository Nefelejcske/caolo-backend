use serde::{ser::SerializeStruct, Serialize, Serializer};

use crate::prelude::*;

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CaoloWorld", 9)?;
        state.serialize_field("time", &self.time())?;
        // ser entities
        state.serialize_field("bots", &self.entities.iterby_bot().collect::<Vec<_>>())?;
        state.serialize_field(
            "structures",
            &self.entities.iterby_structure().collect::<Vec<_>>(),
        )?;
        state.serialize_field(
            "resources",
            &self.entities.iterby_resource().collect::<Vec<_>>(),
        )?;
        // ser users
        state.serialize_field("users", &self.user.iterby_user().collect::<Vec<_>>())?;
        // ser scripts
        state.serialize_field("scripts", &self.scripts.iterby_cao_ir().collect::<Vec<_>>())?;
        // ser rooms
        state.serialize_field("rooms", &self.room.iterby_rooms().collect::<Vec<_>>())?;
        // ser config
        state.serialize_field(
            "room_properties",
            &self.view::<ConfigKey, RoomProperties>().value,
        )?;
        state.serialize_field("game_config", &self.view::<ConfigKey, GameConfig>().value)?;
        state.end()
    }
}
