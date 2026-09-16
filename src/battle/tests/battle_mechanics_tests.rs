use super::*;
use crate::pokemon::PokemonName::{Charizard, Garchomp, Kingambit, Venusaur};
use crate::pokemon::moves::PokemonMoveName::Heat_Wave;
use crate::pokemon::poke_stat::PokemonStatModifier::MINUS_5;
use crate::pokemon::poke_stat::PokemonStatName::*;
use crate::pokemon::*;
use crate::pokemon::moves::{get_move, PokemonMoveName};

#[test]
/// Verify base damage formula using blank Garchomp Draco Meteor vs Kingambit
fn test_move_calc() {
    let garchomp_base_stat = &get_pkmn(Garchomp).base_stats;
    let kingambit_base_stat = &get_pkmn(Kingambit).base_stats;
    let draco_move = get_move(PokemonMoveName::Draco_Meteor);

    let atk_stat = get_full_stat(garchomp_base_stat, None,
        PokemonStatName::SPECIAL_ATTACK);
    let def_stat = get_full_stat(kingambit_base_stat, None,
        PokemonStatName::SPECIAL_DEFENSE);
    let dmg = pkmn_damage_formula(draco_move.power, atk_stat, def_stat);

    let f_dmg = mult_and_round(dmg, 1.5);
    let f2_dmg = mult_and_round(f_dmg, 0.5);
    assert_eq!(f2_dmg, 42);

    let min_dmg = mult_and_round(f2_dmg, 0.85);
    assert_eq!(min_dmg, 35);
}

#[test]
#[ignore = "currently failing lower bound because damage calculation is inaccurate"]
fn test_dmg2_calc() {
    let char_base_stat = &get_pkmn(Charizard).base_stats;
    let venu_base_stat = &get_pkmn(Venusaur).base_stats;
    let pkmn_move = get_move(PokemonMoveName::Heat_Wave);

    let atk_stat = get_full_stat(char_base_stat, None,
        PokemonStatName::SPECIAL_ATTACK);
    let def_stat = get_full_stat(venu_base_stat, None,
        PokemonStatName::SPECIAL_DEFENSE);
    let dmg = pkmn_damage_formula(pkmn_move.power, atk_stat, def_stat);

    let f_dmg = mult_and_round(dmg, 1.5);
    let f2_dmg = mult_and_round(f_dmg, 2.0);
    assert_eq!(f2_dmg, 138);

    let min_dmg = mult_and_round(f2_dmg, 0.85);
    assert_eq!(min_dmg, 116);
}

#[test]
fn test_move_accuracy_sim() {
    let ttar_pkmn = ActivePokemon::quick(PokemonName::Tyranitar);
    let ven_pkmn = ActivePokemon::quick(PokemonName::Venusaur);

    let mut bs = BattleState::simple(ttar_pkmn, ven_pkmn);
    let hydro_pump = get_move(PokemonMoveName::Hydro_Pump);
    BattleState::queue_move(&mut bs.action_queue,
        BattlePosition::F1, &hydro_pump,
        vec![BattlePosition::B1]);

    let mut root_bc = BattleContainer::simple(bs, PkmnRational::ONE());
    root_bc.sim_next_action();

    assert_eq!(root_bc.pct_chance, PkmnRational::ONE());
    assert_eq!(root_bc.battle_ctns.len(), 2);
    assert_eq!(root_bc.battle_ctns[0].pct_chance,
        PkmnRational::ONE() - PkmnRational::from_float(hydro_pump.accuracy));
    assert!(root_bc.battle_ctns[0].battle_state.is_some());
    assert_eq!(root_bc.battle_ctns[0].battle_state
        .as_ref().unwrap().action_queue.len(), 0);
    assert!(root_bc.battle_ctns[1].battle_state.is_some());
    assert_eq!(root_bc.battle_ctns[1].battle_state
        .as_ref().unwrap().action_queue.len(), 1);

    let miss_bc = root_bc.battle_ctns.get_mut(0).unwrap();
    let miss_bs = miss_bc.battle_state.as_mut().unwrap();
    let charizard_idx = miss_bs.f_team.add_poke(ActivePokemon::quick(PokemonName::Charizard));
    miss_bs.send_out(BattlePosition::F2, charizard_idx);
    let rotom_idx = miss_bs.b_team.add_poke(ActivePokemon::quick(PokemonName::Rotom_Wash));
    miss_bs.send_out(BattlePosition::B2, rotom_idx);

    let heat_wave = get_move(Heat_Wave);
    BattleState::queue_move(&mut miss_bs.action_queue,
        BattlePosition::F2, &heat_wave,
        vec![BattlePosition::B1, BattlePosition::B2]);

    miss_bc.sim_next_action();

    assert_eq!(miss_bc.battle_ctns.len(), 4,
        "There should be 4 containers, no-state, b1 hit, b2 hit and b1+b2 hit");
}

fn dummy_bc<'battle>() -> BattleContainer<'battle> {
    let ttar_pkmn = ActivePokemon::quick(PokemonName::Tyranitar);
    let ven_pkmn = ActivePokemon::quick(PokemonName::Venusaur);

    let bs = BattleState::simple(ttar_pkmn, ven_pkmn);
    BattleContainer::simple(bs, PkmnRational::ONE())
}

#[test]
fn test_status_effect_sim() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    let status_action = StatusAction {
        targets: vec![BattlePosition::F1],
        status: PokemonStatus::BURNED,
        accuracy: PkmnRational::ONE(),
    };
    bs.action_queue.push_back(BattleAction::Status(status_action));

    assert_eq!(bs.action_queue.len(), 1);

    root_bc.sim_next_action();

    assert_eq!(root_bc.battle_ctns.len(), 0);
    let new_bs = root_bc.battle_state.as_ref().unwrap();

    assert_eq!(new_bs.action_queue.len(), 0);
    assert_eq!(new_bs.get_active(BattlePosition::F1).unwrap().status, PokemonStatus::BURNED,
        "Pokemon F1 should be burned");
}

#[test]
fn test_stat_modifier_sim() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    let stat_set = StatSet::make_stat_set(
        vec![(PokemonStatName::ATTACK, PokemonStatModifier::MINUS_5)]
    );
    bs.action_queue.push_back(BattleAction::PctActions(
        BattlePctAction::Stat(stat_set), [Some(BattlePosition::F1), None, None, None], 
        PkmnRational::ONE()
    ));

    assert_eq!(bs.action_queue.len(), 1);

    root_bc.sim_next_action();

    let new_bs = root_bc.battle_state.as_mut().unwrap();

    assert_eq!(root_bc.battle_ctns.len(), 0);
    assert_eq!(new_bs.action_queue.len(), 0);
    assert_eq!(new_bs.get_active_mut(BattlePosition::F1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::MINUS_5);
}

#[test]
fn test_multi_stat_modifier_sim() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    let stat_set = StatSet::make_stat_set(
        vec![(PokemonStatName::ATTACK, PokemonStatModifier::MINUS_5)]
    );
    bs.action_queue.push_back(BattleAction::PctActions(
        BattlePctAction::Stat(stat_set), [Some(BattlePosition::F1), None, None, None], PkmnRational::HALF()
    ));
    bs.action_queue.push_back(BattleAction::PctActions(
        BattlePctAction::Stat(stat_set), [Some(BattlePosition::B1), None, None, None], PkmnRational::HALF()
    ));

    assert_eq!(bs.action_queue.len(), 2);

    root_bc.sim_next_action();
    root_bc.sim_next_action(); // simulate 2 actions since each stat occurs separately

    assert_eq!(root_bc.count_tree(), 4);

    assert_eq!(root_bc.battle_ctns[0].pct_chance, PkmnRational::HALF(), "Percentage change should be 1/2");
    assert_eq!(root_bc.battle_ctns[0].battle_ctns[0].pct_chance, PkmnRational::HALF(), "Percentage change should be 1/2");

    // Need better way to get hit/result states for this correctly
    // currently its P,F -> PP, PF, FP, FF
    let no_bs = root_bc.battle_ctns[1].battle_ctns[1].battle_state.as_mut().unwrap();
    assert_eq!(no_bs.action_queue.len(), 0);
    assert_eq!(no_bs.get_active_mut(BattlePosition::F1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::ZERO, "Effect missed, Attack unchanged");
    assert_eq!(no_bs.get_active_mut(BattlePosition::B1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::ZERO, "Effect missed, Attack unchanged");

    let one_bs = root_bc.battle_ctns[0].battle_ctns[1].battle_state.as_mut().unwrap();
    assert_eq!(one_bs.action_queue.len(), 0);
    assert_eq!(one_bs.get_active_mut(BattlePosition::F1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::MINUS_5, "Effect hit, Attack lowered");
    assert_eq!(one_bs.get_active_mut(BattlePosition::B1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::ZERO, "Effect missed, Attack unchanged");
    
    let both_bs = root_bc.battle_ctns[0].battle_ctns[0].battle_state.as_mut().unwrap();
    assert_eq!(both_bs.action_queue.len(), 0);
    assert_eq!(both_bs.get_active_mut(BattlePosition::F1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::MINUS_5);
    assert_eq!(both_bs.get_active_mut(BattlePosition::B1).unwrap().get_active_stat_boost(ATTACK).clone(),
        PokemonStatModifier::MINUS_5);
}

#[test]
fn test_pct_action_flinch() {
    let mut root_bc = dummy_bc();
    let bs = root_bc.battle_state.as_mut().unwrap();

    bs.action_queue.push_back(BattleAction::PctAction(
        BattlePctAction::AddFlag(PokemonBattleState::FLINCHING, true),
        BattlePosition::F1,
        PkmnRational::ONE(),
    ));

    root_bc.sim_next_action();

    assert_eq!(root_bc.battle_ctns.len(), 0);
    let new_bs = root_bc.battle_state.as_ref().unwrap();
    assert_eq!(new_bs.action_queue.len(), 0);
    assert!(new_bs.get_active(BattlePosition::F1).unwrap().battle_status
        .has_flag(PokemonBattleState::FLINCHING));
}