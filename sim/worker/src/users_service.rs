use caolo_sim::{
    components::UserProperties,
    prelude::{UserId, View},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::info;
use uuid::Uuid;

use crate::protos::{cao_common, cao_users};

#[derive(Clone)]
pub struct UsersService {
    world: crate::WorldContainer,
}

impl UsersService {
    pub fn new(world: crate::WorldContainer) -> Self {
        Self { world }
    }
}

#[tonic::async_trait]
impl cao_users::users_server::Users for UsersService {
    type ListUsersStream = ReceiverStream<Result<cao_common::Uuid, Status>>;

    async fn list_users(
        &self,
        request: tonic::Request<cao_common::Empty>,
    ) -> Result<tonic::Response<Self::ListUsersStream>, Status> {
        let addr = request.remote_addr();
        let w = self.world.read().await;
        let users: Vec<UserId> = w.list_users().collect();
        drop(w); // free the lock

        let (tx, rx) = mpsc::channel(512);
        tokio::spawn(async move {
            for u in users {
                let pl = u.0.as_bytes();
                let mut data = Vec::with_capacity(pl.len());
                data.extend_from_slice(pl);
                if tx.send(Ok(cao_common::Uuid { data })).await.is_err() {
                    info!("list users client lost {:?}", addr);
                    break;
                }
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }

    async fn get_user_info(
        &self,
        request: tonic::Request<cao_common::Uuid>,
    ) -> Result<tonic::Response<cao_users::UserInfo>, Status> {
        let user_id = &request.get_ref().data;
        let user_id = Uuid::from_slice(user_id).map_err(|err| {
            Status::invalid_argument(format!("Payload was not a valid UUID: {}", err))
        })?;
        let user_id = UserId(user_id);

        let w = self.world.read().await;
        let props_table: View<UserId, UserProperties> = w.view();
        let properties = props_table.get_by_id(user_id).cloned();
        drop(props_table);
        drop(w); // free the lock

        let result = match properties {
            Some(properies) => cao_users::UserInfo {
                user_id: Some(request.into_inner()),
                level: properies.level as i32,
            },
            None => {
                return Err(Status::not_found(format!(
                    "User {} was not found",
                    user_id.0
                )))
            }
        };

        Ok(tonic::Response::new(result))
    }
}
