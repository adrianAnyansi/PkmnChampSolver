use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::{ActivePokemon, TrainedPokemon};
use crate::battle::{BattleAction, BattlePctAction, BattleState, BattleTeamSide, FieldPosition};
use crate::math::{mult_and_round, PkmnRational};
use crate::pokemon::abilities::{make_ability, PokemonAbilityName};
use crate::pokemon::moves::{get_move, PokemonMoveName};
use crate::pokemon::poke_stat::PokemonNature;
use crate::pokemon::types::PokemonType;
use crate::pokemon::{get_pkmn, Pokemon, PokemonName};

fn neutral_target() -> ActivePokemon<'static, 'static> {
    let pokemon = Box::leak(Box::new(Pokemon {
        name: PokemonName::Venusaur,
        base_stats: get_pkmn(PokemonName::Venusaur).base_stats.clone(),
        abilities: vec![],
        learnset: vec![],
        weight: 100.0,
        types: vec![PokemonType::TYPELESS],
    }));
    let ability = Box::leak(Box::new(make_ability(PokemonAbilityName::Nothing)));
    let trained_pokemon = Box::leak(Box::new(TrainedPokemon::new(
        pokemon,
        ability,
        PokemonNature::Quirky,
        None,
    )));

    ActivePokemon::new(trained_pokemon)
}

#[test]
fn test_blaze_boosts_second_fire_move_below_one_third_hp() {
    let target = neutral_target();
    let blaze = Box::leak(Box::new(make_ability(PokemonAbilityName::Blaze)));
    let trained_charizard = Box::leak(Box::new(TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        blaze,
        PokemonNature::Quirky,
        None,
    )));
    let charizard = ActivePokemon::new(trained_charizard);
    let mut battle_state = BattleState::simple(target, charizard);

    let mut fire_move = get_move(PokemonMoveName::Heat_Wave);
    fire_move.accuracy = 1.0;
    fire_move.hit_actions.clear();

    BattleState::queue_move(
        &mut battle_state.action_queue,
        FieldPosition::B1,
        &fire_move,
        vec![FieldPosition::F1],
    );
    BattleState::queue_move(
        &mut battle_state.action_queue,
        FieldPosition::B1,
        &fire_move,
        vec![FieldPosition::F1],
    );

    let mut battle = BattleContainer::simple(battle_state, PkmnRational::ONE());

    battle.sim_next_action();
    let first_damage = match battle
        .battle_state
        .as_ref()
        .unwrap()
        .action_queue
        .front()
        .unwrap()
    {
        BattleAction::Damage(effect) => effect.calc_damage,
        action => panic!("expected first move to queue damage, got {action}"),
    };

    battle.sim_next_action();
    let state = battle.battle_state.as_mut().unwrap();
    let charizard = state.get_active_mut(FieldPosition::B1).unwrap();
    let max_hp = charizard.get_active_stat(crate::pokemon::poke_stat::PokemonStatName::HEALTH);
    charizard.current_hp = max_hp / 3 - 1;
    assert!(charizard.current_hp * 3 < max_hp);

    battle.sim_next_action();
    let second_damage = match battle
        .battle_state
        .as_ref()
        .unwrap()
        .action_queue
        .front()
        .unwrap()
    {
        BattleAction::Damage(effect) => effect.calc_damage,
        action => panic!("expected second move to queue damage, got {action}"),
    };

    assert_eq!(second_damage, mult_and_round(first_damage, 1.5));
}


#[test]
/// Back Pokemon with Intimidate should queue an Attack drop sourced from B1 that targets the front Pokemon
fn test_exec_set_team_intimidate_queued_in_speed_queue() {
    use crate::pokemon::abilities::{make_ability, PokemonAbility, PokemonAbilityName};
    use crate::pokemon::poke_stat::PokemonNature;

    let nothing: &'static PokemonAbility<'static> =
        Box::leak(Box::new(make_ability(PokemonAbilityName::Nothing)));
    let intimidate: &'static PokemonAbility<'static> =
        Box::leak(Box::new(make_ability(PokemonAbilityName::Intimidate)));
    let front: &'static TrainedPokemon<'static, 'static> = Box::leak(Box::new(
        TrainedPokemon::new(get_pkmn(PokemonName::Tyranitar), nothing, PokemonNature::Quirky, None)));
    let back: &'static TrainedPokemon<'static, 'static> = Box::leak(Box::new(
        TrainedPokemon::new(get_pkmn(PokemonName::Charizard), intimidate, PokemonNature::Quirky, None)));

    let mut bs = BattleState::new();
    bs.exec_set_team(BattleTeamSide::FRONT, &vec![front]);
    bs.exec_set_team(BattleTeamSide::BACK, &vec![back]);

    assert_eq!(bs.speed_queue.len(), 2, "Intimidate queued for both opponents");
    let queued = bs.speed_queue.front().unwrap();

    assert!(queued.team_index.team_side == BattleTeamSide::BACK);
    assert_eq!(queued.team_index.index, 0);

    let BattleAction::PctActions(BattlePctAction::Stat(_), targets, _) = &queued.battle_action
        else { panic!("expected a stat PctActions from Intimidate") };
    let targets: Vec<FieldPosition> = targets.iter().flatten().copied().collect();
    assert_eq!(targets, vec![FieldPosition::F1], "Intimidate should target the opposing front Pokemon");
}


#[test]
fn test_hospitality_heals_ally_one_eighth_on_send_out() {
    let mut battle_state = BattleState::new();
    let front_idx = battle_state.f_team.add_poke(ActivePokemon::quick(PokemonName::Tyranitar));
    let back_idx = battle_state.b_team.add_poke(ActivePokemon::quick(PokemonName::Venusaur));
    battle_state.exec_send_out(FieldPosition::F1, front_idx, None);
    battle_state.exec_send_out(FieldPosition::B1, back_idx, None);

    let b1 = battle_state.get_active_mut(FieldPosition::B1).unwrap();
    let max_hp = b1.get_active_stat(crate::pokemon::poke_stat::PokemonStatName::HEALTH);
    b1.current_hp = max_hp / 2;

    let hospitality = Box::leak(Box::new(make_ability(PokemonAbilityName::Hospitality)));
    let trained_pokemon = Box::leak(Box::new(TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        hospitality,
        PokemonNature::Quirky,
        None,
    )));
    let hospitality_idx = battle_state
        .b_team
        .add_poke(ActivePokemon::new(trained_pokemon));
    battle_state.exec_send_out(FieldPosition::B2, hospitality_idx, None);

    let heal = battle_state
        .speed_queue
        .iter()
        .find_map(|speed_action| match &speed_action.battle_action {
            BattleAction::Heal(heal) => Some(*heal),
            _ => None,
        })
        .expect("Hospitality should queue a healing action");

    assert_eq!(heal.target, FieldPosition::B1);
    assert_eq!(heal.calc_healing, max_hp / 8);
}

#[test]
fn test_rough_skin_damages_attacker_one_eighth_after_move_damage() {
    use crate::battle::DamageSource;
    use crate::pokemon::poke_stat::PokemonStatName;

    let rough_skin = Box::leak(Box::new(make_ability(PokemonAbilityName::Rough_Skin)));
    let trained_garchomp = Box::leak(Box::new(TrainedPokemon::new(
        get_pkmn(PokemonName::Garchomp),
        rough_skin,
        PokemonNature::Quirky,
        None,
    )));
    let garchomp = ActivePokemon::new(trained_garchomp);
    let tyranitar = ActivePokemon::quick(PokemonName::Tyranitar);
    let mut battle_state = BattleState::simple(tyranitar, garchomp);

    let mut attack = get_move(PokemonMoveName::Dragon_Claw);
    attack.accuracy = 1.0;
    attack.hit_actions.clear();
    BattleState::queue_move(
        &mut battle_state.action_queue,
        FieldPosition::F1,
        &attack,
        vec![FieldPosition::B1],
    );

    let attacker_max_hp = battle_state
        .get_active(FieldPosition::F1)
        .unwrap()
        .get_active_stat(PokemonStatName::HEALTH);
    let mut battle = BattleContainer::simple(battle_state, PkmnRational::ONE());

    // Simulating the move queues the move damage
    battle.sim_next_action();
    let move_damage = match battle.battle_state.as_ref().unwrap().action_queue.front().unwrap() {
        BattleAction::Damage(effect) => {
            assert_eq!(effect.target, FieldPosition::B1);
            assert!(matches!(effect.damage_source, DamageSource::Move(_)));
            effect.calc_damage
        }
        action => panic!("expected move damage, got {action}"),
    };
    assert!(move_damage > 0);

    // Applying the move damage queues Rough Skin damage on the attacker
    battle.sim_next_action();
    let state = battle.battle_state.as_ref().unwrap();
    let ability_damage = state
        .action_queue
        .iter()
        .find_map(|action| match action {
            BattleAction::Damage(effect)
                if matches!(effect.damage_source, DamageSource::Ability(PokemonAbilityName::Rough_Skin)) =>
            {
                Some(effect.clone())
            }
            _ => None,
        })
        .expect("Rough Skin should queue damage");

    assert_eq!(ability_damage.target, FieldPosition::F1);
    assert_eq!(ability_damage.calc_damage, mult_and_round(attacker_max_hp, 0.125));

    // Applying it reduces the attacker's HP by 1/8 of its max HP
    battle.sim_next_action();
    let attacker = battle.battle_state.as_ref().unwrap().get_active(FieldPosition::F1).unwrap();
    assert_eq!(attacker.current_hp, attacker_max_hp - ability_damage.calc_damage);
}
