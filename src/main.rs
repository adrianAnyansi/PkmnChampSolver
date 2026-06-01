use crate::battle::PokemonType;
use crate::battle::get_type_multipler;

fn main() {
    println!("Hello, world!");

    analyze();
}

mod battle;
mod pokemon;

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
    let type_result= get_type_multipler(type_a, type_b);
    println!("{type_a} attacks on {type_b} do x{type_result} damage")
}