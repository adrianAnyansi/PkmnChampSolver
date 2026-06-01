/// Contains the pokemon data and rulesets

struct PokemonStats {
    health: i32,
    attack: i32,
    defense: i32,
    sp_attack: i32,
    sp_defense: i32,
    speed: i32
}

enum PokemonName {
    Garchomp
}

enum PokemonAbility {
    SandForce
}

enum PokemonMove {
    Earthquake
}

enum PokemonItem {
    Garchompinite
}

use crate::battle::PokemonType;

pub struct Pokemon {
    name: PokemonName,
    base_stats: PokemonStats,
    trained_stats: PokemonStats,
    ability: PokemonAbility,
    learnset: Vec<PokemonMove>,
    weight: f64,
    type1: PokemonType,
    type2: Option<PokemonType>,
    held_item: PokemonItem
}