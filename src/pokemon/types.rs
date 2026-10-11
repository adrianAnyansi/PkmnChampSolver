use serde::Deserialize;
use serde::{Deserializer};
use crate::math::PkmnRational;

/// Pokemon Type
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum  PokemonType {
    NORMAL,
    FIRE,
    WATER,
    GRASS,
    ELECTRIC,
    BUG,
    FLYING,
    FIGHTING,
    GHOST,
    DARK,
    PSYCHIC,
    FAIRY,
    DRAGON,
    ROCK,
    GROUND,
    ICE,
    STEEL,
    POISON,
    TYPELESS
}

impl<'de> Deserialize<'de> for PokemonType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "NORMAL" => Ok(PokemonType::NORMAL),
            "FIRE" => Ok(PokemonType::FIRE),
            "WATER" => Ok(PokemonType::WATER),
            "GRASS" => Ok(PokemonType::GRASS),
            "ELECTRIC" => Ok(PokemonType::ELECTRIC),
            "BUG" => Ok(PokemonType::BUG),
            "FLYING" => Ok(PokemonType::FLYING),
            "FIGHTING" => Ok(PokemonType::FIGHTING),
            "GHOST" => Ok(PokemonType::GHOST),
            "DARK" => Ok(PokemonType::DARK),
            "PSYCHIC" => Ok(PokemonType::PSYCHIC),
            "FAIRY" => Ok(PokemonType::FAIRY),
            "DRAGON" => Ok(PokemonType::DRAGON),
            "ROCK" => Ok(PokemonType::ROCK),
            "GROUND" => Ok(PokemonType::GROUND),
            "ICE" => Ok(PokemonType::ICE),
            "STEEL" => Ok(PokemonType::STEEL),
            "POISON" => Ok(PokemonType::POISON),
            "TYPELESS" => Ok(PokemonType::TYPELESS),
            _ => Err(serde::de::Error::custom(format!("Invalid type: {}", s))),
        }
    }
}

impl core::fmt::Display for PokemonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PokemonType::NORMAL     => write!(f, "NORMAL"),
            PokemonType::FIRE       => write!(f, "FIRE"),
            PokemonType::WATER      => write!(f, "WATER"),
            PokemonType::GRASS      => write!(f, "GRASS"),
            PokemonType::ELECTRIC   => write!(f, "ELECTRIC"),
            PokemonType::BUG        => write!(f, "BUG"),
            PokemonType::FLYING     => write!(f, "FLYING"),
            PokemonType::FIGHTING   => write!(f, "FIGHTING"),
            PokemonType::GHOST      => write!(f, "GHOST"),
            PokemonType::DARK       => write!(f, "DARK"),
            PokemonType::PSYCHIC    => write!(f, "PSYCHIC"),
            PokemonType::FAIRY      => write!(f, "FAIRY"),
            PokemonType::DRAGON     => write!(f, "DRAGON"),
            PokemonType::ROCK       => write!(f, "ROCK"),
            PokemonType::GROUND     => write!(f, "GROUND"),
            PokemonType::ICE        => write!(f, "ICE"),
            PokemonType::STEEL      => write!(f, "STEEL"),
            PokemonType::POISON     => write!(f, "POISON"),
            PokemonType::TYPELESS     => write!(f, "TYPELESS"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PkmnTypeEffectiveness {
    SUPER_EFFECTIVE,
    NOT_EFFECTIVE,
    IMMUNE,
    REG_EFFECTIVE,
}

impl PkmnTypeEffectiveness {
    pub fn get_rat(self) -> PkmnRational {
        match self {
            Self::SUPER_EFFECTIVE => PkmnRational::new(2, 1),
            Self::NOT_EFFECTIVE => PkmnRational::HALF(),
            Self::IMMUNE => PkmnRational::ZERO(),
            Self::REG_EFFECTIVE => PkmnRational::ONE(),
        }
    }
}

/// Contains the type advantages of the game in a two-dimensional lookup table.
const TYPE_CHART_ARRAY: [[PkmnTypeEffectiveness; 18]; 18] = {
    use PokemonType::*;
    use PkmnTypeEffectiveness::*;
    
    const TUPLE_ARR: &[(PokemonType, PkmnTypeEffectiveness, &[PokemonType])] = &[
        (NORMAL, NOT_EFFECTIVE, &[ROCK, STEEL]),
        (NORMAL, IMMUNE, &[GHOST]),

        (FIRE, NOT_EFFECTIVE, &[FIRE, ROCK, WATER, DRAGON]),
        (FIRE, SUPER_EFFECTIVE, &[GRASS, BUG, STEEL, ICE]),

        (BUG, SUPER_EFFECTIVE, &[GRASS, PSYCHIC, DARK]),
        (BUG, NOT_EFFECTIVE, &[FIRE, FIGHTING, FLYING, GHOST, STEEL, FAIRY, ROCK, POISON]),

        (FLYING, SUPER_EFFECTIVE, &[FIGHTING, GRASS, BUG]),
        (FLYING, NOT_EFFECTIVE, &[ELECTRIC, ROCK, STEEL]),

        (GROUND, IMMUNE, &[FLYING]),
        (GROUND, SUPER_EFFECTIVE, &[POISON, ROCK, FIRE, ELECTRIC, STEEL]),
        (GROUND, NOT_EFFECTIVE, &[BUG, GRASS]),

        (STEEL, NOT_EFFECTIVE, &[STEEL, FIRE, WATER, ELECTRIC]),
        (STEEL, SUPER_EFFECTIVE, &[ICE, ROCK, FAIRY]),
        
        (FIGHTING, SUPER_EFFECTIVE, &[NORMAL, ROCK, ICE, STEEL, DARK]),
        (FIGHTING, NOT_EFFECTIVE, &[FLYING, POISON, BUG, PSYCHIC, FAIRY]),
        (FIGHTING, IMMUNE, &[GHOST]),
        
        (POISON, SUPER_EFFECTIVE, &[GRASS, FAIRY]),
        (POISON, NOT_EFFECTIVE, &[POISON, GROUND, ROCK, GHOST]),
        (POISON, IMMUNE, &[STEEL]),

        (ROCK, SUPER_EFFECTIVE, &[FLYING, BUG, FIRE, ICE]),
        (ROCK, NOT_EFFECTIVE, &[FIGHTING, GROUND, STEEL]),

        (GHOST, SUPER_EFFECTIVE, &[GHOST, PSYCHIC]),
        (GHOST, NOT_EFFECTIVE, &[DARK]),
        (GHOST, IMMUNE, &[NORMAL]),

        (WATER, NOT_EFFECTIVE, &[DRAGON, ELECTRIC, GRASS]),
        (WATER, SUPER_EFFECTIVE, &[ROCK, GROUND, FIRE]),

        (GRASS, NOT_EFFECTIVE, &[DRAGON, BUG, FLYING, POISON, STEEL, GRASS, FIRE]),
        (GRASS, SUPER_EFFECTIVE, &[ROCK, GROUND, WATER]),

        (ELECTRIC, NOT_EFFECTIVE, &[GRASS, DRAGON, ELECTRIC]),
        (ELECTRIC, IMMUNE, &[GROUND]),
        (ELECTRIC, SUPER_EFFECTIVE, &[WATER, FLYING]),

        (PSYCHIC, NOT_EFFECTIVE, &[PSYCHIC, STEEL]),
        (PSYCHIC, IMMUNE, &[DARK]),
        (PSYCHIC, SUPER_EFFECTIVE, &[FIGHTING, POISON]),

        (ICE, NOT_EFFECTIVE, &[WATER, FIRE, STEEL, ICE]),
        (ICE, SUPER_EFFECTIVE, &[FLYING, GRASS, ROCK, DRAGON]),

        (DRAGON, NOT_EFFECTIVE, &[STEEL]),
        (DRAGON, SUPER_EFFECTIVE, &[DRAGON]),
        (DRAGON, IMMUNE, &[FAIRY]),

        (DARK, NOT_EFFECTIVE, &[FIGHTING, DARK, FAIRY]),
        (DARK, SUPER_EFFECTIVE, &[PSYCHIC, GHOST]),

        (FAIRY, NOT_EFFECTIVE, &[FIRE, STEEL, POISON]),
        (FAIRY, SUPER_EFFECTIVE, &[DRAGON, FIGHTING, DARK]),
    ];

    let mut arr = [[REG_EFFECTIVE; 18]; 18];

    let mut iter = 0;

    while iter < TUPLE_ARR.len() {
        let (type_atk, effectiveness, type_defs) = TUPLE_ARR[iter];
        let mut def_iter = 0;
        while def_iter < type_defs.len() {
            arr[type_atk as usize][type_defs[def_iter] as usize] = effectiveness;
            def_iter += 1;
        }
        iter += 1;
    }

    arr

};

/// get the type multipler for an attacking->defending type
pub fn get_type_multipler(type_atk:PokemonType, type_def:PokemonType) -> PkmnTypeEffectiveness {
    if [type_atk, type_def].contains(&PokemonType::TYPELESS) {
        return PkmnTypeEffectiveness::REG_EFFECTIVE
    } 
    TYPE_CHART_ARRAY[type_atk as usize][type_def as usize]
}

#[cfg(test)]
mod tests {

    use crate::pokemon::types::*;
    use PkmnTypeEffectiveness::*;
    
    #[test]
    fn test_type_multiplier () {
        assert_eq!(
            get_type_multipler(PokemonType::NORMAL, PokemonType::NORMAL),
            REG_EFFECTIVE
        );
        
        let type_a = PokemonType::BUG;
        let type_b = PokemonType::FLYING;
        
        let type_result = get_type_multipler(type_a, type_b);
        assert_eq!(type_result, NOT_EFFECTIVE);

        assert_eq!(
            get_type_multipler(PokemonType::GROUND, 
                PokemonType::ELECTRIC),
            SUPER_EFFECTIVE
        );
        assert_eq!(
            get_type_multipler(PokemonType::GROUND, 
                PokemonType::FLYING),
            IMMUNE
        )
    }

    #[test]
    fn typeless_attack_is_regular_effective() {
        assert_eq!(
            get_type_multipler(PokemonType::TYPELESS, PokemonType::FIRE),
            REG_EFFECTIVE
        );
    }

    #[test]
    fn typeless_defender_is_regular_effective() {
        assert_eq!(
            get_type_multipler(PokemonType::FIRE, PokemonType::TYPELESS),
            REG_EFFECTIVE
        );
    }
}