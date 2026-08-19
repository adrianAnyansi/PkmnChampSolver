use crate::battle::battle_processor::{BattleContainer, BattleProcessor};
use crate::battle::{data::ActivePokemon, BattlePosition, BattleState};
use crate::math::PkmnRational;
use crate::pokemon::types::{PokemonType, get_type_multipler};

mod battle;
mod pokemon;
mod math;

#[cfg(test)]
mod move_tests;

use crate::pokemon::PokemonName;
use crate::pokemon::moves::PokemonMoveName::{self};

// #[allow(unused)]

fn main() {
    println!("Pokemon Solver starting up!");

    // garchomp_fight_test();
    // make_pokemon_from_file();
    battle_container_test();

    println!("Pokemon Solver complete!");
}

fn make_pokemon_from_file() {
    // let poke_vec = pokemon::get_stat_json();
    let poke_vec = pokemon::POKEMON_HASH.get(&PokemonName::Garchomp).unwrap();
    println!("Pokemon from json: {poke_vec:#?}");
}

fn garchomp_fight_test() {
    let mut bs = BattleState::new();                  

    let garchomp = pokemon::get_pkmn(PokemonName::Garchomp);
    let bis = pokemon::get_pkmn(PokemonName::Kingambit);
    let draco_move = pokemon::moves::get_move(PokemonMoveName::Draco_Meteor);

    // garchomp.learnset.push(draco_move);
    
    bs.f_poke1 = Some(ActivePokemon::new(garchomp, 
        pokemon::PokemonAbilityName::Sand_Force, 
        pokemon::poke_stat::PokemonNature::Brave, None));
    bs.b_poke1 = Some(ActivePokemon::new(bis, 
        pokemon::PokemonAbilityName::Sand_Force, 
        pokemon::poke_stat::PokemonNature::Brave, None));

    println!("Printing the current battle state:");
    println!("{}", bs.get_print_state());

    // queue
    BattleState::queue_move(
        &mut bs.action_queue,
        BattlePosition::F1,
        &draco_move,
        vec![BattlePosition::B1],
    );
    // & act_garchomp.pokemon.learnset[0]);

    // do move
    bs.perform_turn();
    // There should be a print statement of the move being performed

    // print state again
    println!("Printing the current battle state:");
    println!("{}", bs.get_print_state());
}

fn battle_container_test() {

    println!("Battle processor startup");
    let mut battle_processor = BattleProcessor::new();

    let mut bs = BattleState::new();
    let venusaur = pokemon::get_pkmn(PokemonName::Venusaur);
    let garchomp = pokemon::get_pkmn(PokemonName::Garchomp);

    bs.f_poke1 = Some(
        ActivePokemon::new(venusaur, 
            pokemon::PokemonAbilityName::Nothing, 
            pokemon::poke_stat::PokemonNature::Brave, 
            None)
    );
    bs.f_poke2 = Some(
        ActivePokemon::quick(PokemonName::Kingambit)
    );
    bs.b_poke1 = Some(ActivePokemon::new(garchomp, 
        pokemon::PokemonAbilityName::Nothing, 
        pokemon::poke_stat::PokemonNature::Brave, 
        None));


    println!("Created {}, {} pokemon", venusaur, garchomp);
    let sludge_bomb_move = &pokemon::moves::get_move(PokemonMoveName::Sludge_Bomb);

    BattleState::queue_move(&mut bs.action_queue, 
        BattlePosition::F1, 
        sludge_bomb_move, 
        vec![BattlePosition::B1]);

    let earthquake = &pokemon::moves::get_move(PokemonMoveName::Earthquake);
    BattleState::queue_move(&mut bs.action_queue, 
        BattlePosition::B1, 
        earthquake, 
        vec![BattlePosition::F1, BattlePosition::F2]);

    let parting_shot = &pokemon::moves::get_move(PokemonMoveName::Parting_Shot);
    BattleState::queue_move(&mut bs.action_queue, 
        BattlePosition::F2, 
        parting_shot, 
        vec![BattlePosition::B1, BattlePosition::B2]);

    // make the root container
    let bc = BattleContainer::simple(bs, 
    PkmnRational::ONE());
    battle_processor.battle_ctns.push(bc);


    println!("\nBegin processing\n---\n");
    // Simulate container
    battle_processor.process_all_states_by_one_turn(true);

    println!("Processed the entire turn")

}