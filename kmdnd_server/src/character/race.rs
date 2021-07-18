use serde::{Deserialize, Serialize};

use crate::operation::{AbilityType, SkillType};

use super::{Language, ToolType};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Race {
    Dwarf,
    Elf,
    Halfling,
    Human,
    Dragonborn,
    Gnome,
    HalfElf,
    HalfOrc,
    Tiefling,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RacialTrait {
    AbilityScoreIncrease(Vec<AbilityType>),
    Darkvision,
    DwarvenResiliance,
    DwarvenCombatTraining,
    ToolProficiency(Vec<ToolType>),
    Stonecunning,
    Languages(Vec<Language>),
    DwarvenToughness,
    KeenSenses,
    FeyAncestry,
    Trance,
    ElfWeaponTraining,
    // Cantrip(),
    ExtraLanguage,
    Lucky,
    Brave,
    HalflingNimbleness,
    NaturallyStealthy,
    // DraconicAncestry(DragonType),
    BreathWeapon,
    DamageResistance,
    GnomeCunning,
    ArtificersLore,
    Tinker,
    SkillVersatility(Vec<SkillType>),
    Menacing,
    RelentlessEndurance,
    SavageAttacks,
    HellishResistance,
    InfernalLegacy,
}
