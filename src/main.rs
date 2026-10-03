use crate::battle::battle_processor::{BattleContainer, BattleProcessor};
use crate::battle::{data::{ActivePokemon, TrainedPokemon}, FieldPosition, BattleState};
use crate::math::PkmnRational;

mod battle;
mod pokemon;
mod math;


use crate::pokemon::abilities::{PokemonAbilityLibrary, PokemonAbilityName};
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

    let AbilityLibrary = PokemonAbilityLibrary::new();

    let venusaur_trained = TrainedPokemon::new(venusaur,
        AbilityLibrary.get_ability(PokemonAbilityName::Chlorophyll),
        pokemon::poke_stat::PokemonNature::Brave, None);
    let venusaur_idx = bs.f_team.add_poke(ActivePokemon::new(&venusaur_trained));
    bs.exec_send_out(FieldPosition::F1, venusaur_idx);
    let kingambit_idx = bs.f_team.add_poke(ActivePokemon::quick(PokemonName::Kingambit));
    bs.exec_send_out(FieldPosition::F2, kingambit_idx);
    let garchomp_trained = TrainedPokemon::new(garchomp,
        AbilityLibrary.get_ability(PokemonAbilityName::Nothing),
        pokemon::poke_stat::PokemonNature::Brave, None);
    let garchomp_idx = bs.b_team.add_poke(ActivePokemon::new(&garchomp_trained));
    bs.exec_send_out(FieldPosition::B1, garchomp_idx);


    println!("Created {}, {} pokemon", venusaur, garchomp);
    let sludge_bomb_move = &pokemon::moves::get_move(PokemonMoveName::Sludge_Bomb);

    BattleState::queue_move(&mut bs.action_queue, 
        FieldPosition::F1, 
        sludge_bomb_move, 
        vec![FieldPosition::B1]);

    let earthquake = &pokemon::moves::get_move(PokemonMoveName::Earthquake);
    BattleState::queue_move(&mut bs.action_queue, 
        FieldPosition::B1, 
        earthquake, 
        vec![FieldPosition::F1, FieldPosition::F2]);

    let parting_shot = &pokemon::moves::get_move(PokemonMoveName::Parting_Shot);
    BattleState::queue_move(&mut bs.action_queue, 
        FieldPosition::F2, 
        parting_shot, 
        vec![FieldPosition::B1, FieldPosition::B2]);

    // make the root container
    let bc = BattleContainer::simple(bs, 
    PkmnRational::ONE());
    battle_processor.battle_ctns.push(bc);


    println!("\nBegin processing\n---\n");
    // Simulate container
    battle_processor.process_all_states_by_one_turn(true);

    println!("Processed the entire turn")

}