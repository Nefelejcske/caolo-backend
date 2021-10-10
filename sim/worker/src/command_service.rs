use crate::input::structures;
use crate::{input::rooms, protos::cao_commands};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Clone)]
pub struct CommandService {
    world: crate::WorldContainer,
}

impl std::fmt::Debug for CommandService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandService").finish()
    }
}

impl CommandService {
    pub fn new(world: crate::WorldContainer) -> Self {
        Self { world }
    }
}

#[tonic::async_trait]
impl cao_commands::command_server::Command for CommandService {
    async fn place_structure(
        &self,
        request: Request<cao_commands::PlaceStructureCommand>,
    ) -> Result<Response<cao_commands::CommandResult>, Status> {
        info!("Placing structure");
        let mut w = self.world.write().await;
        structures::place_structure(&mut w, request.get_ref())
            .map(|_: ()| Response::new(cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }

    async fn take_room(
        &self,
        request: tonic::Request<cao_commands::TakeRoomCommand>,
    ) -> Result<tonic::Response<cao_commands::CommandResult>, tonic::Status> {
        info!("Taking room");
        let mut w = self.world.write().await;
        rooms::take_room(&mut w, request.get_ref())
            .map(|_: ()| Response::new(cao_commands::CommandResult {}))
            .map_err(|err| Status::invalid_argument(err.to_string()))
    }
}
