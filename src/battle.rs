// basic data for Pokemon battles

use core::fmt;

/// Pokemon Type
#[derive(Copy, Clone)] // copy trait added for trival enum copy
pub enum PokemonType {
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
    POISON
}

impl fmt::Display for PokemonType {
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
        }
    }
}

const SUPER_EFFECTIVE:f64 = 2f64;
const INEFFECTIVE:f64 = 0.5f64;
const IMMUNE:f64 = 0f64;

/// Contains the type advantages of the game. Will be converted into 
/// a sparse array at runtime for lookup
const TYPE_CHART_ARRAY: [f64; 18*18] = { 
    use PokemonType::*;
    
    // TODO: Use an array of normal
    const TUPLE_ARR: [(PokemonType, PokemonType, f64); 28] = [
        // Normal attacking
        (PokemonType::NORMAL, PokemonType::ROCK, INEFFECTIVE),
        (PokemonType::NORMAL, PokemonType::STEEL, INEFFECTIVE),
        (PokemonType::NORMAL, PokemonType::GHOST, IMMUNE),
        // FIRE attacking
        (PokemonType::FIRE, PokemonType::FIRE, INEFFECTIVE),
        (PokemonType::FIRE, PokemonType::ROCK, INEFFECTIVE),
        (PokemonType::FIRE, PokemonType::WATER, INEFFECTIVE),
        (PokemonType::FIRE, PokemonType::DRAGON, INEFFECTIVE),
        (PokemonType::FIRE, PokemonType::GRASS, SUPER_EFFECTIVE),
        (PokemonType::FIRE, PokemonType::BUG, SUPER_EFFECTIVE),
        (PokemonType::FIRE, PokemonType::STEEL, SUPER_EFFECTIVE),
        (PokemonType::FIRE, PokemonType::ICE, SUPER_EFFECTIVE),
        // Bug attacking
        (BUG, GRASS, SUPER_EFFECTIVE),
        (BUG, PSYCHIC, SUPER_EFFECTIVE),
        (BUG, DARK, SUPER_EFFECTIVE),
        (BUG, FIRE, INEFFECTIVE),
        (BUG, FIGHTING, INEFFECTIVE),
        (BUG, FLYING, INEFFECTIVE),
        (BUG, GHOST, INEFFECTIVE),
        (BUG, STEEL, INEFFECTIVE),
        (BUG, FAIRY, INEFFECTIVE),

        // Flying
        (FLYING, ELECTRIC, INEFFECTIVE),
        (FLYING, FIGHTING, SUPER_EFFECTIVE),
        (FLYING, GRASS, SUPER_EFFECTIVE),
        (FLYING, BUG, SUPER_EFFECTIVE),
        (FLYING, ROCK, INEFFECTIVE),
        (FLYING, STEEL, INEFFECTIVE),

        (GROUND, FLYING, IMMUNE),
        (GROUND, POISON, SUPER_EFFECTIVE)

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
