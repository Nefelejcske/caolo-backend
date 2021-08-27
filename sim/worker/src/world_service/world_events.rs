use std::collections::HashMap;

use caolo_sim::prelude::*;

use crate::protos::cao_world;

type EventsTables<'a> = (View<'a, EntityId, PositionComponent>,);

pub fn events_payload(
    _out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (_entity_positions,): EventsTables,
) {
    // TODO
}
