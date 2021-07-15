use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::{Character, CharacterId, Position};
use crate::database::MongoDatabase;
use crate::encounter::Encounter;
use crate::error::Error;
use crate::item::{DamageType, Weapon};
use crate::operation::{Interaction, InteractionId, RollType};
use crate::violations::Violation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub method: AttackMethod,
    pub targets: Vec<CharacterId>,
}

impl Attack {
    pub async fn submit(
        db: &MongoDatabase,
        campaign_id: CampaignId,
        encounter: &Encounter,
        source_character: Character,
        target_character_id: CharacterId,
        method: AttackMethod,
    ) -> Result<(Attack, Vec<Interaction>, Vec<Violation>), Error> {
        let target_character = db
            .characters()
            .fetch_character_by_campaign_and_id(campaign_id, target_character_id)
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

        let mut violations = vec![];

        let attack_range = method.normal_range();
        let current_range = Position::distance(source_position, target_position);
        if attack_range < current_range {
            violations.push(Violation::AttackNotInRange {
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

        Ok((attack, interactions, violations))
    }

    pub async fn handle_interaction_result(
        &self,
        db: &MongoDatabase,
        campaign_id: CampaignId,
        interaction: &Interaction,
        result: i32,
    ) -> Result<Vec<Interaction>, Error> {
        let new_interactions = match interaction.roll_type {
            RollType::Hit => {
                let target_character_id = self.targets[0]; // TODO:
                let target_character = db
                    .characters()
                    .fetch_character_by_campaign_and_id(campaign_id, target_character_id)
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
                let target_character_id = self.targets[0]; // TODO:
                let target_character = db
                    .characters()
                    .fetch_character_by_campaign_and_id(campaign_id, target_character_id)
                    .await?
                    .ok_or(Error::CharacterDoesNotExistInCampaign {
                        campaign_id,
                        character_id: target_character_id,
                    })?;

                let new_hit_points = i32::max(target_character.current_hit_points - result, 0);
                db.characters()
                    .update_character_hit_points(target_character, new_hit_points)
                    .await?;

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
