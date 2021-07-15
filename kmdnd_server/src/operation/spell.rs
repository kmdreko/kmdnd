use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::{Character, Position};
use crate::database::Database;
use crate::encounter::Encounter;
use crate::error::Error;
use crate::operation::{AbilityType, InteractionId, RollType, SpellTarget};
use crate::violations::Violation;

use super::{Interaction, Operation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cast {
    pub spell: String,
    pub target: SpellTarget,
}

impl Cast {
    pub async fn submit(
        _db: &dyn Database,
        _campaign_id: CampaignId,
        _encounter: &Encounter,
        source_character: Character,
        name: String,
        target: SpellTarget,
    ) -> Result<(Cast, Vec<Interaction>, Vec<Violation>), Error> {
        let spell = Spell::fetch_spell_by_name(&name).ok_or(Error::SpellDoesNotExist { name })?;

        let (cast, interactions, violations) = match spell.name.as_str() {
            "Fireball" => {
                let source_position = source_character.position.as_ref().ok_or(
                    Error::CharacterDoesNotHavePosition {
                        character_id: source_character.id,
                    },
                )?;

                let cast_position = match &target {
                    SpellTarget::Position { position } => position,
                    unexpected_target => {
                        return Err(Error::CastUsesWrongTargetType {
                            expected_type: SpellTargetType::Position,
                            provided_type: unexpected_target.clone(),
                        })
                    }
                };

                let mut violations = vec![];

                let spell_range = match &spell.range {
                    SpellRange::Feet(feet) => *feet,
                    _ => {
                        return Err(Error::ExistentialState(
                            "Expected Fireball to have range in feet".to_string(),
                        ))
                    }
                };
                let cast_distance = Position::distance(source_position, cast_position);
                if cast_distance > spell_range {
                    violations.push(Violation::CastNotInRange {
                        request_character_id: source_character.id,
                        target_position: *cast_position,
                        spell_range,
                        current_range: cast_distance,
                    });
                }

                let interactions = vec![Interaction {
                    id: InteractionId::new(),
                    character_id: source_character.id,
                    roll_type: RollType::Damage,
                    result: None,
                }];

                let cast = Cast {
                    spell: spell.name,
                    target,
                };

                (cast, interactions, violations)
            }
            _ => unimplemented!("other spells not yet implemented"),
        };

        Ok((cast, interactions, violations))
    }

    #[allow(clippy::let_and_return)]
    pub async fn handle_interaction_result(
        &self,
        db: &dyn Database,
        campaign_id: CampaignId,
        encounter: &Encounter,
        operation: &Operation,
        interaction: &Interaction,
        result: i32,
    ) -> Result<Vec<Interaction>, Error> {
        let new_interactions = match self.spell.as_str() {
            "Fireball" => match interaction.roll_type {
                RollType::Damage => {
                    let position = match &self.target {
                        SpellTarget::Position { position } => position,
                        unexpected_target => {
                            return Err(Error::CastUsesWrongTargetType {
                                expected_type: SpellTargetType::Position,
                                provided_type: unexpected_target.clone(),
                            })
                        }
                    };

                    let mut characters_in_encounter = vec![];
                    for &character_id in &encounter.character_ids {
                        let character = db
                            .characters()
                            .fetch_character_by_campaign_and_id(campaign_id, character_id)
                            .await?
                            .ok_or(Error::CharacterDoesNotExistInCampaign {
                                campaign_id,
                                character_id,
                            })?;
                        characters_in_encounter.push(character);
                    }

                    let characters_in_range: Vec<_> = characters_in_encounter
                        .into_iter()
                        .filter(|character| {
                            character
                                .position
                                .map(|character_position| {
                                    character_position.distance(&position) <= 20.0
                                    // TODO: stop hard coding stuff
                                })
                                .unwrap_or(false)
                        })
                        .collect();

                    let interactions = characters_in_range
                        .into_iter()
                        .map(|character| Interaction {
                            id: InteractionId::new(),
                            character_id: character.id,
                            roll_type: RollType::Save(AbilityType::Dexterity),
                            result: None,
                        })
                        .collect();

                    interactions
                }
                RollType::Save(AbilityType::Dexterity) => {
                    let target_character = db
                        .characters()
                        .fetch_character_by_campaign_and_id(campaign_id, interaction.character_id)
                        .await?
                        .ok_or(Error::CharacterDoesNotExistInCampaign {
                            campaign_id,
                            character_id: interaction.character_id,
                        })?;

                    let damage_interaction = operation
                        .interactions
                        .iter()
                        .find(|i| i.roll_type == RollType::Damage)
                        .ok_or_else(|| {
                            Error::ExistentialState(
                                "Expected Fireball to have damage roll interaction".to_string(),
                            )
                        })?;
                    let max_damage = damage_interaction.result.ok_or_else(|| {
                        Error::ExistentialState(
                            "Expected Fireball damage roll to have result".to_string(),
                        )
                    })?;

                    let difficulty_class = 8; // TODO: based on caster's class and stats
                    let damage = if result >= difficulty_class {
                        max_damage / 2
                    } else {
                        max_damage
                    };

                    let new_hit_points = i32::max(target_character.current_hit_points - damage, 0);
                    db.characters()
                        .update_character_hit_points(target_character, new_hit_points)
                        .await?;

                    vec![]
                }
                _ => {
                    vec![]
                }
            },
            _ => vec![],
        };

        Ok(new_interactions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub level: i32,
    pub school: MagicSchool,
    pub casting_time: CastingTime,
    pub range: SpellRange,
    pub target: SpellTargetType,
    pub components: Vec<SpellComponent>,
    pub duration: SpellDuration,
    pub concentration: bool,
    pub description: String,
}

impl Spell {
    pub fn fetch_spell_by_name(name: &str) -> Option<Spell> {
        match name {
            "Fireball" => Some(Spell {
                name: "Fireball".to_string(),
                level: 3,
                school: MagicSchool::Evocation,
                casting_time: CastingTime::Action(1),
                range: SpellRange::Feet(150.0),
                target: SpellTargetType::Position,
                components: vec![
                    SpellComponent::Verbal,
                    SpellComponent::Somatic,
                    SpellComponent::Material(None),
                ],
                duration: SpellDuration::Instantaneous,
                concentration: false,
                description: "blows shit up".to_string(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellTargetType {
    Creature,
    Position,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastingTime {
    Action(i32),
    BonusAction(i32),
    Reaction(String),
    Minute(i32),
    Hour(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellComponent {
    Verbal,
    Somatic,
    Material(Option<i32>), // TODO: currency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellRange {
    Feet(f32),
    Touch,
    Personal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellDuration {
    Instantaneous,
    Round(i32),
    Minute(i32),
    Hour(i32),
    Day(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MagicSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromacy,
    Transmutation,
}
