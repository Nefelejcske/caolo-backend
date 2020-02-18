use super::*;
use crate::model::{self, components, structures::BotDescription, OperationResult, UserId};

#[derive(Debug, Clone)]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: BotDescription,
    pub owner_id: Option<UserId>,
}

pub fn check_spawn_intent(
    intent: &model::structures::SpawnIntent,
    userid: Option<model::UserId>,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let id = intent.id;

    if let Some(userid) = userid {
        match storage
            .entity_table::<components::Structure>()
            .get_by_id(&id)
        {
            Some(_) => {
                let owner_id = storage
                    .entity_table::<components::OwnedEntity>()
                    .get_by_id(&id);
                if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                    return OperationResult::NotOwner;
                }
            }
            None => {
                debug!("Structure not found");
                return OperationResult::InvalidInput;
            }
        }
    }

    if let Some(spawn) = storage
        .entity_table::<components::SpawnComponent>()
        .get_by_id(&id)
    {
        if spawn.spawning.is_some() {
            debug!("Structure is busy");
            return OperationResult::InvalidInput;
        }
    } else {
        debug!("Structure has no spawn component");
        return OperationResult::InvalidInput;
    }

    OperationResult::Ok
}
