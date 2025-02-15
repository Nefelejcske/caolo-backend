mod config;
mod game_loop;
mod input;
mod protos;

mod command_service;
mod health_service;
mod scripting_service;
mod users_service;
mod world_service;

use crate::protos::cao_commands::command_server::CommandServer;
use crate::protos::cao_common::health_server::HealthServer;
use crate::protos::cao_script::scripting_server::ScriptingServer;
use crate::protos::cao_users::users_server::UsersServer;
use crate::protos::cao_world::world_server::WorldServer;
use caolo_sim::executor::{GameConfig, SimpleExecutor};
use std::{env, sync::Arc, time::Duration};
use tracing::{info, Instrument};
use uuid::Uuid;

use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing_subscriber::layer::SubscriberExt;

type WorldContainer = Arc<tokio::sync::RwLock<caolo_sim::prelude::World>>;

fn init() {
    #[cfg(feature = "dotenv")]
    dotenv::dotenv().unwrap_or_default();

    let use_console = std::env::var("CAO_LOG_HUMAN")
        .map(|x| x.parse().unwrap())
        .unwrap_or(true);
    if use_console {
        let collector = tracing_subscriber::fmt()
            .without_time()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(collector).unwrap();
    } else {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let tracer = opentelemetry::sdk::export::trace::stdout::new_pipeline().install_simple();
        let collector = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_opentelemetry::layer().with_tracer(tracer));
        tracing::subscriber::set_global_default(collector).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let now = std::time::Instant::now();

    init();

    let config = config::Config::load();

    info!("Loaded config\n{:#?}", config);

    let script_chunk_size = env::var("CAO_QUEEN_SCRIPT_CHUNK_SIZE")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(1024);

    let tick_latency = Duration::from_millis(config.target_tick_ms);

    info!(
        "Loaded Queen params:\nScript chunk size: {}\nTick latency: {:?}",
        script_chunk_size, tick_latency
    );

    let tag = env::var("CAO_QUEEN_TAG").unwrap_or_else(|_| Uuid::new_v4().to_string());
    let world_span = tracing::error_span!("world-service", queen_tag = tag.as_str());
    let game_loop_span = tracing::error_span!("game-loop", queen_tag = tag.as_str());

    info!("Creating cao executor with tag {}", tag);
    let mut executor = SimpleExecutor;
    info!("Init storage");
    let mut world = executor
        .initialize(GameConfig {
            world_radius: config.world_radius,
            room_radius: config.room_radius,
            queen_tag: tag.clone(),
            ..Default::default()
        })
        .await;

    info!("Starting with {} actors", config.n_actors);

    caolo_sim::init::init_world_entities(&mut world, config.n_actors as usize);

    let addr = env::var("CAO_SERVICE_ADDR")
        .ok()
        .map(|x| x.parse().expect("failed to parse cao service address"))
        .unwrap_or_else(|| "[::1]:50051".parse().unwrap());

    info!("Starting the game loop. Starting the service on {:?}", addr);

    let (outtx, _) = tokio::sync::broadcast::channel(config.world_buff_size as usize);
    let outpayload = Arc::new(outtx);

    let room_bounds = caolo_sim::prelude::Hexagon::from_radius(
        world
            .view::<caolo_sim::indices::ConfigKey, caolo_sim::components::RoomProperties>()
            .unwrap_value()
            .radius as i32,
    );

    let terrain = world
        .view::<caolo_sim::prelude::WorldPosition, caolo_sim::prelude::TerrainComponent>()
        .iter_rooms()
        .map(|(room_id, room_terrain)| {
            (
                room_id.0,
                room_terrain.iter().map(|(_, t)| t).copied().collect(),
            )
        })
        .collect();
    let rooms = world
        .view::<caolo_sim::prelude::Axial, caolo_sim::prelude::RoomComponent>()
        .iter()
        .map(|(room_id, comp)| (room_id, *comp))
        .collect();

    let world = Arc::new(tokio::sync::RwLock::new(world));

    let server = tonic::transport::Server::builder()
        .trace_fn(move |_| tracing::error_span!("service", queen_tag = tag.as_str()))
        .add_service(CommandServer::new(
            crate::command_service::CommandService::new(Arc::clone(&world)),
        ))
        .add_service(ScriptingServer::new(
            crate::scripting_service::ScriptingService::new(Arc::clone(&world)),
        ))
        .add_service(WorldServer::new(crate::world_service::WorldService::new(
            Arc::clone(&outpayload),
            room_bounds,
            Arc::new(terrain),
            Arc::new(rooms),
            world_span,
        )))
        .add_service(HealthServer::new(health_service::HealthService {}))
        .add_service(UsersServer::new(crate::users_service::UsersService::new(
            Arc::clone(&world),
        )))
        .serve(addr);

    let game_loop =
        game_loop::game_loop(world, executor, outpayload, tick_latency).instrument(game_loop_span);

    info!(
        "Initialization done in {:?}",
        std::time::Instant::now() - now
    );
    let (a, _) = futures::join!(server, game_loop);
    a.unwrap();
}
