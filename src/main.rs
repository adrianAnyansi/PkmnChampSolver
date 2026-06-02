use crate::battle::ActivePokemon;
use crate::battle::BattleState;
use crate::battle::PokemonType;
use crate::battle::get_type_multipler;

fn main() {
    println!("Hello, world!");

    // analyze();
    make_battle_state();
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

fn make_battle_state() {
    let mut bs = BattleState::new();

    let mut garchomp = pokemon::get_pkmn(PokemonName::Garchomp);
    let draco_move = pokemon::moves::get_move(PokemonMoveName::Draco_Meteor);

    garchomp.learnset.push(draco_move);
    
    bs.f_poke1 = Some(ActivePokemon::new(garchomp));

    println!("Printing the current battle state:");
    println!("{}", bs.get_print_state())
}