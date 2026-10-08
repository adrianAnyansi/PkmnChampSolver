use std::cell::OnceCell;

use strum::EnumCount;
use strum_macros::{Display, AsRefStr, EnumString, EnumCount as EnumCountMacro};
use serde::Deserialize;

use crate::{battle::{BattleState, DamageEffect, TeamIndex, data::{ActivePokemon, BattleWeatherState}}, math::PkmnRational, pokemon::{Pokemon, abilities::AbilityTriggerFlag::{CausesSpeedChange, OnEnter, OnMoveDamage, OnWeatherChange}, moves::{BitFlagValue128, DamageAmount, FieldTarget, MoveEffect, PokemonBitFlag128, PokemonMove, StatSet}, poke_stat::{PokemonStatModifier, PokemonStatName}, types::PokemonType::{self, FIRE, GRASS}}};

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

pub type OnEnterFn<'simulation> =
    for<'battle> fn(
        &BattleState<'battle, 'simulation>,
        &ActivePokemon<'battle, 'simulation>
    ) -> Vec<MoveEffect>;

pub type OnMoveDamageFn<'simulation> =
    for<'battle> fn(
        &BattleState<'battle, 'simulation>,
        TeamIndex,
        &mut DamageEffect,
    ) -> Vec<MoveEffect>;

pub type OnWeatherChangeFn<'simulation> =
    for <'battle> fn(
        &BattleState<'battle, 'simulation>,
        BattleWeatherState,
    ) -> Vec<MoveEffect>;

// #[derive(Clone)]
pub struct PokemonAbility<'simulation> {
    pub name:PokemonAbilityName,
    pub type_flags: PokemonBitFlag128<AbilityTriggerFlag>,
    
    /// Actions triggered on entering the field
    pub enter_fn:Option<OnEnterFn<'simulation>>,
    /// Actions triggered by MoveDamage
    pub move_damage_fn:Option<OnMoveDamageFn<'simulation>>,
    /// Weather change actions
    pub weather_change_fn:Option<OnWeatherChangeFn<'simulation>>,
    
    /// Modifier for move damage/calculation
    pub move_damage_modifier: Option<MoveDamageModifier<'simulation>>,
    /// Modifier for speed modification
    pub speed_modif_func: Option<SpeedModifier<'simulation>>,


    /// Ties this ability's lifetime to the simulation it was built in
    _marker: std::marker::PhantomData<&'simulation ()>,
}

impl<'simulation> PokemonAbility<'simulation> {
    pub fn new(name:PokemonAbilityName) -> Self {
        PokemonAbility { 
            name, 
            // on_enter: vec![],
            enter_fn: None,
            move_damage_fn: None,
            weather_change_fn: None,
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
        enter_fn: OnEnterFn<'simulation>
    ) -> Self {
        self.enter_fn = Some(enter_fn);
        self.add_flag(OnEnter)
    }

    pub fn add_move_damage_effect(mut self,
        damage_fn: OnMoveDamageFn<'simulation>
    ) -> Self {
        self.move_damage_fn = Some(damage_fn);
        self.add_flag(OnMoveDamage)
    }

    pub fn add_weather_chg_effect(mut self,
        weather_eff: OnWeatherChangeFn<'simulation>
    ) -> Self {
        self.weather_change_fn = Some(weather_eff);
        self.add_flag(OnWeatherChange)
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
            .add_weather_chg_effect(
                |_battle_state:&BattleState, prev_w:BattleWeatherState| {
                    if prev_w == BattleWeatherState::SUN ||
                    _battle_state.weather == BattleWeatherState::SUN {
                        vec![MoveEffect::SpeedChangeFlag]
                    } else {
                        vec![]
                    }
                }
            )
            .add_speed_modifier(
                |battle_state:&BattleState, _source:&ActivePokemon| {
                    if battle_state.weather == BattleWeatherState::SUN {
                        return Some(PkmnRational::new(2, 1))
                    }
                    None
                }
            ).add_flag(OnWeatherChange)
        },
        PokemonAbilityName::Rough_Skin => {
            PokemonAbility::new(ably_name)
            .add_move_damage_effect(
                |_battle_state:&BattleState, _source:TeamIndex, damage:&mut DamageEffect| {
                    // TODO: Check if move is a contact move
                    // damage effect needs a team_index to target a pokemon, not a battle target*
                    return vec![MoveEffect::Damage(FieldTarget::OPPONENT, 
                        DamageAmount::HealthPct(PkmnRational::new(1, 8)))]
                }
            )
        },
        PokemonAbilityName::Intimidate => {
            PokemonAbility::new(ably_name)
            .add_enter_effect(
                |_battle_state:&BattleState, _source:&ActivePokemon| {
                    return vec![MoveEffect::Stat(
                        StatSet::make_stat_set(vec![(PokemonStatName::ATTACK, PokemonStatModifier::MINUS_1)]), 
                        FieldTarget::OPPONENT_ALL, PkmnRational::ONE())]
                }
            )
        },
        PokemonAbilityName::Hospitality => {
            PokemonAbility::new(ably_name)
            .add_enter_effect(
                |_battle_state:&BattleState, _source:&ActivePokemon| {
                    return vec![
                        MoveEffect::Healing(FieldTarget::ALLY, 
                            DamageAmount::HealthPct(PkmnRational::new(1, 8)))
                    ]
                }
            )
        },
        _ => PokemonAbility::new(ably_name),
    }
}