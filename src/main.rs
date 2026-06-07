use crate::battle::{ActivePokemon, BattlePosition, BattleState};
use crate::pokemon::types::{PokemonType, get_type_multipler};

mod battle;
mod pokemon;
// use crate::pokemon::poke_stat::PokemonStats;
use crate::pokemon::PokemonName;
use crate::pokemon::moves::PokemonMoveName;

fn main() {
    println!("Pokemon Solver starting up!");

    // analyze();
    garchomp_fight_test();
    // make_pokemon_from_file();
    println!("Pokemon Solver complete!");
}

fn basic_poke() {
    use crate::pokemon::Pokemon;
    // Make a basic pokemon
    // let poke = Pokemon {
    //     name: PokemonName::Garchomp,
    // }
}


fn analyze() {
    // TODO: take some input
    let type_a = PokemonType::BUG;
    let type_b = PokemonType::FLYING;
    println!("Type A is {type_a} AND B is {type_b}");

    println!("Calculating type matchup...");
    let type_result = get_type_multipler(type_a, type_b);
    println!("{type_a} attacks on {type_b} do x{type_result} damage");

    // let mut poke_str = "Garchomp";
    let poke = pokemon::poke_stat::get_pkmn_stat(PokemonName::Garchomp);

    println!("Pokemon Attack stat is {attack}!", attack = poke.attack);

    let garchomp = pokemon::get_pkmn(PokemonName::Garchomp);
    println!("Pokemon display {}", garchomp);

    // poke_str = "Bisharp"
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
    
    bs.f_poke1 = Some(ActivePokemon::new(garchomp, pokemon::PokemonAbility::Sand_Force, pokemon::PokemonNature::Brave));
    bs.b_poke1 = Some(ActivePokemon::new(bis, pokemon::PokemonAbility::Sand_Force, pokemon::PokemonNature::Brave));

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