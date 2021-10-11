use crate::geometry::Axial;
use crate::terrain::TileTerrainType;
use serde::{Deserialize, Serialize};

/// Represents a connection of a room to another.
/// Length of the Bridge is defined by `radius - offset_end - offset_start`.
/// I choose to represent connections this way because it is much easier to invert them.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RoomConnection {
    pub direction: Axial,
    /// Where the bridge points start on the edge
    pub offset_start: u32,
    /// Where the bridge points end on the edge
    pub offset_end: u32,
}

/// Represents connections a room has to their neighbours. At most 6.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoomConnections(pub [Option<RoomConnection>; 6]);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default, Copy)]
#[serde(rename_all = "camelCase")]
pub struct TerrainComponent(pub TileTerrainType);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoomProperties {
    pub radius: u32,
    pub center: Axial,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoomComponent {
    /// Offset coordinates in world space
    pub offset: Axial,
    pub seed: u64,
}
