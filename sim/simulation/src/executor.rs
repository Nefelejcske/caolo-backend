use std::convert::Infallible;

use tracing::debug;

use crate::{
    components::EntityScript,
    intents,
    map_generation::room::RoomGenerationParams,
    map_generation::MapGenError,
    map_generation::{generate_full_map, overworld::OverworldGenerationParams},
    prelude::{EntityId, FromWorldMut},
    profile,
    systems::{execute_world_update, script_execution::execute_scripts},
    world::World,
};

pub use crate::components::game_config::GameConfig;

/// The simplest executor.
///
/// Just runs a world update
pub struct SimpleExecutor;

impl SimpleExecutor {
    pub async fn forward_bots(
        &self,
        world: &World,
    ) -> Result<Vec<intents::BotIntents>, Infallible> {
        profile!("bots-forward");

        let tick = world.time();
        let s = tracing::error_span!("bots-forward", tick = tick);
        let _e = s.enter();

        debug!("Tick starting");

        let scripts_table = world.view::<EntityId, EntityScript>();
        let executions: Vec<(EntityId, EntityScript)> =
            scripts_table.iter().map(|(id, x)| (id, *x)).collect();

        debug!("Executing scripts");
        let intents = execute_scripts(executions.as_slice(), world).expect("script execution");
        debug!("Executing scripts Done");
        debug!("Done");
        Ok(intents)
    }

    pub async fn apply_intents(
        &mut self,
        world: &mut World,
        intents: Vec<intents::BotIntents>,
    ) -> Result<(), Infallible> {
        profile!("apply-intents");

        let tick = world.time();
        let s = tracing::error_span!("apply-intents", tick = tick);
        let _e = s.enter();

        debug!("Got {} intents", intents.len());
        intents::move_into_storage(world, intents);

        debug!("Executing systems update");
        execute_world_update(world);

        debug!("Executing post-processing");
        world.post_process();

        debug!("Done");

        Ok(())
    }

    pub async fn initialize(&mut self, config: GameConfig) -> World {
        let mut world = World::new();

        execute_map_generation(&mut world, &config)
            .await
            .expect("Failed to generate world map");

        world.config.game_config.value = Some(config);

        world
    }
}

async fn execute_map_generation(world: &mut World, config: &GameConfig) -> Result<(), MapGenError> {
    let world_radius = config.world_radius;
    let room_radius = config.room_radius;
    assert!(room_radius > 6);
    let params = OverworldGenerationParams::builder()
        .with_radius(world_radius as u32)
        .with_room_radius(room_radius)
        .with_min_bridge_len(3)
        .with_max_bridge_len(room_radius - 3)
        .build()
        .unwrap();
    let room_params = RoomGenerationParams::builder()
        .with_radius(room_radius)
        .with_chance_plain(0.13)
        .with_chance_wall(1.0 - 0.13)
        .with_plain_dilation(2)
        .build()
        .unwrap();
    debug!("generating map {:#?} {:#?}", params, room_params);

    generate_full_map(
        &params,
        &room_params,
        None,
        FromWorldMut::from_world_mut(world),
    )
    .await?;

    debug!("world generation done");
    Ok(())
}
