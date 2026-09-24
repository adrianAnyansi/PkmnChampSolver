use std::cell::OnceCell;

use strum::EnumCount;
use strum_macros::{Display, AsRefStr, EnumString, EnumCount as EnumCountMacro};
use serde::Deserialize;

use crate::{battle::{BattleState, data::{ActivePokemon, BattleWeatherState}}, math::PkmnRational, pokemon::{Pokemon, abilities::AbilityTriggerFlag::{CausesSpeedChange, OnWeatherChange}, moves::{BattleTarget, BitFlagValue128, MoveEffect, PokemonBitFlag128, PokemonMove, StatSet}, poke_stat::{PokemonStatModifier, PokemonStatName}, types::PokemonType::{self, FIRE, GRASS}}};

#[allow(non_camel_case_types)]
#[derive(Deserialize, EnumString, Display, EnumCountMacro,
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

// #[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum AbilityTriggerFlag {
    OnEnter,
    OnExit,
    OnMoveDamage,
    OnFaint,
    OnWeatherChange,
    CausesSpeedChange,
}

impl BitFlagValue128 for AbilityTriggerFlag {
    fn as_u128(self) -> u128 {
        self as u128
    }
}

// pub type MoveDamageModifier = for<'battle> fn(
//     &BattleState<'battle, 'simulation>,
//     &ActivePokemon<'battle, 'simulation>,
//     &ActivePokemon<'battle, 'simulation>,
//     &PokemonMove,
// ) -> PkmnRational;

pub type MoveDamageModifier<'simulation> = 
// Box< dyn for<'battle> Fn(
     for<'battle> fn(
        &BattleState<'battle, 'simulation>,
        &ActivePokemon<'battle, 'simulation>,
        &ActivePokemon<'battle, 'simulation>,
        &PokemonMove,
    ) -> Option<PkmnRational>
// >
;

pub type SpeedModifier<'simulation> = 
    for<'battle> fn(
        &BattleState<'battle, 'simulation>,
        &ActivePokemon<'battle, 'simulation>
    ) -> Option<PkmnRational>;

// #[derive(Clone)]
pub struct PokemonAbility<'simulation> {
    pub name:PokemonAbilityName,
    /// Actions triggered on entering the field
    pub on_enter: Vec<MoveEffect>,
    pub type_flags: PokemonBitFlag128<AbilityTriggerFlag>,
    pub move_damage_modifier: Option<MoveDamageModifier<'simulation>>,
    pub speed_modif_func: Option<SpeedModifier<'simulation>>,
    /// Ties this ability's lifetime to the simulation it was built in
    _marker: std::marker::PhantomData<&'simulation ()>,
}

impl<'simulation> PokemonAbility<'simulation> {
    pub fn new(name:PokemonAbilityName) -> Self {
        PokemonAbility { 
            name, 
            on_enter: vec![],
            type_flags: PokemonBitFlag128::<AbilityTriggerFlag>::empty(),
            move_damage_modifier: None,
            speed_modif_func: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_flag(mut self, flag:AbilityTriggerFlag) -> Self {
        self.type_flags.set_flag(flag);
        self
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

    fn add_damage_modifier(mut self, modf_closure:MoveDamageModifier<'simulation>) -> Self{
        self.move_damage_modifier = Some(modf_closure);
        self.type_flags.set_flag(AbilityTriggerFlag::OnMoveDamage);
        self
    }

    fn add_speed_modifier(mut self, modf:SpeedModifier<'simulation>) -> Self {
        self.speed_modif_func = Some(modf);
        self.type_flags.set_flag(CausesSpeedChange);
        self
    }
}

pub struct PokemonAbilityLibrary<'simulation> {
    /// One slot per `PokemonAbilityName` variant, indexed by discriminant, built lazily on first access
    slots: Box<[OnceCell<PokemonAbility<'simulation>>]>,
}

impl<'simulation> PokemonAbilityLibrary<'simulation> {
    pub fn new () -> Self {
        Self {
            slots: (0..PokemonAbilityName::COUNT).map(|_| OnceCell::new()).collect(),
        }
    }

    pub fn get_ability(&self, ably_name: PokemonAbilityName) -> &PokemonAbility<'simulation> {
        self.slots[ably_name as usize].get_or_init(|| make_ability(ably_name))
    }
}

pub fn make_ability<'simulation>(ably_name: PokemonAbilityName) -> PokemonAbility<'simulation> {
    match ably_name {
        PokemonAbilityName::Blaze => PokemonAbility {
            move_damage_modifier: Some(
                |_battle_state, source, _target, pkmn_move| {
                    if PokemonAbility::active_starter_ability_logic(FIRE, 
                        source.get_active_types(),
                    source.get_health_pct(),
                    pkmn_move) {
                        Some(PkmnRational::new(3, 2))
                    } else {
                        None
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
                        return Some(PkmnRational::new(3, 2))
                    }
                    None
                }
            )
        },
        PokemonAbilityName::Chlorophyll => {
            PokemonAbility::new(ably_name)
            .add_speed_modifier(
                |battle_state:&BattleState, source:&ActivePokemon| {
                    if battle_state.weather == BattleWeatherState::SUN {
                        return Some(PkmnRational::new(2, 1))
                    }
                    None
                }
            ).add_flag(OnWeatherChange)
        },
        _ => PokemonAbility::new(ably_name),
    }
}