use crate::model::{
    self,
    components::{Bot, CarryComponent, EnergyComponent, OwnedEntity, PositionComponent, Resource},
    EntityId, OperationResult,
};
use crate::storage::views::View;

pub const DROPOFF_RANGE: u64 = 1;

#[derive(Debug, Clone)]
pub struct DropoffIntent {
    pub bot: EntityId,
    pub structure: EntityId,
    pub amount: u16,
    pub ty: Resource,
}

/// A valid dropoff intent has the following characteristics:
/// - the bot is owned by the user
/// - the bot is carrying resource of type `ty`
/// - the target is not full
/// - the target is within dropoff range
pub fn check_dropoff_intent(
    intent: model::bots::DropoffIntent,
    userid: model::UserId,
    (bots, owners, positions, carry, energy): (
        View<EntityId, Bot>,
        View<EntityId, OwnedEntity>,
        View<EntityId, PositionComponent>,
        View<EntityId, CarryComponent>,
        View<EntityId, EnergyComponent>,
    ),
) -> OperationResult {
    let id = intent.id;
    match bots.get_by_id(&id) {
        Some(_) => {
            let owner_id = owners.get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    if carry
        .get_by_id(&id)
        .map(|carry| carry.carry == 0)
        .unwrap_or(true)
    {
        return OperationResult::Empty;
    }

    let target = intent.target;
    let nearby = positions.get_by_id(&id).and_then(|botpos| {
        positions
            .get_by_id(&target)
            .map(|targetpos| targetpos.0.hex_distance(botpos.0) <= DROPOFF_RANGE)
    });
    match nearby {
        None => {
            error!("Bot or target has no position components {:?}", intent);
            OperationResult::InvalidInput
        }
        Some(false) => OperationResult::NotInRange,
        Some(true) => {
            let capacity = energy.get_by_id(&target);
            if capacity.is_none() {
                error!("Target has no energy component {:?}", intent);
                return OperationResult::InvalidInput;
            }
            let capacity = capacity.unwrap();
            if capacity.energy < capacity.energy_max {
                OperationResult::Ok
            } else {
                OperationResult::Full
            }
        }
    }
}
