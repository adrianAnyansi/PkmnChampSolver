use crate::battle::battle_processor::BattleContainer;
use crate::battle::data::{ActivePokemon, TrainedPokemon};
use crate::battle::{BattleAction, FieldPosition, BattleState};
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
