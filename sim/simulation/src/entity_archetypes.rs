//! helper function to handle entity archetypes

use uuid::Uuid;

use crate::prelude::*;
use crate::query;

/// Initialize a spawn at the given position
pub fn init_structure_spawn(id: EntityId, owner_id: Uuid, pos: WorldPosition, world: &mut World) {
    // TODO tweak these numbas
    query!(
        mutate world
        {
            EntityId, Structure, .insert(id);
            EntityId, SpawnComponent, .insert(id, SpawnComponent::default());
            EntityId, SpawnQueueComponent, .insert(id, SpawnQueueComponent::default());
            EntityId, OwnedEntity, .insert(
                id,
                OwnedEntity {
                    owner_id: UserId(owner_id),
                }
            );
            EntityId, EnergyComponent, .insert(
                id,
                EnergyComponent {
                    energy: 500,
                    energy_max: 500,
                }
            );
            EntityId, EnergyRegenComponent, .insert(id, EnergyRegenComponent { amount: 20 });
            EntityId, HpComponent, .insert(
                id,
                HpComponent {
                    hp: 500,
                    hp_max: 500,
                }
            );
            EntityId, PositionComponent, .insert(id, PositionComponent(pos));
            WorldPosition, EntityComponent, .insert(pos, EntityComponent(id))
                .expect("entities_by_pos insert failed");

        }
    );
}

type InitBotTables = (
    UnsafeView<EntityId, Bot>,
    UnsafeView<EntityId, HpComponent>,
    UnsafeView<EntityId, DecayComponent>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, PositionComponent>,
    UnsafeView<EntityId, OwnedEntity>,
    UnsafeView<EntityId, EntityScript>,
);
pub fn init_bot(
    entity_id: EntityId,
    owner_id: Option<Uuid>,
    pos: WorldPosition,
    (
        mut bots,
        mut hps,
        mut decay,
        mut carry,
        mut positions,
        mut owned,
        mut script_table,
    ): InitBotTables,
    user_default_scripts: View<UserId, EntityScript>,
) {
    bots.insert(entity_id);
    hps.insert(
        entity_id,
        HpComponent {
            hp: 100,
            hp_max: 100,
        },
    );
    decay.insert(
        entity_id,
        DecayComponent {
            interval: 10,
            time_remaining: 10,
            hp_amount: 10,
        },
    );
    carry.insert(
        entity_id,
        CarryComponent {
            carry: 0,
            carry_max: 150,
        },
    );

    positions.insert(entity_id, PositionComponent(pos));

    if let Some(owner_id) = owner_id {
        if let Some(script) = user_default_scripts.get(UserId(owner_id)) {
            script_table.insert(entity_id, *script);
        }

        owned.insert(
            entity_id,
            OwnedEntity {
                owner_id: UserId(owner_id),
            },
        );
    }
}
