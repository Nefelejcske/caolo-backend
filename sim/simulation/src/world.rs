mod world_serde;

use crate::components::*;
use crate::indices::*;
use crate::intents::*;
use crate::storage::{
    self,
    views::{UnsafeView, View},
};
use crate::tables::btree_table::BTreeTable;
use crate::tables::flag_table::SparseFlagTable;
use crate::tables::handle_table::HandleTable;
use crate::tables::morton_hierarchy::MortonGridTable;
use crate::tables::morton_hierarchy::MortonMortonTable;
use crate::tables::morton_table::MortonTable;
use crate::tables::page_table::PageTable;
use crate::tables::unique_table::UniqueTable;
use crate::tables::Component;
use crate::tables::TableId;
use crate::Time;
use crate::{archetype, tables::hex_grid::HexGrid};
use crate::{components::game_config::GameConfig, prelude::Axial};

archetype!(
    module room_store key Axial,
    table RoomConnections : MortonTable<RoomConnections> = room_connections,
    table RoomComponent : MortonTable<RoomComponent> = rooms,
    table OwnedEntity : MortonTable<OwnedEntity> = owner

    iterby rooms
);

archetype!(
    module entity_store key EntityId,

    table Bot : SparseFlagTable<EntityId, Bot>  = bot,
    table PositionComponent : PageTable<PositionComponent> = pos,
    table SpawnBotComponent : PageTable<SpawnBotComponent> = spawnbot,
    table CarryComponent : PageTable<CarryComponent> = carry,
    table Structure : SparseFlagTable<EntityId, Structure> = structure,
    table HpComponent : PageTable<HpComponent> = hp,
    table EnergyRegenComponent : PageTable<EnergyRegenComponent> = energyregen,
    table EnergyComponent : PageTable<EnergyComponent> = energy,
    table ResourceComponent : PageTable<ResourceComponent> = resource,
    table DecayComponent : PageTable<DecayComponent> = decay,
    table EntityScript : PageTable<EntityScript> = script,
    table SpawnComponent : PageTable<SpawnComponent> = spawn,
    table SpawnQueueComponent : PageTable<SpawnQueueComponent> = spawnqueue,
    table OwnedEntity : PageTable<OwnedEntity> = owner,
    table MeleeAttackComponent : PageTable<MeleeAttackComponent> = melee,
    table SayComponent : PageTable<SayComponent> = say,
    table MineEventComponent : PageTable<MineEventComponent> = mine_intents,
    table DropoffEventComponent : PageTable<DropoffEventComponent> = dropoff_intents,
    table RespawnTimer : PageTable<RespawnTimer> = respawn_timer,

    table PathCacheComponent : PageTable<PathCacheComponent> = pathcache,
    table ScriptHistory : PageTable<ScriptHistory> = script_history

    iterby bot
    iterby structure
    iterby resource
);

archetype!(
    module user_store key UserId,

    table UserComponent : SparseFlagTable<UserId, UserComponent> = user,
    table EntityScript: BTreeTable<UserId, EntityScript> = user_default_script,
    table Rooms : BTreeTable<UserId, Rooms> = user_rooms,
    table UserProperties : BTreeTable<UserId, UserProperties> = user_props

    iterby user
);

archetype!(
    module resource_store key EmptyKey,

    table Time : UniqueTable<EmptyKey, Time> = time,
    table Intents<MoveIntent> : UniqueTable<EmptyKey, Intents<MoveIntent>> = move_intents,
    table Intents<SpawnIntent> : UniqueTable<EmptyKey, Intents<SpawnIntent>> = spawn_intents,
    table Intents<MineIntent> : UniqueTable<EmptyKey, Intents<MineIntent>> = mine_intents,
    table Intents<DropoffIntent> : UniqueTable<EmptyKey, Intents<DropoffIntent>> = dropoff_intents,
    table Intents<LogIntent> : UniqueTable<EmptyKey, Intents<LogIntent>> = log_intents,
    table Intents<CachePathIntent> : UniqueTable<EmptyKey, Intents<CachePathIntent>> = update_path_cache_intents,
    table Intents<MutPathCacheIntent> : UniqueTable<EmptyKey, Intents<MutPathCacheIntent>> = mut_path_cache_intents,
    table Intents<MeleeIntent> : UniqueTable<EmptyKey, Intents<MeleeIntent>> = melee_intents,
    table Intents<ScriptHistoryEntry> : UniqueTable<EmptyKey, Intents<ScriptHistoryEntry>> = script_history_intents,
    table Intents<SayIntent> : UniqueTable<EmptyKey, Intents<SayIntent>> = say_intents
);

archetype!(
    module config_store key ConfigKey,

    table RoomProperties : UniqueTable<ConfigKey, RoomProperties> = room_properties,
    table GameConfig : UniqueTable<ConfigKey, GameConfig> = game_config
);

archetype!(
    module positions_store key WorldPosition,
    table TerrainComponent : MortonGridTable<TerrainComponent> = point_terrain,
    table EntityComponent : MortonMortonTable<EntityComponent> = point_entity
);

archetype!(
    module script_store key ScriptId,
    table CompiledScriptComponent : BTreeTable<ScriptId, CompiledScriptComponent> = compiled_script,
    table CaoIrComponent : BTreeTable<ScriptId, CaoIrComponent> = cao_ir

    iterby cao_ir
);

impl<Id: TableId> Component<Id> for LogEntry {
    type Table = BTreeTable<Id, Self>;
}
impl Component<Axial> for TerrainComponent {
    type Table = HexGrid<Self>;
}
impl Component<Axial> for EntityComponent {
    type Table = MortonTable<Self>;
}

pub struct World {
    pub entities: entity_store::Archetype,
    pub room: room_store::Archetype,
    pub user: user_store::Archetype,
    pub config: config_store::Archetype,
    pub resources: resource_store::Archetype,
    pub scripts: script_store::Archetype,
    pub entity_logs: <LogEntry as Component<EntityTime>>::Table,
    pub positions: positions_store::Archetype,

    deferred_deletes: entity_store::DeferredDeletes,

    entity_handles: HandleTable,
}

macro_rules! impl_hastable {
    ($module: ident, $field: ident) => {
        impl<C: Component<$module::Key>> storage::HasTable<$module::Key, C> for World
        where
            $module::Archetype: storage::HasTable<$module::Key, C>,
        {
            fn view(&self) -> View<$module::Key, C> {
                self.$field.view()
            }

            fn unsafe_view(&mut self) -> UnsafeView<$module::Key, C> {
                self.$field.unsafe_view()
            }
        }
    };
}

impl_hastable!(entity_store, entities);
impl_hastable!(room_store, room);
impl_hastable!(user_store, user);
impl_hastable!(config_store, config);
impl_hastable!(positions_store, positions);
impl_hastable!(resource_store, resources);
impl_hastable!(script_store, scripts);

impl storage::HasTable<EntityTime, LogEntry> for World {
    fn view(&self) -> View<EntityTime, LogEntry> {
        View::from_table(&self.entity_logs)
    }

    fn unsafe_view(&mut self) -> UnsafeView<EntityTime, LogEntry> {
        UnsafeView::from_table(&mut self.entity_logs)
    }
}

impl World {
    /// Moving World around in memory would invalidate views, so let's make sure it doesn't
    /// happen.
    pub(crate) fn new() -> Self {
        let mut config: config_store::Archetype = Default::default();
        config.game_config.value = Some(Default::default());

        let mut res = Self {
            config,
            entities: Default::default(),
            room: Default::default(),
            resources: Default::default(),
            entity_logs: Default::default(),
            scripts: Default::default(),
            positions: Default::default(),
            deferred_deletes: Default::default(),
            entity_handles: HandleTable::new(500_000),
            user: Default::default(),
        };

        // initialize the intent tables
        let botints = crate::intents::BotIntents::default();
        crate::intents::move_into_storage(&mut res, vec![botints]);
        res
    }

    pub fn view<Id: TableId, C: Component<Id>>(&self) -> View<Id, C>
    where
        Self: storage::HasTable<Id, C>,
    {
        <Self as storage::HasTable<Id, C>>::view(self)
    }

    pub fn unsafe_view<Id: TableId, C: Component<Id>>(&mut self) -> UnsafeView<Id, C>
    where
        Self: storage::HasTable<Id, C>,
    {
        <Self as storage::HasTable<Id, C>>::unsafe_view(self)
    }

    pub fn time(&self) -> u64 {
        let view = &self.resources.time.value;
        view.map(|Time(t)| t).unwrap_or(0)
    }

    /// Perform post-tick cleanup on the storage
    pub(crate) fn post_process(&mut self) {
        for e in self.deferred_deletes.entityid.iter().copied() {
            self.entity_handles.free(e);
        }
        self.deferred_deletes.execute_all(&mut self.entities);
        self.deferred_deletes.clear();

        self.resources.time.value = self
            .resources
            .time
            .value
            .map(|Time(x)| Time(x + 1))
            .or(Some(Time(1)));
    }

    pub fn insert_entity(&mut self) -> EntityId {
        self.entity_handles.alloc()
    }

    pub fn is_valid_entity(&self, id: EntityId) -> bool {
        self.entity_handles.is_valid(id)
    }

    pub fn queen_tag(&self) -> Option<&str> {
        self.config
            .game_config
            .value
            .as_ref()
            .map(|conf| conf.queen_tag.as_str())
    }

    pub fn list_users(&self) -> impl Iterator<Item = UserId> + '_ {
        self.user.user.iter().map(|(id, _)| id)
    }
}

impl storage::DeferredDeleteById<EntityId> for World
where
    entity_store::DeferredDeletes: storage::DeferredDeleteById<EntityId>,
{
    fn deferred_delete(&mut self, key: EntityId) {
        self.deferred_deletes.deferred_delete(key);
    }

    fn clear_defers(&mut self) {
        self.deferred_deletes.clear_defers();
    }

    fn execute<Store: storage::DeleteById<EntityId>>(&mut self, store: &mut Store) {
        self.deferred_deletes.execute(store);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_env_log::test;

    #[test]
    fn check_world_sanity() {
        let _world = World::new();
    }

    #[test]
    fn test_bot_serialization() {
        let mut world = World::new();

        for _ in 0..4 {
            let _entity = world.insert_entity(); // produce gaps
            let entity = world.insert_entity();

            world.entities.bot.insert(entity);
            world
                .entities
                .melee
                .insert(entity, MeleeAttackComponent { strength: 128 });
            world.entities.pos.insert(
                entity,
                PositionComponent(WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(16, 61),
                }),
            );
        }

        for _ in 0..2 {
            let entity = world.insert_entity();

            world.entities.structure.insert(entity);
            world.entities.pos.insert(
                entity,
                PositionComponent(WorldPosition {
                    room: Axial::new(42, 69),
                    pos: Axial::new(16, 61),
                }),
            );
        }

        let bots: Vec<_> = world.entities.iterby_bot().collect();
        serde_json::to_string_pretty(&bots).unwrap();

        let structures: Vec<_> = world.entities.iterby_structure().collect();
        serde_json::to_string_pretty(&structures).unwrap();
    }
}
