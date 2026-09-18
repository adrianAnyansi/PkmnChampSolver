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

#[test]
fn test_send_out_to_b2_places_pokemon_in_right_slot() {
    let mut bs = BattleState::new();
    let team_idx = bs.b_team.add_poke(ActivePokemon::quick(PokemonName::Charizard));

    bs.send_out(BattlePosition::B2, team_idx);

    let active_poke = bs.get_active(BattlePosition::B2)
        .expect("B2 should contain the sent-out pokemon");
    assert_eq!(active_poke.trained_pokemon.pokemon.name, PokemonName::Charizard);
    assert_eq!(bs.b_poke2, Some(team_idx));
}

#[test]
fn test_return_poke_clears_confusion_from_f2() {
    let mut bs = BattleState::new();
    let team_idx = bs.f_team.add_poke(ActivePokemon::quick(PokemonName::Garchomp));

    bs.send_out(BattlePosition::F2, team_idx);
    bs.get_active_mut(BattlePosition::F2)
        .unwrap()
        .battle_status
        .set_flag(PokemonBattleState::CONFUSED);

    bs.return_poke(BattlePosition::F2);

    assert_eq!(bs.f_poke2, None);
    let returned_poke = bs.f_team.get(team_idx).unwrap();
    assert!(!returned_poke.battle_status.has_flag(PokemonBattleState::CONFUSED),
        "battle_status should be cleared when the Pokémon returns to the team");
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

#[test]
/// Rage Powder should mark its user as center of attention, redirecting a
/// single-target move even when that move was aimed at an ally instead
fn test_rage_powder_redirects_ally_targeted_move() {
    let mut bs = BattleState::new();

    let f1_idx = bs.f_team.add_poke(ActivePokemon::quick(Charizard));
    let f2_idx = bs.f_team.add_poke(ActivePokemon::quick(Garchomp));
    let b1_idx = bs.b_team.add_poke(ActivePokemon::quick(Kingambit));

    bs.send_out(BattlePosition::F1, f1_idx);
    bs.send_out(BattlePosition::F2, f2_idx);
    bs.send_out(BattlePosition::B1, b1_idx);

    let mut root_bc = BattleContainer::simple(bs, PkmnRational::ONE());

    let rage_powder = get_move(PokemonMoveName::Rage_Powder);
    BattleState::queue_move(&mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::F1, &rage_powder, vec![BattlePosition::F1]);

    root_bc.sim_next_action(); // resolve move, queue the CENTER_OF_ATTENTION flag effect
    root_bc.sim_next_action(); // apply the flag effect

    let bs_after_rage_powder = root_bc.battle_state.as_ref().unwrap();
    assert!(bs_after_rage_powder.get_active(BattlePosition::F1).unwrap().battle_status
        .has_flag(PokemonBattleState::CENTER_OF_ATTENTION),
        "F1 should be the center of attention after using Rage Powder");

    // B1 targets its own ally (B2, which is empty) with Sleep Powder
    let sleep_powder = get_move(PokemonMoveName::Sleep_Powder);
    BattleState::queue_move(&mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::B1, &sleep_powder, vec![BattlePosition::B1.get_ally()]);

    root_bc.sim_next_action(); // resolve move: redirect to F1 and branch on accuracy

    assert_eq!(root_bc.battle_ctns.len(), 2,
        "Sleep Powder's accuracy should split the state into a miss and a hit container");

    let hit_bc = &mut root_bc.battle_ctns[1];
    assert_eq!(hit_bc.pct_chance, PkmnRational::from_float(sleep_powder.accuracy));

    hit_bc.sim_next_action(); // apply the redirected sleep status effect

    let hit_bs = hit_bc.battle_state.as_ref().unwrap();
    assert_eq!(hit_bs.get_active(BattlePosition::F1).unwrap().status, PokemonStatus::SLEEP,
        "Sleep Powder should be redirected onto the center-of-attention Pokemon (F1)");
    assert_eq!(hit_bs.get_active(BattlePosition::B1).unwrap().status, PokemonStatus::NONE,
        "Sleep Powder's original ally target should be unaffected");
}

#[test]
/// Wide Guard should mark the field, block a multi-target opponent move (Heat Wave)
/// but let a single-target move (Dragon Claw) through to deal damage
fn test_wide_guard_blocks_multi_target_move_but_not_single_target() {
    let mut root_bc = dummy_bc(); // Tyranitar (F1) vs Venusaur (B1)

    let wide_guard = get_move(PokemonMoveName::Wide_Guard);
    BattleState::queue_move(&mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::F1, &wide_guard, vec![BattlePosition::F1, BattlePosition::F2]);

    root_bc.sim_next_action(); // resolve move, queue the WIDE_GUARD field effect
    root_bc.sim_next_action(); // apply the field effect

    let bs_after_wide_guard = root_bc.battle_state.as_ref()
        .expect("Wide Guard should not branch into multiple states");
    assert!(bs_after_wide_guard.effects.has_flag(PokemonFieldState::WIDE_GUARD),
        "Field effects should have the WIDE_GUARD flag set");

    // Opponent uses Heat Wave, a multi-target move, which should be blocked
    let heat_wave = get_move(Heat_Wave);
    let starting_hp = bs_after_wide_guard.get_active(BattlePosition::F1).unwrap().current_hp;
    BattleState::queue_move(&mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::B1, &heat_wave, vec![BattlePosition::F1]);

    root_bc.sim_next_action();

    assert!(root_bc.message.contains("blocked by Wide Guard"),
        "Heat Wave should be reported as blocked by Wide Guard");
    let bs_after_heat_wave = root_bc.battle_state.as_ref().unwrap();
    assert_eq!(bs_after_heat_wave.get_active(BattlePosition::F1).unwrap().current_hp, starting_hp,
        "Heat Wave should deal no damage while blocked by Wide Guard");
    assert!(bs_after_heat_wave.action_queue.is_empty(),
        "Blocked Heat Wave should not queue any further actions like Damage");

    // Opponent uses Dragon Claw, a single-target move, which should not be blocked
    let dragon_claw = get_move(PokemonMoveName::Dragon_Claw);
    BattleState::queue_move(&mut root_bc.battle_state.as_mut().unwrap().action_queue,
        BattlePosition::B1, &dragon_claw, vec![BattlePosition::F1]);

    root_bc.sim_next_action();

    let bs_after_dragon_claw = root_bc.battle_state.as_ref().unwrap();
    assert!(bs_after_dragon_claw.action_queue.iter().any(|action|
        matches!(action, BattleAction::Damage(dmg_effect) if dmg_effect.target == BattlePosition::F1)),
        "Dragon Claw should queue a Damage effect since Wide Guard doesn't block single-target moves");
}