
use std::collections::HashMap;

use strum_macros::{Display, AsRefStr, EnumString};
use serde::Deserialize;

use crate::{battle::BattleAction, math::PkmnRational, pokemon::{moves::{BattleTarget, BitFlagValue128, MoveEffect, PokemonBitFlag128, StatSet}, poke_stat::{PokemonStatModifier, PokemonStatName}}};

#[allow(non_camel_case_types)]
#[derive(Deserialize, EnumString, Display,
    Debug, Copy, Clone, Eq, PartialEq,
    Hash)]
pub enum PokemonAbilityName {
    Sand_Force,
    Rough_Skin,
    Defiant,
    Sand_Stream,
    Unnerve,
    Supreme_Overlord,
    Sand_Veil,
    Intimidate,
    Blaze,
    Rock_Head,
    Sturdy,
    Sheer_Force,
    Solar_Power,
    Pressure,
    Unburden,
    Poison_Touch,
    Chlorophyll,
    Overgrow,
    Thick_Fat,
    Levitate,
    Flower_Veil,
    Symbiosis,
    Hospitality,
    Heatproof,
    Flame_Body,
    Gale_Wings,
    Nothing
}

#[derive(Clone, Copy)]
pub enum AbilityTriggerFlag {
    OnEnter,
    OnExit,
    OnMoveDamage,
    OnFaint
}

impl BitFlagValue128 for AbilityTriggerFlag {
    fn as_u128(self) -> u128 {
        self as u128
    }
}

struct PokemonAbility {
    pub name:PokemonAbilityName,
    /// Actions triggered on entering the field
    pub on_enter: Vec<MoveEffect>,
    pub type_flags: PokemonBitFlag128<AbilityTriggerFlag>
}

impl<'simulation> PokemonAbility {
    pub fn new(name:PokemonAbilityName) -> Self {
        PokemonAbility { 
            name, 
            on_enter: vec![],
            type_flags: PokemonBitFlag128::<AbilityTriggerFlag>::empty()
        }
    }

    pub fn add_enter_effect(mut self,
        target_type:BattleTarget,
        stat_vec:Vec<(PokemonStatName, PokemonStatModifier)>,
    ) -> Self {
        // TODO: Need to make this generic
        let stat_set = StatSet::make_stat_set(stat_vec);
        self.on_enter.push(MoveEffect::Stat(stat_set, target_type, PkmnRational::ONE()));
        self
    }

}

struct PokemonAbilityLibrary {
    pub ability_map:HashMap<PokemonAbilityName, PokemonAbility>
}

impl<'simulation> PokemonAbilityLibrary {
    pub fn get_ability(&mut self, ably_name:PokemonAbilityName) -> &PokemonAbility {

        let mut ability = PokemonAbility::new(ably_name);
        ability = match ably_name {

            PokemonAbilityName::Blaze => ability.add_enter_effect(BattleTarget::SELF, 
            vec![(PokemonStatName::ATTACK, PokemonStatModifier::PLUS_1)]),
            _ => panic!("Cant do")
        };

        self.ability_map.insert(ably_name, ability);

        &self.ability_map[&ably_name]
    }
}