use crate::character::{CharacterId, Position};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Violation {
    CharacterMovementExceeded {
        character_id: CharacterId,
        maximum_movement: f32,
        current_movement: f32,
        request_movement: f32,
    },
    AttackNotInRange {
        request_character_id: CharacterId,
        target_character_id: CharacterId,
        attack_range: f32,
        current_range: f32,
    },
    CastNotInRange {
        request_character_id: CharacterId,
        target_position: Position,
        spell_range: f32,
        current_range: f32,
    },
}
