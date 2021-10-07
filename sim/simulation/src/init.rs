use crate::prelude::*;
use cao_lang::{compiler::CompileOptions, prelude::*};
use rand::Rng;
use tracing::{debug, trace};
use uuid::Uuid;

/// World should be already initialized with a GameConfig
pub fn init_world_entities(storage: &mut World, n_fake_users: usize) {
    debug!("initializing world");

    let mut rng = rand::thread_rng();

    let mining_script_id = ScriptId(Uuid::new_v4());
    let script: CaoIr = serde_yaml::from_str(include_str!("./programs/mining_program.yaml"))
        .expect("deserialize example program");
    debug!("compiling default program");
    let compiled =
        compile(&script, CompileOptions::new()).expect("failed to compile example program");
    debug!("compilation done");

    crate::query!(
        mutate
        storage
        {
            ScriptId, CompiledScriptComponent,
                .insert(mining_script_id, CompiledScriptComponent(compiled));
        }
    );

    let config = UnwrapView::<ConfigKey, GameConfig>::from_world(storage);

    let radius = config.room_radius;
    debug!("Reset position storage");
    let mut entities_by_pos = storage.unsafe_view::<WorldPosition, EntityComponent>();
    entities_by_pos.clear();
    entities_by_pos
        .table
        .extend(
            storage
                .view::<Axial, RoomComponent>()
                .iter()
                .map(|(roomid, _)| (roomid, Default::default())),
        )
        .expect("entities_by_pos init");
    let bounds = Hexagon {
        center: Axial::new(radius as i32, radius as i32),
        radius: radius as i32,
    };
    let rooms = storage
        .view::<Axial, RoomComponent>()
        .iter()
        .map(|a| a.0)
        .collect::<Vec<_>>();

    let mut taken_rooms = Vec::with_capacity(n_fake_users as usize);
    for i in 0..n_fake_users {
        trace!("initializing room #{}", i);
        let spawnid = storage.insert_entity();

        let room = rng.gen_range(0..rooms.len());
        let room = rooms[room];
        taken_rooms.push(room);

        trace!("initializing room #{} in room {:?}", i, room);
        let user_id = Uuid::new_v4();
        init_spawn(&bounds, spawnid, user_id, Room(room), &mut rng, storage);
        trace!("spawning entities");
        storage
            .unsafe_view::<UserId, EntityScript>()
            .insert(UserId(user_id), EntityScript(mining_script_id));
        let id = storage.insert_entity();
        let pos = uncontested_pos(
            Room(room),
            &bounds,
            &*storage.view::<WorldPosition, EntityComponent>(),
            &*storage.view::<WorldPosition, TerrainComponent>(),
            &mut rng,
        );

        crate::entity_archetypes::init_resource_energy(
            id,
            Room(room),
            pos,
            FromWorldMut::from_world_mut(storage),
            FromWorld::from_world(storage),
        );
        trace!("initializing room #{} done", i);
    }

    debug!("init done");
}

#[allow(clippy::too_many_arguments)] // its just a helper function let it be
fn init_spawn(
    bounds: &Hexagon,
    id: EntityId,
    owner_id: Uuid,
    room: Room,
    rng: &mut impl Rng,
    world: &mut World,
) {
    trace!("init_spawn");
    let pos = uncontested_pos(
        room,
        bounds,
        &*world.view::<WorldPosition, EntityComponent>(),
        &*world.view::<WorldPosition, TerrainComponent>(),
        rng,
    );

    crate::entity_archetypes::init_structure_spawn(id, owner_id, pos, world);
    trace!("init_spawn done");
}

fn uncontested_pos<T: crate::tables::TableRow + Send + Sync + Default>(
    room: Room,
    bounds: &Hexagon,
    positions_table: &crate::tables::morton_hierarchy::MortonMortonTable<T>,
    terrain_table: &<TerrainComponent as Component<WorldPosition>>::Table,
    rng: &mut impl Rng,
) -> WorldPosition {
    const TRIES: usize = 10_000;
    let from = bounds.center - Axial::new(bounds.radius, bounds.radius);
    let to = bounds.center + Axial::new(bounds.radius, bounds.radius);

    let room_positions = positions_table
        .table
        .at(room.0)
        .expect("Given room is missing from positions table");
    let room_terrain = terrain_table
        .table
        .at(room.0)
        .expect("Given room is missing from terrain table");

    for _ in 0..TRIES {
        let x = rng.gen_range(from.q..to.q);
        let y = rng.gen_range(from.r..to.r);

        let pos = Axial::new(x, y);

        trace!("checking pos {:?}", pos);

        if !bounds.contains(pos) {
            trace!("point {:?} is out of bounds {:?}", pos, bounds);
            continue;
        }

        if let Some(TerrainComponent(terrain)) = room_terrain.get(pos) {
            if terrain.is_walkable() && !room_positions.contains_key(pos) {
                return WorldPosition { room: room.0, pos };
            }
        }
    }
    panic!(
        "Failed to find an uncontested_pos in {:?} {:?} in {} iterations",
        from, to, TRIES
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_env_log::test;

    #[test]
    fn can_init_the_game() {
        let mut exc = SimpleExecutor;
        let mut world =
            futures_lite::future::block_on(exc.initialize(crate::executor::GameConfig {
                world_radius: 2,
                room_radius: 10,
                ..Default::default()
            }));

        // smoke test: can the game be even initialized?
        init_world_entities(&mut world, 12);
    }
}
