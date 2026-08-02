// basic data for Pokemon battles


use std::{collections::VecDeque, fmt::write};

pub mod BattleProcessor;

use crate::{math::{div_and_floor, mult_and_round}, pokemon::{Pokemon, PokemonAbility, moves::{BattlePreAction, BattleTarget, PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::get_full_stat}};
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

pub enum BattleAction<'battle> {
    Move(MoveAction<'battle>), // Pokemon is performing a move
    Status(StatusAction),
    Stat(Vec<StatAction>),
    AbilityAction, // Ability 
    /// Pokemon took damage from any source
    Damage(DamageEffect),
    Faint(BattlePosition)
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

/// Move being performed by Pokemon against another Pokemon
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
    pub change: PokemonStatModifier
}

pub struct StatusAction {
    pub targets: Vec<BattlePosition>,
    pub status: PokemonStatus
}

/// Actions needed to be taken by calculated move
pub struct MoveResult {
    /// What type of result occurred here
    pub category: MoveResultEnum,
    /// String displayed when the effect occurs
    pub display_str: String
}

/// Damage Effect calculated by a move or 
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

pub enum BattleEffect {
    Flinch,
    Trapped
}

pub enum DamageSource {
    Move(PokemonMove),
    Ability(PokemonAbility),
    Status(PokemonStatus)
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

    /// TODO: Damage actions, might fold this into perform move
    /// But also needs to handle non-move actions
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
                                source)
                        }
                });
                BattleAction::Stat(
                    stat_acts.collect()
                )
            }
            _ => unimplemented!("Not yet implemented this stat action")
        }
    }

    /// TODO: Return tuple with number targets
    pub fn convert_target_to_position (target:BattleTarget, 
        source: BattlePosition) -> Vec<BattlePosition> {
            use BattlePosition::*;
        match target {
            BattleTarget::SELF => vec![source],
            BattleTarget::ALL => {
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
}


mod test {

// use super::*;
use crate::battle::*;
use crate::pokemon::poke_stat::PokemonStatName::*;

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