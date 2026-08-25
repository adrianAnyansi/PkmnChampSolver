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
    CHARGING
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

// TODO: Need to separate TrainedPokemon from active

/// Represents an active pokemon slot including current hp, status and boosts
// #[allow(Display)]
#[derive(Clone)]
pub struct ActivePokemon {
    pub pokemon: &'static Pokemon,
    /// additional stats, must be 32+32+2 total
    pub trained_stats: PokemonStats,
    pub ability: PokemonAbilityName,
    pub nature: PokemonNature,
    pub status: PokemonStatus,
    pub stat_modifier: [PokemonStatModifier; 5], // temp exclude evasion & acc
    /// current health in the battle
    pub current_hp: i32,
    /// Keep track of conditions in battle
    // pub in_battle_flags: HashMap<String, String>,
    // pub move_history: Vec<PokemonMoveName>,
    /// How many turns active on the field
    pub turns_active: u8,
    /// Flag to track protect in a row count
    pub consec_protect_count: u8,
    /// pointer to keep track of last move executed
    pub last_move_used: Option<PokemonMoveName>,
    pub forced_move: Option<PokemonMoveName>,
    pub battle_status: PokemonBitFlag128<PokemonBattleState>,
}

// TODO: Currently I'm moving the struct instead of referencing
// I don't want multiple structs of base pokemon but it's hard to 
// reason about this while being new to Rust.
// So I'm just going to leave this as a copy for now and remember I'm duplicating
impl ActivePokemon {
    pub fn new (pokemon:&'static Pokemon, 
        ability:PokemonAbilityName, 
        nature:PokemonNature,
        trained_stats: Option<PokemonStats>) -> Self {

            // let combined_stats = trained_stats.clone() + pokemon.base_stats.clone();
            let max_hp = get_full_stat(&pokemon.base_stats, trained_stats.as_ref(), PokemonStatName::HEALTH);
            let trained_stat = trained_stats.unwrap_or_else(|| PokemonStats::empty()); // placeholder
            Self {
                current_hp: max_hp, // copied first
                status: PokemonStatus::NONE,
                stat_modifier: [PokemonStatModifier::ZERO; 5],
                ability,
                nature,
                trained_stats: trained_stat,
                pokemon: pokemon, // this is moved here
                // move_history: Vec::new(),
                turns_active: 0, // first turn effect counter
                consec_protect_count: 0,
                last_move_used: None,
                forced_move: None,
                // in_battle_flags: HashMap::new(), // Keep track of various flags
                battle_status: PokemonBitFlag128::<PokemonBattleState>::empty(),
            }
    }

    pub fn quick(poke_name:PokemonName) -> ActivePokemon {
        let pokemon = pokemon::get_pkmn(poke_name);
        ActivePokemon::new(
            pokemon,
            PokemonAbilityName::Nothing,
            PokemonNature::Quirky,
            None
        )
    }

    /// This will calculate the full stat spread including boosts
    /// so calculate once and update if changes occur
    pub fn get_active_stat(&self, stat_type:PokemonStatName) -> i32 {
        let pkmn = &self.pokemon;

        let comb_stat = get_full_stat(&pkmn.base_stats, Some(&self.trained_stats), stat_type);

        match stat_type {
            PokemonStatName::HEALTH => comb_stat,
            _ => {
                // offset by 1 for the stat modifier
                let stat_idx = (stat_type as usize) - 1;
                comb_stat * self.stat_modifier[stat_idx]
            }
        }
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

    pub fn get_pkmn_type(&self) -> Vec<PokemonType> {
        // TODO: Calc this pokemon's current type based on more factors
        // Filter out TYPELESS to handle the null/None case
        self.pokemon.types.iter()
            .filter(|&&t| t != PokemonType::TYPELESS)
            .copied()
            .collect()
    }

    pub fn get_type_mult(&self, move_type:PokemonType) -> f64 {
        if move_type == PokemonType::TYPELESS {
            return 1.0
        };
        let types = self.get_pkmn_type();
        let mut type_mult = 1.0;
        for type_def in types {
            type_mult *= get_type_multipler(move_type, type_def);
        }
        type_mult
    }

}

impl core::fmt::Display for ActivePokemon {
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
        write!(f, "{}{status_short}{hp_pct_str}", self.pokemon)
    }
}


pub struct ActiveTeam {
    pub pokemon: [ActivePokemon; 6],
    pub used_mega: bool,
    pub has_mega: bool
    // tera
    // gigata?
    // any other team based stuff here
}


#[derive(PartialEq, Clone, Copy, EnumString, Display)]
pub enum BattleWeatherState {
    /// No weather
    NONE,
    SUN,
    RAIN,
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