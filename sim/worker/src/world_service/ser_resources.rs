use std::collections::HashMap;

use super::util::push_room_pl;
use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

type ResourceTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, Axial, RoomComponent>,
    View<'a, EntityId, ResourceComponent>,
    View<'a, EntityId, EnergyComponent>,
    WorldTime,
);

pub fn resource_payload(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (room_entities, rooms, resource, energy, WorldTime(time)): ResourceTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut offset = None;
    let mut accumulator = Vec::with_capacity(128);

    for (next_room, entities) in room_entities {
        // push the accumulator
        if Some(next_room) != room {
            if !accumulator.is_empty() {
                debug_assert!(room.is_some());
                push_room_pl(
                    out,
                    room.unwrap().0,
                    |pl| &mut pl.resources,
                    std::mem::take(&mut accumulator),
                    time as i64,
                );
            }
            room = Some(next_room);
            offset = rooms.get_by_id(next_room.0).map(|x| x.offset);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            let entity_id = *entity_id;
            if let Some(resource) = resource.get_by_id(entity_id) {
                match resource.0 {
                    Resource::Empty => {}
                    Resource::Energy => {
                        accumulator.push(cao_world::Resource {
                            id: entity_id.0.into(),

                            pos: Some(cao_common::WorldPosition {
                                pos: Some(pos.into()),
                                room: room.map(|x| x.0.into()),
                                offset: offset.map(|x| x.into()),
                            }),
                            resource_type: energy.get_by_id(entity_id).copied().map(
                                |EnergyComponent { energy, energy_max }: EnergyComponent| {
                                    cao_world::resource::ResourceType::Energy(cao_world::Bounded {
                                        value: energy.into(),
                                        value_max: energy_max.into(),
                                    })
                                },
                            ),
                        });
                    }
                }
            }
        }
    }
    // push the last accumulator
    if let Some(ref room) = (!accumulator.is_empty()).then(|| ()).and(room) {
        push_room_pl(
            out,
            room.0,
            |pl| &mut pl.resources,
            accumulator,
            time as i64,
        );
    }
}
