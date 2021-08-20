pub mod components;
pub mod entity_archetypes;
pub mod executor;
pub mod geometry;
pub mod indices;
pub mod init;
pub mod map_generation;
pub mod noise;
pub mod pathfinding;
pub mod prelude;
pub mod scripting_api;
pub mod storage;
pub mod tables;
pub mod terrain;

mod intents;
mod systems;
mod utils;
pub mod world;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_sim_version.rs"));
}

#[derive(Clone, Debug, Default, Copy, serde::Serialize, serde::Deserialize)]
pub struct Time(pub u64);

impl<'a> storage::views::FromWorld<'a> for Time {
    fn from_world(w: &'a prelude::World) -> Self {
        Time(w.time())
    }
}
