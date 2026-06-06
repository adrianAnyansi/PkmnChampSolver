use crate::battle::ActivePokemon;
use crate::battle::BattleState;
use crate::pokemon::types::{PokemonType, get_type_multipler};

fn main() {
    println!("Hello, world!");

    // analyze();
    garchomp_fight_test();
}

mod battle;
mod pokemon;
// use crate::pokemon::poke_stat::PokemonStats;
use crate::pokemon::PokemonName;
use crate::pokemon::moves::PokemonMoveName;

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

fn garchomp_fight_test() {
    let mut bs = BattleState::new();                  

    let mut garchomp = pokemon::get_pkmn(PokemonName::Garchomp);
    let mut bis = pokemon::get_pkmn(PokemonName::Bisharp);
    let draco_move = pokemon::moves::get_move(PokemonMoveName::Draco_Meteor);

    garchomp.learnset.push(draco_move);
    
    bs.f_poke1 = Some(ActivePokemon::new(garchomp));
    bs.b_poke1 = Some(ActivePokemon::new(bis));

    println!("Printing the current battle state:");
    println!("{}", bs.get_print_state());

    // queue
    let mut act_garchomp = bs.f_poke1.unwrap();
    let mut act_bis = bs.b_poke1.unwrap();
    
    BattleState::queue_move(
        &mut bs.action_queue,
        &mut act_garchomp, 
        &garchomp.learnset[0],
        vec![&mut act_bis]
    );
    // & act_garchomp.pokemon.learnset[0]);

    // do move
    bs.perform_turn();

    // print state again
    println!("Printing the current battle state:");
    println!("{}", bs.get_print_state());
}