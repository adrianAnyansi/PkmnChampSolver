// moves and information

use serde::Deserialize;
use strum_macros::{Display, EnumString};

use crate::{battle::{self, AddEffect, BattleAction, BattleEffect::{self, Flinch}, BattleState, MoveAction, PokemonStatus::{self, BURNED, PARALYZED, SLEEP}}, math::PkmnRational, pokemon::{moves::BattleTarget::{ANY, OPPONENT, OPPONENT_ALL}, poke_stat::{PokemonStatModifier::{self, MINUS_1, PLUS_1, PLUS_2}, PokemonStatName::{self, ATTACK, DEFENSE, SPECIAL_ATTACK, SPECIAL_DEFENSE}}}};
use crate::pokemon::types::PokemonType;

/// Move type and additional information
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PokemonMoveCategory {
    Physical,
    Special,
    Status
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
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
}

/// Describe an effect that occurs after a move/ability
#[derive(Debug, Copy, Clone)]
pub enum MoveEffect {
    Stat(StatChange),
    Status(PokemonStatus, PkmnRational),
    General(BattleEffect, PkmnRational)
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
            hit_actions: vec![]
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
            hit_actions: vec![]
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
        chance:PkmnRational) -> Self {

            self.hit_actions.push(MoveEffect::Status(status_type, chance));
            self
        }

    pub fn add_flag(mut self,
        flag:&str) -> Self {
            // TODO: Add custom flag for stuff
            self
        }

    pub fn add_flinch(mut self,
        chance:PkmnRational) -> Self {
            // let flinch_chance = 0;
            self.hit_actions.push(MoveEffect::General(
                BattleEffect::Flinch, chance));
            self
    }

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

    pub fn intn_condition_check (&self, 
        battle_state:&BattleState, 
        move_action:&MoveAction) -> bool {
        // let name = self.name;
        match self.name {
            // perform move condition here
            // Draco_Meteor => move_action.
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
#[derive(Display, EnumString, Debug, Deserialize, Copy, Clone)]
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

pub fn get_move<'simulation>(pkmn_move_name:PokemonMoveName) -> PokemonMove {

    use PokemonMoveCategory::*;
    use PokemonType::*;
    use BattleTarget::*;
    use PokemonMoveName::*;

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
        .status_effect(PokemonStatus::POISONED, PkmnRational::new(30, 100)),

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
        .status_effect(SLEEP, PkmnRational::ONE())
        .add_flag("Powder"),

        PokemonMoveName::Heat_Wave => PokemonMove::new(
            pkmn_move_name, FIRE, Special
        ).set_attr(95, PkmnRational::pct(90).float(), OPPONENT_ALL)
        .status_effect(BURNED, PkmnRational::pct(10)),

        
        PokemonMoveName::Solar_Beam => PokemonMove::new(
            pkmn_move_name, GRASS, Special
        ).set_attr(120, 1.0, OPPONENT)
        .add_flag("Charging")
        .add_flag("weather_boost")
        .add_flag("weather_charge"),

        PokemonMoveName::Weather_Ball => PokemonMove::new(
            pkmn_move_name, NORMAL, Special
        ).set_attr(60, 1.0, OPPONENT)
        .add_flag("weather_boost")
        .add_flag("custom_power"),
        
        PokemonMoveName::Protect => PokemonMove::status(
            pkmn_move_name, NORMAL, SELF
        ).set_attr(0, 1.0, SELF)
        .add_flag("protect")
        .add_flag("protect_stall")
        .add_flag("priority +4"),

        PokemonMoveName::Earthquake => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_attr(100, 1.0, ALL_EXCEPT_SELF)
        .add_flag("dig_boost"),
        
        PokemonMoveName::Rock_Slide => PokemonMove::new(
            pkmn_move_name, ROCK, Physical
        ).set_attr(90, 0.85, OPPONENT_ALL)
        .add_flinch(PkmnRational::pct(30)),
        
        PokemonMoveName::Stomping_Tantrum => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_attr(75, 1.0, OPPONENT)
        .add_flag("boost_if_failed_last"),

        PokemonMoveName::Fake_Out => PokemonMove::new(
            pkmn_move_name, NORMAL, Physical
        ).set_attr(40, 1.0, OPPONENT)
        .add_flinch(PkmnRational::ONE())
        .add_flag("custom_use"),
        // TODO: Prevent use after turn 1

        PokemonMoveName::Flare_Blitz => PokemonMove::new(
            pkmn_move_name, FIRE, Physical
        ).set_attr(120, 1.0, OPPONENT)
        .status_effect(BURNED, PkmnRational::pct(10))
        .add_flag("recoil 1/3"),


        PokemonMoveName::Parting_Shot => PokemonMove::status(
            pkmn_move_name, DARK, OPPONENT)
            .stat_change(ATTACK, MINUS_1, OPPONENT, PkmnRational::ONE().float())
            .stat_change(SPECIAL_ATTACK, MINUS_1, OPPONENT, PkmnRational::ONE().float())
            .add_flag("switch self"), // TODO: Add switch effect

        PokemonMoveName::Throat_Chop => PokemonMove::new(
            pkmn_move_name, DARK, Physical
        ).set_power(80)
        .add_flag("Throat_chopped 2"),

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
        .status_effect(BURNED, PkmnRational::pct(20))
        .add_flag("recover 1/2"),

        Rage_Powder => PokemonMove::status(
            pkmn_move_name, BUG, SELF
        ).add_flag("center_of_attention"),

        Trick_Room => PokemonMove::status(
            pkmn_move_name, PSYCHIC, SELF)
        .add_flag("trick room"),

        // reduce stats on hit
        Close_Combat => PokemonMove::new(
            pkmn_move_name,  FIGHTING, Physical
        ).set_power(120)
        .stat_change(DEFENSE, MINUS_1, SELF, PkmnRational::ONE().float())
        .stat_change(SPECIAL_DEFENSE, MINUS_1, SELF, PkmnRational::ONE().float()),

        Dire_Claw => PokemonMove::new(
            pkmn_move_name, POISON, Physical
        ).set_power(80)
        .status_effect(BURNED, PkmnRational::pct(10))
        .status_effect(PARALYZED, PkmnRational::pct(10))
        .status_effect(SLEEP, PkmnRational::pct(10))
        .add_flag("slicing"),
        
        Swords_Dance => PokemonMove::status(
            pkmn_move_name, NORMAL, SELF)
        .stat_change(ATTACK, PLUS_2, SELF, PkmnRational::ONE().float())
        .stat_change(ATTACK, PLUS_2, SELF, PkmnRational::ONE().float()),
        
        Heavy_Slam => PokemonMove::new(
            pkmn_move_name, STEEL, Physical
        ).set_power(0)
        .add_flag("custom_power")
        .add_flag("power based on weight"),

        High_Horsepower => PokemonMove::new(
            pkmn_move_name, GROUND, Physical
        ).set_power(95),
        Wide_Guard => PokemonMove::status(
            pkmn_move_name, ROCK, ALLY_ALL)
        .add_flag("spread_protect")
        .add_flag("protect_stall")
        .add_flag("priority +3"),
        
        Will_O_Wisp => PokemonMove::status(
            pkmn_move_name, FIRE, OPPONENT
        ).set_attr(0, PkmnRational::pct(85).float(), OPPONENT)
        .status_effect(BURNED, PkmnRational::ONE()),
        
        Thunderbolt => PokemonMove::new(
            pkmn_move_name, ELECTRIC, Special
        ).set_power(90)
        .status_effect(PARALYZED, PkmnRational::pct(10)),

        Hydro_Pump => PokemonMove::new(
            pkmn_move_name, WATER, Special
        ).set_power(110).set_attr(120, 0.80, OPPONENT),

        Light_Screen => PokemonMove::status(
            pkmn_move_name, PSYCHIC, ALLY_ALL
        ).add_flag("light_screen"),
        
        Knock_Off => PokemonMove::new(
            pkmn_move_name, DARK, Physical
        ).set_power(65)
        .add_flag("knock_off"),
        
        _ => panic!("Move has not been implemented!")
    }
}
