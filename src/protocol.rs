use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ClientMessage {
    Direction { direction: Direction },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ServerMessage {
    InitClient { client_id: ClientId },
    SpawnRect { entity: Entity, pos: Vec3 },
    RectMove { entity: Entity, pos: Vec3 },
    DespawnRect { entity: Entity },
}
