use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    Ping,
    Pong,
    JoinRoom { player_name: String },
    PlayerStateUpdate { id: String, x: f32, y: f32, rotation: f32 },
}
