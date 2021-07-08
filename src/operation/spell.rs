use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character;
use crate::encounter::Encounter;
use crate::error::Error;
use crate::operation::{AbilityOrSkillType, AbilityType, InteractionId, RollType, SpellTarget};
use crate::violations::Violation;

use super::{Cast, Interaction};

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

    pub async fn submit(
        db: &Database,
        campaign_id: CampaignId,
        encounter: &Encounter,
        name: String,
        target: SpellTarget,
    ) -> Result<(Cast, Vec<Interaction>, Vec<Violation>), Error> {
        let spell = Spell::fetch_spell_by_name(&name).ok_or(Error::SpellDoesNotExist { name })?;

        let (cast, interactions, violations) = match spell.name.as_str() {
            "Fireball" => {
                let position = match target {
                    SpellTarget::Position { position } => position,
                    unexpected_target => {
                        return Err(Error::CastUsesWrongTargetType {
                            expected_type: SpellTargetType::Position,
                            provided_type: unexpected_target,
                        })
                    }
                };

                let violations = vec![];

                let mut characters_in_encounter = vec![];
                for &character_id in &encounter.character_ids {
                    let character = character::db::fetch_character_by_campaign_and_id(
                        &db,
                        campaign_id,
                        character_id,
                    )
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
                                character_position.distance(&position) <= 150.0
                            })
                            .unwrap_or(false)
                    })
                    .collect();

                let interactions = characters_in_range
                    .into_iter()
                    .map(|character| Interaction {
                        id: InteractionId::new(),
                        character_id: character.id,
                        roll_type: RollType::Save(AbilityOrSkillType::Ability(
                            AbilityType::Dexterity,
                        )),
                        result: None,
                    })
                    .collect();

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
