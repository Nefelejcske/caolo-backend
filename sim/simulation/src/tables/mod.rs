//! The game state is represented by a relational model.
//! Tables are generic collections that store game data split by [shape] components.
//!
pub mod btree_table;
pub mod flag_table;
pub mod handle_table;
pub mod hex_grid;
pub mod iterators;
pub mod morton_hierarchy;
pub mod morton_table;
pub mod page_table;
pub mod traits;
pub mod unique_table;

pub use self::iterators::*;
pub use self::morton_hierarchy::*;
pub use self::traits::*;
