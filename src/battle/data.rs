#![allow(non_camel_case_types)]
// Continaing battle state info away from engine impl

use strum_macros::{Display, EnumString};

use crate::pokemon::types::{PokemonType, get_type_multipler};
use crate::pokemon::{self, Pokemon, PokemonAbilityName, PokemonName};
use crate::pokemon::moves::{BitFlagValue128, PokemonBitFlag128, PokemonMoveName};

/// Contains volatile/permanent states affecting pokemon in battle
#[derive(Copy, Clone, Debug)]
pub enum PokemonBattleState {
    PROTECT, // Pokemon is protected, immune to all* damage
    /// Pokemon is flinching and cannot act this turn
    FLINCHING,
    /// Pokemon is charging for next turn
    CHARGING,
    /// Confused state
    CONFUSED,
    /// Infatuation (in love) state
    INFATUATION
}

impl BitFlagValue128 for PokemonBattleState {
    fn as_u128(self) -> u128 {
        self as u128
    }
}


#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq )]
pub enum PokemonStatus {
    NONE,
    BURNED,
    PARALYZED,
    FROZEN,
    SLEEP,
    POISONED,
    TOXIC
}
use crate::battle::data::PokemonStatus::{BURNED, FROZEN, NONE, PARALYZED, POISONED, SLEEP, TOXIC};
use crate::pokemon::poke_stat::{PokemonNature, PokemonStatModifier, PokemonStatName, PokemonStats, get_full_stat};

impl std::fmt::Display for PokemonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            NONE => write!(f, "None"),
            BURNED => write!(f, "Burned"),
            PARALYZED => write!(f, "Paralyze"),
            FROZEN => write!(f, "Frozen"),
            SLEEP => write!(f, "Sleep"),
            POISONED => write!(f, "Poisoned"),
            TOXIC => write!(f, "Toxic")
        }
    }
}

impl PokemonStatus {
    pub fn get_3lt(&self) -> Option<&str> {
        match self {
            NONE => None,
            BURNED => Some("BRN"),
            PARALYZED => Some("PAR"),
            FROZEN => Some("FRZ"),
            SLEEP => Some("SLP"),
            POISONED => Some("PSN"),
            TOXIC => Some("TOX")
        }
    }
}

/// Contains volatile/permanent states affecting the field
enum PokemonFieldState {
    WIDE_GUARD,
}

/// Represents a Pokemon with its species and trainer-selected configuration.
#[derive(Clone)]
pub struct TrainedPokemon {
    pub pokemon: &'static Pokemon,
    /// additional stats, must be 32+32+2 total
    pub trained_stats: PokemonStats,
    pub ability: PokemonAbilityName,
    pub nature: PokemonNature,
}

impl TrainedPokemon {
    pub fn new (pokemon:&'static Pokemon,
        ability:PokemonAbilityName,
        nature:PokemonNature,
        trained_stats: Option<PokemonStats>) -> Self {
        Self {
            pokemon,
            trained_stats: trained_stats.unwrap_or_else(PokemonStats::empty),
            ability,
            nature,
        }
    }

    pub fn get_stat(&self, stat_type:PokemonStatName) -> i32 {
        get_full_stat(&self.pokemon.base_stats, Some(&self.trained_stats), stat_type)
    }

    pub fn get_pkmn_type(&self) -> Vec<PokemonType> {
        self.pokemon.types.iter()
            .filter(|&&t| t != PokemonType::TYPELESS)
            .copied()
            .collect()
    }

    pub fn get_type_mult(&self, move_type:PokemonType) -> f64 {
        if move_type == PokemonType::TYPELESS {
            return 1.0
        };
        self.get_pkmn_type().into_iter()
            .map(|type_def| get_type_multipler(move_type, type_def))
            .product()
    }
}

/// Represents inactive pokemon in battle but not on the field
#[derive(Clone)]
pub struct InActivePokemon<'battle> {
    pub trained_pokemon: &'battle TrainedPokemon,
    pub status: PokemonStatus,
    pub current_hp: u32,

    pub disguise_flag: bool,
    pub eiscue_flag: bool
    // etc
}

/// Represents an active pokemon slot including current hp, status and boosts
#[derive(Clone, Copy)]
pub struct ActivePokemon<'battle> {
    pub trained_pokemon: &'battle TrainedPokemon,

    pub status: PokemonStatus,
    pub stat_modifier: [PokemonStatModifier; 5], // temp exclude evasion & acc
    /// current health in the battle
    pub current_hp: i32,
    // pub move_history: Vec<PokemonMoveName>,
    /// How many turns active on the field
    pub turns_active: u8,
    /// Flag to track protect in a row count
    pub consec_protect_count: u8,
    /// pointer to keep track of last move executed
    pub last_move_used: Option<PokemonMoveName>,
    /// Keep track if previous move failed
    pub last_move_failed: bool,
    pub forced_move: Option<PokemonMoveName>,
    /// All status 
    pub battle_status: PokemonBitFlag128<PokemonBattleState>,
    /// Confusion counter
    pub confusion_count: u8,
}

impl<'battle> ActivePokemon<'battle> {
    pub fn new (trained_pokemon:&'battle TrainedPokemon) -> Self {
            let max_hp = trained_pokemon.get_stat(PokemonStatName::HEALTH);
            Self {
                current_hp: max_hp, // copied first
                status: PokemonStatus::NONE,
                stat_modifier: [PokemonStatModifier::ZERO; 5],
                trained_pokemon,
                // move_history: Vec::new(),
                turns_active: 0, // first turn effect counter
                consec_protect_count: 0,
                last_move_used: None,
                last_move_failed: false,
                forced_move: None,
                // in_battle_flags: HashMap::new(), // Keep track of various flags
                battle_status: PokemonBitFlag128::<PokemonBattleState>::empty(),
                confusion_count: 0,
            }
    }

    pub fn quick(poke_name:PokemonName) -> ActivePokemon<'static> {
        let pokemon = pokemon::get_pkmn(poke_name);
        let trained_pokemon = Box::leak(Box::new(TrainedPokemon::new(
            pokemon,
            PokemonAbilityName::Nothing,
            PokemonNature::Quirky,
            None
        )));
        ActivePokemon::new(trained_pokemon)
    }

    /// This will calculate the full stat spread including boosts
    /// so calculate once and update if changes occur
    pub fn get_active_stat(&self, stat_type:PokemonStatName) -> i32 {
        let comb_stat = self.trained_pokemon.get_stat(stat_type);

        match stat_type {
            PokemonStatName::HEALTH => comb_stat,
            _ => {
                // offset by 1 for the stat modifier
                let stat_idx = (stat_type as usize) - 1;
                comb_stat * self.stat_modifier[stat_idx]
            }
        }
    }

    pub fn get_type_mult(&self, move_type:PokemonType) -> f64 {
        self.trained_pokemon.get_type_mult(move_type)
    }

    pub fn get_active_stat_boost(&mut self, stat_type:PokemonStatName) -> &mut PokemonStatModifier {

        match stat_type {
            PokemonStatName::HEALTH => unimplemented!("Illegal"),
            _ => {
                // offset by 1 for the stat modifier
                let stat_idx = (stat_type as usize) - 1;
                &mut self.stat_modifier[stat_idx]
            }
        }
    }
    pub fn get_active_stat_modf(&self, stat_type:PokemonStatName) -> &PokemonStatModifier {

        match stat_type {
            PokemonStatName::HEALTH => unimplemented!("Illegal"),
            _ => {
                // offset by 1 for the stat modifier
                let stat_idx = (stat_type as usize) - 1;
                &self.stat_modifier[stat_idx]
            }
        }
    }

}

impl core::fmt::Display for ActivePokemon<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hp_pct_str:String = String::new();
        let max_hp = self.get_active_stat(PokemonStatName::HEALTH);
        
        // TODO: Get 3 letter version
        let mut status_short:String = String::new();
        if self.status != PokemonStatus::NONE {
            status_short = self.status.to_string();
        }

        if self.current_hp < max_hp {
            let pct = (self.current_hp as f64 / max_hp as f64) * 100.0;
            hp_pct_str = format!(" {}%", pct.round());
        }
        write!(f, "{}{status_short}{hp_pct_str}", self.trained_pokemon.pokemon)
    }
}

#[derive(Clone, Copy)]
pub struct ActiveTeam<'battle> {
    pub pokemon: [Option<ActivePokemon<'battle>>; 6],
    /// Mega tracking
    pub used_mega: bool,
    pub has_mega: bool
    // tera
    // gigantamax?
    // any other team based stuff here
}

impl<'battle> ActiveTeam<'battle> {
    // TODO: Use Trained Pokemon instead

    /// Create new team from pokemon
    pub fn new_from_pokemon (pokemon_vec:Vec<ActivePokemon>) -> ActiveTeam<'_> {

        let mut pokemon:[Option<ActivePokemon>; 6] = [None; 6];
        for (idx, poke) in pokemon_vec.iter().enumerate() {
            pokemon[idx] = Some(*poke);
        }

        ActiveTeam {
            pokemon,
            used_mega: false,
            has_mega: true
        }
    }

    pub fn empty () -> ActiveTeam<'battle> {
        ActiveTeam::new_from_pokemon(vec![])
    }

    pub fn add_poke(&mut self, new_poke:ActivePokemon<'battle>) -> usize {

        let mut empty_idx:usize = 0;
        for poke in self.pokemon.iter() {
            if poke.is_none() {
                break
            }
            empty_idx += 1;
        }
        if empty_idx < self.pokemon.len() {
            self.pokemon[empty_idx] = Some(new_poke);
        }
        // TODO: Need to error if no empty index
        empty_idx
    }

    pub fn get(&self, index:usize) -> Option<&ActivePokemon<'battle>> {
        if index >= self.pokemon.len() {
            return None
        }
        self.pokemon[index].as_ref()
    }

    pub fn get_mut(&mut self, index:usize) -> Option<&mut ActivePokemon<'battle>> {
        if index >= self.pokemon.len() {
            return None
        }
        self.pokemon[index].as_mut()
    }
}


#[derive(PartialEq, Clone, Copy, EnumString, Display)]
pub enum BattleWeatherState {
    /// No weather
    NONE,
    /// Sun weather
    SUN,
    /// Rain weather
    RAIN,
    /// Sandstorm weather
    SANDSTORM,
    /// No longer possible
    // HAIL,
    /// Boosts Blizzard and Ice defense
    SNOW,
    /// Primal Sun from Origin Groudon
    EXTREME_SUN,
    /// Primal Rain from Origin Kyogre
    EXTREME_RAIN,
    /// Primal weather from Mega Rayquaza
    STRONG_WINDS
}

#[derive(Clone, Copy, EnumString, Display)]
pub enum BattleTerrain {
    NONE,
    ELECTRIC,
    GRASSY,
    PSYCHIC,
    MISTY
}