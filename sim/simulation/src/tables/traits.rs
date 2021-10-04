use crate::components;
use crate::indices::EntityTime;
use serde::Serialize;

/// TableIds may be used as indices of tables
pub trait TableId:
    'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug + Serialize
{
}

impl<T> TableId for T where
    T: 'static
        + Ord
        + PartialOrd
        + Eq
        + PartialEq
        + Copy
        + Default
        + Send
        + std::fmt::Debug
        + Serialize
{
}

/// TableRows may be used as the row type of a table
pub trait TableRow: 'static + std::fmt::Debug {}
impl<T: 'static + std::fmt::Debug> TableRow for T {}

/// Components define both their shape (via their type) and the storage backend that shall be used to
/// store them.
pub trait Component<Id: TableId>: TableRow {
    type Table: Table<Row = Self> + Default;
}

pub trait Table {
    type Id: TableId;
    type Row: TableRow;

    // Id is Copy
    fn delete(&mut self, id: Self::Id) -> Option<Self::Row>;
    fn get(&self, id: Self::Id) -> Option<&Self::Row>;

    fn name() -> &'static str {
        use std::any::type_name;

        type_name::<Self>()
    }
}

pub trait LogTable {
    fn get_logs_by_time(&self, time: u64) -> Vec<(EntityTime, components::LogEntry)>;
}
