use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::ActivePokemon;
use crate::battle::{BattleAction, BattlePosition, BattleState};
use crate::math::PkmnRational;
use crate::pokemon::PokemonName;
use crate::pokemon::moves::{get_move, PokemonMoveName};

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