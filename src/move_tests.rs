use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::{ActivePokemon, BattleWeatherState};
use crate::battle::{BattleAction, BattlePosition, BattleState};
use crate::math::PkmnRational;
use crate::pokemon::PokemonName;
use crate::pokemon::moves::{get_move, get_weather_modify_move, PokemonMoveName};
use crate::pokemon::types::PokemonType;

fn dummy_bc<'battle>() -> BattleContainer<'battle> {
    let garchomp = ActivePokemon::quick(PokemonName::Garchomp);
    let tyranitar = ActivePokemon::quick(PokemonName::Tyranitar);

    let battle_state = BattleState::simple(tyranitar, garchomp);
    BattleContainer::simple(battle_state, PkmnRational::ONE())
}

#[test]
fn test_protect_blocks_earthquake_damage() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    let protect = get_move(PokemonMoveName::Protect);
    let earthquake = get_move(PokemonMoveName::Earthquake);

    BattleState::queue_move(
        &mut bs.action_queue,
        BattlePosition::B1,
        &protect,
        vec![BattlePosition::B1],
    );
    BattleState::queue_move(
        &mut bs.action_queue,
        BattlePosition::F1,
        &earthquake,
        vec![BattlePosition::B1],
    );

    assert_eq!(bs.action_queue.len(), 2);

    root_bc.sim_next_action();
    root_bc.sim_next_action();

    let future_actions = &root_bc
        .battle_state
        .as_ref()
        .expect("battle state should still exist after the queued actions")
        .action_queue;

    assert!(
        future_actions.iter().all(|action| !matches!(action, BattleAction::Damage(_))),
        "Protect should block the Earthquake damage targeting Garchomp"
    );
}

#[test]
fn test_solar_beam_takes_two_move_actions_to_damage() {
    let mut root_bc = dummy_bc();
    let solar_beam = get_move(PokemonMoveName::Solar_Beam);
    let initial_hp = root_bc.battle_state.as_ref().unwrap().f_poke1.as_ref().unwrap().current_hp;

    BattleState::queue_move(
        &mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::B1,
        &solar_beam,
        vec![BattlePosition::F1],
    );
    root_bc.sim_next_action();

    let charged_state = root_bc.battle_state.as_ref().unwrap();
    assert_eq!(charged_state.f_poke1.as_ref().unwrap().current_hp, initial_hp);
    assert!(charged_state.b_poke1.as_ref().unwrap().battle_status.has_flag(
        crate::battle::data::PokemonBattleState::CHARGING
    ));

    BattleState::queue_move(
        &mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::B1,
        &solar_beam,
        vec![BattlePosition::F1],
    );
    root_bc.sim_next_action();

    assert!(root_bc.battle_state.as_ref().unwrap().action_queue.iter()
        .any(|action| matches!(action, BattleAction::Damage(_))));

    root_bc.sim_next_action();
    assert!(root_bc.battle_state.as_ref().unwrap().f_poke1.as_ref().unwrap().current_hp < initial_hp);
}

#[test]
fn test_solar_beam_skips_charge_in_sun() {
    let mut root_bc = dummy_bc();
    let solar_beam = get_move(PokemonMoveName::Solar_Beam);
    let battle_state = root_bc.battle_state.as_mut().unwrap();
    battle_state.weather = BattleWeatherState::SUN;
    let initial_hp = battle_state.f_poke1.as_ref().unwrap().current_hp;

    BattleState::queue_move(
        &mut battle_state.action_queue,
        BattlePosition::B1,
        &solar_beam,
        vec![BattlePosition::F1],
    );
    root_bc.sim_next_action();

    let battle_state = root_bc.battle_state.as_ref().unwrap();
    assert!(!battle_state.b_poke1.as_ref().unwrap().battle_status.has_flag(
        crate::battle::data::PokemonBattleState::CHARGING
    ));
    assert!(battle_state.action_queue.iter()
        .any(|action| matches!(action, BattleAction::Damage(_))));

    root_bc.sim_next_action();
    assert!(root_bc.battle_state.as_ref().unwrap().f_poke1.as_ref().unwrap().current_hp < initial_hp);
}


#[test]
fn test_protect_blocks_will_o_wisp_no_miss() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    let protect = get_move(PokemonMoveName::Protect);
    let inacc_move = get_move(PokemonMoveName::Will_O_Wisp);

    BattleState::queue_move(
        &mut bs.action_queue,
        BattlePosition::B1,
        &protect,
        vec![BattlePosition::B1],
    );
    BattleState::queue_move(
        &mut bs.action_queue,
        BattlePosition::F1,
        &inacc_move,
        vec![BattlePosition::B1],
    );

    assert_eq!(bs.action_queue.len(), 2);

    root_bc.sim_next_action();
    root_bc.sim_next_action();

    assert!( root_bc.battle_ctns.len() == 0, 
    "Only 1 state should exist since protect ignores accuracy check");

    let future_actions = &root_bc
        .battle_state
        .as_ref()
        .expect("battle state should still exist after the queued actions")
        .action_queue;

    assert!(
        future_actions.iter().all(|action| !matches!(action, BattleAction::Status(_))),
        "Protect should block Will-o-Wisp status targeting Garchomp"
    );
}

#[test]
fn test_weather_ball_power_and_type_by_weather() {
    let none_weather_ball = get_weather_modify_move(BattleWeatherState::NONE, PokemonMoveName::Weather_Ball);
    assert_eq!(none_weather_ball.power, 50, "Weather Ball should be 50 BP with no weather");
    assert_eq!(none_weather_ball.r#type, PokemonType::NORMAL, "Weather Ball should stay Normal with no weather");

    let weather_cases = vec![
        (BattleWeatherState::SUN, PokemonType::FIRE),
        (BattleWeatherState::RAIN, PokemonType::WATER),
        (BattleWeatherState::SNOW, PokemonType::ICE),
        (BattleWeatherState::SANDSTORM, PokemonType::ROCK),
    ];

    for (weather, expected_type) in weather_cases {
        let weather_ball = get_weather_modify_move(weather, PokemonMoveName::Weather_Ball);
        assert_eq!(weather_ball.power, 100);
        assert_eq!(weather_ball.r#type, expected_type);
    }
}