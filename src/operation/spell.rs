use serde::{Deserialize, Serialize};

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
