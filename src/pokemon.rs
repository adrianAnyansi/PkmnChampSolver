/// Contains the pokemon data and rulesets


pub mod poke_stat;
pub mod moves;
pub mod types;

pub struct I32Range {
    low: i32,
    high: i32
}
use strum_macros::{Display, AsRefStr};

#[derive(Display, AsRefStr)]
pub enum PokemonName {
    Garchomp,
    Bisharp
}

pub enum PokemonNature {
    Brave
}

pub enum PokemonAbility {
    SandForce
}

pub enum PokemonItem {
    Garchompinite,
    SoftSand
}



use crate::pokemon::types::PokemonType;
use crate::pokemon::moves::PokemonMove;
use crate::pokemon::poke_stat::{PokemonStats, get_pkmn_stat};

pub struct Pokemon {
    pub name: PokemonName,
    pub base_stats: PokemonStats,
    pub trained_stats: Option<PokemonStats>,
    pub ability: PokemonAbility,
    pub nature: PokemonNature,
    pub learnset: Vec<PokemonMove>,
    pub weight: f64,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub held_item: PokemonItem
}

impl core::fmt::Display for Pokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let pk_name = self.name
        let item_str = format!("@ {}", "item_name"); //self.held_item
        write!(f, "{} {}", self.name, item_str)
    }
}

impl Pokemon {

    pub fn has_type(&self, pkm_type:PokemonType) -> bool {
        pkm_type == self.type1 || Some(pkm_type) == self.type2
    }
}


pub fn get_pkmn(pkmn:PokemonName) -> Pokemon {
    match pkmn {
        PokemonName::Garchomp => get_garchomp(),
        PokemonName::Bisharp => unimplemented!()
    }

}

// TODO: Put this in a data file
fn get_garchomp() -> Pokemon {
    Pokemon {
        name: PokemonName::Garchomp,
        base_stats: get_pkmn_stat(PokemonName::Garchomp),
        trained_stats: None,
        ability: PokemonAbility::SandForce,
        nature: PokemonNature::Brave,
        learnset: vec![],
        weight: 95.0,
        type1: PokemonType::DRAGON,
        type2: Some(PokemonType::GROUND),
        held_item: PokemonItem::SoftSand
    }
}