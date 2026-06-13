//! Serde model for MTGJSON's `AtomicCards.json`.
//!
//! Every list/map field is wrapped in `Option` because MTGJSON emits explicit
//! `null` (not just absence) for empty arrays like `keywords` and `producedMana`,
//! and serde's `#[serde(default)]` does not cover an explicit `null`. Stats such
//! as power/toughness/loyalty are strings ("*", "1+*", "X"), and the Vanguard
//! `hand`/`life` fields are strings like "+1"/"-3".

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct AtomicFile {
    #[serde(default)]
    pub meta: Meta,
    pub data: BTreeMap<String, Vec<Card>>,
}

#[derive(Deserialize, Default)]
pub struct Meta {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub date: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    // Present in the data; we key on the map name instead, but parse it for completeness.
    #[allow(dead_code)]
    pub name: Option<String>,
    pub face_name: Option<String>,
    pub side: Option<String>,
    pub layout: Option<String>,

    pub mana_cost: Option<String>,
    pub mana_value: Option<f64>,
    pub face_mana_value: Option<f64>,

    pub colors: Option<Vec<String>>,
    pub color_identity: Option<Vec<String>>,

    #[serde(rename = "type")]
    pub type_line: Option<String>,
    pub types: Option<Vec<String>>,
    pub subtypes: Option<Vec<String>>,
    pub supertypes: Option<Vec<String>>,

    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,

    pub text: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub printings: Option<Vec<String>>,

    pub edhrec_rank: Option<i64>,
    pub edhrec_saltiness: Option<f64>,

    pub is_reserved: Option<bool>,
    pub is_funny: Option<bool>,
    pub is_game_changer: Option<bool>,

    pub hand: Option<String>,
    pub life: Option<String>,

    pub leadership_skills: Option<LeadershipSkills>,
    pub identifiers: Option<Identifiers>,
    pub legalities: Option<BTreeMap<String, String>>,
    pub rulings: Option<Vec<Ruling>>,
}

#[derive(Deserialize, Default)]
pub struct LeadershipSkills {
    pub brawl: Option<bool>,
    pub commander: Option<bool>,
    pub oathbreaker: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identifiers {
    pub scryfall_oracle_id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct Ruling {
    pub date: Option<String>,
    pub text: Option<String>,
}
