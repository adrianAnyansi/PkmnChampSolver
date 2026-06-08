// basic data for Pokemon battles


use std::{collections::VecDeque};

use crate::{math::{div_and_floor, mult_and_round}, pokemon::{Pokemon, PokemonAbility, moves::{PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::get_full_stat}};
use crate::pokemon::{poke_stat::{PokemonStatName, PokemonStats}, types::{PokemonType, get_type_multipler}};
use crate::pokemon::poke_stat::{PokemonStatModifier, PokemonNature};

#[allow(dead_code)]
pub enum PokemonStatus {
    NONE,
    BURNED,
    PARALYZED,
    FROZEN,
    SLEEP,
    POISONED,
}

static LEVEL: i32 = 50;
fn pkmn_damage_formula(power:i32,
    atk_stat:i32, def_stat:i32) -> i32 {

    let level_dmg = (2 * LEVEL) / 5 + 2;
    let power_dmg = level_dmg * power * atk_stat;
    let top_damage = div_and_floor(power_dmg, def_stat);
    let non_mult_dmg = div_and_floor(top_damage, 50) + 2;
    let final_damage = non_mult_dmg;
    // See https://bulbapedia.bulbagarden.net/wiki/Damage#Generation_V_onward for damage formula
    final_damage
}

/// Represents an active pokemon slot including current hp, status and boosts
// #[allow(Display)]
pub struct ActivePokemon {
    pub pokemon: &'static Pokemon,
    pub trained_stats: PokemonStats,
    pub ability: PokemonAbility,
    pub nature: PokemonNature,
    pub status: PokemonStatus,
    pub stat_modifier: [PokemonStatModifier; 5], // temp exclude evasion & acc
    // exclude crit
    current_hp: i32,
}

// TODO: Currently I'm moving the struct instead of referencing
// I don't want multiple structs of base pokemon but it's hard to 
// reason about this while being new to Rust.
// So I'm just going to leave this as a copy for now and remember I'm duplicating
impl ActivePokemon {
    pub fn new (pokemon:&'static Pokemon, 
        ability:PokemonAbility, 
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
            }
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
        write!(f, "{}@", self.pokemon)
    }
}

/// Generic Event representing a current action in the turn state.
/// Will include moves, ability/event resolves, etc.
/// Will think about how to structure this and what types make sense here

pub enum BattleAction<'battle> {
    Move(MoveAction<'battle>), // Pokemon is performing a move
    AbilityAction, // Ability 
    BattleEffect // Other effects 
}

#[derive(Clone, Copy, Debug)]
pub enum BattlePosition {
    F1,
    F2,
    B1,
    B2,
}

pub struct MoveAction<'battle> {
    pub source: BattlePosition,
    pub targets: Vec<BattlePosition>,
    pub pkm_move: &'battle PokemonMove,
}

/// Actions needed to be taken by calculated move
pub struct MoveResult {
    /// What type of result occurred here
    pub category: MoveResultEnum,
    /// String displayed when the effect occurs
    pub display_str: String
}

pub enum MoveResultEnum {
    DAMAGE,
    FAINTED,
    ABILITY_ACTIVATE,
    SECOND_EFFECT
}

/// Represents the state of the battle between any action/resolve.
/// This can include intermediate states
pub struct BattleState<'battle> {
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
    pub action_queue: VecDeque<BattleAction<'battle>>,
    pub turn_num: i32,
}

impl<'battle> BattleState<'battle> {

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
            action_queue: VecDeque::new(),
            turn_num: 0,
        }
    }

    fn get_default_poke_name (poke:&Option<ActivePokemon>) -> String {
        return poke.as_ref().map(|p| p.pokemon.name.to_string()).unwrap_or_else(|| "_".to_string());
    }

    fn get_front_poke(&self) -> String {
        format!("{} {}", 
            BattleState::get_default_poke_name(&self.f_poke1),
            BattleState::get_default_poke_name(&self.f_poke2))
    }

    pub fn get_print_state(&self) -> String {

        let back_row_str = format!("{} {}", 
            BattleState::get_default_poke_name(&self.b_poke1),
            BattleState::get_default_poke_name(&self.b_poke2));

        let field_state = format!("Weather: {}, Other: {}", 
            self.weather, self.terrain);

        let turn_num = self.turn_num;

        format!(
            "*Battle State* Turn: {turn_num}\n\
            Back: \t\t{back_row_str}\n\
            Front: {}\n\
            Field: {field_state}\n\
            ----------------------------",
            self.get_front_poke(),
        )
    }

    /// Add a move to the action queue
    pub fn queue_move (
        action_queue: &mut VecDeque<BattleAction<'battle>>,
        source: BattlePosition,
        pmove:&'battle PokemonMove,
        targets:Vec<BattlePosition>, 
        ) {

        let move_action: MoveAction = MoveAction{
            source,
            targets,
            pkm_move: pmove
        };

        let new_action: BattleAction = BattleAction::Move(move_action);

        action_queue.push_back(new_action);
    }

    fn get_active_mut(&mut self, position: BattlePosition) -> Option<&mut ActivePokemon> {
        match position {
            BattlePosition::F1 => self.f_poke1.as_mut(),
            BattlePosition::F2 => self.f_poke2.as_mut(),
            BattlePosition::B1 => self.b_poke1.as_mut(),
            BattlePosition::B2 => self.b_poke2.as_mut(),
        }
    }

    fn can_perform_move(&self, pkm_move:&PokemonMove, move_action: &MoveAction  ) -> bool {
        // check move conditions
        if !pkm_move.intn_condition_check(
            self, move_action) {
            return false;
        }

        true
    }

    /// Mutate the battle state
    fn perform_move(&mut self, move_action: &mut MoveAction) -> &Self {
        
        let source_active = self.get_active_mut(move_action.source).expect("source must exist");
        let power = move_action.pkm_move.power;
        let move_type = move_action.pkm_move.r#type;

        let atk_stat = match move_action.pkm_move.category {
            Physical => source_active.get_active_stat(PokemonStatName::ATTACK),
            Special => source_active.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
            _ => 1
        };
        let is_stab = [Physical, Special].contains(&move_action.pkm_move.category)
            && source_active.pokemon.has_type(move_action.pkm_move.r#type);
        let num_targets = move_action.targets.len();

        // Calculate everything per target from left->right
        for target_position in &move_action.targets {
            let target_pkmn = self.get_active_mut(*target_position).expect("target must exist");
            
            let poke = &target_pkmn.pokemon;
            println!("Start calc for target {poke}");

            // TODO: Calculate crit, including status and etc effects
            // let is_crit = false;

            let def_stat = match &move_action.pkm_move.category {
                Physical => target_pkmn.get_active_stat(PokemonStatName::DEFENSE),
                Special => target_pkmn.get_active_stat(PokemonStatName::SPECIAL_DEFENSE),
                _ => 1
            };

            let mut def_dmg = pkmn_damage_formula(power, atk_stat, def_stat);
            // TODO: I'll do this later
            // let rng_roll = [0.85, 100.0]; 

            let mut dmg_modifier_list:VecDeque<f64> = VecDeque::new();

            // Damage modifiers
            let stab_mult = if is_stab { 1.5
                // TODO: Adaptability & tera checks
            } else { 1.0 };

            dmg_modifier_list.push_back(stab_mult);
            
            // target multiplier
            let multi_target_mult = if num_targets > 1 {0.75} else {1.0};
            dmg_modifier_list.push_back(multi_target_mult);
            
            // Type multiplier
            let def_type_mult = target_pkmn.get_type_mult(move_type) as f64;
            // TODO: If 0, count as not applicable
            dmg_modifier_list.push_back(def_type_mult);
            
            // status modifiers, burn

            // other modifiers
            
            // Finally
            while let Some(modifier) = dmg_modifier_list.pop_front() {
                def_dmg = mult_and_round(def_dmg, modifier);
            };
            let calc_dmg = def_dmg;
            
            // Subtract health, then roll and apply secondary effects
            // target_pkmn.stat_modifier[0] = 2;
            let final_hp = (target_pkmn.current_hp-calc_dmg).max(0);
            
            // Resolve effects/abilties/etc based on HP
            // target_pkmn
            target_pkmn.current_hp = final_hp;
            // Resolve fainting

            // Return move result
            println!("{poke} took {calc_dmg} damage");

        
        }

        // With each multipler, the float must be floored

        // Run accuracy, determine if exists
        // Roll for critical hit

        self
    }

    pub fn perform_turn(&mut self) {
        // TODO: sort action queue

        // Can you perform this move in this battle state
        while self.action_queue.len() > 0 {
            
            let action = match self.action_queue.pop_front() {
                Some(action) => action,
                None => break
            };

            match action {
                BattleAction::Move(mut move_action) => {
                    if !self.can_perform_move(&move_action.pkm_move, &move_action) {
                        continue
                    };
                    println!("{} used {}!",
                        self.get_active_mut(move_action.source).unwrap(), 
                        move_action.pkm_move.name);
                    self.perform_move(&mut move_action);
                }
                _ => {
                    println!("Not yet implemented")
                }
            }


        }
        

    }
}


mod test {
    use crate::pokemon::poke_stat::PokemonStatName::{SPECIAL_ATTACK, SPECIAL_DEFENSE};

use super::*;

    #[test]
    fn test_move_calc() {
        // Using the test of Garchomp Draco Meteor on Kingambit
        let base_stat = PokemonStats {
            hp: 0,
            attack: 0,
            defense: 0,
            sp_attack: 80,
            sp_defense: 85,
            speed: 0
        };
        let power = 130;
        let atk_stat = get_full_stat(&base_stat, None, SPECIAL_ATTACK);
        let def_stat = get_full_stat(&base_stat, None, SPECIAL_DEFENSE);
        let dmg = pkmn_damage_formula(power, atk_stat, def_stat);

        // stab bonus
        let f_dmg = mult_and_round(dmg, 1.5);
        // type_multiplier
        let f2_dmg = mult_and_round(f_dmg, 0.5);
        assert_eq!(f2_dmg, 42);
    }
}