// basic data for Pokemon battles
#![allow(dead_code)]

use std::{collections::VecDeque};

pub mod battle_processor;
pub mod data;

#[cfg(test)]
#[path = "battle/tests/move_test.rs"]
mod move_test;
#[cfg(test)]
#[path = "battle/tests/battle_mechanics.rs"]
mod battle_mechanics;



use crate::battle;
use crate::battle::DamageAfterEffect::Drain;
use crate::battle::data::{ActivePokemon, BattleTerrain, BattleWeatherState, PokemonBattleState, PokemonStatus};
use crate::pokemon::moves::PokemonMoveFlag::{IGNORE_ACC, INCRM_PROTECT_COUNTER, PROTECT, PROTECT_ACC, RECOIL_1_3RD, RECOIL_1_4TH};
use crate::pokemon::moves::{MoveEffect, PokemonBitFlag128, PokemonMoveFlag, PokemonMoveName, format_pkmn_message, get_charge_message, get_custom_base_power, get_move, get_weather_modify_move};
use crate::pokemon::poke_stat::PokemonStatName::HEALTH;
use crate::{battle::battle_processor::BattleContainer};
use crate::math::{BinCombination8, PkmnRational, div_and_floor, gen_power_set, mult_and_round}; 
use crate::pokemon::{self, Pokemon, PokemonAbilityName, PokemonName, moves::{BattlePreAction, BattleTarget, PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::get_full_stat};
use crate::pokemon::{poke_stat::{PokemonStatName, PokemonStats}, types::{PokemonType, get_type_multipler}};
use crate::pokemon::poke_stat::{PokemonStatModifier, PokemonNature};



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




/// Generic Event representing a current action in the turn state.
/// Will include moves, ability/event resolves, etc.
/// Will think about how to structure this and what types make sense here
#[derive(Clone)]
pub enum BattleAction<'battle> {
    /// Pokemon Move being performed
    Move(MoveAction<'battle>), 
    /// Status being enacted by move or effect
    Status(StatusAction),
    /// Stat modifier change being enacted by move or effect
    Stat(Vec<StatAction>),
    /// Volatile status effect
    VolatileStatus(BattlePosition, PokemonBattleState, PkmnRational),
    /// Ability effect
    AbilityAction, 
    /// Pokemon took damage from any source
    Damage(DamageEffect),
    /// Healing
    Heal(HealEffect),

    /// Pokemon is fainting
    Faint(BattlePosition),
    /// Protect state
    Protect(PokemonMoveName, BattlePosition, PkmnRational), 
    /// Add this message to the battle state, no action
    Message(String),

    HitAction(MoveAction<'battle>, String),
    /// Set flag on active pokemon, <position, state, set>
    SetFlag(BattlePosition, PokemonBattleState, bool),
    /// Force pokemon to use move
    ForceMove(BattlePosition, PokemonMoveName),
    
    // TODO: Transition stat/status/protect to this version
    /// Subset to contain % action effects, no miss case. <action, target, %>
    PctAction(BattlePctAction, BattlePosition, PkmnRational),

    /// PctActions with multiple targets
    PctActions(BattlePctAction, [Option<BattlePosition>; 4], PkmnRational)
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

/// Subset of BattleAction with actions that typically have % of occurring
#[derive(Clone)]
pub enum BattlePctAction {
    Stat(StatAction),
    Status(PokemonStatus),
    AddFlag(PokemonBattleState, bool)
}


/// The selected position on the battlefield
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattlePosition {
    F1,
    F2,
    B1,
    B2,
}

impl std::fmt::Display for BattlePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            BattlePosition::F1 => write!(f, "Front1"),
            BattlePosition::F2 => write!(f, "Front2"),
            BattlePosition::B1 => write!(f, "Back_1"),
            BattlePosition::B2 => write!(f, "Back_2"),
        }
    }
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

/// Status action being effected a pokemon
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
    pub damage_source: DamageSource,
    pub dmg_after_effect: (BattlePosition, Option<(DamageAfterEffect, PkmnRational)>)
}

/// Heal effect calculated from another move or effect
#[derive(Clone, Copy)]
pub struct HealEffect {
    pub target: BattlePosition,
    pub calc_healing: i32,
    /// TODO: Re-using damage until this more details are needed
    pub heal_source: DamageSource,
}

/// Used to calculate recoil or healing after
#[derive(Clone, Copy)]
pub enum DamageAfterEffect {
    /// Recoil damage from moves
    Recoil,
    /// Healing from moves
    Drain
}


pub struct AddEffect {
    pub target: BattlePosition,
    pub effect_type: BattleEffect,
    pub damage_source: String
}

/// Enum representing effects from moves & etc
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)] 
pub enum BattleEffect {
    Flinch,
    Trapped,
    Confused,
    Infatuation,
    Drowsy,
    Magnet_Rise,
    Encore,
    Leech_Seed,
    Bound,
    Protect,
    Charging
}

#[derive(Clone, Copy)]
pub enum DamageSource {
    Move(PokemonMoveName),
    Ability(PokemonAbilityName),
    Status(PokemonStatus),
    Recoil(PokemonMoveName),
    Heal(PokemonMoveName)
}

impl std::fmt::Display for DamageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DamageSource::Move(move_name)|
            DamageSource::Recoil(move_name)|
            DamageSource::Heal(move_name) => write!(f, "DamageSource({move_name})"),
            DamageSource::Ability(ability_name) => write!(f, "DamageSource({ability_name})"),
            DamageSource::Status(status_name) => write!(f, "DamageSource({status_name})")
            
        }
    }
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
    pub f_poke1: Option<ActivePokemon<'battle>>,
    pub f_poke2: Option<ActivePokemon<'battle>>,
    pub b_poke1: Option<ActivePokemon<'battle>>,
    pub b_poke2: Option<ActivePokemon<'battle>>,
    /// Current Weather
    pub weather: BattleWeatherState,
    /// active terrain (only 1) on the field
    pub terrain: BattleTerrain,
    /// Other effects not included yet
    pub effects: i32,
    /// Room moves (Trick, Wonder, Magic)
    pub room: i32,
    /// This will contain the many per battle effects that don't fit neatly
    /// i.e Rage Fist, Disguise, etc.
    pub internal_state: i32,
    // pub current_action: Option<String>,
    // NOTE: If speed/ability/etc order is hard to order, create a different queue
    pub action_queue: VecDeque<BattleAction<'battle>>,
    /// Current battle turn number
    pub turn_num: i32,
    /// Action number
    pub action_num: i32,
    /// Strings to display for actions
    /// TODO: Move this out of battle state for easier cloning
    pub action_strs: Vec<String>,
    /// Bool flag when turn is complete
    pub turn_complete: bool
}

impl<'battle> BattleState<'battle> {

    pub fn new () -> Self {
        BattleState {
            f_poke1: None,
            f_poke2: None,
            b_poke1: None,
            b_poke2: None,
            // Will implement this properly later in the future idc rn
            weather: BattleWeatherState::NONE,
            terrain: BattleTerrain::NONE,
            effects: 0,
            room: 0,
            internal_state: 0,
            // current_action: None,
            action_queue: VecDeque::new(),
            turn_num: 1,
            action_num: 0,
            // Keep track of messages from actions/debug this frame
            action_strs: Vec::new(),
            turn_complete: false
        }
    }

    pub fn simple (f_poke:ActivePokemon<'battle>, 
        b_poke:ActivePokemon<'battle>) -> Self {
            let mut bs = BattleState::new();
            bs.f_poke1 = Some(f_poke);
            bs.b_poke1 = Some(b_poke);

            bs
    }

    fn get_default_poke_name (poke:&Option<ActivePokemon<'battle>>) -> String {
        return poke.as_ref().map(|p| p.trained_pokemon.pokemon.name.to_string()).unwrap_or_else(|| "_".to_string());
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


    fn get_active_mut<'a>(&'a mut self, position: BattlePosition) -> Option<&'a mut ActivePokemon<'battle>> {
        match position {
            BattlePosition::F1 => self.f_poke1.as_mut(),
            BattlePosition::F2 => self.f_poke2.as_mut(),
            BattlePosition::B1 => self.b_poke1.as_mut(),
            BattlePosition::B2 => self.b_poke2.as_mut(),
        }
    }

    pub fn get_active<'a>(&'a self, position: BattlePosition) -> Option<&'a ActivePokemon<'battle>> {
        match position {
            BattlePosition::F1 => self.f_poke1.as_ref(),
            BattlePosition::F2 => self.f_poke2.as_ref(),
            BattlePosition::B1 => self.b_poke1.as_ref(),
            BattlePosition::B2 => self.b_poke2.as_ref(),
        }
    }

    /// Fixed-order [F1, F2, B1, B2] view of the 4 active slots, always in sync with the fields.
    fn get_all_active<'a>(&'a self) -> [Option<&'a ActivePokemon<'battle>>; 4] {
        [
            self.f_poke1.as_ref(),
            self.f_poke2.as_ref(),
            self.b_poke1.as_ref(),
            self.b_poke2.as_ref(),
        ]
    }

    /// Mutable counterpart of [`BattleState::get_all_active`], same [F1, F2, B1, B2] order.
    fn get_all_active_mut<'a>(&'a mut self) -> [Option<&'a mut ActivePokemon<'battle>>; 4] {
        [
            self.f_poke1.as_mut(),
            self.f_poke2.as_mut(),
            self.b_poke1.as_mut(),
            self.b_poke2.as_mut(),
        ]
    }


    // Can perform checks

    fn can_perform_stat(&self, stat_action:&StatAction) -> Vec<BattlePosition> {

            // TODO: Check if blocked by pokemon ability/item
            // TODO: Check field conditions

            let valid_targets:Vec<BattlePosition> = stat_action.targets.iter().filter(
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
            ).copied().collect();
            
            
            valid_targets

        }

    fn can_perform_move(&self, pkm_move:&PokemonMove, move_action: &MoveAction  ) -> (bool, String) {
        // check move conditions
        if !pkm_move.intn_condition_check(
            self, move_action) {
            return (false, "But it failed!".to_string());
        }
        
        // Field checks
        let source_poke = self.get_active(move_action.source);
        if let Some(source_act_poke) = source_poke {

            if source_act_poke.battle_status.has_flag(PokemonBattleState::FLINCHING) {
                return (false, format!("{source_act_poke} flinched!")) // skip flinching, add message
            }
        }
        
        // Ability check
        // Item check

        (true, "".to_string())
    }

    /// Get Recoil/Heal damage from effect
    fn get_damage_after_effect(pkm_move:&PokemonMove) 
        -> Option<(DamageAfterEffect, PkmnRational)> {
        
        use PokemonMoveFlag::*;
        // recoil flags
        if pkm_move.flags.has_flag(RECOIL_1_3RD) {
            return Some((DamageAfterEffect::Recoil, PkmnRational::new(1, 3)))
        } else if pkm_move.flags.has_flag(RECOIL_1_4TH) {
            return Some((DamageAfterEffect::Recoil, PkmnRational::new(1, 4)))
        }

        if pkm_move.flags.has_flag(HEAL_1_2HF) {
            return Some(
                (DamageAfterEffect::Drain, PkmnRational::new(1, 2))
            )
        }
        None
    }

    /// Simulate a stat change
    fn sim_stat(&self, stat_action: StatAction) -> Vec<BattleContainer<'battle>> {

        let change_dir = if stat_action.change.direction() == 1 {"rose"} else {"fell"};
        
        // Get all targets that are hit
        let valid_targets = self.can_perform_stat(&stat_action);
        let prob_set = gen_power_set(vec![
            stat_action.pct_chance; valid_targets.len()]);

        let apply_func = 
            |cloned_state:&mut BattleState, target_idx:u8| -> String {
                let position = valid_targets[target_idx as usize];
                cloned_state.exec_stat_change(position, &stat_action);
                // stat
                let target_poke = cloned_state.get_active(position).unwrap();
                let stat_msg = format!("{}'s {} {change_dir} to [{:?}]!", target_poke.trained_pokemon.pokemon,
                    stat_action.stat_name, 
                    target_poke.get_active_stat_modf(stat_action.stat_name),
                );
                stat_msg
            };
        
        return self.spawn_bcs_for_power_set(&prob_set, apply_func, None);
        // Old implementation

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
                clone_state.exec_stat_change(*target_pos, &stat_action);
                let target_poke = clone_state.get_active(*target_pos).unwrap();
                stat_msg.push_str(&format!("{}'s {} {change_dir} to [{:?}]!", target_poke.trained_pokemon.pokemon,
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
        
    }

    /// Simulate a volatile status being applied
    fn exec_vol_status(&mut self, vol_status:PokemonBattleState, b_position:BattlePosition) {

        // TODO: check item/ability/field effects for volatile status block/change

        let target_poke = self.get_active_mut(b_position);

        if let Some(target_act_poke) = target_poke {
            // TODO: For CONFUSED/etc, set additional variables
            target_act_poke.battle_status.set_flag(vol_status);
            
            let msg = match vol_status {
                PokemonBattleState::FLINCHING => format!("{target_act_poke} flinched!"), // flinch is not shown until attacking
                PokemonBattleState::CONFUSED => format!("{target_act_poke} is confused!"),
                PokemonBattleState::INFATUATION => format!("{target_act_poke} is in love!"),
                _ => panic!("Invalid volatile status")
            };
            if msg != "" {
                self.action_strs.push(msg);
            }
        }
        // b_clone
    }

    /// Simulate a pokemon protecting
    fn sim_protect(&self, move_name:PokemonMoveName, target_pos:BattlePosition, acc:PkmnRational) -> Vec<BattleContainer<'battle>> {

        let mut result_vec:Vec<BattleContainer> = Vec::new();

        // Create a failed state if acc is less than 1
        if acc.float() < PkmnRational::ONE().float() {
            let mut null_state = self.clone();
            if let Some(prot_pkmn) = null_state.get_active_mut(target_pos) {
                prot_pkmn.consec_protect_count = 0
            }
            
            let msg = format!("The move failed!");
            null_state.action_strs.push(msg);
            result_vec.push(BattleContainer::simple(null_state, 
                PkmnRational::ONE() - acc));
        }

        // Protect succeeded case
        let mut cloned_state = self.clone();
        let source_poke = cloned_state.get_active_mut(target_pos);
        
        if let Some(source_act_poke) = source_poke {

            source_act_poke.consec_protect_count += 1; // increment success counter

            if [PokemonMoveName::Protect].contains(&move_name) {
                source_act_poke.battle_status.set_flag(
                    PokemonBattleState::PROTECT
                );
            }
            // TODO: Add non-generic protect
            
            let msg = format!("{} protected itself!", source_act_poke);
            cloned_state.action_strs.push(msg);
        }

        result_vec.push(BattleContainer::simple(cloned_state, 
            acc));

        result_vec
    }

    /// Simulate a charge start-up and return a new Battle state
    /// Can return true if the move is skipping charge
    fn exec_charge_for_move(&mut self, move_action:&MoveAction) -> bool {

        // let source_status = &source_act_pkmn.battle_status;
        let source_poke = self.get_active(move_action.source).unwrap();
        let bypass_charge ;

        if !source_poke.battle_status.has_flag(PokemonBattleState::CHARGING) {
            let b_actions;
            (bypass_charge, b_actions) = self.gen_pre_charge_action(move_action);
            let mut_poke = self.get_active_mut(move_action.source).unwrap();
            if !bypass_charge { // add charging flag
                mut_poke.battle_status.set_flag(PokemonBattleState::CHARGING);
            }
            
            // Always sow 
            let charge_msg = format_pkmn_message(
                get_charge_message(move_action.pkm_move.name),
                mut_poke, None);
            self.action_strs.push(charge_msg);
            // add actions & quit move early
            for b_action in b_actions {
                self.action_queue.push_front(b_action);
            }
        } else {
            let mut_poke = self.get_active_mut(move_action.source).unwrap();
            mut_poke.battle_status.clear_flag(
                PokemonBattleState::CHARGING
            ); // Clear charging state, no other event triggered
            bypass_charge = true
            // continue with the move eval
        }

        return bypass_charge;

    }

    fn gen_pre_charge_action(&self, move_action:&MoveAction) -> (bool, Vec<BattleAction<'battle>>) {

        // let source_act_pkmn = 
        //     self.get_active(move_action.source).expect("Source pkmn must exist");
        
        // Check if charge is bypassed by static effect
        let bypass_charge = match move_action.pkm_move.name {
            PokemonMoveName::Solar_Beam => self.weather == BattleWeatherState::SUN,
            _ => false
        };

        if bypass_charge {
            // evaluate the full move
            return (true, vec![])
            // TODO: Return pre-charge actions if needed
        }

        // TODO: if power herb, return a consume item action before the move

        (false, vec![])
    }

    // Perform generic NULL(miss) case for all moves
    fn perform_generic_null_case (&mut self, target_pos:BattlePosition, move_action: &MoveAction) {
        
        let source_pkmn = self.get_active_mut(target_pos);

        if let Some(source_act_pkmn) = source_pkmn {

            // Reset protect counter if miss/failed
            if move_action.pkm_move.flags.has_flag(INCRM_PROTECT_COUNTER) {
                source_act_pkmn.consec_protect_count = 0;
            }
        }
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
    fn sim_move(&mut self, move_action: &mut MoveAction) -> Vec<BattleContainer<'battle>> {

        let source_mut = self.get_active_mut(move_action.source);
        source_mut.unwrap().turns_active += 1;

        let source_act_pkmn = self.get_active(move_action.source).expect("source must exist");

        // Actions that always occur before actions
        // let mut base_queue:Vec<BattleAction> = vec![];
        // Always create a base clone
        let mut base_clone = self.clone();

        // TODO: Move to separate function probably
        // If charging move, perform charge
        if move_action.pkm_move.flags.has_flag(PokemonMoveFlag::CHARGING) {
            let bypass_charge = base_clone.exec_charge_for_move(move_action);
            if !bypass_charge {
                return vec![BattleContainer::simple(base_clone, 
                    PkmnRational::ONE())];
            }
        }

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

        // TODO: Set last_move_failed as false

        // List of actions per target
        let num_valid_targets = move_targets.len();
        
        let mut result_queue:Vec<Vec<BattleAction>> = vec![];

        let move_acc = BattleState::calc_move_accuracy(move_action, source_act_pkmn);

        // Check valid targets on field and calc hit
        for (target_act_pkmn, target_pos) in &move_targets {
            
            // Contains resulting actions from move hit
            let mut result_act_vec: Vec<BattleAction> = vec![];
            let def_poke = &target_act_pkmn.trained_pokemon.pokemon;
            
            if target_act_pkmn.battle_status.has_flag(PokemonBattleState::PROTECT) {
                // TODO: BattleAction::Message() for hidden effects?
                // Bypass hit check and trigger on-hit actions
                result_act_vec.extend(
                    [BattleAction::Message(format!("{target_act_pkmn} protected itself!"))
                    ]
                );
                // TODO: For certain protects King Shield, add effect here
                // if move_action.pkm_move.is_attack() {
                //     // TODO: Considering 
                //     result_act_vec.push(BattleAction::HitAction(move_action.clone(), "Protected".to_string()));
                // }
                continue;
            }

            // TODO: Calculate crit, including status and etc effects
            let mut active_move = move_action.pkm_move.clone();

            if move_action.pkm_move.is_attack() {
                println!("INT[{}/{}] Start damage calc for target {def_poke}",
                    self.turn_num, self.action_num);
                
                // TODO: Damage needs to be calculated on hit not before hit
                let move_damage = BattleState::calc_move_damage(&mut active_move, 
                    num_valid_targets, source_act_pkmn, target_act_pkmn, self);

                let bat_pos = **target_pos;
                // TODO: Some moves may do more than just damage
                // This damage should be accurate pre-ability/etc modification
                let dmg_after_effect = BattleState::get_damage_after_effect(move_action.pkm_move);
                // if let Some((effect_type, modifier)) =  {
                //     dmg_after_effect_source = Some((move_action.source, effect_type, modifier));
                // }

                // Trigger DamageEffect
                let dmg_effect = DamageEffect {
                    target: bat_pos,
                    calc_damage: move_damage,
                    damage_source: DamageSource::Move(move_action.pkm_move.name),
                    dmg_after_effect: (move_action.source, dmg_after_effect)
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
            // TODO: Most vec will go unused, look for better logic
            // generate on hit effects to the action queue
            for hit_action in &active_move.hit_actions {

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
                    MoveEffect::General(BattleEffect::Protect, target, rat) => {
                        Some(BattleAction::Protect(active_move.name, 
                                move_action.source, move_acc))
                    },
                    MoveEffect::General(BattleEffect::Flinch, target, rat) => {
                        let f_target_pos = BattleState::convert_effect_target_to_position(
                            *target, move_action.source, Some(**target_pos));
                        Some(BattleAction::PctAction(
                            BattlePctAction::AddFlag(PokemonBattleState::FLINCHING, true), 
                                f_target_pos, *rat))
                        // Some(BattleAction::VolatileStatus(f_target_pos, 
                        //         PokemonBattleState::FLINCHING, *rat))
                    }
                    MoveEffect::Status(_, target, _rat) |
                    MoveEffect::General(_, target, _rat) => {

                        let status_target_pos = BattleState::convert_effect_target_to_position(
                            *target, move_action.source, Some(**target_pos));
                        
                        Some(BattleState::convert_effect_to_baction(hit_action, status_target_pos))
                    },
                    _ => {println!("WARNING: MoveEffect not implemented"); None}
                };

                if let Some(action) = b_action {
                    // NOTE: hit_Action order should not matter*
                    // cloned_state.action_queue.push_front(action);
                    result_act_vec.push(action);
                }
            }
            
            result_queue.push(result_act_vec);

            
        }

        let prob_set = gen_power_set(
            vec![move_acc; num_valid_targets]);

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

            if bin_comb == 0 {
                // TODO: Should not be a case where source is null
                // let source_target = cloned_state.get_active_mut(move_action.source).unwrap();
                cloned_state.perform_generic_null_case(move_action.source, move_action);
            }

            for (idx,target_pos) in move_targets.iter().enumerate() {
                
                    if (bin_comb >> idx) & 0b1 == 1 {
                        add_to_queue(&mut cloned_state, &result_queue[idx]);
                    }
                    // if missed, add message here
                    else if (bin_comb >> idx) & 0b1 == 0 && move_action.pkm_move.is_attack() {
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
    // fn base_power_calc(move_action: &MoveAction, act_poke:&ActivePokemon) -> i32 {
    //     let base_power = move_action.pkm_move.power;

    //     let atk_stat = match move_action.pkm_move.category {
    //         Physical => act_poke.get_active_stat(PokemonStatName::ATTACK),
    //         Special => act_poke.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
    //         _ => 1
    //     };

    //     // TODO: Special power calcs

    //     return base_power
    // }

    fn calc_move_damage(pkm_move: &mut PokemonMove,
        num_valid_targets:usize, 
        atk_poke:&ActivePokemon, def_poke:&ActivePokemon,
        battle_state:&BattleState) -> i32 {

            // Would like to cache this but base_power on weight or Foul Play
            // Requires more thought

            let atk_stat = match pkm_move.category {
                Physical => atk_poke.get_active_stat(PokemonStatName::ATTACK),
                Special => atk_poke.get_active_stat(PokemonStatName::SPECIAL_ATTACK),
                _ => 1
            };

            let mut effective_move = pkm_move.clone();
            // TODO: Add custom power flag 
            if pkm_move.flags.has_flag(PokemonMoveFlag::WEATHER_MODIFY) {
                effective_move = get_weather_modify_move(battle_state.weather, pkm_move.name);
                
            }
            if pkm_move.flags.has_flag(PokemonMoveFlag::CUSTOM_POWER) {
                effective_move = get_custom_base_power(battle_state, atk_poke, pkm_move.name);
            }
            // let base_power = effective_move.power;

            let def_stat = match &effective_move.category {
                Physical => def_poke.get_active_stat(PokemonStatName::DEFENSE),
                Special => def_poke.get_active_stat(PokemonStatName::SPECIAL_DEFENSE),
                _ => panic!("Non attacking move in damage calculation")
            };

            let move_type = effective_move.r#type;
            // TODO Custom type flag, edit move action
            // if move_action.pkm_move.flags.has_flag(PokemonMoveFlag::CUSTOM_TYPE)

            let base_damage = pkmn_damage_formula(effective_move.power, atk_stat, def_stat);

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
            if atk_poke.trained_pokemon.pokemon.has_type(move_type) {
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


    /// calculate pre-target pokemon accuracy
    fn calc_move_accuracy(move_action: &MoveAction, 
        source_act_pkmn:&ActivePokemon) -> PkmnRational {
        
        // TODO: Accuracy can change based on defender, validate this
        // TODO: If can't miss, override to 1
        let mut move_acc = PkmnRational::from_float(move_action.pkm_move.accuracy);
        
        // Check if move need protect accuracy
        if move_action.pkm_move.flags.has_flag(PROTECT_ACC) {
            // let last_move = get_move(source_act_pkmn.last_move_used.unwrap() );
            // if last_move.flags.has_flag(PROTECT) {
                move_acc = PkmnRational::new(1, 
                    source_act_pkmn.consec_protect_count as u32 * 3);
            // }
        }

        if move_action.pkm_move.flags.has_flag(IGNORE_ACC) 
        || move_action.pkm_move.target_type == BattleTarget::SELF {
            move_acc = PkmnRational::ONE(); //
        } else {
            // TODO: Item/ability accuracy modifications
            // move_acc = 1.0;
        }

        move_acc
    }


    /// Return if this is a valid target for this move
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
                (poke.trained_pokemon.pokemon.has_type(PokemonType::GRASS)) {
                return false;
            }

            return true;
        } else {
            // No target on field
            return false;
        }
        
    }
    
    /// Spawn N BattleContainers based on multiple independent* events with apply & fail functions
    fn spawn_bcs_for_power_set<F> (&self, prob_set:&[PkmnRational], 
        mut apply_func:F, mut fail_func:Option<F>
    ) -> Vec<BattleContainer<'battle>>
        where F: FnMut(&mut BattleState<'battle>, u8) -> String,
    {
        let mut result_bcs = vec![];
        
        // Get bin_comb (each indep event occuring) and prob_i (probability)
        for (bin_comb, prob_i) in prob_set.iter().enumerate() {
            if *prob_i == PkmnRational::ZERO() {continue;}
        
            let mut int_clone = self.clone();
            let mut ret_strings: Vec<String> = vec![];

            // For each event, apply change or transform
            for idx in 0..prob_set.len() {
                if BinCombination8::index(bin_comb as u8, idx as u8) {
                    let app_str = apply_func(&mut int_clone, idx as u8);
                    ret_strings.push(app_str);
                } else if let Some(ref mut fail_f) = fail_func {
                    let fail_str = fail_f(&mut int_clone, idx as u8);
                    ret_strings.push(fail_str);
                }
            }
            result_bcs.push(
                BattleContainer::simple(int_clone, *prob_i)
                .add_msg(ret_strings.join("\n"))
            );
        }
        result_bcs
    }

    /// Create 2 states, with 1 applying a simulation change
    fn spawn_bc_for_single_prob<F> (&self, mut apply_func:F, prob:PkmnRational) 
        -> Vec<BattleContainer<'battle>>
        where F: FnMut(&mut BattleState<'battle>) 
    {
        let mut result_bcs = vec![];
        if prob != PkmnRational::ZERO() {
            let mut bs_clone = self.clone();
            apply_func(&mut bs_clone); // apply change
            result_bcs.push(
                BattleContainer::simple(bs_clone, prob)
            );
        }

        // NOTE: Cloning the unmodified state
        if prob != PkmnRational::ONE() {
            let base_bc = BattleContainer::simple(self.clone(), 
                PkmnRational::ONE() - prob);
            result_bcs.push(base_bc);
        }

        result_bcs
    }

    /// Perform a statistic change on the pokemon
    fn exec_stat_change(&mut self, target_pos:BattlePosition, stat_action: &StatAction) {
        let target_poke = self.get_active_mut(target_pos);
        let target_act_pkmn = target_poke.expect("Non-null");
    
        let stat_ref= target_act_pkmn.get_active_stat_boost(stat_action.stat_name);
        // let pre_boost = *stat_ref;
        *stat_ref += stat_action.change;
    }

    /// Perform a status change on the battle state
    fn exec_status_change(&mut self, 
        position:BattlePosition, 
        // target_act_poke:&mut ActivePokemon,
        status_action: &StatusAction) {
        
        let target_act_poke:&mut ActivePokemon = self.get_active_mut(position).unwrap();

        let type_prevention = |status:PokemonStatus, has_type:PokemonType| -> bool {
            return status_action.status == status &&
                target_act_poke.trained_pokemon.pokemon.has_type(has_type)
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

        // TODO: Check ability prevention
        // TODO: Check field & etc prevention

        if target_act_poke.status == PokemonStatus::NONE {
            target_act_poke.status = status_action.status
        }
    }

    /// Simulate damage step, creating multiple universes if damage range
    /// causes multiple effects
    fn sim_damage(&self, dmg_effect:DamageEffect) -> Vec<BattleContainer<'battle>> {

        // TODO: Check if any abilities block/mitigate the damage, i.e disguise
        // NOTE: Might be bad to check here, lets assume damage is always accurate

        let mut cloned_state = self.clone();

        let target_pkmn = 
            cloned_state.get_active_mut(dmg_effect.target).unwrap();

        // let curr_hp = target_pkmn.current_hp;
        let total_hp = target_pkmn.get_active_stat(HEALTH);
        // TODO: Any pre-damage takes (items, abilities, endure)
        let dmg_done = dmg_effect.calc_damage.min(target_pkmn.current_hp);

        target_pkmn.current_hp -= dmg_done;
        let final_hp = target_pkmn.current_hp;
        
        let fainted = target_pkmn.current_hp == 0;
        let dmg_msg = format!("{target_pkmn} took {dmg_done} damage!");
        
        
        // If HP is 0, faint and perform fainting actions and ignore other effects
        // NOTE: Faint needs to wait for after effects, so needs to be in its own queue
        if fainted {
            cloned_state.action_queue.push_front(
                BattleAction::Faint(dmg_effect.target)
            );
        }
        
        // Trigger any health effects (abilities, berries, etc)
        // TODO: Think about how to calc this easily into ratio
        // maybe compare ratio to threshold rational
        let health_ratio =  final_hp / total_hp;

        // Trigger if recoil/recovery if move permits (or do within move)
        if let (target, Some((effect_type, modifier))) = dmg_effect.dmg_after_effect {
            let calc_damage = mult_and_round(dmg_done,  modifier.float());
            // let move_name = dmg_effect.damage_source;

            let battle_action = match effect_type {
                battle::DamageAfterEffect::Recoil => {
                    BattleAction::Damage( DamageEffect {
                    target,
                    calc_damage,
                    damage_source: DamageSource::Recoil(PokemonMoveName::Heat_Wave),
                    dmg_after_effect: (target, None)
                    })
                    
                },
                Drain => {
                    BattleAction::Heal( HealEffect {
                        target,
                        calc_healing: calc_damage,
                        heal_source: DamageSource::Heal(PokemonMoveName::Heat_Wave),
                    })
                }
            };
            cloned_state.action_queue.push_front(battle_action);
        }

        // TODO: currently create 1 BC, if ablities/items have % chance generate
        let mut bc = BattleContainer::simple(cloned_state, PkmnRational::ONE());
        bc.message = dmg_msg;

        vec![bc]
    }

    fn sim_healing(&mut self, heal_effect:HealEffect) -> Vec<BattleContainer<'battle>> {

        let target_pkmn = self.get_active_mut(heal_effect.target).unwrap();
        
        let total_hp = target_pkmn.get_active_stat(HEALTH);

        // TODO: BIG ROOT or ability check
        let healing_done = heal_effect.calc_healing.max(total_hp - target_pkmn.current_hp);

        target_pkmn.current_hp += healing_done;
        let final_hp = target_pkmn.current_hp;

        // TODO: Check anything that triggers from HP gain/etc
        // TODO: Think about message for healing which is different for multiple actions
        let heal_msg = format!("{target_pkmn} healed {healing_done}!");

        vec![BattleContainer::one(self.clone(), Some(heal_msg))]
    }

    /// Create a list of battle states created from 1 action on the action queue
    #[allow(unused_mut)] // some actions need to be modified
    pub fn sim_action(&mut self, mut action:BattleAction) -> Vec<BattleContainer<'battle>> {
        // TODO: sort action queue

        match action {
            BattleAction::Move(move_action) => {
                let (can_perform, fail_message) = self.can_perform_move(&move_action.pkm_move, &move_action);
                if !can_perform {
                    self.action_strs.push(fail_message);
                    let source_act_poke = self.get_active_mut(move_action.source).unwrap();
                    // TODO: Certain effects do not set this, flinch does
                    source_act_poke.last_move_failed = true;
                    return vec![BattleContainer::simple(self.clone(), PkmnRational::ONE())];
                }
                
                // TODO: Check abilities & etc with field to edit move if needed
                
                // NOTE: Cloning as move may need modification
                return self.sim_move(&mut move_action.clone())
            },
            BattleAction::Stat(mut stat_actions) => {
                let mut all_vecs = vec![];
                for stat_action in stat_actions {
                    all_vecs.extend(self.sim_stat(stat_action));
                }
                return all_vecs
            },
            BattleAction::Status(mut status_action) => {
                return self.sim_status(status_action)
            }
            BattleAction::Damage(mut dmg_action) => {
                return self.sim_damage(dmg_action);
                // TODO: implement this
            },
            BattleAction::Heal(mut heal_effect) => {
                self.sim_healing(heal_effect)
            },
            BattleAction::Protect(move_name, position, accuracy ) => {
                // Check the protect_counter in the function
                return self.sim_protect(move_name, position, accuracy);
            }
            BattleAction::PctAction(BattlePctAction::AddFlag(flag, set_value), position , accuracy ) => {
                // NOTE: Should be used for no miss cases
                let vol_status = flag; // TODO: Check volatile subset status
                let apply_func = |cloned_state:&mut BattleState| {
                    cloned_state.exec_vol_status(vol_status, position)
                };

                return self.spawn_bc_for_single_prob(apply_func, accuracy);
                // let result = self.spawn_bcs_for_power_set(prob_set, apply_func, apply_func);
                // return BattleContainer::simple(battle_state, pct_chance)
                // return BattleState::sim_vol_status(self, vol_status, position, accuracy);
                    
            },
            BattleAction::PctActions(
                BattlePctAction::Stat(stat_action), targets, rat) => {

                    let valid_targets: Vec<BattlePosition> = targets.into_iter().flatten().collect();
                    let prob_set = gen_power_set(vec![rat; valid_targets.len()]);
                    let apply_func = 
                    |cloned_state:&mut BattleState, target_idx:u8| {
                        let position = valid_targets[target_idx as usize];
                        cloned_state.exec_stat_change(position, &stat_action);
                        String::from("Stringy")
                    };

                    return self.spawn_bcs_for_power_set(&prob_set, apply_func, None);
                },
            // BattleAction::VolatileStatus(position, vol_status, accuracy) => {
            //     return self.sim_vol_status(vol_status, position, accuracy)
            // }
            _ => {
                    println!("Not yet implemented {action}");
                    vec![BattleContainer::one(self.clone(), 
                        Some("Not impl".to_string()))]
            }
        }
        
        
        // return vec![];

        // Ok can nested states occur? yes
        // For each state, the internal state will create a clone and modify that state to return

        
    }


    /// Resolve end of turn effects, set flag for turn complete
    pub fn mark_end_of_turn(&mut self) -> &mut Self {
        
        // TODO: Effect order
        // Weather effect
        // Terrain
        // Future-Sight/Wish/Etc
        // Binding damage
        // Perish Song
        // Items/Abilities (speed order)

        // TODO: Go through all abilties & etc to resolve end of turn stuff

        for poke in self.get_all_active_mut() {
            let Some(act_poke) = poke else { continue };
            
            // clear volatile flags
            act_poke.battle_status.clear_flag(PokemonBattleState::FLINCHING)
            .clear_flag(PokemonBattleState::PROTECT);
        }

        // Mark turn is complete
        self.turn_complete = true;
        self
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
            MoveEffect::General(battle_effect, target, rat ) => {
                match battle_effect {
                    // TODO: Think about this
                    // BattleEffect::Flinch => BattleAction::SetFlag(target, (), ()),
                    _ => panic!("BattleEffect {:?} is not implemented, ignoring", battle_effect)
                }
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
