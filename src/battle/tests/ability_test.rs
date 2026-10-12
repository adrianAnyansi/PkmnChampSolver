use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::{ActivePokemon, TrainedPokemon};
use crate::battle::{BattleAction, BattlePctAction, BattleState, BattleTeamSide, FieldPosition, MoveAction, SpeedBattleAction};
use crate::battle::data::BattleWeatherState;
use crate::math::{mult_and_round, PkmnRational};
use crate::pokemon::abilities::{get_ability, PokemonAbility, PokemonAbilityName};
use crate::pokemon::moves::{get_move, get_move_priority, DamageAmount, MoveEffect, PokemonMoveName};
use crate::pokemon::poke_stat::{PokemonNature, PokemonStatName};
use crate::pokemon::types::PokemonType;
use crate::pokemon::{get_pkmn, Pokemon, PokemonName};
use std::sync::LazyLock;

fn raw_damage(amount: DamageAmount) -> u32 {
    match amount {
        DamageAmount::RawDamage(damage) => damage,
        other => panic!("expected raw damage, got {other:?}"),
    }
}

static NEUTRAL_POKEMON: LazyLock<Pokemon> = LazyLock::new(|| Pokemon {
    name: PokemonName::Venusaur,
    base_stats: get_pkmn(PokemonName::Venusaur).base_stats.clone(),
    abilities: vec![],
    learnset: vec![],
    weight: 100.0,
    types: vec![PokemonType::TYPELESS],
});

/// Typeless target; the caller owns the TrainedPokemon so ActivePokemon can borrow it
fn neutral_trained() -> TrainedPokemon<'static> {
    TrainedPokemon::new(
        &NEUTRAL_POKEMON,
        get_ability(PokemonAbilityName::Nothing),
        PokemonNature::Quirky,
        None,
    )
}

#[test]
fn test_blaze_boosts_second_fire_move_below_one_third_hp() {
    let trained_target = neutral_trained();
    let target = ActivePokemon::new(&trained_target);
    let blaze = get_ability(PokemonAbilityName::Blaze);
    let trained_charizard = TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        blaze,
        PokemonNature::Quirky,
        None,
    );
    let charizard = ActivePokemon::new(&trained_charizard);
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
        BattleAction::Damage(effect) => raw_damage(effect.dmg_amt),
        action => panic!("expected first move to queue damage, got {action}"),
    };

    battle.sim_next_action();
    let state = battle.battle_state.as_mut().unwrap();
    let charizard = state.get_active_mut(FieldPosition::B1).unwrap();
    let max_hp = u32::try_from(
        charizard.get_active_stat(crate::pokemon::poke_stat::PokemonStatName::HEALTH),
    )
    .expect("max HP should be nonnegative");
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
        BattleAction::Damage(effect) => raw_damage(effect.dmg_amt),
        action => panic!("expected second move to queue damage, got {action}"),
    };

    assert_eq!(
        i32::try_from(second_damage).expect("damage should fit in i32"),
        mult_and_round(
            i32::try_from(first_damage).expect("damage should fit in i32"),
            1.5,
        )
    );
}


#[test]
/// Back Pokemon with Intimidate should queue an Attack drop sourced from B1 that targets the front Pokemon
fn test_exec_set_team_intimidate_queued_in_speed_queue() {
    use crate::pokemon::abilities::{get_ability, PokemonAbility, PokemonAbilityName};
    use crate::pokemon::poke_stat::PokemonNature;

    let nothing = get_ability(PokemonAbilityName::Nothing);
    let intimidate = get_ability(PokemonAbilityName::Intimidate);
    let front = TrainedPokemon::new(get_pkmn(PokemonName::Tyranitar), nothing, PokemonNature::Quirky, None);
    let back = TrainedPokemon::new(get_pkmn(PokemonName::Charizard), intimidate, PokemonNature::Quirky, None);

    let mut bs = BattleState::new();
    bs.exec_set_team(BattleTeamSide::FRONT, &vec![&front]);
    bs.exec_set_team(BattleTeamSide::BACK, &vec![&back]);

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
    let max_hp = u32::try_from(
        b1.get_active_stat(crate::pokemon::poke_stat::PokemonStatName::HEALTH),
    )
    .expect("max HP should be nonnegative");
    b1.current_hp = max_hp / 2;

    let hospitality = get_ability(PokemonAbilityName::Hospitality);
    let trained_pokemon = TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        hospitality,
        PokemonNature::Quirky,
        None,
    );
    let hospitality_idx = battle_state
        .b_team
        .add_poke(ActivePokemon::new(&trained_pokemon));
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
fn test_drought_sets_sun_on_send_out() {
    let drought = get_ability(PokemonAbilityName::Drought);
    let trained_pokemon = TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        drought,
        PokemonNature::Quirky,
        None,
    );

    let mut battle_state = BattleState::new();
    let idx = battle_state.b_team.add_poke(ActivePokemon::new(&trained_pokemon));
    assert_eq!(battle_state.weather, BattleWeatherState::NONE);

    battle_state.exec_send_out(FieldPosition::B1, idx, None);

    let set_weather = battle_state
        .speed_queue
        .iter()
        .find_map(|speed_action| match &speed_action.battle_action {
            BattleAction::Generic(MoveEffect::SetWeather(weather), _) => {
                Some(speed_action.battle_action.clone()).filter(|_| *weather == BattleWeatherState::SUN)
            }
            _ => None,
        })
        .expect("Drought should queue a SUN SetWeather action");

    battle_state.sim_action(set_weather);
    assert_eq!(battle_state.weather, BattleWeatherState::SUN);
}

#[test]
fn test_sand_stream_sets_sandstorm_on_send_out() {
    let sand_stream = get_ability(PokemonAbilityName::Sand_Stream);
    let trained_pokemon = TrainedPokemon::new(
        get_pkmn(PokemonName::Tyranitar),
        sand_stream,
        PokemonNature::Quirky,
        None,
    );

    let mut battle_state = BattleState::new();
    let idx = battle_state.b_team.add_poke(ActivePokemon::new(&trained_pokemon));
    assert_eq!(battle_state.weather, BattleWeatherState::NONE);

    battle_state.exec_send_out(FieldPosition::B1, idx, None);

    let set_weather = battle_state
        .speed_queue
        .iter()
        .find_map(|speed_action| match &speed_action.battle_action {
            BattleAction::Generic(MoveEffect::SetWeather(BattleWeatherState::SANDSTORM), _) => {
                Some(speed_action.battle_action.clone())
            }
            _ => None,
        })
        .expect("Sand Stream should queue a SANDSTORM SetWeather action");

    battle_state.sim_action(set_weather);
    assert_eq!(battle_state.weather, BattleWeatherState::SANDSTORM);
}

#[test]
fn test_chlorophyll_reorders_speed_queue_after_sun_weather() {
    let nothing = get_ability(PokemonAbilityName::Nothing);
    let chlorophyll = get_ability(PokemonAbilityName::Chlorophyll);

    let charizard = TrainedPokemon::new(
        get_pkmn(PokemonName::Charizard),
        nothing,
        PokemonNature::Quirky,
        None,
    );
    let venusaur = TrainedPokemon::new(
        get_pkmn(PokemonName::Venusaur),
        chlorophyll,
        PokemonNature::Quirky,
        None,
    );

    let mut battle_state = BattleState::simple(
        ActivePokemon::new(&charizard),
        ActivePokemon::new(&venusaur),
    );

    let charizard_move = get_move(PokemonMoveName::Flare_Blitz);
    let venusaur_move = get_move(PokemonMoveName::Solar_Beam);
    let charizard_idx = battle_state.get_active_team_idx(FieldPosition::F1).unwrap();
    let venusaur_idx = battle_state.get_active_team_idx(FieldPosition::B1).unwrap();

    battle_state.speed_queue.push_back(SpeedBattleAction {
        priority: get_move_priority(&charizard_move),
        team_index: charizard_idx,
        battle_action: BattleAction::Move(MoveAction {
            source: FieldPosition::F1,
            targets: vec![FieldPosition::B1],
            pkm_move: &charizard_move,
        }),
    });
    battle_state.speed_queue.push_back(SpeedBattleAction {
        priority: get_move_priority(&venusaur_move),
        team_index: venusaur_idx,
        battle_action: BattleAction::Move(MoveAction {
            source: FieldPosition::B1,
            targets: vec![FieldPosition::F1],
            pkm_move: &venusaur_move,
        }),
    });

    battle_state.sort_speed_queue();
    assert!(battle_state.speed_queue.front().unwrap().team_index == charizard_idx,
        "Charizard should move first while the weather is normal");

    battle_state.exec_set_weather(BattleWeatherState::SUN, false);
    let weather_flag = battle_state
        .action_queue
        .iter()
        .find(|action| matches!(action, BattleAction::Generic(MoveEffect::SpeedChangeFlag, _)))
        .expect("setting sun should create a speed-change flag action");
    assert!(matches!(weather_flag, BattleAction::Generic(MoveEffect::SpeedChangeFlag, _)));

    let consumed = battle_state
        .sim_action(weather_flag.clone())
        .into_iter()
        .next()
        .expect("speed change flag should be consumed as an action");
    assert!(consumed.battle_state.is_some());
    assert!(battle_state.speed_dirty_flag,
        "sun weather should mark the speed queue dirty after the flag is consumed");
    assert!(
        battle_state.get_active_pokemon_speed(venusaur_idx) ==
            (battle_state.get_active(FieldPosition::B1).unwrap().get_active_stat(PokemonStatName::SPEED) as u32 * 2),
        "Chlorophyll doubles Venusaur's speed in Sun"
    );

    battle_state.sort_speed_queue();
    assert!(battle_state.speed_queue.front().unwrap().team_index == venusaur_idx,
        "Venusaur should move first after its Chlorophyll speed boost in Sun");
}

#[test]
fn test_rough_skin_damages_attacker_one_eighth_after_move_damage() {
    use crate::battle::DamageSource;
    use crate::pokemon::poke_stat::PokemonStatName;

    let rough_skin = get_ability(PokemonAbilityName::Rough_Skin);
    let trained_garchomp = TrainedPokemon::new(
        get_pkmn(PokemonName::Garchomp),
        rough_skin,
        PokemonNature::Quirky,
        None,
    );
    let garchomp = ActivePokemon::new(&trained_garchomp);
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

    let attacker_max_hp = u32::try_from(
        battle_state
            .get_active(FieldPosition::F1)
            .unwrap()
            .get_active_stat(PokemonStatName::HEALTH),
    )
    .expect("max HP should be nonnegative");
    let mut battle = BattleContainer::simple(battle_state, PkmnRational::ONE());

    // Simulating the move queues the move damage
    battle.sim_next_action();
    let move_damage = match battle.battle_state.as_ref().unwrap().action_queue.front().unwrap() {
        BattleAction::Damage(effect) => {
            assert_eq!(effect.target, FieldPosition::B1);
            assert!(matches!(effect.damage_source, DamageSource::Move(_)));
            raw_damage(effect.dmg_amt)
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
    assert!(matches!(
        ability_damage.dmg_amt,
        DamageAmount::HealthPct(pct) if pct == PkmnRational::new(1, 8)
    ));
    let expected_ability_damage = attacker_max_hp / 8;

    // Applying it reduces the attacker's HP by 1/8 of its max HP
    battle.sim_next_action();
    let attacker = battle.battle_state.as_ref().unwrap().get_active(FieldPosition::F1).unwrap();
    assert_eq!(attacker.current_hp, attacker_max_hp - expected_ability_damage);
}
