// basic data for Pokemon battles

use core::fmt;


/// Pokemon Type
#[allow(dead_code)]
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

use crate::pokemon::{Pokemon, moves::PokemonMove};

#[allow(dead_code)]
pub enum PokemonStatus {
    NONE,
    BURNED,
    PARALYZED,
    FROZEN,
    SLEEP,
    POISONED,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum PokemonStatModifier {
    ZERO = 0,
    MINUS_1 = -1,
    MINUS_2 = -2,
    MINUS_3 = -3,
    MINUS_4 = -4,
    MINUS_5 = -5,
    MINUS_6 = -6,
    PLUS_1 = 1,
    PLUS_2 = 2,
    PLUS_3 = 3,
    PLUS_4 = 4,
    PLUS_5 = 5,
    PLUS_6 = 6
}

fn get_stat_modify(poke_mod:PokemonStatModifier) -> f64 {
    use PokemonStatModifier::*;

    match poke_mod {
        ZERO => 1.0,
        MINUS_1 => 2.0/3.0,
        MINUS_2 => 2.0/4.0,
        MINUS_3 => 2.0/5.0,
        MINUS_4 => 2.0/6.0,
        MINUS_5 => 2.0/7.0,
        MINUS_6 => 2.0/8.0,
        PLUS_1  => 3.0/2.0,
        PLUS_2  => 4.0/2.0,
        PLUS_3  => 5.0/2.0,
        PLUS_4  => 6.0/2.0,
        PLUS_5  => 7.0/2.0,
        PLUS_6  => 8.0/2.0,
    }
}

/// Represents an active pokemon slot including current hp, status and boosts
pub struct ActivePokemon {
    pokemon: Pokemon,
    status: PokemonStatus,
    stat_modifier: [i8; 5], // temp exclude evasion & acc
    // exclude crit
    current_hp: i32,
}

// TODO: Currently I'm moving the struct instead of referencing
// I don't want multiple structs of base pokemon but it's hard to 
// reason about this while being new to Rust.
// So I'm just going to leave this as a copy for now and remember I'm duplicating
impl ActivePokemon {
    pub fn new (pokemon:Pokemon) -> Self {
        Self {
            current_hp: pokemon.base_stats.health, // copied first
            pokemon: pokemon, // this is moved here
            status: PokemonStatus::NONE,
            stat_modifier: [0;5],
        }
    }
}

/// Generic Event representing a current action in the turn state.
/// Will include moves, ability/event resolves, etc.
/// Will think about how to structure this and what types make sense here
pub struct BattleAction {
    name: String,
}

pub struct MoveAction<'a> {
    source: &'a Pokemon,
    target: Vec<Pokemon>,
    moveDet: &'a PokemonMove,
}

/// Represents the state of the battle between any action/resolve.
/// This can include intermediate states
pub struct BattleState {
    pub f_poke1: Option<ActivePokemon>,
    pub f_poke2: Option<ActivePokemon>,
    pub b_poke1: Option<ActivePokemon>,
    pub b_poke2: Option<ActivePokemon>,
    pub weather: String,
    pub terrain: String,
    pub effects: String,
    pub room: String,
    /// This will contain the many per battle effects that don't fit neatly
    /// i.e Rage Fist, Disguise, etc.
    pub internal_state: String,
    pub current_action: Option<String>,
    pub action_queue: Vec<BattleAction>,
    pub turn_num: i32,
}

impl BattleState {

    pub fn new () -> Self {
        BattleState {
            f_poke1: None,
            f_poke2: None,
            b_poke1: None,
            b_poke2: None,
            // Will implement this properly later in the future idk
            weather: "None".to_string(),
            terrain: "None".to_string(),
            effects: "None".to_string(),
            room: "None".to_string(),
            internal_state: "_".to_string(),
            current_action: None,
            action_queue: vec![],
            turn_num: 0,
        }
    }

    fn get_default_poke_name (poke:&Option<ActivePokemon>) -> String {
        return poke.as_ref().map(|p| p.pokemon.name.to_string()).unwrap_or_else(|| "_".to_string());
    }

    fn get_front_poke(&self) -> String {
        format!("Front: {} {}", 
            BattleState::get_default_poke_name(&self.f_poke1),
            BattleState::get_default_poke_name(&self.f_poke2))
    }

    pub fn get_print_state(&self) -> String {

        let back_row_str = format!("Back: {} {}", 
            BattleState::get_default_poke_name(&self.b_poke1),
            BattleState::get_default_poke_name(&self.b_poke2));

        let field_state = format!("Weather: {}, Other: {}", 
            self.weather, self.terrain);

        let turn_num = self.turn_num;

        format!(
            "*Battle State* Turn: {turn_num}\n\
            {back_row_str}\n\
            {}\n\
            Field: {field_state}",
            self.get_front_poke(),
        )
    }
}
