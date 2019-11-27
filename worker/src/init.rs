use caolo_api::{point::Point, Script, ScriptId};
use caolo_sim::model;
use caolo_sim::prelude::*;
use caolo_sim::storage::Storage;

const PROGRAM: &str = r#"{
    "nodes": {
        "0": {
            "instruction": "ScalarInt",
            "scalar": {
                "Integer": 4
            }
        },
        "1": {
            "instruction": "ScalarInt",
            "scalar": {
                "Integer": 5
            }
        },
        "2": {
            "instruction": "Add",
            "inputs": [0, 1]
        },
        "3": {
            "instruction": "Call",
            "string": "log_scalar",
            "inputs": [2]
        }
    }
}"#;

pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default(); // TODO randomize
    let script_id = model::ScriptId(script_id);
    storage
        .scripts_table_mut::<model::ScriptComponent>()
        .insert(
            script_id,
            model::ScriptComponent(Script {
                compiled: None,
                script: serde_json::from_str(PROGRAM).expect("deserialize"),
            }),
        );

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        storage
            .entity_table_mut::<model::EntityScript>()
            .insert(id, model::EntityScript { script_id });
        storage
            .entity_table_mut::<model::Bot>()
            .insert(id, model::Bot {});
        storage
            .entity_table_mut::<model::PositionComponent>()
            .insert(id, model::PositionComponent(Point::new(0, 0)));
        storage
            .entity_table_mut::<model::CarryComponent>()
            .insert(id, Default::default());
    }
    storage
}
