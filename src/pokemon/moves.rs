// moves and information

use serde::Deserialize;
use strum_macros::{Display, EnumString};

use crate::{battle::{self, AddEffect, BattleAction, BattleEffect::{self, Flinch}, BattleState, MoveAction}, pokemon::{moves::{BattleTarget::OPPONENT, PokemonMoveName::{Draco_Meteor, Kowtow_Cleave}}, poke_stat::{PokemonStatModifier, PokemonStatName}}};
use crate::pokemon::types::PokemonType;

/// Move type and additional information
#[derive(PartialEq, Copy, Clone)]
pub enum PokemonMoveCategory {
    Physical,
    Special,
    Status
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum BattleTarget {
    /// Target 1 opponent
    OPPONENT,
    /// Target your ally but not yourself
    ALLY,
    /// Target any ally including self
    ALLY_ANY,
    /// Target self only
    SELF,
    /// Target both opponents
    OPPONENT_ALL,
    /// Target you and your ally
    ALLY_ALL,
    /// Target anyone on the field
    ANY,
    /// Target all users except self
    ALL,
    /// Target all users including self
    ALL_SELF
}

#[allow(dead_code)]
pub struct PokemonMove {
    pub name: PokemonMoveName,
    pub power: i32,
    pub r#type: PokemonType,
    // Explanation of the move type
    pub category: PokemonMoveCategory,
    pub accuracy: f64, // note need to convert to proper i32
    pp: i32,
    pub priority: i8,
    contact: bool,
    pub target_type: BattleTarget
}

/// Indicates a change in stat boosts
pub struct StatChange {
    pub target_type: BattleTarget,
    pub name: PokemonStatName,
    pub change: PokemonStatModifier,
    pub accuracy: f64
}

// TODO: Move to NumberConstants module for qol
const ONE_THIRD:f64 = 1.0/3.0;

/// Effects incurred by a move
// trait MoveSecondEffect {
//     /// Actions that modify battle state after hitting
//     fn afterSuccess (battle_state:&BattleState);
//     /// Can this move be performed
//     fn canPerform(battle_state:&BattleState);
//     /// Actions that occur before move is performed
//     fn beforeAction (battle_state:&BattleState) -> bool;
// }

impl PokemonMove {
    pub fn new (move_name:PokemonMoveName, 
        move_type:PokemonType, 
        catg:PokemonMoveCategory) -> Self {
        PokemonMove {
            name: move_name,
            r#type: move_type,
            category: catg,
            power: 50,
            accuracy: 1.0,
            pp: 32,
            contact: false,
            priority: 0,
            target_type: OPPONENT
        }
    }

    pub fn set_attr(mut self,
        power:i32, acc:f64, target:BattleTarget 
    ) -> Self {
        self.power = power;
        self.accuracy = acc;
        self.target_type = target;
        self
    }

    pub fn after_hit (&self, 
        battle_state:&BattleState) -> Vec<BattlePreAction> {
        use PokemonMoveName::*;
        match self.name {
            Draco_Meteor => {
                // get move_performer
                // TODO: Determine from JSON
                let stat_change = StatChange {
                    target_type: BattleTarget::SELF,
                    name: PokemonStatName::SPECIAL_ATTACK, 
                    change: PokemonStatModifier::MINUS_2,
                    accuracy: 1.0
                };
                let v_stat = vec![stat_change];
                vec![BattlePreAction::Stat(v_stat)]
            },
            Iron_Head => {
                return vec![
                    BattlePreAction::Effect(
                        vec![BattlePreEffect {
                        target_type: OPPONENT,
                        effect_type: BattleEffect::Flinch,
                        accuracy: ONE_THIRD,
                        damage_source: PokemonMoveName::Iron_Head.to_string()
                    }
                ])]
            },
            Rock_Slide => {
                return vec![BattlePreAction::Effect(vec![
                    BattlePreEffect {
                        target_type: OPPONENT,
                        effect_type: BattleEffect::Flinch,
                        accuracy: ONE_THIRD,
                        damage_source: PokemonMoveName::Rock_Slide.to_string()
                    }
                ])]
            }
            // TODO: Make a generator that takes a list of default stuff and returns
            _ => return vec![] // no effect
        }
    }

    pub fn intn_condition_check (&self, 
        battle_state:&BattleState, 
        move_action:&MoveAction) -> bool {
        // let name = self.name;
        match self.name {
            // perform move condition here
            // Draco_Meteor => move_action.
            _ => return true
        }
    }

    /// Override the default accuracy check if necessary for this move
    pub fn accuracy_check (&self,
        battle_state:&BattleState) -> f64 {
            // Perfect accuracy moves ignore accuracy check
            if self.accuracy > 1.0 {
                return 1.0;
            }
            match self.name {
                // Put 1-HIT KO into here
                // If minimize/submerged etc
                _ => {
                    return self.accuracy
                }
            }
        }

    fn get_flinch_chance( &self, target_type:BattleTarget, chance:f64) -> BattlePreEffect {
        return BattlePreEffect {
            effect_type: Flinch,
            target_type: target_type,
            accuracy: chance,
            damage_source: "Flinched".to_string()
        }
    }
}

pub enum BattlePreAction {
    Stat(Vec<StatChange>),
    Effect(Vec<BattlePreEffect>)
}

pub struct BattlePreEffect {
    pub target_type: BattleTarget,
    pub effect_type: BattleEffect,
    pub accuracy: f64,
    pub damage_source: String 
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Display, Debug, Deserialize)]
pub enum PokemonMoveName {
    Draco_Meteor,
    Kowtow_Cleave,
    Iron_Head,
    Night_Slash,
    Crunch,
    Stone_Edge,
    Dragon_Claw,
    Earthquake,
    Rock_Slide

}

pub fn get_move(pkmn_move:PokemonMoveName) -> PokemonMove {
    match pkmn_move {
        PokemonMoveName::Draco_Meteor => PokemonMove {
            name: pkmn_move,
            r#type: PokemonType::DRAGON,
            power: 130,
            category: PokemonMoveCategory::Special,
            accuracy: 0.9,
            pp: 12,
            contact: false,
            priority: 0,
            target_type: OPPONENT
        },
        PokemonMoveName::Iron_Head => PokemonMove {
            name: pkmn_move,
            r#type: PokemonType::STEEL,
            power: 80,
            category: PokemonMoveCategory::Physical,
            accuracy: 1.0,
            pp: 24,
            contact: true,
            priority: 0,
            target_type: OPPONENT
        },
        PokemonMoveName::Dragon_Claw => PokemonMove::new(
            PokemonMoveName::Dragon_Claw, 
            PokemonType::DRAGON, 
            PokemonMoveCategory::Physical 
        ).set_attr(80, 1.0, BattleTarget::ALL),
        
        _ => panic!("Move has not been implemented!")
    }
}
