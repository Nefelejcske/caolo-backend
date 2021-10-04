use super::*;
use crate::components::LogEntry;
use crate::indices::EntityTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    data: BTreeMap<Id, Row>,
}

impl<Id, Row> BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn iter(&self) -> impl TableIterator<Id, &Row> {
        self.data.iter().map(|(id, row)| (*id, row))
    }

    pub fn iter_mut(&mut self) -> impl TableIterator<Id, &mut Row> {
        self.data.iter_mut().map(|(id, row)| (*id, row))
    }

    pub fn get(&self, id: Id) -> Option<&Row> {
        self.data.get(&id)
    }

    pub fn get_by_id_mut(&mut self, id: Id) -> Option<&mut Row> {
        self.data.get_mut(&id)
    }

    pub fn get_by_ids(&self, ids: &[Id]) -> Vec<(Id, &Row)> {
        self.data
            .iter()
            .filter(move |(i, _)| ids.iter().any(|x| *i == x))
            .map(move |(i, v)| (*i, v))
            .collect()
    }

    pub fn contains(&self, id: Id) -> bool {
        self.data.get(&id).is_some()
    }

    pub fn insert(&mut self, id: Id, row: Row) -> Option<Row> {
        self.data.insert(id, row)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl<Id, Row> Table for BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    type Id = Id;
    type Row = Row;

    fn delete(&mut self, id: Id) -> Option<Row> {
        self.data.remove(&id)
    }

    fn get(&self, id: Id) -> Option<&Row> {
        BTreeTable::get(self, id)
    }
}

impl LogTable for BTreeTable<EntityTime, LogEntry> {
    fn get_logs_by_time(&self, time: u64) -> Vec<(EntityTime, LogEntry)> {
        self.data
            .iter()
            .filter(|(t, _)| t.1 == time)
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
}
