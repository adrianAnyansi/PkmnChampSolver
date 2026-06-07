use serde::Deserialize;

/// Statistics of a Pokemon
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct PokemonStats {
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub sp_attack: i32,
    pub sp_defense: i32,
    pub speed: i32
}

pub enum PokemonStatName {
    HEALTH,
    ATTACK,
    DEFENSE,
    SPECIAL_ATTACK,
    SPECIAL_DEFENSE,
    SPEED
}

pub fn gen_pkmn_stat(
    hp:i32, atk:i32, def:i32, satk:i32, sdef:i32, spd:i32
) -> PokemonStats {
    return PokemonStats {
        hp: hp, attack: atk, defense: def,
        sp_attack: satk, sp_defense: sdef, speed: spd
    }
}

use crate::pokemon::PokemonName;

/// Get the base stats for a Pokemon in Champions
pub fn get_pkmn_stat(pkmn_name:PokemonName) -> PokemonStats {
    let stat = match pkmn_name {
        PokemonName::Garchomp => gen_pkmn_stat(108, 130, 95, 80, 85, 102),
        PokemonName::Kingambit => gen_pkmn_stat(100, 135, 120, 60, 85, 50),
        _ => {
            gen_pkmn_stat(0, 0, 0, 0, 0, 0)
        }
    };
    stat
}
