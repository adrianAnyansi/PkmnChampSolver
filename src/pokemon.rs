/// Contains the pokemon data and rulesets


pub mod poke_stat;
pub mod moves;
pub mod types;

pub struct I32Range {
    low: i32,
    high: i32
}
use strum_macros::{Display, AsRefStr, EnumString};
use serde::Deserialize;

use std::sync::LazyLock as Lazy;

#[derive(Display, AsRefStr, EnumString, Deserialize, 
    Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PokemonName {
    Garchomp,
    Kingambit,
    Tyranitar
}


#[allow(non_camel_case_types)]
#[derive(Deserialize, EnumString, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PokemonAbility {
    Sand_Force,
    Rough_Skin,
    Defiant,
    Sand_Stream,
    Unnerve,
    Supreme_Overlord,
    Sand_Veil
}

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PokemonItem {
    Garchompinite,
    SoftSand
}



use crate::pokemon::types::PokemonType;
use crate::pokemon::moves::PokemonMoveName;
use crate::pokemon::poke_stat::{PokemonStats};
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use serde::{Deserializer};


#[derive(Debug)]
pub struct Pokemon {
    pub name: PokemonName,
    pub base_stats: PokemonStats,
    // pub trained_stats: Option<PokemonStats>,
    pub abilities: Vec<PokemonAbility>,
    // pub nature: PokemonNature,
    pub learnset: Vec<PokemonMoveName>,
    pub weight: f64,
    pub types: Vec<PokemonType>,
    // pub held_item: PokemonItem
}

impl<'de> Deserialize<'de> for Pokemon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PokemonData {
            #[serde(default)]
            name: Option<String>,
            base_stats: PokemonStats,
            trained_stats: Option<PokemonStats>,
            #[serde(default)]
            abilities: Vec<String>,
            #[serde(default)]
            learnset: Option<Vec<String>>,
            weight: f64,
            types: Vec<PokemonType>,
        }

        let data = PokemonData::deserialize(deserializer)?;
        let pokemon_label = data
            .name
            .as_deref()
            .unwrap_or("<unknown>");
        let abilities = data.abilities.into_iter()
            .filter_map(|ability| {
                match PokemonAbility::from_str(&ability) {
                    Ok(parsed_ability) => Some(parsed_ability),
                    Err(_) => {
                        eprintln!(
                            "Warning: Ignoring unknown ability '{}' for pokemon '{}'",
                            ability, pokemon_label
                        );
                        None
                    }
                }
            })
            .collect();
        let learnset = data.learnset.unwrap_or_default().into_iter()
            .filter_map(|move_name| {
                match PokemonMoveName::from_str(&move_name) {
                    Ok(parsed_move) => Some(parsed_move),
                    Err(_) => {
                        eprintln!(
                            "Warning: Ignoring unknown move '{}' for pokemon '{}'",
                            move_name, pokemon_label
                        );
                        None
                    }
                }
            })
            .collect();
        let mut types = data.types;
        
        // Pad with TYPELESS if only one type
        while types.len() < 2 {
            types.push(PokemonType::TYPELESS);
        }
        
        // Ensure we don't have more than 2 types
        types.truncate(2);

        Ok(Pokemon {
            name: PokemonName::Garchomp, // This will be overwritten by get_stat_json
            base_stats: data.base_stats,
            // trained_stats: data.trained_stats,
            abilities,
            learnset,
            weight: data.weight,
            types,
        })
    }
}

impl core::fmt::Display for Pokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: With all info moved to Active, name should be fine right now
        // let detail_str = "";
        write!(f, "{}", self.name)
    }
}

impl Pokemon {

    pub fn has_type(&self, pkm_type:PokemonType) -> bool {
        self.types.iter().any(|&t| t == pkm_type && t != PokemonType::TYPELESS)
    }
}

// static POKE_JSON_FILENAME: &str = "src/data/pokemon.json";
static POKE_JSON_STR: &str = include_str!("data/pokemon.json");
pub static POKEMON_HASH: Lazy<HashMap<PokemonName, Pokemon>> = Lazy::new(|| get_stat_json());
pub fn get_stat_json() -> HashMap<PokemonName, Pokemon> {
    // let file = std::fs::File::open(POKE_JSON_FILENAME).expect("Failed to open stat json file");
    // let reader = std::io::BufReader::new(file);

    
    // First deserialize as a raw HashMap<String, Value>
    // let raw_map: HashMap<String, serde_json::Value> = serde_json::from_reader(reader)
    //     .expect("Failed to parse pokemon json file");

    let raw_map: HashMap<String, serde_json::Value> = serde_json::from_str(POKE_JSON_STR)
        .expect("Failed to parse pokemon json file");
    
    // Then manually deserialize each entry and filter out those with unknown pokemon names
    let poke_map: HashMap<PokemonName, Pokemon> = raw_map.into_iter()
        .filter_map(|(k, v)| {
            // Try to parse the pokemon name
            match PokemonName::from_str(&k) {
                Ok(name) => {
                    // Try to deserialize the pokemon data
                    match serde_json::from_value::<Pokemon>(v) {
                        Ok(mut pokemon) => {
                            pokemon.name = name;
                            Some((name, pokemon))
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to deserialize pokemon '{}': {}", k, e);
                            None
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Warning: Ignoring unknown pokemon '{}' in JSON file", k);
                    None
                }
            }
        })
        .collect();
    poke_map
}


pub fn get_pkmn(pkmn:PokemonName) -> &'static Pokemon {
    match POKEMON_HASH.get(&pkmn) {
        Some(pokemon) => pokemon,
        None => unimplemented!("{} data not implemented yet", pkmn),
    }
}
