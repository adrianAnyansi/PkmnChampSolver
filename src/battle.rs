// basic data for Pokemon battles
#![allow(dead_code)]

use std::clone;
use std::collections::HashMap;
use std::hash::Hash;
use std::{collections::VecDeque};

pub mod battle_processor;
// use crate::batt

use crate::battle::PokemonStatus::{BURNED, FROZEN, NONE, PARALYZED, POISONED, SLEEP, TOXIC};
use crate::pokemon::moves::PokemonMoveFlag::{IGNORE_ACC, PROTECT, PROTECT_ACC, PROTECT_COUNTER};
use crate::pokemon::moves::{MoveEffect, PokemonMoveFlag, PokemonMoveName, get_move};
use crate::pokemon::poke_stat::PokemonStatName::HEALTH;
use crate::{battle::battle_processor::BattleContainer};
use crate::math::{PkmnRational, div_and_floor, gen_power_set, mult_and_round}; 
use crate::pokemon::{self, Pokemon, PokemonAbilityName, PokemonName, moves::{BattlePreAction, BattleTarget, PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::get_full_stat};
use crate::pokemon::{poke_stat::{PokemonStatName, PokemonStats}, types::{PokemonType, get_type_multipler}};
use crate::pokemon::poke_stat::{PokemonStatModifier, PokemonNature};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, )]
pub enum PokemonStatus {
    NONE,
    BURNED,
    PARALYZED,
    FROZEN,
    SLEEP,
    POISONED,
    TOXIC
}

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

static LEVEL: i32 = 50;
fn pkmn_damage_formula(power:i32,
    atk_stat:i32, 
    def_stat:i32) -> i32 {

    let level_dmg = (2 * LEVEL) / 5 + 2;
    let power_dmg = level_dmg * power * atk_stat;
    let top_damage = div_and_floor(power_dmg, def_stat);
    let non_mult_dmg = div_and_floor(top_damage + 2*50, 50);
    let final_damage = non_mult_dmg;
    // See https://bulbapedia.bulbagarden.net/wiki/Damage#Generation_V_onward for damage formula
    final_damage
}

/// Represents an active pokemon slot including current hp, status and boosts
// #[allow(Display)]
#[derive(Clone)]
pub struct ActivePokemon {
    pub pokemon: &'static Pokemon,
    pub trained_stats: PokemonStats,
    pub ability: PokemonAbilityName,
    pub nature: PokemonNature,
    pub status: PokemonStatus,
    pub stat_modifier: [PokemonStatModifier; 5], // temp exclude evasion & acc
    // exclude crit
    current_hp: i32,
    /// Keep track of conditions in battle
    pub in_battle_flags: HashMap<String, String>,
    // pub move_history: Vec<PokemonMoveName>,
    pub turns_active: u8,
    pub protect_count: u8,
    pub last_move_used: Option<PokemonMoveName>
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
                protect_count: 0,
                last_move_used: None,
                in_battle_flags: HashMap::new() // Keep track of various flags
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
        let mut hp_pct_str:String = "".to_string();
        let max_hp = self.get_active_stat(PokemonStatName::HEALTH);
        
        if self.current_hp < max_hp {
            let pct = (self.current_hp as f64 / max_hp as f64) * 100.0;
            hp_pct_str = format!(" {}%", pct.round());
        }
        write!(f, "{}-{hp_pct_str}", self.pokemon)
    }
}

/// Generic Event representing a current action in the turn state.
/// Will include moves, ability/event resolves, etc.
/// Will think about how to structure this and what types make sense here
#[derive(Clone)]
pub enum BattleAction<'battle> {
    Move(MoveAction<'battle>), // Pokemon is performing a move
    Status(StatusAction),
    Stat(Vec<StatAction>),
    AbilityAction, // Ability 
    /// Pokemon took damage from any source
    Damage(DamageEffect),
    Faint(BattlePosition),
    Protect(PokemonMoveName, BattlePosition)
}

impl core::fmt::Display for BattleAction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleAction::Move(act) => 
                write!(f, "BattleAction::Move({})", act.pkm_move.name),
            BattleAction::Stat(act) => 
                write!(f, "BattleAction::Stat({})", act[0].stat_name),
            BattleAction::Damage(act) => 
                write!(f, "BattleAction::Damage({})", act.damage_source),
            BattleAction::Status(act) =>
                write!(f, "BattleAction::Status({})", act),
            BattleAction::Faint(act) => 
                write!(f, "BattleAction::Faint({:?})", act),
            _ => write!(f, "BattleAction<>")
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum BattlePosition {
    F1,
    F2,
    B1,
    B2,
}

impl BattlePosition {

    pub fn get_opposing(self) -> BattlePosition {
        use BattlePosition::*;
        match self {
            B1 => F1,
            B2 => F2,
            F1 => B1,
            F2 => B2
        }
    }

    pub fn get_ally(self) -> BattlePosition {
        use BattlePosition::*;
        match self {
            B1 => B2,
            B2 => B1,
            F1 => F2,
            F2 => F1
        }
    }

    pub fn get_opposing_team(self) -> Vec<BattlePosition> {
        use BattlePosition::*;
        match self {
            B1 | B2 => vec![F1,F2],
            F1 | F2 => vec![B1,B2]
        }
    }

    pub fn get_ally_team(self) -> Vec<BattlePosition> {
        use BattlePosition::*;
        match self {
            B1 | B2 => vec![B1,B2],
            F1 | F2 => vec![F1,F2]
        }
    }
}

// I wrote this cause I wanted to be able to copy the battle structs
// but I shouldn't be copying Actions by value, explicit clones are better
// But I'm leaving this in case needed
struct BattlePositionVec {
    pub targets: [BattlePosition; 4]
}

/// Move being performed by Pokemon
#[derive(Debug, Clone)]
pub struct MoveAction<'battle> {
    pub source: BattlePosition,
    pub targets: Vec<BattlePosition>,
    pub pkm_move: &'battle PokemonMove,
}

/// Stat Modifier Change being performed
#[derive(Clone)]
pub struct StatAction {
    pub targets: Vec<BattlePosition>,
    pub stat_name: PokemonStatName,
    pub change: PokemonStatModifier,
    pub pct_chance: PkmnRational
}

#[derive(Clone)]
pub struct StatusAction {
    pub targets: Vec<BattlePosition>,
    pub status: PokemonStatus,
    pub accuracy: PkmnRational
}

impl std::fmt::Display for StatusAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "StatusAct: {} {:?} {}", 
            self.status, self.targets, self.accuracy.str_pct())
    }
}

/// Battle action with generic target/accuracy 
pub struct BattleActionCtn<'battle> {
    /// Action type
    pub action: BattleAction<'battle>,
    // action type/enum
    pub targets: Vec<BattlePosition>,
    pub accuracy: PkmnRational

}

/// Actions needed to be taken by calculated move
pub struct MoveResult {
    /// What type of result occurred here
    pub category: MoveResultEnum,
    /// String displayed when the effect occurs
    pub display_str: String
}

/// Damage Effect calculated by a move or other source
#[derive(Clone)]
pub struct DamageEffect {
    pub target: BattlePosition,
    pub calc_damage: i32,
    // TODO: This needs to handle multiple things like Ability damage,
    // Whirlpool, regular moves, Status
    pub damage_source: String 
}

pub struct AddEffect {
    pub target: BattlePosition,
    pub effect_type: BattleEffect,
    pub damage_source: String
}

/// Enum representing effects from moves & etc
#[derive(Clone, Copy, Debug)] 
pub enum BattleEffect {
    Flinch,
    Trapped,
    Confused
}

pub enum DamageSource {
    Move(PokemonMove),
    Ability(PokemonAbilityName),
    Status(PokemonStatus)
}

#[allow(non_camel_case_types)]
pub enum MoveResultEnum {
    DAMAGE,
    FAINTED,
    ABILITY_ACTIVATE,
    SECOND_EFFECT
}

/// Represents the state of the battle between any action/resolve.
/// This can include intermediate states
#[derive(Clone)]
pub struct BattleState<'battle> {
    pub f_poke1: Option<ActivePokemon>,
    pub f_poke2: Option<ActivePokemon>,
    pub b_poke1: Option<ActivePokemon>,
    pub b_poke2: Option<ActivePokemon>,
    pub weather: String,
    // active terrain (only 1) on the field
    pub terrain: String,
    pub effects: String,
    pub room: String,
    /// This will contain the many per battle effects that don't fit neatly
    /// i.e Rage Fist, Disguise, etc.
    pub internal_state: String,
    // pub current_action: Option<String>,
    // NOTE: If speed/ability/etc order is hard to order, create a different queue
    pub action_queue: VecDeque<BattleAction<'battle>>,
    pub turn_num: i32,
    pub action_num: i32,
    pub action_strs: Vec<String>
}

impl<'battle> BattleState<'battle> {

    pub fn new () -> Self {
        BattleState {
            f_poke1: None,
            f_poke2: None,
            b_poke1: None,
            b_poke2: None,
            // Will implement this properly later in the future idc rn
            weather: "None".to_string(),
            terrain: "None".to_string(),
            effects: "None".to_string(),
            room: "None".to_string(),
            internal_state: "_".to_string(),
            // current_action: None,
            action_queue: VecDeque::new(),
            turn_num: 1,
            action_num: 0,
            // Keep track of messages from actions/debug this frame
            action_strs: Vec::new()
        }
    }

    pub fn simple (f_poke:ActivePokemon, 
        b_poke:ActivePokemon) -> Self {
            let mut bs = BattleState::new();
            bs.f_poke1 = Some(f_poke);
            bs.b_poke1 = Some(b_poke);

            bs
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
            self.b_poke1.as_ref().map_or("_".to_string(), |poke| poke.to_string()),
            // BattleState::get_default_poke_name(&self.b_poke1),
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

    fn get_active(&self, position: BattlePosition) -> Option<&ActivePokemon> {
        match position {
            BattlePosition::F1 => self.f_poke1.as_ref(),
            BattlePosition::F2 => self.f_poke2.as_ref(),
            BattlePosition::B1 => self.b_poke1.as_ref(),
            BattlePosition::B2 => self.b_poke2.as_ref(),
        }
    }

    fn can_perform_stat(&self, stat_action:&StatAction) -> Vec<BattlePosition> {

            // TODO: Check if blocked by pokemon ability/item
            // TODO: Check field conditions

            let valid_targets:Vec<&BattlePosition> = stat_action.targets.iter().filter(
                |pos:&&BattlePosition| {
                    let t_act_poke = self.get_active(**pos);

                    // Ensure that target pokemon exists
                    if t_act_poke.is_none() {
                        return false
                    }

                    // Check if blocked by ability or item (Clear Body, Covert Cloak)

                    let curr_pkmn = t_act_poke.expect("Non-null effect");
                    
                    // Check if stat is maxed, then cancel
                    let stat_ref = curr_pkmn.get_active_stat_modf(stat_action.stat_name);
                    if *stat_ref == PokemonStatModifier::MINUS_6 && stat_action.change.direction() == -1 
                    || *stat_ref == PokemonStatModifier::PLUS_6 && stat_action.change.direction() == 1 {
                        return false
                    }
                    true
                }
            ).collect();
            
            // NOTE: ugh- the position is getting copied by iter()
            return valid_targets.iter().copied().copied().collect()

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
    #[deprecated(note="Modifies state directly")]
    fn perform_move(&mut self, move_action: &mut MoveAction) -> &Self {
        
        let source_act_pkmn = self.get_active_mut(move_action.source).expect("source must exist");
        let power = move_action.pkm_move.power;
        let move_type = move_action.pkm_move.r#type;

        let atk_stat = match move_action.pkm_move.category {
            Physical => source_act_pkmn.get_active_stat(PokemonStatName::ATTACK),
            Special => source_act_pkmn.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
            _ => 1
        };
        let is_stab = [Physical, Special].contains(&move_action.pkm_move.category)
            && source_act_pkmn.pokemon.has_type(move_action.pkm_move.r#type);
        let num_targets = move_action.targets.len();
        let mut result_act_vec: Vec<BattleAction> = vec![];

        // Calculate everything per target from left->right
        for target_position in &move_action.targets {
            let target_pkmn = self.get_active_mut(*target_position).expect("target must exist");
            
            let poke = &target_pkmn.pokemon;
            println!("*Start damage calc {} for target {poke}", move_action.pkm_move.name);

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
            
            // Finally sum these up
            while let Some(modifier) = dmg_modifier_list.pop_front() {
                def_dmg = mult_and_round(def_dmg, modifier);
            };
            let calc_dmg = def_dmg;
            
            // Subtract health, then roll and apply secondary effects
            let dmg_effect = DamageEffect {
                target: target_position.clone(),
                calc_damage: calc_dmg,
                damage_source: format!("{poke} took {calc_dmg} damage!")
            };

            // Create a damage effect that the Battle system can resolve
            // Any health changes resolve in damage step
            result_act_vec.push(BattleAction::Damage(dmg_effect));
            
        
        }
        
        // Check after_hit effects
        let hit_effects = move_action.pkm_move.after_hit(self);
        let resolved_actions = hit_effects.iter().map(|pre_action|
            BattleState::convert_to_action(pre_action, move_action.source));
        for act in resolved_actions {
            result_act_vec.push(act);
        }
        while let Some(action) = result_act_vec.pop() {
            self.action_queue.push_front(action);
        }

        self
    }

    /// Perform stat modifier change for a single pokemon
    #[deprecated(note="use sim_stat & perform_stat_change instead")]
    fn perform_stat(&mut self, stat_action: StatAction) -> &Self {

        for target in stat_action.targets {
            let target_poke = self.get_active_mut(target);
            if target_poke.is_none() { continue; }

            // TODO: Check if stat is blocked by ability or other (forgor)
            let target_act_pkmn = target_poke.expect("Non-null");
            
            let stat_ref= target_act_pkmn.get_active_stat_boost(stat_action.stat_name);
            let pre_boost = *stat_ref;
            *stat_ref += stat_action.change;


            let stat_changed = pre_boost != *stat_ref;
            // TODO: Check if abilities trigger on ability change
            // TODO: Log flag for stat change
            if (stat_changed) {
                let change_dir = if [PokemonStatModifier::MINUS_1, PokemonStatModifier::MINUS_2].contains(&stat_action.change) {"fell"} else {"rose"};
                let final_value = *stat_ref;
                println!("{}'s {} {change_dir} [{:?}]!", target_act_pkmn.pokemon, 
                    stat_action.stat_name, 
                    final_value,
                );
            }
        }
        
        self
    }

    /// Simulate a stat change
    fn sim_stat(&self, stat_action: StatAction) -> Vec<BattleContainer<'battle>> {

        let change_dir = if stat_action.change.direction() == 1 {"rose"} else {"fell"};
        
        // Get all targets that are hit
        let valid_targets = self.can_perform_stat(&stat_action);
        let prob_set = gen_power_set(vec![
            stat_action.pct_chance; valid_targets.len()]);
        let mut result_vecs:Vec<BattleContainer> = vec![];

        for (bin_comb, rat) in prob_set.iter().enumerate() {
            if prob_set[bin_comb] == PkmnRational::ZERO() { continue; }

            let mut clone_state = self.clone();
            let mut stat_msg = String::new();

            for (idx, target_pos) in valid_targets.iter().enumerate() {
                if (bin_comb >> idx) & 0b1 == 0 {
                    let target_poke = clone_state.get_active(*target_pos).unwrap();
                    stat_msg.push_str(&format!("{} avoided the stat change!", target_poke));
                    continue
                }
                clone_state.perform_stat_change(*target_pos, &stat_action);
                let target_poke = clone_state.get_active(*target_pos).unwrap();
                stat_msg.push_str(&format!("{}'s {} {change_dir} to [{:?}]!", target_poke.pokemon, 
                    stat_action.stat_name, 
                    target_poke.get_active_stat_modf(stat_action.stat_name),
                ));
            }
            result_vecs.push(
                BattleContainer::simple(clone_state, *rat)
                .add_msg(stat_msg)
            );
        }

        result_vecs
        
        // Build states with permutation of each if not 100%
        // if stat_action.pct_chance == PkmnRational::ONE() {
        //     // 1 state with every change
        //     for target_pos in valid_targets {
        //         state_clone.perform_stat_change(target_pos, &stat_action);
        //     }
        //     let act_poke = state_clone.get_active(*target_pos).unwrap();
        //     let change_dir = if stat_action.change.direction() == 1 {"rose"} else {"fell"};
        //     let tmp_msg = format!("{}'s {} {change_dir} [{:?}]!", act_poke.pokemon, 
        //                     stat_action.stat_name, 
        //                     act_poke.get_active_stat_modf(stat_action.stat_name),
        //                 );
        //     return vec![*BattleContainer::simple(state_clone, 
        //         PkmnRational::ONE()).add_msg(&tmp_msg)]
        // } else {
        //     const POWER_SET:[u32; 2] = [0b1, 0b11];
        //     let inverse_chance = PkmnRational::ONE() - stat_action.pct_chance;
        //     let pct_container = 
        //     match valid_targets.len() {
        //         1 => vec![inverse_chance, stat_action.pct_chance],
        //         2 => vec![
        //             inverse_chance ^ 2,
        //             (inverse_chance ^ 1) * stat_action.pct_chance ^ 1,
        //             (inverse_chance ^ 1) * stat_action.pct_chance ^ 1,
        //             stat_action.pct_chance ^ 2,
        //         ],
        //         _ => panic!("Invalid number of targets {}", valid_targets.len())
        //     };

        //     // No need to manual calc the NULL case anymore
        //     let mut result_bcs = vec![
        //         BattleContainer::simple(state_clone, 
        //         PkmnRational::ONE() - stat_action.pct_chance ^ valid_targets.len() as u32
        //     )];
        //     let num_combs = POWER_SET[valid_targets.len()];

        //     let fn_stat_change = 
        //     |state: &mut BattleState<'battle>, target_pos:&BattlePosition| {
        //         state.perform_stat_change(*target_pos, &stat_action);
        //     };

        //     for bin_comb in 1..num_combs {
        //         let mut int_clone = self.clone();
        //         let mut msg = String::new();
        //         for (idx,target_pos) in valid_targets.iter().enumerate() {
        //             if (bin_comb >> idx) & 0b1 == 1 {
        //                 int_clone.perform_stat_change(*target_pos, &stat_action);
        //                 let act_poke = int_clone.get_active(*target_pos).unwrap();
        //                 let change_dir = if stat_action.change.direction() == 1 {"rose"} else {"fell"};
        //                 let tmp_msg = format!("{}'s {} {change_dir} [{:?}]!", act_poke.pokemon, 
        //                     stat_action.stat_name, 
        //                     act_poke.get_active_stat_modf(stat_action.stat_name),
        //                 );
        //                 msg.push_str(&tmp_msg);
        //                 msg.push('\n');
        //             }
        //         }
        //         result_bcs.push(
        //             BattleContainer::simple(int_clone, 
        //             pct_container[bin_comb as usize])
        //         );
                
        //         // Hacky method of getting this to the bc
        //         let last = result_bcs.last_mut().unwrap();
        //         last.message = msg;
        //     }
            
        //     return result_bcs
        // }

        // TODO: Log stat change flag on the pokemon
            // if (stat_changed) {
            //     let change_dir = if [PokemonStatModifier::MINUS_1, PokemonStatModifier::MINUS_2].contains(&stat_action.change) {"fell"} else {"rose"};
            //     let final_value = *stat_ref;
            //     println!("{}'s {} {change_dir} [{:?}]!", target_act_pkmn.pokemon, 
            //         stat_action.stat_name, 
            //         final_value,
            //     );
            // }
        
        // self
    }

    /// Simulate performing a status to the change
    fn sim_status(&self, status_action: StatusAction) -> Vec<BattleContainer<'battle>> {

        // TODO: Validate status is not blocked by field/battle
        // let valid_targets = self.can_perform_status(&status_action);
        // NOTE: Targets is always 1 for status
        let valid_targets = &status_action.targets;
        // TODO: If target is impossible, quit returning base state

        // Build result vec
        let mut result_vecs:Vec<BattleContainer> = vec![];
        // TODO: Put this into the status Action like a normal person
        let prob_set = gen_power_set(
            vec![status_action.accuracy; valid_targets.len()]);
        
        // for each combination in the power_set, I need to edit the cloned battle state
        // and then add to queue if anything triggers
        for (bin_comb, rat) in prob_set.iter().enumerate() {
            if prob_set[bin_comb] == PkmnRational::ZERO() { continue; }

            let mut status_msg = String::new();
            let mut clone_state = self.clone();
            for (idx,target) in valid_targets.iter().enumerate() {
                // perform status change on clone
                if (bin_comb >> idx) & 0b1 == 0 {
                    status_msg.push_str("status chance fail");
                    continue
                }
                clone_state.exec_status_change(*target, &status_action);
                status_msg.push_str(&format!("{} was statused [{}]", 
                    clone_state.get_active(*target).unwrap(), status_action.status));
            }
            result_vecs.push(
                BattleContainer::simple(clone_state, *rat));
            result_vecs.last_mut().unwrap().message = status_msg;
        }

        result_vecs
    }

    // Simulate a move hit and create states from this
    fn sim_move(&self, move_action: &MoveAction) -> Vec<BattleContainer<'battle>> {
        let source_act_pkmn = self.get_active(move_action.source).expect("source must exist");

        let move_msg = format!("{} used {}!",
                        self.get_active(move_action.source).unwrap(), 
                        move_action.pkm_move.name);
        
        // TODO: Do valid target function check
        let move_targets:Vec<(&ActivePokemon, &BattlePosition)> = move_action.targets.iter().filter_map(
            |position| {
                let poke = self.get_active(*position);
                if poke.is_some() {
                    return Some((poke.unwrap(), position))
                } else {
                    return None
                }
            }
        )
        .collect();
        let num_valid_targets = move_targets.len();
        let mut result_queue:Vec<Vec<BattleAction>> = vec![];

        // Check valid targets on field and calc hit
        for (target_act_pkmn, target_pos) in &move_targets {
            
            // Contains resulting actions from move hit
            let mut result_act_vec: Vec<BattleAction> = vec![];
            let def_poke = &target_act_pkmn.pokemon;

            // TODO: Calculate crit, including status and etc effects

            if move_action.pkm_move.is_attack() {
                println!("INT[{}/{}] Start damage calc for target {def_poke}",
                    self.turn_num, self.action_num);
                let move_damage = BattleState::calc_move_damage(move_action, 
                    num_valid_targets, source_act_pkmn, target_act_pkmn);

                let bat_pos = **target_pos;
                // Trigger DamageEffect
                let dmg_effect = DamageEffect {
                    target: bat_pos,
                    calc_damage: move_damage,
                    damage_source: format!("{target_act_pkmn} took {move_damage} damage!")
                };
                
                result_act_vec.push(BattleAction::Damage(dmg_effect));
            } else {
                // status move dont do damage effects, only 2nd targets
                println!("INT[{}/{}] Status move target {def_poke}",
                    self.turn_num, self.action_num);
            }
            
            // Process secondary effects and add to queue
            // -------------------------------------------------------------
            use crate::pokemon::moves::MoveEffect;
            if move_action.pkm_move.hit_actions.len() > 0 {
                
                // generate on hit effects to the action queue
                for hit_action in &move_action.pkm_move.hit_actions {

                    // TODO: All move_effects should contain targetting and pct_chance
                    // So the battle_action conversion is generic
                    let b_action:Option<BattleAction> = match hit_action {
                        effect @ MoveEffect::Stat(stat_change) => {
                            let stat_target_pos = BattleState::convert_effect_target_to_position(
                                stat_change.target_type, move_action.source, Some(**target_pos));

                            // NOTE: Stats are not combined*
                            Some(BattleState::convert_effect_to_baction(
                                effect,
                                stat_target_pos))
                            },
                        MoveEffect::Status(_, target, _rat) |
                        MoveEffect::General(_, target, _rat) => {
                            let status_target_pos = BattleState::convert_effect_target_to_position(
                                *target, move_action.source, Some(**target_pos));
                            
                            Some(BattleState::convert_effect_to_baction(hit_action, status_target_pos))
                        }
                        _ => {println!("WARNING: MoveEffect not implemented"); None}
                    };

                    if let Some(action) = b_action {
                        // NOTE: hit_Action order should not matter*
                        // cloned_state.action_queue.push_front(action);
                        result_act_vec.push(action);
                    }
                }
            } // 2nd effects are done
            
            result_queue.push(result_act_vec);

            
        }

        // TODO: Accuracy can change based on defender, validate this
        // TODO: If can't miss, override to 1
        let mut move_acc = move_action.pkm_move.accuracy;
        
        // Check if move need protect accuracy
        if move_action.pkm_move.flags.has_flag(PROTECT_ACC) {
            // let last_move = get_move(source_act_pkmn.last_move_used.unwrap() );
            // if last_move.flags.has_flag(PROTECT) {
                move_acc = PkmnRational::new(1, 
                    source_act_pkmn.protect_count as u32 * 3).float();
            // }
        }

        if move_action.pkm_move.flags.has_flag(IGNORE_ACC) {
            move_acc = 1.0; //
        } else {
            // TODO: Item/ability accuracy modifications
            // move_acc = 1.0;
        }


        // Increment protect counter if protect counter move
        // move to protect phase
        if move_action.pkm_move.flags.has_flag(PROTECT_COUNTER) {
            source_act_pkmn.protect_count += 1;
        } else {
            source_act_pkmn.protect_count = 0;
        }


        let prob_set = gen_power_set(
            vec![PkmnRational::from_float(move_acc); num_valid_targets]);

        let add_to_queue = 
        |state:&mut BattleState<'battle>, battle_actions:&Vec<BattleAction<'battle>>| {
            for ba in battle_actions.iter().rev() {
                // NOTE this is cloned because multiple borrow occurs
                state.action_queue.push_front(ba.clone());
            }
        };

        let mut result_bcs:Vec<BattleContainer> = Vec::new();
        for (bin_comb, rat) in prob_set.iter().enumerate() {
            if *rat == PkmnRational::ZERO() {continue;}
            let mut cloned_state = self.clone();
            let mut bc_msg:String = move_msg.clone();
            
            // Add attack to history
            // let atk_pkmn = cloned_state.get_active_mut(move_action.source).unwrap();
            // atk_pkmn.move_history.push(move_action.pkm_move.name);

            for (idx,target_pos) in move_targets.iter().enumerate() {
                    if (bin_comb >> idx) & 0b1 == 1 {
                        add_to_queue(&mut cloned_state, &result_queue[idx]);
                        
                        // int_clone.perform_stat_change(*target_pos, &stat_action);
                    }
                    // if missed, add message here
                    else {
                        let missed_pkmn_opt = cloned_state.get_active(*target_pos.1);
                        if let Some(missed_pkmn) = missed_pkmn_opt {
                            bc_msg.push_str(
                                &format!("\nAttack missed on {}", missed_pkmn).to_string()
                            );
                        }
                    }
                }
            let mut new_bc = BattleContainer::simple(cloned_state, *rat);
            new_bc.message = bc_msg;
            result_bcs.push(new_bc)
        }
        // Now return all the new battle_states that occur
        result_bcs
    }

    /// Base power calculation for moves, TODO: covering special power calcs
    fn base_power_calc(move_action: &MoveAction, act_poke:&ActivePokemon) -> i32 {
        let base_power = move_action.pkm_move.power;

        let atk_stat = match move_action.pkm_move.category {
            Physical => act_poke.get_active_stat(PokemonStatName::ATTACK),
            Special => act_poke.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
            _ => 1
        };

        // TODO: Special power calcs

        return base_power
    }

    fn calc_move_damage(move_action: &MoveAction,
        num_valid_targets:usize, 
        atk_poke:&ActivePokemon, def_poke:&ActivePokemon) -> i32 {

            // Would like to cache this but base_power on weight or Foul Play
            // Requires more thought

            let atk_stat = match move_action.pkm_move.category {
                Physical => atk_poke.get_active_stat(PokemonStatName::ATTACK),
                Special => atk_poke.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
                _ => 1
            };

            let base_power = move_action.pkm_move.power;

            let def_stat = match &move_action.pkm_move.category {
                Physical => def_poke.get_active_stat(PokemonStatName::DEFENSE),
                Special => def_poke.get_active_stat(PokemonStatName::SPECIAL_DEFENSE),
                _ => 1
            };

            let move_type = move_action.pkm_move.r#type;
            let base_damage = pkmn_damage_formula(base_power, atk_stat, def_stat);

            // ------------
            
            let mut dmg_modifier_list:VecDeque<f64> = VecDeque::new();

            // Multi-Targets (x0.75 in doubles)
            if num_valid_targets > 1 {
                dmg_modifier_list.push_back(0.75);
            }

            // Parental Bond check (0.25 if duplicate strike)

            // Weather multiplier x1.5 or x0.5
            // GlaiveRush x2
            // Critical hit x1.5
            // Random factor [x85, x100] then /100

            /*
            STAB (x1.5 regular, x2.0 adaptability, x1.5 override on pledge combo
            TERA x1.5 on og type but not tera, x2 if tera type == og type or ADPT no tera
                x2.25 if match tera and APT
             */
            if atk_poke.pokemon.has_type(move_type) {
                dmg_modifier_list.push_back(1.5);
            }
            // 
            /* Type effectiveness
                See Forest Curse, Burn Up, Double Shock and Trick-or-Treat for type changes
                Typeless moves ignores changes
                Grounded flying = 1, Ring Target(?)
                Scrappy, Foresight, Odor Sleuth, Miracle Eye
                Freeze-Dry
                Flying Press (use both)
                Strong Wings
                Tar Shot = Fire x2
             */
            let resist_mult = def_poke.get_type_mult(move_type);
            if resist_mult != 1.0 {
                dmg_modifier_list.push_back(resist_mult);
            }

            // Burn check, or Guts
            // Other (In speed order), see https://bulbapedia.bulbagarden.net/wiki/Damage

            let mut calc_dmg = base_damage;
            while let Some(modifier) = dmg_modifier_list.pop_front() {
                calc_dmg = mult_and_round(calc_dmg, modifier);
            }
            calc_dmg
        }

    /// Return if this is a valid target for this
    fn is_valid_target(&self, move_action: &MoveAction, target_pos:BattlePosition) -> bool {

        if let Some(poke) = self.get_active(target_pos) {
            // check there's no type immunity to move
            // TODO: Needs custom text (not affected)
            // check ability (Scrappy), Grounded, etc
            if poke.get_type_mult(move_action.pkm_move.r#type) == 0.0 {
                return false;
            }
            // Protect check

            // Powder check
            if move_action.pkm_move.flags.has_flag(PokemonMoveFlag::POWDER) && 
                (poke.pokemon.has_type(PokemonType::GRASS)) {
                return false;
            }

            return true;
        } else {
            // No target on field
            return false;
        }
        
    }
    
    // TODO: Complete this later, its important
    fn spawn_bcs_for_power_set<F> (self, prob_set:&[PkmnRational], 
        mut apply_func:F)
        where
            F: FnMut(&mut BattleState<'battle>),
         {

        let mut result_bcs = vec![];
        // indep events are based on targets so far
        let valid_targets:Vec<BattlePosition> = Vec::new();
        for bin_comb in 1..prob_set.len() {
            if prob_set[bin_comb as usize] == PkmnRational::ZERO() {continue;}
        
            let mut int_clone = self.clone();
            for (idx,target_pos) in valid_targets.iter().enumerate() {
                if (bin_comb >> idx) & 0b1 == 1 {
                    apply_func(&mut int_clone);
                    // int_clone.perform_stat_change(*target_pos, &stat_action);
                }
            }
            result_bcs.push(
                BattleContainer::simple(int_clone, 
                prob_set[bin_comb as usize])
            );
        }
    }

    fn perform_stat_change(&mut self, target_pos:BattlePosition, stat_action: &StatAction) {
        let target_poke = self.get_active_mut(target_pos);
        let target_act_pkmn = target_poke.expect("Non-null");
    
        let stat_ref= target_act_pkmn.get_active_stat_boost(stat_action.stat_name);
        // let pre_boost = *stat_ref;
        *stat_ref += stat_action.change;
    }

    #[deprecated(note="Modifies state directly")]
    fn perform_status_change(&mut self, target_pos:BattlePosition, status_action: &StatusAction) {
        let target_poke = self.get_active_mut(target_pos);
        let target_act_pkmn = target_poke.expect("Must be non-null");

        target_act_pkmn.status = status_action.status;
    }

    fn exec_status_change(&mut self, 
        position:BattlePosition, 
        // target_act_poke:&mut ActivePokemon,
        status_action: &StatusAction) {
        
        let target_act_poke:&mut ActivePokemon = self.get_active_mut(position).unwrap();

        let type_prevention = |status:PokemonStatus, has_type:PokemonType| -> bool {
            return status_action.status == status &&
                target_act_poke.pokemon.has_type(has_type)
        };

        use PokemonStatus::*;
        use PokemonType::*;
        // Type prevention
        if type_prevention(BURNED, FIRE) ||
            type_prevention(FROZEN, ICE) ||
            type_prevention(POISONED, POISON) ||
            type_prevention(TOXIC, POISON) ||
            type_prevention(PARALYZED, ELECTRIC) {
                return
            }

        // Check ability prevention
        // Check field & etc prevention

        if target_act_poke.status != PokemonStatus::NONE {
            target_act_poke.status = status_action.status
        }
    }

    /// TODO: Damage actions, might fold this into perform move
    /// But also needs to handle non-move actions
    #[deprecated(note="Modifies state directly")]
    fn perform_damage(&mut self, dmg_effect:DamageEffect) {

        // TODO: Check if any abilities block/mitigate the damage
        // NOTE: Might be bad to check here, lets assume damage is always accurate

        let target_pkmn = self.get_active_mut(dmg_effect.target).unwrap();
        let dmg_done = dmg_effect.calc_damage.min(target_pkmn.current_hp);
        // let final_hp = (target_pkmn.current_hp-dmg_effect.calc_damage).max(0);

        // Trigger any health effects (abilities, berries, etc)
        // Trigger if recoil/recovery if move permits (or do within move)
        target_pkmn.current_hp -= dmg_done;
        
        println!("{}", dmg_effect.damage_source);

        // If HP is 0, faint and perform fainting actions
        if target_pkmn.current_hp == 0 {
            self.action_queue.push_front(
                BattleAction::Faint(dmg_effect.target)
            );
        }
    }

    fn sim_damage(&self, dmg_effect:DamageEffect) -> Vec<BattleContainer<'battle>> {

        // TODO: Check if any abilities block/mitigate the damage
        // NOTE: Might be bad to check here, lets assume damage is always accurate

        let mut cloned_state = self.clone();

        let target_pkmn = 
            cloned_state.get_active_mut(dmg_effect.target).unwrap();

        
        let curr_hp = target_pkmn.current_hp;
        // TODO: Any pre-damage takes (items, abilities, endure)
        let dmg_done = dmg_effect.calc_damage.min(target_pkmn.current_hp);

        target_pkmn.current_hp -= dmg_done;
        
        let fainted = target_pkmn.current_hp == 0;
        let dmg_msg = format!("{target_pkmn} took {dmg_done} damage!");
        // TODO: Think about how to calc this easily into ratio
        // maybe compare ratio to threshold rational
        let health_ratio =  target_pkmn.current_hp / target_pkmn.get_active_stat(HEALTH);
        
        // If HP is 0, faint and perform fainting actions and ignore other effects
        if fainted {
            cloned_state.action_queue.push_front(
                BattleAction::Faint(dmg_effect.target)
            );
        }
        // Check pokemon/field for any faint effects

        // Trigger any health effects (abilities, berries, etc)
        // Trigger if recoil/recovery if move permits (or do within move)
        // TODO: currently create 1 BC, if ablities/items have % chance generate
        let mut bc = BattleContainer::simple(cloned_state, PkmnRational::ONE());
        bc.message = dmg_msg;

        vec![bc]
    }

    pub fn perform_turn(&mut self) {
        // TODO: sort action queue
        // Iter until action_queue is empty (any lingering moves should be status)

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
                BattleAction::Stat(stat_actions) => {
                    // Latent Stat Actions, no condition checks
                    for stat_action in stat_actions {
                        self.perform_stat(stat_action);
                    }
                }
                BattleAction::Damage(dmg_effect) => {
                    self.perform_damage(dmg_effect)
                }
                _ => {
                    println!("Not yet implemented {action}")
                }
            }


        }
        

    }

    /// Create a list of battle states created from 1 action on the action queue
    #[allow(unused_mut)] // some actions need to be modified
    pub fn sim_action(&mut self, mut action:BattleAction) -> Vec<BattleContainer<'battle>> {
        // TODO: sort action queue

        match action {
            BattleAction::Move(move_action) => {
                if !self.can_perform_move(&move_action.pkm_move, &move_action) {
                    // TODO: set move as failed
                    // Return bc with failed state
                    return vec![];
                }
                // TODO: Check abilities & etc with field to edit move if needed
                // println!("{} used {}!",
                //         self.get_active(move_action.source).unwrap(), 
                //         move_action.pkm_move.name);
                
                return self.sim_move(&move_action)
            },
            BattleAction::Stat(mut stat_actions) => {
                for stat_action in stat_actions {
                    return self.sim_stat(stat_action)
                }
            },
            BattleAction::Status(mut status_action) => {
                return self.sim_status(status_action)
            }
            BattleAction::Damage(mut dmg_action) => {
                // return self.perform_damage(dmg_action);
                return self.sim_damage(dmg_action);
                // TODO: implement this
            }
            _ => {
                    println!("Not yet implemented {action}")
            }
        }
        
        
        return vec![];

        // Ok can nested states occur? yes
        // For each state, the internal state will create a clone and modify that state to return

        
    }

    /// Convert a PreAction to a BattleAction
    #[deprecated]
    pub fn convert_to_action (pre_action:&BattlePreAction, source:BattlePosition) 
    -> BattleAction<'battle> {
        match pre_action {
            BattlePreAction::Stat(stat_change) => {
                let stat_acts = 
                stat_change.iter().map(|stat_c| {
                        StatAction {
                            stat_name: stat_c.name,
                            change: stat_c.change,
                            // NOTE: Only Moves can have ambigious targeting so this should not create new universes
                            targets: BattleState::convert_target_to_position(
                                stat_c.target_type, 
                                source),
                            pct_chance: PkmnRational::from_float(stat_c.accuracy)
                        }
                });
                BattleAction::Stat(
                    stat_acts.collect()
                )
            }
            _ => unimplemented!("Not yet implemented this action")
        }
    }

    /// Convert a MoveEffect to an Action with Damage/Status/etc
    /// Targets must be determined before calling this
    pub fn convert_effect_to_baction (move_effect:&MoveEffect, 
        effect_target_pos:BattlePosition)
    -> BattleAction<'battle> {

        match move_effect {
            MoveEffect::Status(status_chg, 
                    _target,  rat) => {
                // NOTE: StatusAction has a single target
                let status_act = 
                    StatusAction {
                        targets: vec![effect_target_pos],
                        status: *status_chg,
                        accuracy: *rat
                    };
                BattleAction::Status(status_act)
            },
            MoveEffect::Stat(stat_c) => {
                // TODO: Is multiple targets/stats worth it
                let stat_act = 
                        StatAction {
                            stat_name: stat_c.name,
                            change: stat_c.change,
                            targets: vec![effect_target_pos],
                            pct_chance: PkmnRational::from_float(stat_c.accuracy)
                        };
                    
                // NOTE: multiple stats are allowed but not used right now
                BattleAction::Stat(vec![stat_act])
            },
            _ => panic!("Not like this")
        }
    }

    /// TODO: Return tuple with number of targets
    pub fn convert_target_to_position (target:BattleTarget, 
        source: BattlePosition) -> Vec<BattlePosition> {
            use BattlePosition::*;
        match target {
            BattleTarget::SELF => vec![source],
            BattleTarget::ALL_EXCEPT_SELF => {
                let mut vec = source.get_opposing_team();
                vec.push(source.get_ally());
                vec
            },
            BattleTarget::ALLY => vec![source.get_ally()],
            BattleTarget::ALLY_ALL => source.get_ally_team(),
            BattleTarget::ALLY_ANY => source.get_ally_team(),
            BattleTarget::ANY => vec![F1, F2, B1, B2],
            BattleTarget::OPPONENT => vec![source.get_opposing()],
            BattleTarget::OPPONENT_ALL => source.get_opposing_team(),
            BattleTarget::ALL_SELF => vec![F1, F2, B1, B2],
        }
    }

    /// Convert Effect Target to Position, intended for non-move targeting
    pub fn convert_effect_target_to_position (
        target: BattleTarget,
        source: BattlePosition,
        dest: Option<BattlePosition>,
    ) -> BattlePosition {
        match target {
            BattleTarget::SELF => source,
            BattleTarget::OPPONENT | BattleTarget::ALLY => dest.unwrap_or(source),
            _ => panic!("Invalid target {:?}", target)
        }
    }
}


mod test {

use std::ops::Deref;

use crate::battle::PokemonStatus::BURNED;
// use super::*;
use crate::battle::*;
use crate::pokemon::PokemonName::{Charizard, Garchomp, Kingambit, Talonflame, Venusaur};
use crate::pokemon::moves::PokemonMoveName::Heat_Wave;
use crate::pokemon::poke_stat::PokemonStatModifier::MINUS_5;
use crate::pokemon::poke_stat::PokemonStatName::*;
use crate::pokemon::*;
use crate::pokemon::moves::{get_move, PokemonMoveName};

    #[test]
    /// Verify base damage formula using blank Garchomp Draco Meteor vs Kingambit
    fn test_move_calc() {
        let garchomp_base_stat = &get_pkmn(Garchomp).base_stats;
        let kingambit_base_stat = &get_pkmn(Kingambit).base_stats;
        let draco_move = get_move(PokemonMoveName::Draco_Meteor);

        let atk_stat = get_full_stat(garchomp_base_stat, None, 
            PokemonStatName::SPECIAL_ATTACK);
        let def_stat = get_full_stat(kingambit_base_stat, None, 
            PokemonStatName::SPECIAL_DEFENSE);
        let dmg = pkmn_damage_formula(draco_move.power, atk_stat, def_stat);

        // stab bonus (Dragon-type)
        let f_dmg = mult_and_round(dmg, 1.5);
        // type_multiplier (Steel resist)
        let f2_dmg = mult_and_round(f_dmg, 0.5);
        assert_eq!(f2_dmg, 42); // max damage roll is 42

        let min_dmg = mult_and_round(f2_dmg, 0.85);
        assert_eq!(min_dmg, 35); // min damage roll is 35
    }

    #[test]
    fn test_dmg2_calc() {
        let char_base_stat = &get_pkmn(Charizard).base_stats;
        let venu_base_stat = &get_pkmn(Venusaur).base_stats;
        let pkmn_move = get_move(PokemonMoveName::Heat_Wave);

        let atk_stat = get_full_stat(char_base_stat, None, 
            PokemonStatName::SPECIAL_ATTACK);
        let def_stat = get_full_stat(venu_base_stat, None, 
            PokemonStatName::SPECIAL_DEFENSE);
        let dmg = pkmn_damage_formula(pkmn_move.power, atk_stat, def_stat);

        // stab bonus
        let f_dmg = mult_and_round(dmg, 1.5);
        // type_multiplier 
        let f2_dmg = mult_and_round(f_dmg, 2.0);
        assert_eq!(f2_dmg, 138); // max damage roll

        // TODO: 0.85 is not the accurate way to calc?
        // Need a builder that will return 1 number in a consistent way
        let min_dmg = mult_and_round(f2_dmg, 0.85);
        assert_eq!(min_dmg, 116); // min damage roll is 35
    }

    #[test]
    fn test_move_accuracy_sim() {

        let ttar_pkmn = ActivePokemon::quick(PokemonName::Tyranitar);
        let ven_pkmn = ActivePokemon::quick(PokemonName::Venusaur);

        let mut bs = BattleState::simple(ttar_pkmn, ven_pkmn);
        let hydro_pump = get_move(PokemonMoveName::Hydro_Pump);
        BattleState::queue_move(&mut bs.action_queue, 
            BattlePosition::F1, &hydro_pump, 
            vec![BattlePosition::B1]);

        let mut root_bc = BattleContainer::simple(bs, PkmnRational::ONE());
        root_bc.sim_next_action();

        assert_eq!(root_bc.pct_chance, PkmnRational::ONE());
        // expect 2 bcs, one where hydro misses, another with hit
        assert_eq!(root_bc.battle_ctns.len(), 2);
        // 1st bc should have 1-pct_chance and no queue
        assert_eq!(root_bc.battle_ctns[0].pct_chance, 
            PkmnRational::ONE() - PkmnRational::from_float(hydro_pump.accuracy));
        assert!(root_bc.battle_ctns[0].battle_state.is_some());
        assert_eq!(root_bc.battle_ctns[0].battle_state
            .as_ref().unwrap().action_queue.len(), 0);
        // 2nd bc should have hydro pump damage
        assert!(root_bc.battle_ctns[1].battle_state.is_some());
        assert_eq!(root_bc.battle_ctns[1].battle_state
            .as_ref().unwrap().action_queue.len(), 1);

        
        // get the miss case, pull out of vector
        let miss_bc = root_bc.battle_ctns.get_mut(0).unwrap();
        let miss_bs = miss_bc.battle_state.as_mut().unwrap();
        miss_bs.f_poke2 = 
            Some(ActivePokemon::quick(PokemonName::Charizard));
        
        miss_bs.b_poke2 = 
            Some(ActivePokemon::quick(PokemonName::Rotom_Wash));

        let heat_wave = get_move(Heat_Wave);
        BattleState::queue_move(&mut miss_bs.action_queue, 
            BattlePosition::F2, &heat_wave, 
        vec![BattlePosition::B1, BattlePosition::B2]);

        // Simulate heat wave on 2 targets
        miss_bc.sim_next_action();

        // there should be 4 ctns, nothing, b1 hit, b2 hit, b1 & b2 hit
        assert_eq!(miss_bc.battle_ctns.len(), 4);
        // TODO: Complete this test on accuracy and states and etc


    }

    fn dummy_bc<'battle> () -> BattleContainer<'battle> {
        let ttar_pkmn = ActivePokemon::quick(PokemonName::Tyranitar);
        let ven_pkmn = ActivePokemon::quick(PokemonName::Venusaur);

        let mut bs = BattleState::simple(ttar_pkmn, ven_pkmn);
        // let hydro_pump = get_move(PokemonMoveName::Hydro_Pump);
        // BattleState::queue_move(&mut bs.action_queue, 
        //     BattlePosition::F1, &hydro_pump, 
        //     vec![BattlePosition::B1]);

        let mut root_bc:BattleContainer<'battle> = BattleContainer::simple(bs, PkmnRational::ONE());
        root_bc
    }

    #[test]
    fn test_status_effect_sim () {

        let mut root_bc = dummy_bc();
        // {
        let bs = root_bc.battle_state.as_mut().unwrap();

        let status_action = StatusAction {
            targets: vec![BattlePosition::F1],
            status: PokemonStatus::BURNED,
            accuracy: PkmnRational::ONE()
        };
        bs.action_queue.push_back(BattleAction::Status(status_action));
        // }

        assert_eq!(bs.action_queue.len(), 1);

        root_bc.sim_next_action();

        assert_eq!(root_bc.battle_ctns.len(), 0);
        // Should be processed, get new state
        let new_bs = root_bc.battle_state.as_ref().unwrap();
        
        assert_eq!(new_bs.action_queue.len(), 0);
        // pokemon F1 should be burnt
        assert_eq!(new_bs.f_poke1.as_ref().unwrap().status, PokemonStatus::BURNED);
        
    }

    #[test]
    fn test_stat_modifier_sim () {

        let mut root_bc = dummy_bc();
        // {
        let bs = root_bc.battle_state.as_mut().unwrap();

        let stat_action = StatAction {
            targets: vec![BattlePosition::F1],
            stat_name: ATTACK,
            change: MINUS_5,
            pct_chance: PkmnRational::ONE()
        };
        bs.action_queue.push_back(BattleAction::Stat(vec![stat_action]));
        // }
        
        // 1 action to process
        assert_eq!(bs.action_queue.len(), 1);

        root_bc.sim_next_action();

        // Should be processed, get new state
        let mut new_bs = root_bc.battle_state.as_mut().unwrap();
        
        assert_eq!(root_bc.battle_ctns.len(), 0);        
        assert_eq!(new_bs.action_queue.len(), 0);
        // pokemon F1 should be -5 special attack
        // have to get mut ref since i only have mut stat function
        assert_eq!(new_bs.f_poke1.as_mut().unwrap().get_active_stat_boost(ATTACK).clone(), 
            PokemonStatModifier::MINUS_5);
        
    }


}