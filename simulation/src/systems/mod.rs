pub mod decay_system;
pub mod energy_system;
pub mod intent_execution;
pub mod mineral_system;
pub mod pathfinding;
pub mod positions_system;
pub mod script_execution;
pub mod spawn_system;

use crate::profile;
use crate::storage::{
    views::{HasNew, HasNewMut},
    Storage,
};

pub trait System<'a> {
    // Requiring these traits instead of From impl disallows Storage as an `update` parameter
    // Thus requiring callers to explicitly state their dependencies
    type Mut: HasNewMut;
    type Const: HasNew<'a>;

    fn update(&mut self, m: Self::Mut, c: Self::Const);
}

pub fn execute_world_update(storage: &mut Storage) {
    profile!("execute_world_update");

    let mut energy_sys = energy_system::EnergySystem;
    update(&mut energy_sys, storage);

    let mut spawn_sys = spawn_system::SpawnSystem;
    update(&mut spawn_sys, storage);

    let mut decay_sys = decay_system::DecaySystem;
    update(&mut decay_sys, storage);

    let mut mineral_sys = mineral_system::MineralSystem;
    update(&mut mineral_sys, storage);

    let mut positions_sys = positions_system::PositionSystem;
    update(&mut positions_sys, storage);
}

#[inline]
fn update<'a, Sys: System<'a>>(sys: &mut Sys, storage: &'a mut Storage) {
    sys.update(Sys::Mut::new(storage), Sys::Const::new(storage));
}
