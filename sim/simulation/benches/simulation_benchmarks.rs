mod btree_table;
mod morton_table;
mod pathfinding_benches;
mod table_join;

use criterion::criterion_main;

criterion_main!(
    morton_table::morton_benches,
    btree_table::btree_benches,
    table_join::join_benches,
    pathfinding_benches::pathfinding_benches
);
