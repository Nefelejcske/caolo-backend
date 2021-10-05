use crate::components;
use crate::indices::{EntityId, UserId};
use crate::scripting_api::OperationResult;
use crate::storage::views::View;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MineIntent {
    pub bot: EntityId,
    pub resource: EntityId,
}

type CheckInput<'a> = (
    View<'a, EntityId, components::Bot>,
    View<'a, EntityId, components::OwnedEntity>,
    View<'a, EntityId, components::PositionComponent>,
    View<'a, EntityId, components::ResourceComponent>,
    View<'a, EntityId, components::EnergyComponent>,
    View<'a, EntityId, components::CarryComponent>,
);

pub fn check_mine_intent(
    intent: &MineIntent,
    userid: UserId,
    (bots_table, owner_ids_table, positions_table, resources_table, energy_table, carry_table): CheckInput,
) -> OperationResult {
    let bot = intent.bot;

    let s = tracing::debug_span!("check_mine_intent", entity_id = bot.to_string().as_str());
    let _e = s.enter();

    match bots_table.contains(&bot) {
        true => {
            let owner_id = owner_ids_table.get(bot);
            if owner_id.map(|bot| bot.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        false => return OperationResult::InvalidInput,
    };

    let botpos = match positions_table.get(bot) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    let target = intent.resource;
    let mineralpos = match positions_table.get(target) {
        Some(pos) => pos,
        None => {
            debug!("{} has no position", target);
            return OperationResult::InvalidInput;
        }
    };

    match carry_table.get(bot) {
        Some(carry) => {
            if carry.carry >= carry.carry_max {
                debug!("{} is full", bot);
                return OperationResult::Full;
            }
        }
        None => {
            debug!("{} has no carry component", bot);
            return OperationResult::InvalidInput;
        }
    }

    if botpos.0.room != mineralpos.0.room {
        trace!(
            "NotInSameRoom bot: {:?} mineral: {:?}",
            botpos.0.room,
            mineralpos.0.room,
        );
        return OperationResult::NotInRange;
    }
    if botpos.0.pos.hex_distance(mineralpos.0.pos) > 1 {
        trace!(
            "NotInRange bot: {:?} mineral: {:?}, distance: {}",
            botpos.0.pos,
            mineralpos.0.pos,
            botpos.0.pos.hex_distance(mineralpos.0.pos)
        );
        return OperationResult::NotInRange;
    }

    match resources_table.get(target) {
        Some(components::ResourceComponent(components::Resource::Energy)) => {
            match energy_table.get(target) {
                Some(energy) => {
                    if energy.energy > 0 {
                        OperationResult::Ok
                    } else {
                        OperationResult::Empty
                    }
                }
                None => {
                    debug!("Mineral has no energy component!");
                    OperationResult::InvalidInput
                }
            }
        }
        Some(_) | None => {
            debug!("{} is not a resource!", target);
            OperationResult::InvalidInput
        }
    }
}
