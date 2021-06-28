use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::{self, Character, CharacterId, Position};
use crate::encounter::Encounter;
use crate::error::Error;
use crate::item::{DamageType, Weapon};
use crate::operation::{Interaction, InteractionId, RollType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub method: AttackMethod,
    pub targets: Vec<CharacterId>,
}

impl Attack {
    pub async fn submit(
        db: &Database,
        campaign_id: CampaignId,
        encounter: &Encounter,
        source_character: Character,
        target_character_id: CharacterId,
        method: AttackMethod,
    ) -> Result<(Attack, Vec<Interaction>), Error> {
        let target_character = character::db::fetch_character_by_campaign_and_id(
            &db,
            campaign_id,
            target_character_id,
        )
        .await?
        .ok_or(Error::CharacterNotInCampaign {
            campaign_id,
            character_id: target_character_id,
        })?;

        if !encounter.character_ids.contains(&target_character_id) {
            return Err(Error::CharacterNotInEncounter {
                campaign_id,
                encounter_id: encounter.id,
                character_id: target_character_id,
            });
        }

        let source_position =
            source_character
                .position
                .as_ref()
                .ok_or(Error::CharacterDoesNotHavePosition {
                    character_id: source_character.id,
                })?;
        let target_position =
            target_character
                .position
                .as_ref()
                .ok_or(Error::CharacterDoesNotHavePosition {
                    character_id: target_character.id,
                })?;

        let attack_range = method.normal_range();
        let current_range = Position::distance(source_position, target_position);
        if attack_range < current_range {
            return Err(Error::AttackNotInRange {
                request_character_id: source_character.id,
                target_character_id: target_character.id,
                attack_range,
                current_range,
            });
        }

        let interactions = vec![Interaction {
            id: InteractionId::new(),
            character_id: source_character.id,
            roll_type: RollType::Hit,
            result: None,
        }];

        let attack = Attack {
            method,
            targets: vec![target_character.id],
        };

        Ok((attack, interactions))
    }

    pub async fn handle_interaction_result(
        &self,
        db: &Database,
        campaign_id: CampaignId,
        interaction: &Interaction,
        result: i32,
    ) -> Result<Vec<Interaction>, Error> {
        let new_interactions = match interaction.roll_type {
            RollType::Hit => {
                let target_character_id = self.targets[0]; // TODO:
                let target_character = character::db::fetch_character_by_campaign_and_id(
                    &db,
                    campaign_id,
                    target_character_id,
                )
                .await?
                .ok_or(Error::CharacterDoesNotExistInCampaign {
                    campaign_id,
                    character_id: target_character_id,
                })?;

                if target_character.stats.armor_class <= result {
                    vec![Interaction {
                        id: InteractionId::new(),
                        character_id: interaction.character_id,
                        roll_type: RollType::Damage,
                        result: None,
                    }]
                } else {
                    vec![]
                }
            }
            RollType::Damage => {
                // TODO: do actual damage
                vec![]
            }
            _ => {
                vec![]
            }
        };

        Ok(new_interactions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum AttackMethod {
    Unarmed(DamageType),
    Weapon(Weapon),
    ImprovisedWeapon(Weapon), // TODO: maybe Item
}

impl AttackMethod {
    pub fn normal_range(&self) -> f32 {
        match self {
            AttackMethod::Unarmed(_) => 5.0,
            AttackMethod::Weapon(weapon) => weapon.normal_range(),
            AttackMethod::ImprovisedWeapon(weapon) => weapon.normal_range(),
        }
    }
}
