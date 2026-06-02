// moves and information

use crate::battle::PokemonType;

/// Move type and additional information
enum PokemonMoveType {
    Physical,
    Special,
    Status
}

pub struct PokemonMove {
    name: PokemonMoveName,
    power: i32,
    r#type: PokemonType,
    // Explanation of the move type
    category: PokemonMoveType,
    accuracy: f64, // note need to convert to proper i32
    pp: i32,
    priority: i8,
    contact: bool,
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
    fn afterSuccess (&self, battle_state:&BattleState) {
        use PokemonMoveName::*;
        let move_result = None; // Move result here
        match self.name {
            Draco_Meteor => {
                // get move_performer
                // reduce sp_atk by 2 if possible
            },
            _ => return // no effect
        }
    }
}


pub enum PokemonMoveName {
    Draco_Meteor,
    Kowtow_Cleave
}

pub fn get_pokemon_move(pkmn_move:PokemonMoveName) -> PokemonMove {
    match pkmn_move {
        PokemonMoveName::Draco_Meteor => PokemonMove {
            name: pkmn_move,
            r#type: PokemonType::DRAGON,
            power: 130,
            category: PokemonMoveType::Special,
            accuracy: 0.9,
            pp: 12,
            contact: false,
            priority: 0
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