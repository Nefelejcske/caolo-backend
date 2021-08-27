use crate::components::HpComponent;
use crate::indices::*;
use crate::profile;
use crate::storage::views::{DeferredDeleteEntityView, View};
use tracing::{debug, trace};

pub fn death_update(mut delete: DeferredDeleteEntityView, (hps,): (View<EntityId, HpComponent>,)) {
    profile!("DeathSystem update");
    debug!("update death system called");

    hps.iter().for_each(|(id, hp)| {
        if hp.hp == 0 {
            trace!("Entity {:?} has died, deleting", id);
            unsafe {
                delete.delete_entity(id);
            }
        }
    });

    debug!("update death system done");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{query, world::World};
    use crate::{storage::views::FromWorld, storage::views::FromWorldMut};

    #[test]
    fn test_dead_entity_is_deleted() {
        let mut store = World::new();

        let entity_1 = store.insert_entity();
        let entity_2 = store.insert_entity();
        query!(
            mutate
            store
            {
                EntityId, HpComponent, .insert_or_update(entity_1, HpComponent {
                    hp: 0,
                    hp_max: 123
                });
                EntityId, HpComponent, .insert_or_update(entity_2, HpComponent {
                    hp: 50,
                    hp_max: 123
                });
            }
        );

        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![entity_1, entity_2]);

        death_update(
            FromWorldMut::from_world_mut(&mut store),
            FromWorld::from_world(&mut store),
        );
        store.post_process();

        let entities: Vec<_> = store
            .view::<EntityId, HpComponent>()
            .iter()
            .map(|(id, _)| id)
            .collect();

        assert_eq!(entities, vec![entity_2]);
    }
}
