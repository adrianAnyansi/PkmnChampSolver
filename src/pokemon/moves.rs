// moves and information

use crate::{battle::{BattleState, PokemonType}, pokemon::moves::PokemonMoveTarget::OPPONENT};

/// Move type and additional information
pub enum PokemonMoveCategory {
    Physical,
    Special,
    Status
}

pub enum PokemonMoveTarget {
    OPPONENT,
    ALLY,
    SELF,
    OPPONENT_ALL,
    ALLY_ALL,
    ANY
}

#[allow(dead_code)]
pub struct PokemonMove {
    name: PokemonMoveName,
    power: i32,
    r#type: PokemonType,
    // Explanation of the move type
    category: PokemonMoveCategory,
    accuracy: f64, // note need to convert to proper i32
    pp: i32,
    priority: i8,
    contact: bool,
    target: PokemonMoveTarget
}

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
            target: OPPONENT
        }
    }

    pub fn set_attr(mut self,
        power:i32, acc:f64, target:PokemonMoveTarget 
    ) -> Self {
        self.power = power;
        self.accuracy = acc;
        self.target = target;
        self
    }

    fn afterSuccess (&self, battle_state:&BattleState) {
        use PokemonMoveName::*;
        // let move_result = None; // Move result here
        match self.name {
            Draco_Meteor => {
                // get move_performer
                // reduce sp_atk by 2 if possible
            },
            _ => return // no effect
        }
    }
}


#[allow(dead_code)]
pub enum PokemonMoveName {
    Draco_Meteor,
    Kowtow_Cleave
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
            target: OPPONENT
        },
        _ => panic!("Move has not been implemented!")
    }
}

// pub fn get_pokemon_move_eff(pkmn_move:PokemonMoveName) -> {
//     match pkmn_move {
//         PokemonMoveName::Draco_Meteor => MoveSecondEffect {
            
//         }
        
//     }
// }