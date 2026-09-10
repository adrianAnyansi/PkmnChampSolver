// moves and information
#![allow(dead_code)]


use serde::Deserialize;
use strum_macros::{Display, EnumString};

use crate::{battle::{ BattleEffect::{self, Flinch}, BattlePosition, BattleState, MoveAction, data::{ActivePokemon, BattleWeatherState::{self, SANDSTORM, SNOW, STRONG_WINDS}}, }, math::PkmnRational, pokemon::{moves::{BattleTarget::{ANY, OPPONENT, OPPONENT_ALL}, PokemonMoveName::{Fake_Out, Solar_Beam, Stomping_Tantrum, }}, poke_stat::{PokemonStatModifier::{self, MINUS_1, PLUS_1, PLUS_2}, PokemonStatName::{self, ATTACK, DEFENSE, SPECIAL_ATTACK, SPECIAL_DEFENSE}}, types::PokemonType::ICE}};
use crate::battle::data::PokemonStatus::{self, BURNED, PARALYZED, SLEEP};
use crate::pokemon::types::PokemonType;

/// Move type and additional information
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PokemonMoveCategory {
    Physical,
    Special,
    Status
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, EnumString, Display, PartialEq)]
pub enum BattleTarget {
    /// Target 1 opponent
    OPPONENT,
    /// Target your ally but not yourself
    ALLY,
    /// Target any ally including self
    ALLY_ANY,
    /// Target self only
    SELF,
    /// Target both opponents
    OPPONENT_ALL,
    /// Target you and your ally
    ALLY_ALL,
    /// Target anyone on the field
    ANY,
    /// Target all users except self
    ALL_EXCEPT_SELF,
    /// Target all users including self
    ALL_SELF
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PokemonMove {
    pub name: PokemonMoveName,
    pub power: i32,
    pub r#type: PokemonType,
    // Explanation of the move type
    pub category: PokemonMoveCategory,
    pub accuracy: f64, // note need to convert to proper i32
    pp: i32,
    pub priority: i8,
    contact: bool,
    pub target_type: BattleTarget,
    pub hit_actions: Vec<MoveEffect>,

    pub flags: PokemonBitFlag128<PokemonMoveFlag>
}

/// Describe an effect that occurs after a move/ability
#[derive(Debug, Copy, Clone)]
pub enum MoveEffect {
    Stat(StatChange),
    Status(PokemonStatus, BattleTarget, PkmnRational),
    General(BattleEffect, BattleTarget, PkmnRational),
    /// Charge move, Source, Target
    Charge(BattlePosition)
}

/// Indicates a change in stat boosts
#[derive(Clone, Copy, Debug)]
pub struct StatChange {
    pub target_type: BattleTarget,
    pub name: PokemonStatName,
    pub change: PokemonStatModifier,
    pub accuracy: f64
}

impl StatChange {
    pub fn new (target_type:BattleTarget, 
        stat_name:PokemonStatName,
        change_amt:PokemonStatModifier,
        acc:f64) -> StatChange {
            StatChange {
                target_type,
                name: stat_name,
                change: change_amt,
                accuracy: acc
            }
        }
}

// TODO: Move to NumberConstants module for qol
const ONE_THIRD:f64 = 1.0/3.0;

/// Effects incurred by a move
// trait MoveSecondEffect {
//     /// Actions that modify battle state after hitting
//     fn afterSuccess (battle_state:&BattleState);
//     /// Can this move be performed
//     fn canPerform(battle_state:&BattleState);
//     /// Actions that occur before move is performed
//     fn beforeAction (battle_state:&BattleState) -> bool;
// }

impl<'simulation> PokemonMove {
    pub fn new (move_name:PokemonMoveName, 
        move_type:PokemonType, 
        catg:PokemonMoveCategory) -> Self {
        PokemonMove {
            name: move_name,
            r#type: move_type,
            category: catg,
            power: 50,
            accuracy: 1.0,
            pp: 32,
            contact: false,
            priority: 0,
            target_type: ANY,
            hit_actions: vec![],
            flags: PokemonBitFlag128::new(vec![])
        }
    }

    pub fn status (move_name:PokemonMoveName,
        move_type:PokemonType,
        target: BattleTarget
    ) -> Self {
        PokemonMove {
            name: move_name,
            r#type: move_type,
            category: PokemonMoveCategory::Status,
            power: 0,
            accuracy: 1.0,
            pp: 32,
            contact: false,
            priority: 0,
            target_type: target,
            hit_actions: vec![],
            flags: PokemonBitFlag128::new(vec![])
        }
    }

    pub fn set_attr(mut self,
        power:i32, acc:f64, target:BattleTarget 
    ) -> Self {
        self.power = power;
        self.accuracy = acc;
        self.target_type = target;
        self
    }

    pub fn set_power(mut self, power:i32) -> Self {
        self.power = power;
        self
    }

    pub fn set_target(mut self, target: BattleTarget) -> Self {
        self.target_type = target;
        self
    }

    pub fn stat_change(mut self, 
        name:PokemonStatName,
        change:PokemonStatModifier,
        target_type:BattleTarget,
        acc:f64
        ) -> Self {

        let stat_chg = StatChange { target_type, name, change, accuracy: acc };
        self.hit_actions.push(MoveEffect::Stat(stat_chg));
        self
    }

    pub fn status_effect(mut self,
        status_type:PokemonStatus,
        chance:PkmnRational,
        target:BattleTarget,) -> Self {

            self.hit_actions.push(MoveEffect::Status(status_type, target, chance));
            self
        }

    pub fn add_dummy_flag(mut self,
        flag:&str) -> Self {
            // TODO: Add custom flag for stuff
            self
        }

    pub fn add_flag(mut self, flag:PokemonMoveFlag) -> Self {
            self.flags.set_flag(flag);
            self
        }

    pub fn add_flinch(mut self,
        chance:PkmnRational) -> Self {
            self.hit_actions.push(MoveEffect::General(
                BattleEffect::Flinch, BattleTarget::OPPONENT, chance));
            self
    }

    pub fn add_generic(mut self, b_effect:BattleEffect,
    target_pos:BattleTarget, rat:PkmnRational) -> Self {
        // let real_rat = rat.unwrap_or(PkmnRational::ONE());
        self.hit_actions.push(MoveEffect::General(b_effect, target_pos, rat));
        self
    }


    // Move issues

    pub fn is_attack(&self) -> bool {
        self.category != PokemonMoveCategory::Status
    }

    pub fn is_status(&self) -> bool {
        self.category == PokemonMoveCategory::Status
    }

    #[deprecated]
    pub fn after_hit (&self, 
        battle_state:&BattleState) -> Vec<BattlePreAction> {
        use PokemonMoveName::*;
        match self.name {
            Draco_Meteor => {
                // get move_performer
                // TODO: Determine from JSON
                let stat_change = StatChange {
                    target_type: BattleTarget::SELF,
                    name: PokemonStatName::SPECIAL_ATTACK, 
                    change: PokemonStatModifier::MINUS_2,
                    accuracy: 1.0
                };
                let v_stat = vec![stat_change];
                vec![BattlePreAction::Stat(v_stat)]
            },
            Iron_Head => {
                return vec![
                    BattlePreAction::Effect(
                        vec![BattlePreEffect {
                        target_type: OPPONENT,
                        effect_type: BattleEffect::Flinch,
                        accuracy: ONE_THIRD,
                        damage_source: PokemonMoveName::Iron_Head.to_string()
                    }
                ])]
            },
            Rock_Slide => {
                return vec![BattlePreAction::Effect(vec![
                    BattlePreEffect {
                        target_type: OPPONENT,
                        effect_type: BattleEffect::Flinch,
                        accuracy: ONE_THIRD,
                        damage_source: PokemonMoveName::Rock_Slide.to_string()
                    }
                ])]
            }
            // TODO: Make a generator that takes a list of default stuff and returns
            _ => return vec![] // no effect
        }
    }

    #[deprecated(note="move to data backed version")]
    pub fn intn_condition_check (&self, 
        battle_state:&BattleState, 
        move_action:&MoveAction) -> bool {
        // let name = self.name;
        match self.name {
            // perform move condition here
            // Draco_Meteor => move_action.
            Fake_Out => {
                let source = battle_state.get_active(move_action.source);
                source.unwrap().turns_active == 0
            }
            _ => return true
        }
    }

    /// Override the default accuracy check if necessary for this move
    pub fn accuracy_check (&self,
        battle_state:&BattleState) -> f64 {
            // Perfect accuracy moves ignore accuracy check
            if self.accuracy > 1.0 {
                return 1.0;
            }
            match self.name {
                // Put 1-HIT KO into here
                // If minimize/submerged etc
                _ => {
                    return self.accuracy
                }
            }
        }

    fn get_flinch_chance( &self, target_type:BattleTarget, chance:f64) -> BattlePreEffect {
        return BattlePreEffect {
            effect_type: Flinch,
            target_type: target_type,
            accuracy: chance,
            damage_source: "Flinched".to_string()
        }
    }
}

pub enum BattlePreAction {
    Stat(Vec<StatChange>),
    Effect(Vec<BattlePreEffect>)
}

pub struct BattlePreEffect {
    pub target_type: BattleTarget,
    pub effect_type: BattleEffect,
    pub accuracy: f64,
    pub damage_source: String 
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Display, EnumString, Debug, Deserialize, Copy, Clone, PartialEq)]
pub enum PokemonMoveName {
    Draco_Meteor,
    Kowtow_Cleave,
    Iron_Head,
    Night_Slash,
    Crunch,
    Stone_Edge,
    Dragon_Claw,
    Earthquake,
    Rock_Slide,

    Sleep_Powder,
    Sludge_Bomb,
    Earth_Power,
    Protect,

    Heat_Wave,
    Solar_Beam,
    Weather_Ball,

    Stomping_Tantrum,
    
    Fake_Out,
    Flare_Blitz,
    Parting_Shot,
    Throat_Chop,

    Moonblast,
    Dazzling_Gleam,
    Calm_Mind,

    Matcha_Gotcha,
    Rage_Powder,
    Trick_Room,

    Close_Combat,
    Dire_Claw,

    Brave_Bird,
    Swords_Dance,

    Heavy_Slam,
    High_Horsepower,
    Wide_Guard,

    Will_O_Wisp,
    Thunderbolt,
    Hydro_Pump,
    Light_Screen,

    Knock_Off,
    Dragon_Dance

}

#[allow(non_camel_case_types)]
/// Flag of important move/item/ability effects
#[derive(Clone, Copy, Debug)]
pub enum PokemonMoveFlag {
    POWDER, // Powder moves are ignored by Grass, OverCoat & Safety Goggles
    SLICING, // Move boosted by Sharpness
    // PULSE, // Move boosted by Pulse (Mega Launcher)
    BALLISTIC, // Blocked by Bullet Proof
    // BITING, // Strong Jaw
    // PUNCHING, // Iron Fist & Punching Glove
    SOUND, // Throat Chop, Throat Spray and etc
    WIND, // Boosted by Wind power

    /// Move has a charging state
    CHARGING,
    /// Custom power/behaviour based on weather
    WEATHER_MODIFY,
    /// Custom power in general
    CUSTOM_POWER,

    PROTECT,    // Apply protect to this pokemon
    PROTECT_ACC, // Modify accuracy by consecutive protects if used
    INCRM_PROTECT_COUNTER, // Increment the protect counter if consecutive

    /// Priority +1 move
    PRIORITY_1,
    PRIORITY_3,    
    PRIORITY_4,
    PRIORITY_MINUS_1,
    PRIORITY_MINUS_6,

    IGNORE_ACC, // This move ignores accuracy checks

    /// 1/3 recoil damage
    RECOIL_1_3RD,
    /// 1/4 recoil damage
    RECOIL_1_4TH,

    /// 1/2 healing drain
    HEAL_1_2HF,
}

pub trait BitFlagValue128: Copy {
    fn as_u128(self) -> u128;
}

impl BitFlagValue128 for PokemonMoveFlag {
    fn as_u128(self) -> u128 {
        self as u128
    }
}

/// Move flag will have 128 slots, once enum increases, add another flag
#[derive(Clone, Debug)]
pub struct PokemonBitFlag128<T = PokemonMoveFlag>
where
    T: BitFlagValue128,
{
    flag: u128,
    _marker: std::marker::PhantomData<T>,
}

impl<T> PokemonBitFlag128<T>
where
    T: BitFlagValue128,
{
    pub fn new(init_flags: Vec<T>) -> Self {
        let mut pkmn_flag = PokemonBitFlag128::<T> {
            flag: 0,
            _marker: std::marker::PhantomData,
        };
        pkmn_flag.set_flags(init_flags);
        pkmn_flag
    }

    pub fn empty() -> Self {
        return PokemonBitFlag128::<T> {
            flag: 0,
            _marker: std::marker::PhantomData,
        };
    }

    /// Check flag is set in BitFlag
    pub fn has_flag(&self, flag_id: T) -> bool {
        let flag_int = flag_id.as_u128();
        self.flag & (1u128 << flag_int) != 0
    }
    
    pub fn set_flag(&mut self, flag_id: T) -> &mut Self {
        self.flag |= 1u128 << flag_id.as_u128();
        self
    }

    pub fn set_flags(&mut self, flags: Vec<T>) -> &mut Self {
        for flag in flags {
            self.set_flag(flag);
        }
        self
    }

    pub fn clear_flag(&mut self, flag_id: T) -> &mut Self {
        self.flag |= 0u128 << flag_id.as_u128();
        self
    }
}


// TODO: Convert this back into the generic to get
// the string of all flags set to true
impl<T> std::fmt::Display for PokemonBitFlag128<T>
where
    T: BitFlagValue128,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_flags_builds_with_all_flags() {
        let mut bit_flag = PokemonBitFlag128::<PokemonMoveFlag>::new(vec![]);
        bit_flag.set_flags(vec![PokemonMoveFlag::POWDER]);

        assert!(bit_flag.has_flag(PokemonMoveFlag::POWDER));
    }
}

pub fn get_move<'simulation>(pkmn_move_name:PokemonMoveName) -> PokemonMove {

    use PokemonMoveCategory::*;
    use PokemonType::*;
    use BattleTarget::*;
    use PokemonMoveName::*;
    use PokemonMoveFlag::*;

    match pkmn_move_name {
        PokemonMoveName::Draco_Meteor => PokemonMove::new(
            pkmn_move_name, DRAGON, Special
        ).set_attr(130, 0.9, OPPONENT)
        .stat_change(PokemonStatName::SPECIAL_ATTACK, 
        PokemonStatModifier::MINUS_2,
    BattleTarget::SELF, 1.0),

        PokemonMoveName::Iron_Head => PokemonMove::new(
            pkmn_move_name, STEEL, Physical
        ).set_power(80)
        .add_flinch(PkmnRational::new(20, 100)),

        PokemonMoveName::Dragon_Claw => PokemonMove::new(
            PokemonMoveName::Dragon_Claw, 
            PokemonType::DRAGON, 
            PokemonMoveCategory::Physical 
        ).set_attr(80, 1.0, BattleTarget::OPPONENT),

        PokemonMoveName::Sludge_Bomb => PokemonMove::new(
            PokemonMoveName::Sludge_Bomb,
            PokemonType::POISON,
            PokemonMoveCategory::Special,
        ).set_attr(90, 1.0, OPPONENT)
        .status_effect(PokemonStatus::POISONED, 
             PkmnRational::new(30, 100),
            BattleTarget::OPPONENT),

        PokemonMoveName::Earth_Power => PokemonMove::new(
            PokemonMoveName::Earth_Power,
            PokemonType::GROUND,
            Special
        ).set_attr(80, 1.0, OPPONENT)
        .stat_change(SPECIAL_DEFENSE, MINUS_1, OPPONENT, 
        PkmnRational::pct(10).float()),
        
        PokemonMoveName::Sleep_Powder => PokemonMove::status(
            pkmn_move_name, GRASS, ANY
        ).set_attr(0, 0.75, ANY)
        .status_effect(SLEEP, PkmnRational::ONE(), OPPONENT)
        .add_flag(PokemonMoveFlag::POWDER),

        PokemonMoveName::Heat_Wave => PokemonMove::new(
            pkmn_move_name, FIRE, Special
        ).set_attr(95, PkmnRational::pct(90).float(), OPPONENT_ALL)
        .status_effect(BURNED, PkmnRational::pct(10), BattleTarget::OPPONENT,),

        
        PokemonMoveName::Solar_Beam => PokemonMove::new(
            pkmn_move_name, GRASS, Special
        ).set_attr(120, 1.0, OPPONENT)
        .add_flag(PokemonMoveFlag::CHARGING)
        .add_flag(PokemonMoveFlag::WEATHER_MODIFY)
        .add_dummy_flag("weather_boost")
        .add_dummy_flag("weather_charge"),

        PokemonMoveName::Weather_Ball => PokemonMove::new(
            pkmn_move_name, NORMAL, Special
        ).set_attr(60, 1.0, OPPONENT)
        .add_flag(PokemonMoveFlag::BALLISTIC)
        .add_flag(PokemonMoveFlag::WEATHER_MODIFY),
        
        PokemonMoveName::Protect => PokemonMove::status(
            pkmn_move_name, NORMAL, SELF
        ).set_attr(0, 1.0, SELF)
        .add_flag(PokemonMoveFlag::PROTECT)
        .add_flag(PokemonMoveFlag::INCRM_PROTECT_COUNTER)
        .add_flag(PokemonMoveFlag::PROTECT_ACC)
        .add_flag(PokemonMoveFlag::PRIORITY_4)
        .add_generic(BattleEffect::Protect, BattleTarget::SELF, PkmnRational::ONE())
        .add_dummy_flag("priority +4"),

        PokemonMoveName::Earthquake => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_attr(100, 1.0, ALL_EXCEPT_SELF)
        .add_dummy_flag("boost damage against dig"),
        
        PokemonMoveName::Rock_Slide => PokemonMove::new(
            pkmn_move_name, ROCK, Physical
        ).set_attr(90, 0.85, OPPONENT_ALL)
        .add_flinch(PkmnRational::pct(30)),
        
        PokemonMoveName::Stomping_Tantrum => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_attr(75, 1.0, OPPONENT)
        .add_flag(CUSTOM_POWER),

        PokemonMoveName::Fake_Out => PokemonMove::new(
            pkmn_move_name, NORMAL, Physical
        ).set_attr(40, 1.0, OPPONENT)
        .add_flinch(PkmnRational::ONE())
        .add_flag(PRIORITY_3)
        .add_dummy_flag("priority +3")
        .add_dummy_flag("custom_use"), // Cant be selected in Champions after turn 1
        // TODO: Prevent use after turn 1

        PokemonMoveName::Flare_Blitz => PokemonMove::new(
            pkmn_move_name, FIRE, Physical
        ).set_attr(120, 1.0, OPPONENT)
        .status_effect(BURNED, PkmnRational::pct(10), BattleTarget::OPPONENT,)
        .add_flag(RECOIL_1_3RD)
        .add_dummy_flag("recoil 1/3"),


        PokemonMoveName::Parting_Shot => PokemonMove::status(
            pkmn_move_name, DARK, OPPONENT)
            .stat_change(ATTACK, MINUS_1, OPPONENT, PkmnRational::ONE().float())
            .stat_change(SPECIAL_ATTACK, MINUS_1, OPPONENT, PkmnRational::ONE().float())
            .add_dummy_flag("switch self"), // TODO: Add switch effect

        PokemonMoveName::Throat_Chop => PokemonMove::new(
            pkmn_move_name, DARK, Physical
        ).set_power(80)
        .add_dummy_flag("Throat_chopped 2turns"),

        PokemonMoveName::Moonblast => PokemonMove::new(
            pkmn_move_name, FAIRY, Special
        ).set_power(95)
        .stat_change(ATTACK, MINUS_1, OPPONENT, PkmnRational::pct(10).float()),

        Dazzling_Gleam => PokemonMove::new(
            pkmn_move_name, FAIRY, Special
        ).set_power(80).set_target(OPPONENT_ALL),

        Calm_Mind => PokemonMove::status(
            pkmn_move_name, NORMAL, SELF
        ).stat_change(SPECIAL_ATTACK,PLUS_1, SELF, PkmnRational::ONE().float())
        .stat_change(SPECIAL_DEFENSE,PLUS_1, SELF, PkmnRational::ONE().float()),

        Matcha_Gotcha => PokemonMove::new(
            pkmn_move_name, GRASS, Special
        ).set_power(80).set_target(OPPONENT_ALL)
        .add_flag(HEAL_1_2HF)
        .status_effect(BURNED, PkmnRational::pct(20), BattleTarget::OPPONENT,)
        .add_dummy_flag("recover 1/2"),

        Rage_Powder => PokemonMove::status(
            pkmn_move_name, BUG, SELF
        ).add_dummy_flag("center_of_attention"),

        Trick_Room => PokemonMove::status(
            pkmn_move_name, PSYCHIC, SELF)
        .add_dummy_flag("trick room"),

        // reduce stats on hit
        Close_Combat => PokemonMove::new(
            pkmn_move_name,  FIGHTING, Physical
        ).set_power(120)
        .stat_change(DEFENSE, MINUS_1, SELF, PkmnRational::ONE().float())
        .stat_change(SPECIAL_DEFENSE, MINUS_1, SELF, PkmnRational::ONE().float()),

        Dire_Claw => PokemonMove::new(
            pkmn_move_name, POISON, Physical
        ).set_power(80)
        .status_effect(BURNED, PkmnRational::pct(10), BattleTarget::OPPONENT,)
        .status_effect(PARALYZED, PkmnRational::pct(10), BattleTarget::OPPONENT,)
        .status_effect(SLEEP, PkmnRational::pct(10), BattleTarget::OPPONENT,)
        .add_flag(PokemonMoveFlag::SLICING),
        
        Swords_Dance => PokemonMove::status(
            pkmn_move_name, NORMAL, SELF)
        .stat_change(ATTACK, PLUS_2, SELF, PkmnRational::ONE().float())
        .stat_change(ATTACK, PLUS_2, SELF, PkmnRational::ONE().float()),
        
        Heavy_Slam => PokemonMove::new(
            pkmn_move_name, STEEL, Physical
        ).set_power(0)
        .add_dummy_flag("custom_power")
        .add_dummy_flag("power based on weight"),

        High_Horsepower => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_power(95),
        Wide_Guard => PokemonMove::status(
            pkmn_move_name, ROCK, ALLY_ALL)
        .add_dummy_flag("spread_protect")
        .add_dummy_flag("protect_stall")
        .add_dummy_flag("priority +3"),
        
        Will_O_Wisp => PokemonMove::status(
            pkmn_move_name, FIRE, OPPONENT
        ).set_attr(0, PkmnRational::pct(85).float(), OPPONENT)
        .status_effect(BURNED, PkmnRational::ONE(), BattleTarget::OPPONENT),
        
        Thunderbolt => PokemonMove::new(
            pkmn_move_name, ELECTRIC, Special
        ).set_power(90)
        .status_effect(PARALYZED, PkmnRational::pct(10), BattleTarget::OPPONENT,),

        Hydro_Pump => PokemonMove::new(
            pkmn_move_name, WATER, Special
        ).set_power(110).set_attr(120, 0.80, OPPONENT),

        Light_Screen => PokemonMove::status(
            pkmn_move_name, PSYCHIC, ALLY_ALL
        ).add_dummy_flag("light_screen"),
        
        Knock_Off => PokemonMove::new(
            pkmn_move_name, DARK, Physical
        ).set_power(65)
        .add_dummy_flag("knock_off"),
        
        _ => panic!("Move has not been implemented!")
    }
}

use crate::pokemon::moves::PokemonMoveName::*;


pub fn get_charge_message(move_name:PokemonMoveName) -> String {
    match move_name {
        PokemonMoveName::Solar_Beam => "{source_name} absorbed sunlight!".to_string(),
        _ => format!("{} is charging!", move_name)
    }
}

pub fn format_pkmn_message(message_template:String,
    source_poke:&ActivePokemon,
    dest_poke:Option<ActivePokemon>) -> String {

        message_template
        .replace("{source_poke}", &source_poke.to_string())
        .replace("{dest_poke}", &dest_poke.map_or("dest_poke".to_string(), |p| p.to_string()))
    }

/// Return moves modified by weather
pub fn get_weather_modify_move(weather:BattleWeatherState, move_name:PokemonMoveName) -> PokemonMove {
    use crate::battle::data::BattleWeatherState::*;
    match move_name {
        Weather_Ball => {
            let mut base_weather_ball = get_move(move_name);
            base_weather_ball.power = 100;
            match weather {
                SUN => base_weather_ball.r#type = PokemonType::FIRE,
                RAIN => base_weather_ball.r#type = PokemonType::WATER,
                SNOW  => base_weather_ball.r#type = PokemonType::ICE,
                SANDSTORM => base_weather_ball.r#type = PokemonType::ROCK,
                _ => base_weather_ball.power = 50,
            }
            base_weather_ball
        },
        Solar_Beam => {
            let mut base_move = get_move(move_name);
            if ![BattleWeatherState::NONE, BattleWeatherState::SUN].contains(&weather) {
                base_move.power = 60;
            }
            base_move
        },
        _ => panic!("Not implemented move for weather modify")
    }
}

/// Get custom base_power based on current battle/etc
pub fn get_custom_base_power(battle_state:&BattleState, source:&ActivePokemon, 
    move_name:PokemonMoveName) -> PokemonMove {
    match move_name {
        Stomping_Tantrum => {
            let mut base_move = get_move(move_name);
            // let source_act_poke = battle_state.get_active(source);
            if source.last_move_failed {
                base_move.power = 140;
            }
            base_move
        },
        _ => panic!("Not implemented in custom_base_power")
    }
}

