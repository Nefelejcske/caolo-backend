use super::*;
use crate::components::{EntityComponent, PositionComponent};
use crate::indices::WorldPosition;
use crate::profile;
use crate::world::World;
use cao_lang::{prelude::*, StrPointer};
use std::convert::TryFrom;
use tracing::{trace, warn};

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum FindConstant {
    Resource = 1,
    Spawn = 2,
    EnemyBot = 3,
}

impl TryFrom<Value> for FindConstant {
    type Error = Value;
    fn try_from(i: Value) -> Result<Self, Value> {
        let op = match i {
            Value::Integer(1) => FindConstant::Resource,
            Value::Integer(2) => FindConstant::Spawn,
            Value::Integer(3) => FindConstant::EnemyBot,
            _ => return Err(i),
        };
        Ok(op)
    }
}

pub fn parse_find_constant(
    vm: &mut Vm<ScriptExecutionData>,
    param: StrPointer,
) -> Result<(), ExecutionError> {
    profile!("parse_find_constant");
    trace!("parse_find_constant");
    let param = unsafe {
        param.get_str().ok_or_else(|| {
            trace!("parse_find_constant called with invalid param");
            ExecutionError::invalid_argument(
                "parse_find_constant called with non-string param".to_owned(),
            )
        })?
    };
    let constant = match param {
        "resource" | "RESOURCE" | "Resource" => FindConstant::Resource,
        "spawn" | "SPAWN" | "Spawn" => FindConstant::Spawn,
        "enemy_bot" | "ENEMY_BOT" | "EnemyBot" => FindConstant::EnemyBot,
        _ => {
            trace!(
                "parse_find_constant got an invalid constant value {}",
                param
            );
            return Err(ExecutionError::invalid_argument(format!(
                "parse_find_constant got in invalid constant value {}",
                param
            )));
        }
    };
    vm.stack_push(constant as i64)?;
    Ok(())
}

/// Return OperationResult and an EntityId if the Operation succeeded
pub fn find_closest_by_range(
    vm: &mut Vm<ScriptExecutionData>,
    param: FindConstant,
) -> Result<(), ExecutionError> {
    profile!("find_closest_by_range");

    let aux = vm.get_aux();
    let entity_id = aux.entity_id;

    let s = tracing::warn_span!(
        "find_closest_by_range",
        entity_id = entity_id.to_string().as_str()
    );
    let _e = s.enter();

    trace!("find_closest_by_range {:?}", param);

    let position = match vm
        .get_aux()
        .storage()
        .view::<EntityId, PositionComponent>()
        .get(entity_id)
    {
        Some(p) => p.0,
        None => {
            warn!("{:?} has no PositionComponent", entity_id);
            return Err(ExecutionError::InvalidArgument { context: None });
        }
    };

    trace!("Executing find_closest_by_range {:?}", position);

    param.execute(vm, position)
}

impl FindConstant {
    pub fn execute(
        self,
        vm: &mut Vm<ScriptExecutionData>,
        position: WorldPosition,
    ) -> Result<(), ExecutionError> {
        trace!("Executing find {:?}", self);

        let storage = vm.get_aux().storage();
        let user_id = vm.get_aux().user_id;
        let candidate = match self {
            FindConstant::Resource => {
                let resources = storage.view::<EntityId, components::ResourceComponent>();
                find_closest_entity_impl(storage, position, |id| resources.contains(id))
            }
            FindConstant::Spawn => {
                let owner = storage.view::<EntityId, components::OwnedEntity>();
                let spawns = storage.view::<EntityId, components::SpawnComponent>();
                find_closest_entity_impl(storage, position, |id| {
                    spawns.contains(id)
                        && owner.get(id).map(|owner_id| owner_id.owner_id) == user_id
                })
            }
            FindConstant::EnemyBot => {
                let owner = storage.view::<EntityId, components::OwnedEntity>();
                let bots = storage.view::<EntityId, components::Bot>();
                find_closest_entity_impl(storage, position, |id| {
                    bots.contains(&id) && owner.get(id).map(|owner_id| owner_id.owner_id) != user_id
                })
            }
        }?;
        match candidate {
            Some(entity) => {
                tracing::debug!("Found entity {:?}", entity);
                let id: u64 = entity.into();
                vm.stack_push(id as i64)?;
            }
            None => {
                trace!("No stuff was found");
                vm.stack_push(Value::Nil)?;
            }
        }
        Ok(())
    }
}

fn find_closest_entity_impl<F>(
    storage: &World,
    position: WorldPosition,
    filter: F,
) -> Result<Option<EntityId>, ExecutionError>
where
    F: Fn(EntityId) -> bool,
{
    let WorldPosition { room, pos } = position;
    let entities_by_pos = storage.view::<WorldPosition, EntityComponent>();

    let room = entities_by_pos
        .table
        .at(room)
        .ok_or_else(|| ExecutionError::InvalidArgument {
            context: "find_closest_by_range called on invalid room"
                .to_string()
                .into(),
        })?;

    // search the whole room
    let candidate = room.find_closest_by_filter(pos, |_, entity| filter(entity.0));
    let candidate = candidate.map(|(_, _, EntityComponent(id))| *id);
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use crate::{systems::script_execution::get_alloc, world::World};
    use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

    fn init_resource_storage(
        entity_id: EntityId,
        center_pos: WorldPosition,
        expected_id: EntityId,
        expected_pos: WorldPosition,
    ) -> World {
        let mut seed = [0; 32];
        thread_rng().fill(&mut seed);
        let mut rng = SmallRng::from_seed(seed);

        let mut storage = World::new();

        let mut entity_positions = storage.unsafe_view::<EntityId, PositionComponent>();
        let mut position_entities = storage.unsafe_view::<WorldPosition, EntityComponent>();

        position_entities
            .insert(expected_pos, EntityComponent(expected_id))
            .expect("Initial insert 2");

        for _ in 0..128 {
            let id = storage.insert_entity();
            let pos = loop {
                let q = rng.gen_range(0..256);
                let r = rng.gen_range(0..256);

                let pos = Axial::new(q, r);
                if center_pos.pos.hex_distance(pos) > center_pos.pos.hex_distance(expected_pos.pos)
                {
                    break pos;
                }
            };
            position_entities
                .insert(
                    WorldPosition {
                        room: Axial::new(0, 0),
                        pos,
                    },
                    EntityComponent(id),
                )
                .expect("Initial insert 3");
        }

        // make every one of these a resource
        for (_, entity_id) in position_entities.iter_rooms().flat_map(|(room_id, room)| {
            room.iter().map(move |(pos, item)| {
                (
                    WorldPosition {
                        room: room_id.0,
                        pos,
                    },
                    item,
                )
            })
        }) {
            storage
                .unsafe_view::<EntityId, components::ResourceComponent>()
                .insert(
                    entity_id.0,
                    components::ResourceComponent(components::Resource::Energy),
                );
        }

        // the querying entity is not a resource

        entity_positions.insert(entity_id, PositionComponent(center_pos));
        position_entities
            .insert(center_pos, EntityComponent(entity_id))
            .expect("Initial insert 1");
        storage
    }

    #[test]
    fn finds_closest_returns_itself_when_appropriate() {
        let entity_id = EntityId::new(1024, 0);
        let center_pos = WorldPosition {
            room: Axial::new(0, 0),
            pos: Axial::new(14, 14),
        };

        let expected_id = EntityId::new(2040, 0);
        let expected_pos = WorldPosition {
            room: Axial::new(0, 0),
            pos: Axial::new(69, 69),
        };

        let mut storage = init_resource_storage(entity_id, center_pos, expected_id, expected_pos);
        storage
            .unsafe_view::<EntityId, components::ResourceComponent>()
            .insert(
                entity_id,
                components::ResourceComponent(components::Resource::Energy),
            );
        let data =
            ScriptExecutionData::new(&storage, Default::default(), entity_id, None, get_alloc());
        let mut vm = Vm::new(data).unwrap();

        let constant = FindConstant::Resource;

        find_closest_by_range(&mut vm, constant).expect("find_closest_by_range exec");

        let res_id = vm.stack_pop();
        if let Value::Integer(p) = res_id {
            let res_id = <EntityId as From<u64>>::from(p.try_into().unwrap());

            assert_eq!(res_id, entity_id);
        } else {
            panic!("Expected pointer, got {:?}", res_id);
        }
    }

    #[test]
    fn finds_closest_resources_as_expected() {
        let entity_id = EntityId::new(1024, 0);
        let center_pos = WorldPosition {
            room: Axial::new(0, 0),
            pos: Axial::new(14, 14),
        };

        let expected_id = EntityId::new(2040, 0);
        let expected_pos = WorldPosition {
            room: Axial::new(0, 0),
            pos: Axial::new(69, 69),
        };

        let storage = init_resource_storage(entity_id, center_pos, expected_id, expected_pos);
        let data = ScriptExecutionData::new(
            &storage,
            Default::default(),
            entity_id,
            Default::default(),
            get_alloc(),
        );
        let mut vm = Vm::new(data).unwrap();

        let constant = FindConstant::Resource;

        find_closest_by_range(&mut vm, constant).expect("find_closest_by_range exec");

        let res_id = vm.stack_pop();
        if let Value::Integer(p) = res_id {
            let res_id = EntityId::new(p.try_into().expect("expected entity id"), 0);

            assert_eq!(res_id, expected_id);
        } else {
            panic!("Expected pointer, got {:?}", res_id);
        }
    }
}
