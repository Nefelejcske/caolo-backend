use cao_lang::prelude::*;
use caolo_api::{point::Point, ScriptId};
use caolo_sim::model;
use caolo_sim::storage::Storage;

const PROGRAM: &str = r#"{"nodes":{"0":{"node":{"ScalarInt":{"value":69}},"children":[1]},"1":{"node":{"ScalarInt":{"value":420}},"children":[2]},"2":{"node":{"Call":{"function":"make_point"}},"children":[3]},"3":{"node":{"Call":{"function":"bots::move_bot"}},"children":[5]},"4":{"node":{"Call":{"function":"make_operation_result"}},"children":[9]},"5":{"node":{"ScalarInt":{"value":0}},"children":[4]},"6":{"node":{"JumpIfTrue":{"nodeid":7}},"children":[10]},"7":{"node":{"StringLiteral":{"value":"Moving :)"}},"children":[8]},"8":{"node":{"Call":{"function":"console_log"}},"children":[]},"9":{"node":{"Equals":null},"children":[6]},"10":{"node":{"StringLiteral":{"value":"No moverino :("}},"children":[11]},"11":{"node":{"Call":{"function":"console_log"}},"children":[]},"12":{"node":{"Start":null},"children":[0]}},"name":"placeholder"}"#;

pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default(); // TODO randomize
    let script_id = model::ScriptId(script_id);
    let script: CompilationUnit =
        serde_json::from_str(PROGRAM).expect("deserialize example program");
    let compiled = Compiler::compile(script.clone()).expect("failed to compile example program");
    storage
        .scripts_table_mut::<model::ScriptComponent>()
        .insert_or_update(script_id, model::ScriptComponent(compiled));

    let mut rng = rand::thread_rng();

    let terrain = storage.point_table_mut::<model::TerrainComponent>();

    for _ in 0..1000 {
        let pos = uncontested_pos(terrain, &mut rng);
        terrain.insert(pos, model::TerrainComponent(model::TileTerrainType::Wall));
    }

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        storage
            .entity_table_mut::<model::EntityScript>()
            .insert_or_update(id, model::EntityScript { script_id });
        storage
            .entity_table_mut::<model::Bot>()
            .insert_or_update(id, model::Bot {});
        storage
            .entity_table_mut::<model::CarryComponent>()
            .insert_or_update(id, Default::default());
        storage
            .entity_table_mut::<model::OwnedEntity>()
            .insert_or_update(
                id,
                model::OwnedEntity {
                    owner_id: Default::default(),
                },
            );

        let pos = {
            let entities_by_pos = storage.point_table::<model::EntityComponent>();
            uncontested_pos(entities_by_pos, &mut rng)
        };

        let positions = storage.entity_table_mut::<model::PositionComponent>();
        positions.insert_or_update(id, model::PositionComponent(pos));
    }
    storage
}

fn uncontested_pos<T: caolo_sim::tables::TableRow + Send + Sync>(
    positions_table: &caolo_sim::tables::MortonTable<Point, T>,
    rng: &mut rand::rngs::ThreadRng,
) -> caolo_api::point::Point {
    use rand::Rng;

    let mut pos = Point::default();
    loop {
        pos.x = rng.gen_range(0, 500);
        pos.y = rng.gen_range(0, 500);

        if positions_table.get_by_id(&pos).is_none() {
            break;
        }
    }
    pos
}
