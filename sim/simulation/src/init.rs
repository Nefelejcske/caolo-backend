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
    let script: CompilationUnit =
        serde_json::from_str(include_str!("./programs/mining_program.json"))
            .expect("deserialize example program");
    debug!("compiling default program");
    let compiled = compile(script, CompileOptions::new().with_breadcrumbs(false))
        .expect("failed to compile example program");
    debug!("compilation done");

    crate::query!(
        mutate
        storage
        {
            ScriptId, ScriptComponent,
                .insert_or_update(mining_script_id, ScriptComponent(compiled));
        }
    );

    let center_walking_script_id = ScriptId(Uuid::new_v4());
    let script: CompilationUnit =
        serde_json::from_str(include_str!("./programs/center_walking_program.json"))
            .expect("deserialize example program");
    debug!("compiling default program");
    let compiled = compile(script, CompileOptions::new().with_breadcrumbs(false))
        .expect("failed to compile example program");
    debug!("compilation done");

    crate::query!(
        mutate
        storage
        {
            ScriptId, ScriptComponent,
                .insert_or_update(center_walking_script_id, ScriptComponent(compiled));
        }
    );

    let config = UnwrapView::<ConfigKey, GameConfig>::new(storage);

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

        let room = rng.gen_range(0, rooms.len());
        let room = rooms[room];
        taken_rooms.push(room);

        trace!("initializing room #{} in room {:?}", i, room);
        let user_id = Uuid::new_v4();
        init_spawn(&bounds, spawnid, user_id, Room(room), &mut rng, storage);
        trace!("spawning entities");
        storage
            .unsafe_view::<UserId, EntityScript>()
            .insert_or_update(UserId(user_id), EntityScript(center_walking_script_id));
        let spawn_pos = storage
            .view::<EntityId, PositionComponent>()
            .get_by_id(spawnid)
            .expect("spawn should have position")
            .0;
        for _ in 0..2 {
            let botid = storage.insert_entity();
            init_bot(
                botid,
                mining_script_id,
                user_id,
                spawn_pos,
                FromWorldMut::new(storage),
            );
        }
        for _ in 0..2 {
            let botid = storage.insert_entity();
            init_bot(
                botid,
                center_walking_script_id,
                user_id,
                spawn_pos,
                FromWorldMut::new(storage),
            );
        }
        let id = storage.insert_entity();
        init_resource(
            &bounds,
            id,
            Room(room),
            &mut rng,
            FromWorldMut::new(storage),
            FromWorld::new(storage),
        );
        trace!("initializing room #{} done", i);
    }

    debug!("init done");
}

type InitBotMuts = (
    UnsafeView<EntityId, MeleeAttackComponent>,
    UnsafeView<EntityId, HpComponent>,
    UnsafeView<EntityId, EntityScript>,
    UnsafeView<EntityId, Bot>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, OwnedEntity>,
    UnsafeView<EntityId, PositionComponent>,
    UnsafeView<WorldPosition, EntityComponent>,
);

fn init_bot(
    id: EntityId,
    script_id: ScriptId,
    owner_id: Uuid,
    pos: WorldPosition,
    (
        mut melee,
        mut hp,
        mut entity_scripts,
        mut bots,
        mut carry_component,
        mut owners,
        mut positions,
        mut entities_by_pos,
    ): InitBotMuts,
) {
    entity_scripts.insert_or_update(id, EntityScript(script_id));
    bots.insert(id);
    carry_component.insert_or_update(
        id,
        CarryComponent {
            carry: 0,
            carry_max: 50,
        },
    );
    owners.insert_or_update(
        id,
        OwnedEntity {
            owner_id: UserId(owner_id),
        },
    );

    positions.insert_or_update(id, PositionComponent(pos));
    entities_by_pos
        .table
        .at_mut(pos.room)
        .expect("expected bot pos to be in the table")
        .insert(pos.pos, EntityComponent(id))
        .expect("entities_by_pos insert");

    melee.insert_or_update(id, MeleeAttackComponent { strength: 5 });
    hp.insert_or_update(id, HpComponent { hp: 50, hp_max: 50 });
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

type InitResourceMuts = (
    UnsafeView<EntityId, PositionComponent>,
    UnsafeView<EntityId, ResourceComponent>,
    UnsafeView<EntityId, EnergyComponent>,
    UnsafeView<WorldPosition, EntityComponent>,
);

type InitResourceConst<'a> = (View<'a, WorldPosition, TerrainComponent>,);

fn init_resource(
    bounds: &Hexagon,
    id: EntityId,
    room: Room,
    rng: &mut impl Rng,
    (mut positions_table, mut resources_table, mut energy_table, mut entities_by_pos, ): InitResourceMuts,
    (terrain,): InitResourceConst,
) {
    resources_table.insert_or_update(id, ResourceComponent(Resource::Energy));
    energy_table.insert_or_update(
        id,
        EnergyComponent {
            energy: 250,
            energy_max: 250,
        },
    );

    let pos = uncontested_pos(room, bounds, &*entities_by_pos, &*terrain, rng);

    positions_table.insert_or_update(id, PositionComponent(pos));
    entities_by_pos
        .table
        .at_mut(room.0)
        .expect("expected room to be in entities_by_pos table")
        .insert(pos.pos, EntityComponent(id))
        .expect("entities_by_pos insert");
}

fn uncontested_pos<T: crate::tables::TableRow + Send + Sync>(
    room: Room,
    bounds: &Hexagon,
    positions_table: &crate::tables::morton_hierarchy::MortonMortonTable<T>,
    terrain_table: &<TerrainComponent as Component<WorldPosition>>::Table,
    rng: &mut impl Rng,
) -> WorldPosition {
    const TRIES: usize = 10_000;
    let from = bounds.center - Axial::new(bounds.radius, bounds.radius);
    let to = bounds.center + Axial::new(bounds.radius, bounds.radius);
    for _ in 0..TRIES {
        let x = rng.gen_range(from.q, to.q);
        let y = rng.gen_range(from.r, to.r);

        let pos = Axial::new(x, y);

        trace!("checking pos {:?}", pos);

        if !bounds.contains(pos) {
            trace!("point {:?} is out of bounds {:?}", pos, bounds);
            continue;
        }

        let pos = WorldPosition { room: room.0, pos };

        if let Some(TerrainComponent(terrain)) = terrain_table.get_by_id(pos) {
            if terrain.is_walkable() && !positions_table.contains_key(&pos) {
                return pos;
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

    #[test]
    fn can_init_the_game() {
        let mut exc = SimpleExecutor;
        let mut world = exc
            .initialize(crate::executor::GameConfig {
                world_radius: 2,
                room_radius: 10,
                ..Default::default()
            })
            .unwrap();

        // smoke test: can the game be even initialized?
        init_world_entities(&mut *world, 12);
    }
}