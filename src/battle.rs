// basic data for Pokemon battles


use std::{collections::VecDeque};

use crate::{pokemon::{Pokemon, moves::{PokemonMove, PokemonMoveCategory::{Physical, Special}}, poke_stat::PokemonStatName, types::{PokemonType, get_type_multipler}}};

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

impl PokemonStatModifier {
    pub fn value () -> i32 {
        return 1
    }
}

fn pkmn_damage_formula(power:i32,
    atk_stat:i32, def_stat:i32) -> i32 {
    const LEVEL:i32 = 50;

    #[allow(unused_parens)]
    let final_damage = (
        (((2 * LEVEL)/5 + 2) 
            * power * atk_stat / def_stat) 
        / 50 + 2
        // * targets - multiple targets is 0.75
        // * weather boost
        // * GlaiveRush
        // * critical (if true, 1.5)
        // * random factor (85-100 / 100 int)
        // STAB
        // Type effectiveness
        // burn
        // other
    );
    final_damage
}

/// Represents an active pokemon slot including current hp, status and boosts
// #[allow(Display)]
pub struct ActivePokemon {
    pub pokemon: Pokemon,
    pub status: PokemonStatus,
    pub stat_modifier: [i8; 5], // temp exclude evasion & acc
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

    pub fn get_stat(&self, stat_type:PokemonStatName) -> i32 {
        let pkmn = &self.pokemon;

        let stat_array = [
            pkmn.base_stats.health,
            pkmn.base_stats.attack,
            pkmn.base_stats.defense,
            pkmn.base_stats.sp_attack,
            pkmn.base_stats.sp_defense,
            pkmn.base_stats.speed,
        ];

        // pull trained stats
        let final_arr = pkmn.trained_stats.as_ref().map(|s| 
            [stat_array[0] + s.health,
            stat_array[1] + s.attack,
            stat_array[2] + s.defense,
            stat_array[3] + s.sp_attack,
            stat_array[4] + s.sp_defense,
            stat_array[5] + s.speed]
        ).unwrap_or(
            stat_array
        );

        
        // NOTE I don't know if this single unwrap is better above or per call?
        // Pull stat boosts
        match stat_type {
            PokemonStatName::HEALTH => final_arr[0],
            PokemonStatName::ATTACK => {
                final_arr[1] * self.stat_modifier[0] as i32
            },
            PokemonStatName::DEFENSE => {
                final_arr[2] * self.stat_modifier[1] as i32
            },
            PokemonStatName::SPECIAL_ATTACK => {
                final_arr[3] * self.stat_modifier[2] as i32
            },
            PokemonStatName::SPECIAL_DEFENSE => {
                final_arr[4] * self.stat_modifier[3] as i32
            },
            PokemonStatName::SPEED => {
                final_arr[5] * self.stat_modifier[4] as i32
            },
        }
    }

    pub fn get_pkmn_type(&self) -> Vec<PokemonType> {
        // TODO: Calc this pokemon's current type based on more factors
        let mut type_list = vec![self.pokemon.type1];
        if self.pokemon.type2.is_some() {
            type_list.push(self.pokemon.type2.unwrap());
        }
        type_list
    }

    pub fn get_type_mult(&self, move_type:PokemonType) -> f64 {
        if move_type == PokemonType::TYPELESS {return 1.0};
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

pub struct MoveAction<'battle> {
    pub source: &'battle mut ActivePokemon,
    pub targets: Vec<&'battle mut ActivePokemon>,
    // TODO: Could move lifetime to a turn but uncessary
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
    ABILITY_ACTIVATE
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
            \t\t\t{back_row_str}\n\
            {}\n\
            Field: {field_state}",
            self.get_front_poke(),
        )
    }

    /// Add a move to the action queue
    /// TODO: Should pass the indexes + move instead? Since I cant borrow mutables apparently
    pub fn queue_move (
        // &'battle mut self, 
        action_queue: &mut VecDeque<BattleAction<'battle>>,
        source_pkmn:&'battle mut ActivePokemon, 
        pmove:&'battle PokemonMove,
        targets:Vec<&'battle mut ActivePokemon>, 
        ) {

        let move_action: MoveAction = MoveAction{
            source: source_pkmn,
            targets: targets,
            pkm_move: pmove
        };

        let new_action: BattleAction = BattleAction::Move(move_action);

        action_queue.push_back(new_action);
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
        
        let source_active = &move_action.source;
        let power = move_action.pkm_move.power;
        let move_type = move_action.pkm_move.r#type;
        
        let atk_stat = match move_action.pkm_move.category {
            Physical => source_active.get_stat(PokemonStatName::ATTACK),
            Special => source_active.get_stat(PokemonStatName::SPECIAL_ATTACK),
            _ => 1
        };
        let num_targets = move_action.targets.len();

        // Calculate everything per target from left->right
        for target_pkmn in &mut move_action.targets {
            
            let poke = &target_pkmn.pokemon;
            println!("Start calc for poke {poke}");

            // TODO: Calculate crit, including status and etc effects
            // let is_crit = false;

            let def_stat = match &move_action.pkm_move.category {
                Physical => target_pkmn.get_stat(PokemonStatName::DEFENSE),
                Special => target_pkmn.get_stat(PokemonStatName::SPECIAL_DEFENSE),
                _ => 1
            };

            let def_dmg = pkmn_damage_formula(power, atk_stat, def_stat);
            // TODO: I'll do this later
            // let rng_roll = [0.85, 100.0]; 

            let mut dmg_modifier_list:Vec<f32> = vec![];

            // Damage modifiers
            let is_stab: bool = 
            if [Physical, Special].contains(&move_action.pkm_move.category) {
                // TODO: With Soak/Terastilization, use a type check based on active state
                move_action.source.pokemon.has_type(move_action.pkm_move.r#type);
                true
            } else {
                false
            };

            // TODO: I calc with separate since some moves care about type effectiveness
            let stab_mult = if is_stab { 1.5
                // TODO: Adaptability & tera checks
            } else { 1.0 };

            dmg_modifier_list.push(stab_mult);
            
            // target multiplier
            let target_mult = if num_targets > 1 {0.75} else {1.0};
            dmg_modifier_list.push(target_mult);
            
            // Type multiplier
            let type_mult = target_pkmn.get_type_mult(move_type) as f32;
            // TODO: If 0, count as not applicable
            dmg_modifier_list.push(type_mult);
            
            // status modifiers, burn

            // other modifiers
            
            // Finally
            let calc_dmg = def_dmg;
            
            // Subtract health, then roll and apply secondary effects
            // target_pkmn.stat_modifier[0] = 2;
            let final_hp = (target_pkmn.current_hp-calc_dmg).min(0);
            
            // Resolve effects/abilties/etc based on HP
            // target_pkmn
            target_pkmn.current_hp = final_hp;
            // Resolve fainting


        
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
                    println!("Target {} used {}!",
                        move_action.source, move_action.pkm_move.name);
                    self.perform_move(&mut move_action);
                }
                _ => {
                    println!("Not yet implemented")
                }
            }


        }
        

    }
}
