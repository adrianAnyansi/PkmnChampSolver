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

impl PokemonStats {
    pub fn get_arr(&self) -> Vec<i32> {
        vec![self.hp, self.attack, self.defense, self.sp_attack, self.sp_defense, self.speed]
    }

    pub fn get_stat_by_name(&mut self, stat_name: PokemonStatName) -> &mut i32 {
        match stat_name {
            PokemonStatName::HEALTH => &mut self.hp,
            PokemonStatName::ATTACK => &mut self.attack,
            PokemonStatName::DEFENSE => &mut self.defense,
            PokemonStatName::SPECIAL_ATTACK => &mut self.sp_attack,
            PokemonStatName::SPECIAL_DEFENSE => &mut self.sp_defense,
            PokemonStatName::SPEED => &mut self.speed
        }
    }

    pub fn check_stat_by_name(self, stat_name: PokemonStatName) -> i32 {
        match stat_name {
            PokemonStatName::HEALTH => self.hp,
            PokemonStatName::ATTACK => self.attack,
            PokemonStatName::DEFENSE => self.defense,
            PokemonStatName::SPECIAL_ATTACK => self.sp_attack,
            PokemonStatName::SPECIAL_DEFENSE => self.sp_defense,
            PokemonStatName::SPEED => self.speed
        }
    }

    pub fn empty() -> Self {
        PokemonStats {
            hp: 0,
            attack: 0,
            defense: 0,
            sp_attack: 0,
            sp_defense: 0,
            speed: 0
        }
    }

}

// pub fn get_health_stat(base_stat:&PokemonStats, trained_stats:Option<&PokemonStats>) -> i32 {
//     let trained = trained_stats.unwrap_or_else(|| &PokemonStats::empty());
//     let combined_stat = *base_stat + *trained;
//     // combined_stat.hp + 75
//     75
// }

pub fn get_full_stat(base_stat:&PokemonStats, 
    trained_stats:Option<&PokemonStats>, 
    stat_name: PokemonStatName) -> i32 {
        // NOTE: I'm not happy with the ref -> temp -> clone here
        // I need to add a completely impl for + between references so I have to do this
        // because of the move-by-default Rust, rewrite this later
    let val = PokemonStats::empty();
    let trained = trained_stats.unwrap_or_else(|| &val);
    let combined_stat = base_stat.clone() + trained.clone();
    match stat_name {
        PokemonStatName::HEALTH => combined_stat.check_stat_by_name(PokemonStatName::HEALTH) + 75,
        _ => combined_stat.check_stat_by_name(stat_name) + 20,
    }

    // TODO: Multiply by Nature as well
}

impl std::ops::Add for PokemonStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PokemonStats {
            hp: self.hp + rhs.hp,
            attack: self.attack + rhs.attack,
            defense: self.defense + rhs.defense,
            sp_attack: self.sp_attack + rhs.sp_attack,
            sp_defense: self.sp_defense + rhs.sp_defense,
            speed: self.speed + rhs.speed
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PokemonStatName {
    HEALTH,
    ATTACK,
    DEFENSE,
    SPECIAL_ATTACK,
    SPECIAL_DEFENSE,
    SPEED
}

impl core::fmt::Display for PokemonStatName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PokemonStatName::*;
        match self {
            HEALTH => write!(f, "Health"),
            ATTACK => write!(f, "Attack"),
            DEFENSE => write!(f, "Defense"),
            SPECIAL_ATTACK => write!(f, "Special Attack"),
            SPECIAL_DEFENSE => write!(f, "Special Defense"),
            SPEED => write!(f, "Speed"),
        }
    }
}


pub fn gen_pkmn_stat(
    hp:i32, atk:i32, def:i32, satk:i32, sdef:i32, spd:i32
) -> PokemonStats {
    return PokemonStats {
        hp: hp, attack: atk, defense: def,
        sp_attack: satk, sp_defense: sdef, speed: spd
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PokemonStatModifier {
    ZERO = 0,
    MINUS_1 = -1,
    MINUS_2 = -2,
    MINUS_3 = -3,
    MINUS_4 = -4,
    MINUS_5 = -5,
    MINUS_6 = -6,
    PLUS_1 = 1,
    PLUS_2 = 2,
    PLUS_3 = 3,
    PLUS_4 = 4,
    PLUS_5 = 5,
    PLUS_6 = 6
}

// TODO: I would like a trait that can be applied to stat modifiers and etc
// This way every multiply is natively handled in order without thought
/// Convert the enum modifier to a float and apply to int with floor
impl std::ops::Mul<i32> for PokemonStatModifier {
    type Output = i32;

    fn mul(self, base_int: i32) -> Self::Output {
        let mult_val = self.mult();
        let rhs_value = base_int as f64;
        (mult_val * rhs_value) as i32
    }
}

impl std::ops::Mul<PokemonStatModifier> for i32 {
    type Output = i32;

    fn mul(self, rhs: PokemonStatModifier) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Add for PokemonStatModifier {
    type Output = PokemonStatModifier;

    fn add(self, rhs: Self) -> Self::Output {
        let val = self as i32 + rhs as i32;
        PokemonStatModifier::cast_int(val)
    }
}

impl std::ops::AddAssign for PokemonStatModifier {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}


use PokemonStatModifier::*;
impl PokemonStatModifier {
    pub fn mult (&self) -> f64 {
        use PokemonStatModifier::*;
        match self {
            ZERO => 1.0,
            MINUS_1 => 2.0/3.0,
            MINUS_2 => 2.0/4.0,
            MINUS_3 => 2.0/5.0,
            MINUS_4 => 2.0/6.0,
            MINUS_5 => 2.0/7.0,
            MINUS_6 => 2.0/8.0,
            PLUS_1  => 3.0/2.0,
            PLUS_2  => 4.0/2.0,
            PLUS_3  => 5.0/2.0,
            PLUS_4  => 6.0/2.0,
            PLUS_5  => 7.0/2.0,
            PLUS_6  => 8.0/2.0,
        }
    }

    const ORDERED: [PokemonStatModifier; 13] = [
        MINUS_6, MINUS_5, MINUS_4, MINUS_3, MINUS_2, MINUS_1, 
        ZERO,
        PLUS_1, PLUS_2, PLUS_3, PLUS_4, PLUS_5, PLUS_6, 
    ];
    pub fn cast_int(int_val:i32) -> PokemonStatModifier {
        let clamped_val = (int_val.min(6).max(-6) + 6) as usize;
        // TODO: This is not a safe index technically from the compiler viewpoint
        PokemonStatModifier::ORDERED[clamped_val]
    }
}


#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PokemonNature {
    // ATK+ Natures
    Brave,      // +ATK, -SPE
    Adamant,    // +ATK, -SPA
    Lonely,     // +ATK, -DEF
    Naughty,    // +ATK, -SPD
    
    // DEF+ Natures
    Bold,       // +DEF, -ATK
    Impish,     // +DEF, -SPA
    Lax,        // +DEF, -SPD
    Relaxed,    // +DEF, -SPE
    
    // SPA+ Natures
    Modest,     // +SPA, -ATK
    Quiet,      // +SPA, -SPE
    Rash,       // +SPA, -SPD
    Mild,       // +SPA, -DEF
    
    // SPD+ Natures
    Calm,       // +SPD, -ATK
    Careful,    // +SPD, -SPA
    Sassy,      // +SPD, -SPE
    Gentle,     // +SPD, -DEF
    
    // SPE+ Natures
    Timid,      // +SPE, -ATK
    Jolly,      // +SPE, -SPA
    Hasty,      // +SPE, -DEF
    Naive,      // +SPE, -SPD
    
    // Neutral Natures
    Bashful,
    Docile,
    Hardy,
    Quirky,
    Serious,
}


use crate::{math::mult_and_round, pokemon::poke_stat::PokemonStatName::HEALTH};
static NATURE_MODIFIER: f64 = 1.1;
impl PokemonNature {

    pub fn stat_changes(&self) -> (PokemonStatName, PokemonStatName) {
        use crate::pokemon::poke_stat::{PokemonStatName::{ATTACK, SPECIAL_ATTACK, SPECIAL_DEFENSE, SPEED, DEFENSE}};

        match self {
            // ATK+ Natures
            PokemonNature::Brave => (ATTACK, SPEED),
            PokemonNature::Adamant => (ATTACK, SPECIAL_ATTACK),
            PokemonNature::Lonely => (ATTACK, DEFENSE),
            PokemonNature::Naughty => (ATTACK, SPECIAL_DEFENSE),
            
            // DEF+ Natures
            PokemonNature::Bold => (DEFENSE, ATTACK),
            PokemonNature::Impish => (DEFENSE, SPECIAL_ATTACK),
            PokemonNature::Lax => (DEFENSE, SPECIAL_DEFENSE),
            PokemonNature::Relaxed => (DEFENSE, SPEED),
            
            // SPA+ Natures
            PokemonNature::Modest => (SPECIAL_ATTACK, ATTACK),
            PokemonNature::Quiet => (SPECIAL_ATTACK, SPEED),
            PokemonNature::Rash => (SPECIAL_ATTACK, SPECIAL_DEFENSE),
            PokemonNature::Mild => (SPECIAL_ATTACK, DEFENSE),
            
            // SPD+ Natures
            PokemonNature::Calm => (SPECIAL_DEFENSE, ATTACK),
            PokemonNature::Careful => (SPECIAL_DEFENSE, SPECIAL_ATTACK),
            PokemonNature::Sassy => (SPECIAL_DEFENSE, SPEED),
            PokemonNature::Gentle => (SPECIAL_DEFENSE, DEFENSE),
            
            // SPE+ Natures
            PokemonNature::Timid => (SPEED, ATTACK),
            PokemonNature::Jolly => (SPEED, SPECIAL_ATTACK),
            PokemonNature::Hasty => (SPEED, DEFENSE),
            PokemonNature::Naive => (SPEED, SPECIAL_DEFENSE),
            
            // Neutral Natures
            PokemonNature::Bashful => (SPECIAL_ATTACK, SPECIAL_ATTACK),
            PokemonNature::Docile => (ATTACK, ATTACK),
            PokemonNature::Hardy => (DEFENSE, DEFENSE),
            PokemonNature::Quirky => (SPECIAL_DEFENSE, SPECIAL_DEFENSE),
            PokemonNature::Serious => (SPEED, SPEED),
        }
    }

    pub fn get_stat_multiplier(&self, poke_stat: PokemonStats) -> PokemonStats {
        let (boosted_stat, reduced_stat) = self.stat_changes();
        if boosted_stat == reduced_stat {
            return poke_stat; // Neutral nature, no changes
        }
        let mut copy_stat = poke_stat.clone();

        // Update boosted & reduced stat here
        *copy_stat.get_stat_by_name(boosted_stat) = 
            mult_and_round(
                *copy_stat.get_stat_by_name(boosted_stat), 
                NATURE_MODIFIER);
        
        *copy_stat.get_stat_by_name(reduced_stat) = 
            mult_and_round(
                *copy_stat.get_stat_by_name(reduced_stat), 
                1.0/NATURE_MODIFIER);
        // let mut copy_arr = copy_stat.get_arr();
        // let boosted_idx = match boosted_stat {
        //     PokemonStatName::HEALTH => 0,
        //     _ => 1
        // };
        // copy_arr[boosted_idx] = (copy_arr[boosted_idx] as f64 * NATURE_MODIFIER) as i32;
        // let reduced_idx = reduced_stat as usize;
        // copy_arr[reduced_idx] = (copy_arr[reduced_idx] as f64 / NATURE_MODIFIER) as i32;
        // TODO: Dont be lazy, make index access based on the enum instead
        copy_stat
    }
}

#[cfg(test)]
mod tests {

use super::*;
    // use PokemonStatModifier::*;

    #[test]
    fn pokemon_stat_modifier_value() {
        assert_eq!(PokemonStatModifier::ZERO.mult(), 1.0);
        assert_eq!(PokemonStatModifier::MINUS_1.mult(), 2.0/3.0);
        assert_eq!(PokemonStatModifier::PLUS_1.mult(), 3.0/2.0);
    }

    #[test]
    fn pokemon_stat_modifier_mul_flooring_both_sides() {
        // stat check
        assert_eq!(PokemonStatModifier::PLUS_1 * 30, (3.0/2.0 * 30.0) as i32);
        assert_eq!(30 * PokemonStatModifier::PLUS_1, (3.0/2.0 * 30.0) as i32);

        // floor testing
        assert_eq!(PokemonStatModifier::MINUS_2 * 58, (2.0/4.0 * 58.0) as i32);
        assert_eq!(58 * PokemonStatModifier::MINUS_2, (2.0/4.0 * 58.0) as i32);
    }
}

mod test_poke_stat {
    use super::*;

    #[test]
    fn nature_stat_changes() {
        assert_eq!(PokemonNature::Brave.stat_changes(), (PokemonStatName::ATTACK, PokemonStatName::SPEED));
        assert_eq!(PokemonNature::Careful.stat_changes(), (PokemonStatName::SPECIAL_DEFENSE, PokemonStatName::SPECIAL_ATTACK));
    }

    #[test]
    fn stat_mult() {
        let base_stat = gen_pkmn_stat(100, 150, 200, 250, 300, 350);
        let modified_stat = PokemonNature::Brave.get_stat_multiplier(base_stat);
        // assert_ne!(modified_stat.attack, base_stat.attack); // Attack should be modified
        assert_eq!(modified_stat.hp, 100); // HP should be unchanged
        assert_eq!(modified_stat.attack, (150.0 * NATURE_MODIFIER) as i32); // Attack should be boosted
        assert_eq!(modified_stat.speed, (350.0 / NATURE_MODIFIER) as i32); // Speed should be reduced
    }
}