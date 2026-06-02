/// Contains the pokemon data and rulesets


pub mod poke_stat;
mod moves;

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

pub enum PokemonMove {
    Earthquake
}

pub enum PokemonItem {
    Garchompinite,
    SoftSand
}



use crate::battle::PokemonType;
// use crate::data::getPokemon;
use crate::pokemon::poke_stat::{PokemonStats, get_pkmn_stat};

pub struct Pokemon {
    name: PokemonName,
    base_stats: PokemonStats,
    trained_stats: Option<PokemonStats>,
    ability: PokemonAbility,
    nature: PokemonNature,
    learnset: Vec<PokemonMove>,
    weight: f64,
    type1: PokemonType,
    type2: Option<PokemonType>,
    held_item: PokemonItem
}

impl core::fmt::Display for Pokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let pk_name = self.name
        let item_str = format!("@ {}", "item_name"); //self.held_item
        write!(f, "{} {}", self.name, item_str)
    }
}


pub fn get_pkmn(pkmn:PokemonName) -> Pokemon {
    return Pokemon {
        name: PokemonName::Garchomp, 
        base_stats: get_pkmn_stat(PokemonName::Garchomp), 
        trained_stats: None, 
        ability: PokemonAbility::SandForce, 
        nature: PokemonNature::Brave, 
        learnset: Vec::new(), 
        weight: 209.4,
        type1: PokemonType::GROUND, 
        type2: Some(PokemonType::DRAGON), 
        held_item: PokemonItem::SoftSand
    };
}