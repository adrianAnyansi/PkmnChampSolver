use std::collections::HashMap;

use strum_macros::{Display, AsRefStr, EnumString};
use serde::Deserialize;

use crate::{battle::{BattleState, data::{ActivePokemon, BattleWeatherState}}, math::PkmnRational, pokemon::{moves::{BattleTarget, BitFlagValue128, MoveEffect, PokemonBitFlag128, PokemonMove, StatSet}, poke_stat::{PokemonStatModifier, PokemonStatName}, types::PokemonType::{self, FIRE, GRASS}}};

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

// pub type MoveDamageModifier = for<'battle> fn(
//     &BattleState<'battle>,
//     &ActivePokemon<'battle>,
//     &ActivePokemon<'battle>,
//     &PokemonMove,
// ) -> PkmnRational;

pub type MoveDamageModifier = 
// Box< dyn for<'battle> Fn(
     for<'battle> fn(
        &BattleState<'battle>,
        &ActivePokemon<'battle>,
        &ActivePokemon<'battle>,
        &PokemonMove,
    ) -> PkmnRational
// >
;

// #[derive(Clone)]
pub struct PokemonAbility {
    pub name:PokemonAbilityName,
    /// Actions triggered on entering the field
    pub on_enter: Vec<MoveEffect>,
    pub type_flags: PokemonBitFlag128<AbilityTriggerFlag>,
    pub move_damage_modifier: Option<MoveDamageModifier>,
}

impl<'simulation> PokemonAbility {
    pub fn new(name:PokemonAbilityName) -> Self {
        PokemonAbility { 
            name, 
            on_enter: vec![],
            type_flags: PokemonBitFlag128::<AbilityTriggerFlag>::empty(),
            move_damage_modifier: None,
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

// pub fn add_flag(mut self, flag:AbilityTriggerFlag)

    // I cant add a function and 
    pub fn add_damage_calc(mut self) -> Self {
        self
    }

    fn active_starter_ability_logic(boost_type:PokemonType, 
        curr_types:&[PokemonType], health_pct:PkmnRational,
        pkmn_move:&PokemonMove) -> bool {
        return pkmn_move.r#type == boost_type &&
            (health_pct <= PkmnRational::new(1, 3) && 
            curr_types.contains(&boost_type))
    }

    fn add_damage_modifier(mut self, modf_closure:MoveDamageModifier) -> Self{
        self.move_damage_modifier = Some(modf_closure);
        self.type_flags.set_flag(AbilityTriggerFlag::OnMoveDamage);
        self
    }
}

fn make_ability(ably_name: PokemonAbilityName) -> PokemonAbility {
    match ably_name {
        PokemonAbilityName::Blaze => PokemonAbility {
            move_damage_modifier: Some(
                |_battle_state, source, _target, pkmn_move| {
                    if PokemonAbility::active_starter_ability_logic(FIRE, 
                        source.get_active_types(),
                    source.get_health_pct(),
                    pkmn_move) {
                    PkmnRational::new(3, 2)
                } else {
                    PkmnRational::ONE()
                }
            }),
            ..PokemonAbility::new(ably_name)
        },
        PokemonAbilityName::Overgrow => {
            PokemonAbility::new(ably_name).add_damage_modifier(
                |_battle_state, source, _target, pkmn_move| {
                    if PokemonAbility::active_starter_ability_logic(GRASS, 
                        source.get_active_types(), 
                        source.get_health_pct(),
                        pkmn_move) {
                        PkmnRational::new(3, 2)
                    } else {
                        PkmnRational::ONE()
                    }
                }
            )
        },
        PokemonAbilityName::Chlorophyll => {
            PokemonAbility::new(ably_name)
            
        }
        _ => PokemonAbility::new(ably_name),
    }
}
