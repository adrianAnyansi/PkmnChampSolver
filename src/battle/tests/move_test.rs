use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::{ActivePokemon, BattleWeatherState, PokemonBattleState};
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
fn test_rock_slide_flinches_before_heat_wave() {
    let mut root_bc = dummy_bc();
    let rock_slide = get_move(PokemonMoveName::Rock_Slide);
    let heat_wave = get_move(PokemonMoveName::Heat_Wave);

    let battle_state = root_bc.battle_state.as_mut().unwrap();
    BattleState::queue_move(
        &mut battle_state.action_queue,
        BattlePosition::F1,
        &rock_slide,
        vec![BattlePosition::B1],
    );
    BattleState::queue_move(
        &mut battle_state.action_queue,
        BattlePosition::B1,
        &heat_wave,
        vec![BattlePosition::F1],
    );

    root_bc.sim_next_action();

    assert_eq!(root_bc.battle_ctns.len(), 2);
    assert!(root_bc.battle_state.is_none());

    let (miss_states, hit_states) = root_bc.battle_ctns.split_at_mut(1);
    let miss_bc = &miss_states[0];
    let hit_bc = &mut hit_states[0];
    assert_eq!(miss_bc.pct_chance, PkmnRational::ONE() - PkmnRational::from_float(rock_slide.accuracy));
    assert_eq!(hit_bc.pct_chance, PkmnRational::from_float(rock_slide.accuracy));

    hit_bc.sim_next_action();
    hit_bc.sim_next_action();

    let hit_state = hit_bc.battle_state.as_ref().unwrap();
    assert!(hit_state.f_poke1.as_ref().unwrap().battle_status
        .has_flag(PokemonBattleState::FLINCHING));
    assert!(hit_state.action_strs.iter().any(|message| message == "Tyranitar flinched!"));
    assert!(hit_state.action_queue.iter().any(|action| matches!(action, BattleAction::Move(move_action)
        if move_action.pkm_move.name == PokemonMoveName::Heat_Wave)));

    hit_bc.sim_next_action();

    let hit_state = hit_bc.battle_state.as_ref().unwrap();
    assert_eq!(hit_state.f_poke1.as_ref().unwrap().current_hp,
        ActivePokemon::quick(PokemonName::Tyranitar).current_hp);
    assert!(hit_state.action_queue.is_empty());
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

#[test]
fn test_weather_ball_damage_boosts_and_changes_type_in_sun_with_dummy_bc() {
    fn weather_ball_damage_in_weather(weather: BattleWeatherState) -> i32 {
        let mut root_bc = dummy_bc();
        let battle_state = root_bc.battle_state.as_mut().unwrap();
        battle_state.weather = weather;

        let weather_ball = get_move(PokemonMoveName::Weather_Ball);
        let initial_hp = battle_state.f_poke1.as_ref().unwrap().current_hp;

        BattleState::queue_move(
            &mut battle_state.action_queue,
            BattlePosition::B1,
            &weather_ball,
            vec![BattlePosition::F1],
        );

        root_bc.sim_next_action();
        root_bc.sim_next_action();

        let final_hp = root_bc
            .battle_state
            .as_ref()
            .unwrap()
            .f_poke1
            .as_ref()
            .unwrap()
            .current_hp;

        initial_hp - final_hp
    }

    let none_damage = weather_ball_damage_in_weather(BattleWeatherState::NONE);
    let sun_damage = weather_ball_damage_in_weather(BattleWeatherState::SUN);

    assert!(sun_damage > none_damage,
        "Weather Ball should deal more damage in sun than with no weather");
    // TODO: This test should explicitly check the damage on a neutral pokemon resistance
    // or figure out a way to directly get moves before execution

}
