use crate::battle::battle_processor::{BattleContainer, BattleProcessor};
use crate::battle::{data::{ActivePokemon, TrainedPokemon}, BattlePosition, BattleState};
use crate::math::PkmnRational;
use crate::pokemon::types::{PokemonType, get_type_multipler};

mod battle;
mod pokemon;
mod math;


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



fn battle_container_test() {

    println!("Battle processor startup");
    let mut battle_processor = BattleProcessor::new();

    let mut bs = BattleState::new();
    let venusaur = pokemon::get_pkmn(PokemonName::Venusaur);
    let garchomp = pokemon::get_pkmn(PokemonName::Garchomp);

    let venusaur_trained = TrainedPokemon::new(venusaur,
        pokemon::PokemonAbilityName::Nothing,
        pokemon::poke_stat::PokemonNature::Brave, None);
    bs.f_poke1 = Some(ActivePokemon::new(&venusaur_trained));
    bs.f_poke2 = Some(
        ActivePokemon::quick(PokemonName::Kingambit)
    );
    let garchomp_trained = TrainedPokemon::new(garchomp,
        pokemon::PokemonAbilityName::Nothing,
        pokemon::poke_stat::PokemonNature::Brave, None);
    bs.b_poke1 = Some(ActivePokemon::new(&garchomp_trained));


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