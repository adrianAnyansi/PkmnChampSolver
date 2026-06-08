use serde::Deserialize;
use serde::{Deserializer};

/// Pokemon Type
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum  PokemonType {
    NORMAL,
    FIRE,
    WATER,
    GRASS,
    ELECTRIC,
    BUG,
    FLYING,
    FIGHTING,
    GHOST,
    DARK,
    PSYCHIC,
    FAIRY,
    DRAGON,
    ROCK,
    GROUND,
    ICE,
    STEEL,
    POISON,
    TYPELESS
}

impl<'de> Deserialize<'de> for PokemonType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "NORMAL" => Ok(PokemonType::NORMAL),
            "FIRE" => Ok(PokemonType::FIRE),
            "WATER" => Ok(PokemonType::WATER),
            "GRASS" => Ok(PokemonType::GRASS),
            "ELECTRIC" => Ok(PokemonType::ELECTRIC),
            "BUG" => Ok(PokemonType::BUG),
            "FLYING" => Ok(PokemonType::FLYING),
            "FIGHTING" => Ok(PokemonType::FIGHTING),
            "GHOST" => Ok(PokemonType::GHOST),
            "DARK" => Ok(PokemonType::DARK),
            "PSYCHIC" => Ok(PokemonType::PSYCHIC),
            "FAIRY" => Ok(PokemonType::FAIRY),
            "DRAGON" => Ok(PokemonType::DRAGON),
            "ROCK" => Ok(PokemonType::ROCK),
            "GROUND" => Ok(PokemonType::GROUND),
            "ICE" => Ok(PokemonType::ICE),
            "STEEL" => Ok(PokemonType::STEEL),
            "POISON" => Ok(PokemonType::POISON),
            "TYPELESS" => Ok(PokemonType::TYPELESS),
            _ => Err(serde::de::Error::custom(format!("Invalid type: {}", s))),
        }
    }
}

impl core::fmt::Display for PokemonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PokemonType::NORMAL     => write!(f, "NORMAL"),
            PokemonType::FIRE       => write!(f, "FIRE"),
            PokemonType::WATER      => write!(f, "WATER"),
            PokemonType::GRASS      => write!(f, "GRASS"),
            PokemonType::ELECTRIC   => write!(f, "ELECTRIC"),
            PokemonType::BUG        => write!(f, "BUG"),
            PokemonType::FLYING     => write!(f, "FLYING"),
            PokemonType::FIGHTING   => write!(f, "FIGHTING"),
            PokemonType::GHOST      => write!(f, "GHOST"),
            PokemonType::DARK       => write!(f, "DARK"),
            PokemonType::PSYCHIC    => write!(f, "PSYCHIC"),
            PokemonType::FAIRY      => write!(f, "FAIRY"),
            PokemonType::DRAGON     => write!(f, "DRAGON"),
            PokemonType::ROCK       => write!(f, "ROCK"),
            PokemonType::GROUND     => write!(f, "GROUND"),
            PokemonType::ICE        => write!(f, "ICE"),
            PokemonType::STEEL      => write!(f, "STEEL"),
            PokemonType::POISON     => write!(f, "POISON"),
            PokemonType::TYPELESS     => write!(f, "TYPELESS"),
        }
    }
}

const SUPER_EFFECTIVE:f64 = 2f64;
const NOT_EFFECTIVE:f64 = 0.5f64;
const IMMUNE:f64 = 0f64;

/// Contains the type advantages of the game. Will be converted into 
/// a sparse array at runtime for lookup
const TYPE_CHART_ARRAY: [f64; 18*18] = { 
    use PokemonType::*;
    
    // TODO: Use an array of normal
    const TUPLE_ARR: [(PokemonType, PokemonType, f64); 138] = [
        // Normal attacking
        (NORMAL, ROCK, NOT_EFFECTIVE),
        (NORMAL, STEEL, NOT_EFFECTIVE),
        (NORMAL, GHOST, IMMUNE),
        // FIRE attacking
        (FIRE, FIRE, NOT_EFFECTIVE),
        (FIRE, ROCK, NOT_EFFECTIVE),
        (FIRE, WATER, NOT_EFFECTIVE),
        (FIRE, DRAGON, NOT_EFFECTIVE),
        (FIRE, GRASS, SUPER_EFFECTIVE),
        (FIRE, BUG, SUPER_EFFECTIVE),
        (FIRE, STEEL, SUPER_EFFECTIVE),
        (FIRE, ICE, SUPER_EFFECTIVE),
        // Bug attacking
        (BUG, GRASS, SUPER_EFFECTIVE),
        (BUG, PSYCHIC, SUPER_EFFECTIVE),
        (BUG, DARK, SUPER_EFFECTIVE),
        (BUG, FIRE, NOT_EFFECTIVE),
        (BUG, FIGHTING, NOT_EFFECTIVE),
        (BUG, FLYING, NOT_EFFECTIVE),
        (BUG, GHOST, NOT_EFFECTIVE),
        (BUG, STEEL, NOT_EFFECTIVE),
        (BUG, FAIRY, NOT_EFFECTIVE),

        // Flying
        (FLYING, FIGHTING, SUPER_EFFECTIVE),
        (FLYING, GRASS, SUPER_EFFECTIVE),
        (FLYING, BUG, SUPER_EFFECTIVE),
        (FLYING, ELECTRIC, NOT_EFFECTIVE),
        (FLYING, ROCK, NOT_EFFECTIVE),
        (FLYING, STEEL, NOT_EFFECTIVE),
        // Ground
        (GROUND, FLYING, IMMUNE),
        (GROUND, POISON, SUPER_EFFECTIVE),
        (GROUND, ROCK, SUPER_EFFECTIVE),
        (GROUND, FIRE, SUPER_EFFECTIVE),
        (GROUND, ELECTRIC, SUPER_EFFECTIVE),
        (GROUND, STEEL, SUPER_EFFECTIVE),
        (GROUND, BUG, NOT_EFFECTIVE),
        (GROUND, GRASS, NOT_EFFECTIVE),
        // Steel
        (STEEL, STEEL, NOT_EFFECTIVE),
        (STEEL, FIRE, NOT_EFFECTIVE),
        (STEEL, WATER, NOT_EFFECTIVE),
        (STEEL, ELECTRIC, NOT_EFFECTIVE),
        (STEEL, ICE, SUPER_EFFECTIVE),
        (STEEL, ROCK, SUPER_EFFECTIVE),
        (STEEL, FAIRY, SUPER_EFFECTIVE),
        // Fighting
        (FIGHTING, NORMAL, SUPER_EFFECTIVE),
        (FIGHTING, ROCK, SUPER_EFFECTIVE),
        (FIGHTING, ICE, SUPER_EFFECTIVE),
        (FIGHTING, STEEL, SUPER_EFFECTIVE),
        (FIGHTING, DARK, SUPER_EFFECTIVE),
        
        (FIGHTING, FLYING, NOT_EFFECTIVE),
        (FIGHTING, POISON, NOT_EFFECTIVE),
        (FIGHTING, BUG, NOT_EFFECTIVE),
        (FIGHTING, PSYCHIC, NOT_EFFECTIVE),
        (FIGHTING, FAIRY, NOT_EFFECTIVE),
        (FIGHTING, GHOST, IMMUNE),

        // Poison
        (POISON, GRASS, SUPER_EFFECTIVE),
        (POISON, FAIRY, SUPER_EFFECTIVE),
        (POISON, POISON, NOT_EFFECTIVE),
        (POISON, GROUND, NOT_EFFECTIVE),
        (POISON, ROCK, NOT_EFFECTIVE),
        (POISON, GHOST, NOT_EFFECTIVE),        
        (POISON, STEEL, IMMUNE),

        (ROCK, FLYING, SUPER_EFFECTIVE),
        (ROCK, BUG, SUPER_EFFECTIVE),
        (ROCK, FIRE, SUPER_EFFECTIVE),
        (ROCK, ICE, SUPER_EFFECTIVE),
        (ROCK, FIGHTING, NOT_EFFECTIVE),
        (ROCK, GROUND, NOT_EFFECTIVE),
        (ROCK, STEEL, NOT_EFFECTIVE),

        (BUG, PSYCHIC, SUPER_EFFECTIVE),
        (BUG, DARK, SUPER_EFFECTIVE),
        (BUG, GRASS, SUPER_EFFECTIVE),
        (BUG, ROCK, NOT_EFFECTIVE),
        (BUG, STEEL, NOT_EFFECTIVE),
        (BUG, FIGHTING, NOT_EFFECTIVE),
        (BUG, FLYING, NOT_EFFECTIVE),
        (BUG, FIRE, NOT_EFFECTIVE),
        (BUG, FAIRY, NOT_EFFECTIVE),
        (BUG, GHOST, NOT_EFFECTIVE),
        (BUG, POISON, NOT_EFFECTIVE),

        (GHOST, GHOST, SUPER_EFFECTIVE),
        (GHOST, PSYCHIC, SUPER_EFFECTIVE),
        (GHOST, DARK, NOT_EFFECTIVE),
        (GHOST, NORMAL, IMMUNE),

        (FIRE, FIRE, NOT_EFFECTIVE),
        (FIRE, ROCK, NOT_EFFECTIVE),
        (FIRE, WATER, NOT_EFFECTIVE),
        (FIRE, DRAGON, NOT_EFFECTIVE),
        (FIRE, BUG, SUPER_EFFECTIVE),
        (FIRE, GRASS, SUPER_EFFECTIVE),
        (FIRE, ICE, SUPER_EFFECTIVE),
        (FIRE, STEEL, SUPER_EFFECTIVE),

        (WATER, DRAGON, NOT_EFFECTIVE),
        (WATER, ELECTRIC, NOT_EFFECTIVE),
        (WATER, GRASS, NOT_EFFECTIVE),
        (WATER, ROCK, SUPER_EFFECTIVE),
        (WATER, GROUND, SUPER_EFFECTIVE),
        (WATER, FIRE, SUPER_EFFECTIVE),

        (GRASS, DRAGON, NOT_EFFECTIVE),
        (GRASS, BUG, NOT_EFFECTIVE),
        (GRASS, FLYING, NOT_EFFECTIVE),
        (GRASS, POISON, NOT_EFFECTIVE),
        (GRASS, STEEL, NOT_EFFECTIVE),
        (GRASS, GRASS, NOT_EFFECTIVE),
        (GRASS, FIRE, NOT_EFFECTIVE),
        (GRASS, ROCK, SUPER_EFFECTIVE),
        (GRASS, GROUND, SUPER_EFFECTIVE),
        (GRASS, WATER, SUPER_EFFECTIVE),

        (ELECTRIC, GRASS, NOT_EFFECTIVE),
        (ELECTRIC, DRAGON, NOT_EFFECTIVE),
        (ELECTRIC, ELECTRIC, NOT_EFFECTIVE),
        (ELECTRIC, GROUND, IMMUNE),
        (ELECTRIC, WATER, SUPER_EFFECTIVE),
        (ELECTRIC, FLYING, SUPER_EFFECTIVE),
        
        (PSYCHIC, PSYCHIC, NOT_EFFECTIVE),
        (PSYCHIC, STEEL, NOT_EFFECTIVE),
        (PSYCHIC, DARK, IMMUNE),
        (PSYCHIC, FIGHTING, SUPER_EFFECTIVE),
        (PSYCHIC, POISON, SUPER_EFFECTIVE),

        (ICE, WATER, NOT_EFFECTIVE),
        (ICE, FIRE, NOT_EFFECTIVE),
        (ICE, STEEL, NOT_EFFECTIVE),
        (ICE, ICE, NOT_EFFECTIVE),
        (ICE, FLYING, SUPER_EFFECTIVE),
        (ICE, GRASS, SUPER_EFFECTIVE),
        (ICE, ROCK, SUPER_EFFECTIVE),
        (ICE, DRAGON, SUPER_EFFECTIVE),

        (DRAGON, STEEL, NOT_EFFECTIVE),
        (DRAGON, DRAGON, SUPER_EFFECTIVE),
        (DRAGON, FAIRY, IMMUNE),

        (DARK, FIGHTING, NOT_EFFECTIVE),
        (DARK, DARK, NOT_EFFECTIVE),
        (DARK, FAIRY, NOT_EFFECTIVE),
        (DARK, PSYCHIC, SUPER_EFFECTIVE),
        (DARK, GHOST, SUPER_EFFECTIVE),

        (FAIRY, FIRE, NOT_EFFECTIVE),
        (FAIRY, STEEL, NOT_EFFECTIVE),
        (FAIRY, POISON, NOT_EFFECTIVE),
        (FAIRY, DRAGON, SUPER_EFFECTIVE),
        (FAIRY, FIGHTING, SUPER_EFFECTIVE),
        (FAIRY, DARK, SUPER_EFFECTIVE),
    ];

    let mut arr: [f64; 324] = [1f64; 18*18];

    let mut iter = 0;

    while iter < TUPLE_ARR.len() {
        let (type_atk, type_def, mult) = TUPLE_ARR[iter];
        let hash_idx = type_hash_func(type_atk, type_def);
        arr[hash_idx] = mult;
        iter += 1;
    }

    arr

};

/// used to get the multipler for type matchups
const fn type_hash_func(type_atk:PokemonType, type_def:PokemonType) -> usize {
    let hash_idx = (type_atk as i32) * 18 + (type_def as i32);
    return hash_idx as usize;
}

/// get the type multipler for an attacking->defending type
pub fn get_type_multipler(type_atk:PokemonType, type_def:PokemonType) -> f64 {
    let mult = TYPE_CHART_ARRAY[type_hash_func(type_atk, type_def)];
    mult
}