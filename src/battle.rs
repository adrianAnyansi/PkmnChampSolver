// basic data for Pokemon battles
#![allow(dead_code)]

use std::{collections::VecDeque};

pub mod battle_processor;
pub mod data;

#[cfg(test)]
#[path = "battle/tests/move_tests.rs"]
mod move_test;
#[cfg(test)]
#[path = "battle/tests/battle_mechanics_tests.rs"]
mod battle_mechanics;
#[cfg(test)]
#[path = "battle/tests/ability_test.rs"]
mod ability_test;



use strum_macros::Display;

use crate::battle;
use crate::battle::DamageAfterEffect::Drain;
use crate::battle::data::{ActivePokemon, ActiveTeam, BattleFieldEffect, BattleTerrain, BattleWeatherState, PokemonBattleState, PokemonFieldState, PokemonStatus, TrainedPokemon, VolatileEnums};
use crate::pokemon::moves::PokemonMoveFlag::{IGNORE_ACC, INCRM_PROTECT_COUNTER, PROTECT, PROTECT_ACC, RECOIL_1_3RD, RECOIL_1_4TH};
use crate::pokemon::moves::{MoveEffect, PokemonBitFlag128, PokemonMoveFlag, PokemonMoveName, StatModf, StatSet, format_pkmn_message, get_charge_message, get_custom_base_power, get_move, get_move_priority, get_weather_modify_move};
use crate::pokemon::poke_stat::PokemonStatName::HEALTH;
use crate::{battle::battle_processor::BattleContainer};
use crate::math::{BinCombination8, PkmnRational, div_and_floor, gen_power_set, mult_and_round}; 
use crate::pokemon::abilities::PokemonAbilityName;
use crate::pokemon::{self, Pokemon, moves::{BattlePreAction, FieldTarget, PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::get_full_stat};
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

/// Contains the speed order of the battle
#[derive(Clone)]
pub struct SpeedBattleAction<'battle> {
    pub priority: i8,
    // pub act_poke:&'battle ActivePokemon<'battle, 'simulation>,
    pub team_index: TeamIndex,
    pub battle_action: BattleAction<'battle>
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
    /// Volatile status effect
    VolatileStatus(FieldPosition, PokemonBattleState, PkmnRational),
    /// Ability effect
    AbilityAction, 
    /// Pokemon took damage from any source
    Damage(DamageEffect),
    /// Healing
    Heal(HealEffect),
    /// Protect state
    Protect(PokemonMoveName, FieldPosition, PkmnRational), 

    /// Add this message to the battle state, no action
    Message(String),

    /// Pokemon is fainting
    Faint(FieldPosition),
    /// Pokemon is being sent back into team
    Return(FieldPosition),
    /// Pokemon entering the field
    SendOut(FieldPosition, usize),
    /// Team choice, first 2* pokemon will be sent out. BattlePosition is for the team
    ChooseTeam(FieldPosition, u8),

    /// TODO: Separate action of hit? Dunno
    HitAction(MoveAction<'battle>, String),


    /// Set flag on active pokemon, <position, state, set>
    SetFlag(FieldPosition, PokemonBattleState, bool),
    /// Force pokemon to use move
    ForceMove(FieldPosition, PokemonMoveName),

    
    // TODO: Transition stat/status/protect to this version
    /// Subset to contain % action effects, no miss case. <action, target, %>
    PctAction(BattlePctAction, FieldPosition, PkmnRational),

    /// PctActions with multiple targets
    PctActions(BattlePctAction, [Option<FieldPosition>; 4], PkmnRational)
}

impl core::fmt::Display for BattleAction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleAction::Move(act) => 
                write!(f, "BattleAction::Move({})", act.pkm_move.name),
            // BattleAction::Stat(act) => 
            //     write!(f, "BattleAction::Stat({})", act[0].stat_name),
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
    /// Add stat
    Stat(StatSet),
    /// Add Status pokemon
    Status(PokemonStatus),
    /// Add battle_status flag
    AddFlag(PokemonBattleState, bool),

    /// Add Field status
    // AddField(BattleFieldEffect, bool)
    AddField(PokemonFieldState, bool)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BattleTeamSide {
    FRONT,
    BACK
}

#[derive(Clone, Copy, PartialEq)]
pub struct TeamIndex {
    pub team_side: BattleTeamSide,
    pub index: usize
}

/// The selected position on the battlefield
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldPosition {
    F1,
    F2,
    B1,
    B2,
}

impl std::fmt::Display for FieldPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            FieldPosition::F1 => write!(f, "Front1"),
            FieldPosition::F2 => write!(f, "Front2"),
            FieldPosition::B1 => write!(f, "Back_1"),
            FieldPosition::B2 => write!(f, "Back_2"),
        }
    }
}

impl FieldPosition {
    
    pub fn get_team(self) -> BattleTeamSide {
        use FieldPosition::*;
        match self {
            B1 | B2 => BattleTeamSide::BACK,
            F1 | F2 => BattleTeamSide::FRONT,
        }
    }

    pub fn get_opposing(self) -> FieldPosition {
        use FieldPosition::*;
        match self {
            B1 => F1,
            B2 => F2,
            F1 => B1,
            F2 => B2
        }
    }

    pub fn get_ally(self) -> FieldPosition {
        use FieldPosition::*;
        match self {
            B1 => B2,
            B2 => B1,
            F1 => F2,
            F2 => F1
        }
    }

    pub fn get_opposing_team(self) -> [FieldPosition;2] {
        use FieldPosition::*;
        match self {
            B1 | B2 => [F1,F2],
            F1 | F2 => [B1,B2]
        }
    }

    pub fn get_ally_team(self) ->  [FieldPosition;2] {
        use FieldPosition::*;
        match self {
            B1 | B2 => [B1,B2],
            F1 | F2 => [F1,F2]
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum PriorityTier {
    SWITCH = 9,
    MEGA = 8,
    /// Any pre-turn priority
    PRE_TURN = 6,
    // highest built in priority is +5 in champions
    PRIORITY_5 = 5,
}

/// Move being performed by Pokemon
#[derive(Debug, Clone)]
pub struct MoveAction<'battle> {
    pub source: FieldPosition,
    pub targets: Vec<FieldPosition>,
    pub pkm_move: &'battle PokemonMove,
}


/// Status action being effected a pokemon
#[derive(Clone)]
pub struct StatusAction {
    pub targets: Vec<FieldPosition>,
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
    pub targets: Vec<FieldPosition>,
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
    pub target: FieldPosition,
    pub calc_damage: i32,
    // TODO: This needs to handle multiple things like Ability damage,
    // Whirlpool, regular moves, Status
    pub damage_source: DamageSource,
    pub dmg_after_effect: (FieldPosition, Option<(DamageAfterEffect, PkmnRational)>)
}

/// Heal effect calculated from another move or effect
#[derive(Clone, Copy)]
pub struct HealEffect {
    pub target: FieldPosition,
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
    pub target: FieldPosition,
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

#[derive(Clone, Copy)]
pub enum BattleStatePhase {
    /// Must fill the field from the team
    PreBattle,
    /// Perform switches and onEnter turns
    PreTurn,
    /// Regular moves and etc
    InTurn,
    /// End of turn 
    EndTurn
}

/// Represents the state of the battle between any action/resolve.
/// This can include intermediate states
#[derive(Clone)]
pub struct BattleState<'battle, 'simulation: 'battle> {
    /// Front facing team
    pub f_team: ActiveTeam<'battle, 'simulation>,
    /// Back facing team
    pub b_team: ActiveTeam<'battle, 'simulation>,

    /// front face pokemon, left
    f_poke_idx1: Option<usize>,
    f_poke_idx2: Option<usize>,

    /// Back face pokemon, left
    b_poke_idx1: Option<usize>,
    b_poke_idx2: Option<usize>,

    /// Current Weather
    pub weather: BattleWeatherState,
    /// active terrain (only 1) on the field
    pub terrain: BattleTerrain,
    /// Other effects not included yet
    pub effects: PokemonBitFlag128<PokemonFieldState>,
    /// Room moves (Trick, Wonder, Magic)
    pub room: i32,
    /// This will contain the many per battle effects that don't fit neatly
    /// i.e Rage Fist, Disguise, etc.
    pub internal_state: i32,
    // pub current_action: Option<String>,
    
    // NOTE: If speed/ability/etc order is hard to order, create a different queue
    pub action_queue: VecDeque<BattleAction<'battle>>,
    // Speed ordered queue
    pub speed_queue:VecDeque<SpeedBattleAction<'battle>>,


    /// Current battle turn number
    pub turn_num: i32,
    /// Action number
    pub action_num: i32,
    /// Strings to display for actions
    /// TODO: Move this out of battle state for easier cloning
    pub action_strs: Vec<String>,
    /// Bool flag when turn is complete
    pub turn_complete: bool,
    /// Keep track of state in battle
    pub turn_phase: BattleStatePhase,
}

impl<'battle, 'simulation: 'battle> BattleState<'battle, 'simulation> {

    pub fn new () -> Self {
        BattleState {
            f_team: ActiveTeam::empty(),
            b_team: ActiveTeam::empty(),

            f_poke_idx1: None,
            f_poke_idx2: None,
            b_poke_idx1: None,
            b_poke_idx2: None,

            weather: BattleWeatherState::NONE,
            terrain: BattleTerrain::NONE,
            effects: PokemonBitFlag128::<PokemonFieldState>::empty(),
            room: 0,
            internal_state: 0,
            // current_action: None,
            action_queue: VecDeque::new(),
            speed_queue: VecDeque::new(),
            turn_num: 0, // turn 0 is send out state
            action_num: 0,
            // Keep track of messages from actions/debug this frame
            action_strs: Vec::new(), // TODO: Move to battle container
            turn_complete: false,
            turn_phase: BattleStatePhase::PreTurn
        }
    }

    pub fn simple (f_poke:ActivePokemon<'battle, 'simulation>, 
        b_poke:ActivePokemon<'battle, 'simulation>) -> Self {
            let mut bs = BattleState::new();
            let f_idx = bs.f_team.add_poke(f_poke);
            let b_idx = bs.b_team.add_poke(b_poke);

            bs.exec_send_out(FieldPosition::F1, f_idx, Some(PriorityTier::PRE_TURN as i8));
            bs.exec_send_out(FieldPosition::B1, b_idx, Some(PriorityTier::PRE_TURN as i8));

            bs
    }

    // ==============================================
    // TEAM MECHANICS

    fn get_active_idx_mut(&mut self, position: FieldPosition) -> &mut Option<usize> {
        match position {
            FieldPosition::F1 => &mut self.f_poke_idx1,
            FieldPosition::F2 => &mut self.f_poke_idx2,
            FieldPosition::B1 => &mut self.b_poke_idx1,
            FieldPosition::B2 => &mut self.b_poke_idx2,
        }
    }

    /// Get TeamIndex from ActiveTeam
    fn get_active_team_idx(&self, position:FieldPosition) -> Option<TeamIndex> {
        let mut team_side = BattleTeamSide::FRONT;
        if [FieldPosition::B1, FieldPosition::B2].contains(&position) {
            team_side = BattleTeamSide::BACK;
        }

        let Some(index) = (match position {
            FieldPosition::F1 => self.f_poke_idx1,
            FieldPosition::F2 => self.f_poke_idx2,
            FieldPosition::B1 => self.b_poke_idx1,
            FieldPosition::B2 => self.b_poke_idx2,
        }) else {
            return None
        };

        Some(TeamIndex { team_side, index })
    }

    /// Send out pokemon from ActiveTeam to field. Trigger abilities/items/effects
    pub fn exec_send_out(&mut self, position: FieldPosition, 
        team_index: usize, int_priority: Option<i8>) {

        // TODO: Change from InActivePokemon to ActivePokemon

        // Add pokemon to the field
        let position_ref = self.get_active_idx_mut(position);
        if position_ref.is_some() {
            panic!("Send out a pokemon while field was occupied.")
        }
        *position_ref = Some(team_index);

        if let Some( entered_poke) = self.get_active_mut(position) {

            entered_poke.actions_taken = 0; // Reset actions taken

        } else {
            panic!("Missing(No) Pokemon was sent out!")
        }

        let true_index = TeamIndex { team_side: position.get_team(), index: team_index };
        // separate cause of mutable borrow
        let effects: Vec<MoveEffect> = match self.get_active(position) {
            Some(act_poke) => match act_poke.trained_pokemon.ability.enter_fn {
                Some(enter_fn) => enter_fn(self, act_poke),
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        let priority = int_priority.unwrap_or(0);
        // TODO: Trigger/Queue items with ON_ENTER flags
        // TODO: target -> position -> battle action sucks, do better T
        for eff in effects {
            let target_positions = match eff {
                MoveEffect::Stat(_, t, _) | MoveEffect::Status(_, t, _)
                | MoveEffect::General(_, t, _) | MoveEffect::AddFlag(_, t, _) =>
                    BattleState::convert_target_to_position(t, position),
                _ => vec![position],
            };
            for target_pos in target_positions {
                let ba = BattleState::convert_effect_to_baction(&eff, target_pos);
                // TODO: Ability priority depends on when switch in occurred
                self.queue_action_with_speed(true_index, ba, Some(priority));
            }
        }

        
    }

    pub fn exec_return_poke(&mut self, position: FieldPosition) {
        // Trigger ON_EXIT abilities/items/etc
        
        // Clear battle_status*
        let return_poke = self.get_active_mut(position).unwrap();

        return_poke.battle_status.clear_all();
        // Keep status & certain flags*
        
        // Remove pokemon from field
        let field_index_ref = self.get_active_idx_mut(position);
        *field_index_ref = None;
        
    }

    pub fn queue_speed_action(&mut self, battle_action:BattleAction<'battle>) {

        self.action_queue.push_back(battle_action);
        // sort speed 
    }

    pub fn queue_pre_turn_switch(&mut self, 
    // act_poke:&'battle ActivePokemon<'battle, 'simulation>, 
    position:FieldPosition) {
        // TODO: Get current position
        // let bat_pos = self.get_active_idx_mut(position);
        let team_index = self.get_active_team_idx(position).unwrap();
        
        let speed_action = SpeedBattleAction{
            priority: 6,
            // act_poke,
            team_index,
            battle_action: BattleAction::Return(position)
        };

        self.speed_queue.push_back(speed_action);
    }

    /// Queue move with current speed order
    pub fn queue_move_with_speed (
        &self,
        speed_queue: &mut VecDeque<SpeedBattleAction<'battle>>,
        source: FieldPosition,
        pkm_move:&'battle PokemonMove,
        targets:Vec<FieldPosition>, 
        ) {

        let move_action: MoveAction = MoveAction{
            source, targets, pkm_move
        };

        // TODO: Better way to get reference
        // let act_poke = self.get_active(source).unwrap();
        let team_index = self.get_active_team_idx(source).unwrap();
        let battle_action: BattleAction = BattleAction::Move(move_action);
        
        let speed_action: SpeedBattleAction = SpeedBattleAction {
            priority: get_move_priority(pkm_move),
            // act_poke,
            team_index,
            battle_action
        };

        speed_queue.push_back(speed_action);
    }

    pub fn queue_action_with_speed (
        &mut self,
        team_index:TeamIndex,
        battle_action:BattleAction<'battle>,
        int_priority: Option<i8>
    ) {
        // let mut speed_queue  = std::mem::take(&mut self.speed_queue);
        let priority = int_priority.unwrap_or(0);
        
        self.speed_queue.push_back( SpeedBattleAction { 
            priority, team_index, battle_action }
        );
        // self.speed_queue = speed_queue;
    }

    /// Get number of valid actions
    pub fn get_num_valid_actions() -> u8 {
        // TODO: Check for speed ties
        1
    }

    /// Pop next action for processing
    pub fn pop_next_action (
        &mut self
    ) -> Option<BattleAction<'battle>> {
        if let Some(action) = self.action_queue.pop_front() {
            Some(action)
        } else if let Some(speed_action) = self.speed_queue.pop_front() {
            Some(speed_action.battle_action)
        } else {
            None
        }
    }

    // TODO: Use param speed_tie to pop specific speed action

    /// Sort by priority then speed, both descending; ties keep queue order
    pub fn sort_speed_queue (&mut self) {
        // TODO: See if the take can be avoided, sort_by_key is by element so value has to be cached on the element
        let mut queue = std::mem::take(&mut self.speed_queue);
        queue.make_contiguous().sort_by_key(|sa| std::cmp::Reverse(
            (sa.priority, self.get_active_pokemon_speed(sa.team_index))
        ));
        self.speed_queue = queue;
    }


    // -----------------------------------------------------------
    // Utilty functions



    /// Get pokemon name
    fn get_default_poke_name (poke:Option<&ActivePokemon<'battle, 'simulation>>) -> String {
        return poke.as_ref().map(
            |p| 
            p.trained_pokemon.pokemon.name.to_string()).unwrap_or_else(|| "_".to_string()
        );
    }

    fn get_front_poke(&self) -> String {
        format!("{} {}", 
            BattleState::get_default_poke_name(self.get_active(FieldPosition::F1)),
            BattleState::get_default_poke_name(self.get_active(FieldPosition::F2))
        )
    }

    pub fn get_print_state(&self) -> String {

        let back_row_str = format!("{} {}", 
            BattleState::get_default_poke_name(self.get_active(FieldPosition::B1)),
            BattleState::get_default_poke_name(self.get_active(FieldPosition::B2))
        );

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
        source: FieldPosition,
        pmove:&'battle PokemonMove,
        targets:Vec<FieldPosition>, 
        ) {

        let move_action: MoveAction = MoveAction{
            source,
            targets,
            pkm_move: pmove
        };

        let new_action: BattleAction = BattleAction::Move(move_action);

        action_queue.push_back(new_action);
    }


    fn get_active_mut<'a>(&'a mut self, position: FieldPosition) -> Option<&'a mut ActivePokemon<'battle, 'simulation>> {
        match position {
            FieldPosition::F1 => self.f_team.get_mut(self.f_poke_idx1.unwrap_or(100) as usize),
            FieldPosition::F2 => self.f_team.get_mut(self.f_poke_idx2.unwrap_or(100) as usize),
            FieldPosition::B1 => self.b_team.get_mut(self.b_poke_idx1.unwrap_or(100) as usize),
            FieldPosition::B2 => self.b_team.get_mut(self.b_poke_idx2.unwrap_or(100) as usize),
        }
    }

    pub fn get_active<'a>(&'a self, position: FieldPosition) -> Option<&'a ActivePokemon<'battle, 'simulation>> {
        match position {
            FieldPosition::F1 => self.f_team.get(self.f_poke_idx1.unwrap_or(100) as usize),
            FieldPosition::F2 => self.f_team.get(self.f_poke_idx2.unwrap_or(100) as usize),
            FieldPosition::B1 => self.b_team.get(self.b_poke_idx1.unwrap_or(100) as usize),
            FieldPosition::B2 => self.b_team.get(self.b_poke_idx2.unwrap_or(100) as usize),
        }
    }

    
    pub fn get_team_by_side(&self, team_side:BattleTeamSide) -> &ActiveTeam<'battle, 'simulation> {
        match team_side {
            BattleTeamSide::BACK => &self.b_team,
            BattleTeamSide::FRONT => &self.f_team,
        }
    }

    // ========================================================================
    // Can perform checks

    /// Check if stat is entirely blocked, if so quit simulation
    fn can_perform_stat(&self,
        stat_modf:&StatModf, target_pos:FieldPosition) -> bool
    {

            // TODO: Check if blocked by pokemon ability/item
            // TODO: Check field conditions

            let t_act_poke = self.get_active(target_pos);
            if t_act_poke.is_none() {
                return false;
            }

            // Check if blocked by ability/item/etc
            let curr_pkmn = t_act_poke.expect("Non-null effect");

            // each stat needs to be evaluated separately or modified
            let stat_ref = curr_pkmn.get_active_stat_modf(stat_modf.name);
            if *stat_ref == PokemonStatModifier::MINUS_6 && stat_modf.chg.direction() == -1
            || *stat_ref == PokemonStatModifier::PLUS_6 && stat_modf.chg.direction() == 1 {
                return false
            }

            true

            // let valid_targets:Vec<BattlePosition> = stat_action.targets.iter().filter(
            //     |pos:&&BattlePosition| {
            //         let t_act_poke = self.get_active(**pos);

            //         // Ensure that target pokemon exists
            //         if t_act_poke.is_none() {
            //             return false
            //         }

            //         // Check if blocked by ability or item (Clear Body, Covert Cloak)

            //         let curr_pkmn = t_act_poke.expect("Non-null effect");
                    
            //         // Check if stat is maxed, then cancel
            //         let stat_ref = curr_pkmn.get_active_stat_modf(stat_action.stat_name);
            //         if *stat_ref == PokemonStatModifier::MINUS_6 && stat_action.change.direction() == -1 
            //         || *stat_ref == PokemonStatModifier::PLUS_6 && stat_action.change.direction() == 1 {
            //             return false
            //         }
            //         true
            //     }
            // ).copied().collect();
            
            
            // valid_targets

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

    /// Simulate multiple* stat changes for a pokemon in order
    fn sim_stat(&self, stat_set:&StatSet, target_pos:FieldPosition, rat:PkmnRational) -> Vec<BattleContainer<'battle, 'simulation>> {

        // Calculate and validate final stat changes
        let valid_modfs:Vec<StatModf> = // for Some(stat_modf) in stat_set.arr {
            stat_set.arr.into_iter().flatten().filter_map( | stat_modf | {
                // Modify stat change based on Simple, Contary, etc
                let active_stat_modf = stat_modf;

                let can_perform = self.can_perform_stat(&stat_modf, target_pos);
                if !can_perform {
                    return None
                }

                Some(active_stat_modf)
            }).collect();

        let apply_func = |cloned_state: &mut BattleState| {
            for stat_modf in &valid_modfs {
                cloned_state.exec_stat_change(target_pos, &stat_modf);
                // let change_dir = if stat_modf.chg.direction() == 1 {"rose"} else {"fell"};
                // TODO Do string tracking + into final container
            }
        };

        return self.spawn_bc_for_single_prob(apply_func, rat);

        // Old new implementation

        // let valid_targets = self.can_perform_stat(&stat_action);
        // let change_dir = if stat_action.change.direction() == 1 {"rose"} else {"fell"};
        
        // // Get all targets that are hit
        // let prob_set = gen_power_set(vec![rat; valid_targets.len()]);

        // let apply_func = 
        //     |cloned_state:&mut BattleState, target_idx:u8| -> String {
        //         let position = valid_targets[target_idx as usize];
        //         cloned_state.exec_stat_change(position, &stat_action);
        //         // stat
        //         let target_poke = cloned_state.get_active(position).unwrap();
        //         let stat_msg = format!("{}'s {} {change_dir} to [{:?}]!", target_poke.trained_pokemon.pokemon,
        //             stat_action.stat_name, 
        //             target_poke.get_active_stat_modf(stat_action.stat_name),
        //         );
        //         stat_msg
        //     };
        
        // return self.spawn_bcs_for_power_set(&prob_set, apply_func, None);
        
        // // Old implementation
        // let mut result_vecs:Vec<BattleContainer> = vec![];

        // for (bin_comb, rat) in prob_set.iter().enumerate() {
        //     if prob_set[bin_comb] == PkmnRational::ZERO() { continue; }

        //     let mut clone_state = self.clone();
        //     let mut stat_msg = String::new();

        //     for (idx, target_pos) in valid_targets.iter().enumerate() {
        //         if (bin_comb >> idx) & 0b1 == 0 {
        //             let target_poke = clone_state.get_active(*target_pos).unwrap();
        //             stat_msg.push_str(&format!("{} avoided the stat change!", target_poke));
        //             continue
        //         }
        //         clone_state.exec_stat_change(*target_pos, &stat_action);
        //         let target_poke = clone_state.get_active(*target_pos).unwrap();
        //         stat_msg.push_str(&format!("{}'s {} {change_dir} to [{:?}]!", target_poke.trained_pokemon.pokemon,
        //             stat_action.stat_name, 
        //             target_poke.get_active_stat_modf(stat_action.stat_name),
        //         ));
        //     }
        //     result_vecs.push(
        //         BattleContainer::simple(clone_state, *rat)
        //         .add_msg(stat_msg)
        //     );
        // }

        // result_vecs
        
    }

    /// Simulate a volatile status being applied
    fn exec_vol_status(&mut self, vol_status:PokemonBattleState, b_position:FieldPosition) {

        // TODO: check item/ability/field effects for volatile status block/change

        let target_poke = self.get_active_mut(b_position);

        if let Some(target_act_poke) = target_poke {
            // TODO: For CONFUSED/etc, set additional variables
            target_act_poke.battle_status.set_flag(vol_status);
            
            let msg = match vol_status {
                PokemonBattleState::FLINCHING => format!("{target_act_poke} flinched!"), // flinch is not shown until attacking
                PokemonBattleState::CONFUSED => format!("{target_act_poke} is confused!"),
                PokemonBattleState::INFATUATION => format!("{target_act_poke} is in love!"),
                PokemonBattleState::CENTER_OF_ATTENTION => format!("{target_act_poke} became the center of attention!"),
                _ => panic!("Invalid volatile status")
            };
            if msg != "" {
                self.action_strs.push(msg);
            }
        }
        // b_clone
    }

    /// Simulate a pokemon protecting
    fn sim_protect(&self, move_name:PokemonMoveName, target_pos:FieldPosition, acc:PkmnRational) -> Vec<BattleContainer<'battle, 'simulation>> {

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
    fn perform_generic_null_case (&mut self, target_pos:FieldPosition, move_action: &MoveAction) {
        
        let source_pkmn = self.get_active_mut(target_pos);

        if let Some(source_act_pkmn) = source_pkmn {

            // Reset protect counter if miss/failed
            if move_action.pkm_move.flags.has_flag(INCRM_PROTECT_COUNTER) {
                source_act_pkmn.consec_protect_count = 0;
            }
        }
    }

    /// Simulate performing a status to the change
    fn sim_status(&self, status_action: StatusAction) -> Vec<BattleContainer<'battle, 'simulation>> {

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
    fn sim_move(&mut self, move_action: &mut MoveAction) -> Vec<BattleContainer<'battle, 'simulation>> {

        let source_mut = self.get_active_mut(move_action.source);
        source_mut.unwrap().actions_taken += 1;

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

        // Wide guard protection
        if self.effects.has_flag(PokemonFieldState::WIDE_GUARD) && 
            move_action.pkm_move.target_type.is_multi_target_opp() {
                // Move is blocked by protect
                return vec![BattleContainer::one(base_clone, 
                    Some(format!("Move {} was blocked by Wide Guard!", move_action.pkm_move.name)))]
            }
        
        // Center of attention redirection
        self.exec_move_redirection(move_action);
        
        // TODO: Do valid target function check
        let move_targets:Vec<(&ActivePokemon, &FieldPosition)> = move_action.targets.iter().filter_map(
            |position| {
                if let Some(poke) = self.get_active(*position) {
                    return Some((poke, position))
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
            // use crate::pokemon::moves::MoveEffect;

            // generate on hit effects to the action queue

            for hit_action in &active_move.hit_actions {

                let effect_target_pos = match hit_action {
                    MoveEffect::Stat(_, battle_target, _rat) |
                    MoveEffect::Status(_, battle_target, _rat) |
                    MoveEffect::General(_, battle_target, _rat) |
                    MoveEffect::AddFlag(_, battle_target , _rat )
                    => {
                        BattleState::convert_effect_target_to_position(
                            *battle_target, move_action.source, Some(**target_pos))
                    },
                    // MoveEffect::AddFieldFlag(_ , _rat )
                    // => {
                    //     move_action.source
                    // },
                    MoveEffect::Charge(_) => {
                        panic!("Not yet implemented")
                    },
                    _ => {
                        move_action.source
                    }
                };

                let b_action = match hit_action {
                    // Protect overrides* see if best data model for this
                    MoveEffect::General(BattleEffect::Protect, _target, _rat) => {
                        BattleAction::Protect(active_move.name, 
                                move_action.source, move_acc)
                    },
                    // Wide Guard sets a field-wide flag rather than a per-pokemon one
                    // MoveEffect::General(BattleEffect::WideGuard, _target, rat) => {
                    //     BattleAction::PctAction(
                    //         BattlePctAction::AddField(PokemonFieldState::WIDE_GUARD, true),
                    //         move_action.source, *rat)
                    // },
                    _ => BattleState::convert_effect_to_baction(
                            hit_action,
                            effect_target_pos)
                };

                if true {
                    // NOTE: hit_Action order should not matter*
                    result_act_vec.push(b_action);
                }
            }

            
            
            result_queue.push(result_act_vec);

            
        }

        let prob_set = gen_power_set(
            vec![move_acc; num_valid_targets]);

        let add_to_queue = 
        |state:&mut BattleState<'battle, 'simulation>, battle_actions:&Vec<BattleAction<'battle>>| {
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

    fn exec_move_redirection(&self, move_action: &mut MoveAction) {
        // TODO: Ignore if Stalwart or move flag
        if move_action.pkm_move.target_type.is_single_target() {
            let oppose_team = move_action.source.get_opposing_team();
            let redirect_to = oppose_team.into_iter().find(|bp| {
                if let Some(act_poke) = self.get_active(*bp) {
                    return act_poke.battle_status.has_flag(PokemonBattleState::CENTER_OF_ATTENTION)
                }
                false
            });
            if let Some(redirect_target) = redirect_to {
                move_action.targets = vec![redirect_target]
            }
        }
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
        atk_poke:&ActivePokemon<'battle, 'simulation>,
        def_poke:&ActivePokemon<'battle, 'simulation>,
        battle_state:&BattleState<'battle, 'simulation>) -> i32 {

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
            
            if let Some(ability_fn) = atk_poke.trained_pokemon.ability.move_damage_modifier {
                let dmg = ability_fn(battle_state, atk_poke, def_poke, pkm_move);
                if let Some(dmg_stat) = dmg {
                    dmg_modifier_list.push_back(dmg_stat.float());
                }
            }


            let resist_mult = def_poke.get_type_mult(move_type);
            if resist_mult != 1.0 {
                dmg_modifier_list.push_back(resist_mult);
            }

            // Burn check, or Guts
            // Other (In speed order), see https://bulbapedia.bulbagarden.net/wiki/Damage

            // TODO: Add blaze/overgrow/etc

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
        || move_action.pkm_move.target_type == FieldTarget::SELF {
            move_acc = PkmnRational::ONE(); //
        } else {
            // TODO: Item/ability accuracy modifications
            // move_acc = 1.0;
        }

        move_acc
    }


    /// Return if this is a valid target for this move
    fn is_valid_target(&self, move_action: &MoveAction, target_pos:FieldPosition) -> bool {

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
    ) -> Vec<BattleContainer<'battle, 'simulation>>
        where F: FnMut(&mut BattleState<'battle, 'simulation>, u8) -> String,
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
        -> Vec<BattleContainer<'battle, 'simulation>>
        where F: FnMut(&mut BattleState<'battle, 'simulation>) 
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
    fn exec_stat_change(&mut self, target_pos:FieldPosition, stat_modf:&StatModf) {
        let target_poke = self.get_active_mut(target_pos);
        let target_act_pkmn = target_poke.expect("Non-null");
    
        let stat_ref = target_act_pkmn.get_active_stat_boost(stat_modf.name);
        // let pre_boost = *stat_ref;

        *stat_ref += stat_modf.chg;
    }

    /// Perform a status change on the battle state
    fn exec_status_change(&mut self, 
        position:FieldPosition, 
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
    fn sim_damage(&self, dmg_effect:DamageEffect) -> Vec<BattleContainer<'battle, 'simulation>> {

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
        let _health_ratio =  final_hp / total_hp;

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

    fn sim_healing(&mut self, heal_effect:HealEffect) -> Vec<BattleContainer<'battle, 'simulation>> {

        let target_pkmn = self.get_active_mut(heal_effect.target).unwrap();
        
        let total_hp = target_pkmn.get_active_stat(HEALTH);

        // TODO: BIG ROOT or ability check
        let healing_done = heal_effect.calc_healing.max(total_hp - target_pkmn.current_hp);

        target_pkmn.current_hp += healing_done;
        let _final_hp = target_pkmn.current_hp;

        // TODO: Check anything that triggers from HP gain/etc
        // TODO: Think about message for healing which is different for multiple actions
        let heal_msg = format!("{target_pkmn} healed {healing_done}!");

        vec![BattleContainer::one(self.clone(), Some(heal_msg))]
    }

    /// Create a list of battle states created from 1 action on the action queue
    #[allow(unused_mut)] // some actions need to be modified
    pub fn sim_action(&mut self, mut action:BattleAction) -> Vec<BattleContainer<'battle, 'simulation>> {
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
            // BattleAction::Stat(mut stat_actions) => {
            //     let mut all_vecs = vec![];
            //     for stat_action in stat_actions {
            //         all_vecs.extend(self.sim_stat(stat_action));
            //     }
            //     return all_vecs
            // },
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
            },

            BattleAction::SendOut(position, index) => {
                self.exec_send_out(position, index, None);
                // TODO: Better logging
                vec![BattleContainer::one(self.clone(), Some("Sent out pokemon".to_string()))]
            },
            BattleAction::Return(position) => {
                self.exec_return_poke(position);
                let team_side = position.get_team();
                vec![BattleContainer::one(self.clone(), Some(format!("Returned pokemon index {position} for side {team_side:?}")))]
            },

            BattleAction::PctAction(BattlePctAction::AddFlag(flag, _set_value), position , accuracy ) => {
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
            BattleAction::PctAction(
                BattlePctAction::AddField(field_state, _set_value),
                _position, accuracy) => {

                let apply_func = |cloned_state:&mut BattleState| {
                    cloned_state.effects.set_flag(field_state);
                    cloned_state.action_strs.push("Wide Guard protects the team!".to_string());
                };

                return self.spawn_bc_for_single_prob(apply_func, accuracy);
            },
            BattleAction::PctActions(
                BattlePctAction::Stat(stat_set), targets, rat) => {

                    // TODO: Multi target stat change is not implemented
                    return self.sim_stat(&stat_set, targets[0].unwrap(), rat);

                    // return self.spawn_bcs_for_power_set(&prob_set, apply_func, None);
                },
            // BattleAction::VolatileStatus(position, vol_status, accuracy) => {
            //     return self.sim_vol_status(vol_status, position, accuracy)
            // }
            _ => {
                    // panic!("Not implemented action sent")
                    println!("[WARN] Not yet implemented {action}");
                    vec![BattleContainer::one(self.clone(), 
                        Some("Not impl".to_string()))]
            }
        }

        // Ok can nested states occur? yes
        // For each state, the internal state will create a clone and modify that state to return
        
    }

    // ===================================================================
    // battle state things

    /// Set team and send out 1-2 pokemon to the field
    pub fn exec_set_team(&mut self, team_side:BattleTeamSide, 
        team:&Vec<&'battle TrainedPokemon<'battle, 'simulation>>) {

        // set team
        let battle_team:&mut ActiveTeam<'battle, 'simulation>;
        let field_pos: [FieldPosition;2];
        if team_side == BattleTeamSide::FRONT {
            battle_team = &mut self.f_team;
            field_pos = [FieldPosition::F1, FieldPosition::F2];
        } else {
            battle_team = &mut self.b_team;
            field_pos = [FieldPosition::B1, FieldPosition::B2];
        }
        
        // NOTE: borrow checked stuff, rethink eventually 
        for trained_poke in team {
            let act_poke = ActivePokemon::new(trained_poke);
            battle_team.add_poke(act_poke);
        }
        
        let team_len = team.len();
        // generate send_out actions for this team
        const MAX_FIELD:usize = 2; // TODO: Move to constant based on battle type
        let max_fields = MAX_FIELD.min(team_len);
        
        for team_index in 0..max_fields {
            self.exec_send_out(field_pos[team_index], 
                team_index, Some(PriorityTier::PRE_TURN as i8));
        }

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

        // TODO: Go through all abilties & etc to resolve end of turn stuff ``

        use FieldPosition::*;
        for pos in [F1, F2, B1, B2] {
            let Some(act_poke) = self.get_active_mut(pos) else { continue };
            
            // clear volatile flags
            act_poke.battle_status.clear_flag(PokemonBattleState::FLINCHING)
            .clear_flag(PokemonBattleState::PROTECT);
        }

        self.effects.clear_mask(VolatileEnums::FIELD_STATES);

        // Mark turn is complete
        self.turn_complete = true;
        self
    }

    /// Check if team
    pub fn has_team_lost(&self, team_side:BattleTeamSide) -> bool {
        let team = self.get_team_by_side(team_side);
        let num_alive = team.get_alive_indexes();
        return num_alive.len() > 0
    }


    // ====================================================
    // Convert & internal utility

    /// Convert a MoveEffect to an Action with Damage/Status/etc
    /// Targets must be determined before calling this
    pub fn convert_effect_to_baction (move_effect:&MoveEffect, 
        effect_target_pos:FieldPosition)
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
            MoveEffect::Stat(stat_set, _target, rat) => {
                    
                // NOTE: multiple stats are allowed but not used right now
                // BattleAction::Stat(vec![stat_act])
                
                BattleAction::PctActions(
                    BattlePctAction::Stat(*stat_set), 
                    [Some(effect_target_pos), None, None, None],
                     *rat)
                    
            },
            MoveEffect::General(BattleEffect::Flinch, _target, rat) => {
                BattleAction::PctAction(
                    BattlePctAction::AddFlag(PokemonBattleState::FLINCHING, true), 
                        effect_target_pos, *rat)
            }
            MoveEffect::General(battle_effect, _target, _rat ) => {
                match battle_effect {
                    // TODO: Think about this
                    // BattleEffect::Flinch => BattleAction::SetFlag(target, (), ()),
                    _ => panic!("BattleEffect {:?} is not implemented, ignoring", battle_effect)
                }
            },
            MoveEffect::AddFlag(battle_flag, _target, rat ) => {
                BattleAction::PctAction(BattlePctAction::AddFlag(*battle_flag, true), 
                        effect_target_pos, *rat)
            },
            MoveEffect::AddFieldFlag(field_flag, rat ) => {
                BattleAction::PctAction(BattlePctAction::AddField(*field_flag, true), 
                        effect_target_pos, *rat)
            },
            _ => panic!("Not like this")
        }
    }

    /// TODO: Return tuple with number of targets
    pub fn convert_target_to_position (target:FieldTarget, 
        source: FieldPosition) -> Vec<FieldPosition> {
            use FieldPosition::*;
        match target {
            FieldTarget::SELF => vec![source],
            FieldTarget::ALL_EXCEPT_SELF => {
                let mut vec = source.get_opposing_team().to_vec();
                vec.push(source.get_ally());
                vec
            },
            FieldTarget::ALLY => vec![source.get_ally()],
            FieldTarget::ALLY_ALL => source.get_ally_team().to_vec(),
            FieldTarget::ALLY_ANY => source.get_ally_team().to_vec(),
            FieldTarget::ANY_EXCEPT_SELF => {
                let mut vec = source.get_opposing_team().to_vec();
                vec.push(source.get_ally());
                vec
            }
            FieldTarget::OPPONENT => vec![source.get_opposing()],
            FieldTarget::OPPONENT_ALL => source.get_opposing_team().to_vec(),
            FieldTarget::ALL_AND_SELF => vec![F1, F2, B1, B2],
        }
    }

    /// Convert Effect Target to Position, intended for non-move targeting
    pub fn convert_effect_target_to_position (
        target: FieldTarget,
        source: FieldPosition,
        dest: Option<FieldPosition>,
    ) -> FieldPosition {
        match target {
            FieldTarget::SELF => source,
            FieldTarget::OPPONENT | FieldTarget::ALLY => dest.unwrap_or(source),
            _ => panic!("Invalid target {:?}", target)
        }
    }

    fn get_active_pokemon_speed(&self, team_index:TeamIndex) -> u32 {
        let team;
        if team_index.team_side == BattleTeamSide::FRONT {
            team = &self.f_team;
        } else {
            team = &self.b_team;
        }
        let act_poke = team.get(team_index.index).unwrap();
        // let act_poke = self.get_active(position);
        let speed = act_poke.get_active_stat(PokemonStatName::SPEED);
        // TODO: Check paralysis, Trick Room, etc
        speed as u32
    }
}
