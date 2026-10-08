# Battle State Ownership Flow

## Current flow

The simulation currently follows this path:

```text
BattleProcessor::process_all_states_by_one_turn
    -> BattleContainer::sim_next_action(&mut self)
        -> self.battle_state.as_mut()
        -> BattleState::sim_action(&mut self, action)
            -> sim_* methods
            -> clone BattleState into BattleContainer values
        -> replace self.battle_state with returned state(s)
```

The key ownership problem is in `BattleContainer::sim_next_action`. It
temporarily borrows the contained state with `as_mut()`, so `sim_action`
cannot move that state into a `BattleContainer`. It must clone it instead.

## Recommended ownership change

Keep `BattleContainer::sim_next_action(&mut self)` unchanged at the public
API level. This preserves existing callers and recursive processing of nested
containers.

Temporarily take ownership of the state using `Option::take()`:

```rust
let Some(mut b_state) = self.battle_state.take() else {
    for battle_ctn in &mut self.battle_ctns {
        battle_ctn.sim_next_action();
    }
    return;
};

let action = match b_state.action_queue.pop_front() {
    Some(action) => action,
    None => {
        self.battle_state = Some(b_state);
        return;
    }
};

b_state.action_num += 1;
let new_bcs = b_state.sim_action(action);
```

`Option::take()` replaces `self.battle_state` with `None` and returns the
original `BattleState` by value. This removes the borrow and allows
`sim_action` to own the state.

The result should then be transferred back into the container:

```rust
match new_bcs.len() {
    0 => {
        self.message = "No state change occurred from action*, review".to_string();
    }
    1 => {
        let mut bc = new_bcs.into_iter().next().unwrap();
        self.battle_state = bc.battle_state.take();
        self.message = bc.message;
        self.name = bc.name;
    }
    _ => {
        self.battle_ctns = new_bcs;
        self.battle_state = None;
    }
}
```

## Change `sim_action` to consume the state

Change the signature from:

```rust
pub fn sim_action(
    &mut self,
    mut action: BattleAction,
) -> Vec<BattleContainer<'battle, 'simulation>>
```

to:

```rust
pub fn sim_action(
    mut self,
    action: BattleAction,
) -> Vec<BattleContainer<'battle, 'simulation>>
```

Deterministic actions can then return the owned state directly instead of
cloning it. For example, healing should use an owning helper:

```rust
fn sim_healing(
    mut self,
    heal_effect: HealEffect,
) -> Vec<BattleContainer<'battle, 'simulation>> {
    let target_pkmn = self.get_active_mut(heal_effect.target).unwrap();

    let total_hp = target_pkmn.get_active_stat(HEALTH);
    let healing_done = heal_effect
        .calc_healing
        .min(total_hp - target_pkmn.current_hp);

    target_pkmn.current_hp += healing_done;
    let heal_msg = format!("{target_pkmn} healed {healing_done}!");

    vec![BattleContainer::one(self, Some(heal_msg))]
}
```

The same direct ownership approach applies to single-result actions such as:

- failed moves;
- `SendOut`;
- `Return`;
- setting weather;
- other deterministic actions that currently call `self.clone()`.

## Cloning is still required for branching

Not every clone should be removed. Actions that produce multiple independent
outcomes still need separate states, including:

- status accuracy branches;
- stat-change branches;
- protect branches;
- damage branches;
- move accuracy branches;
- `spawn_bc_for_power_set`;
- `spawn_bc_for_single_prob`.

Those functions use one state as a base and clone it for each possible result.
That is appropriate unless the branching model is later redesigned to consume
one state and clone only the additional branches.

## Why `BattleContainer::sim_next_action` should not consume `self`

Changing `sim_next_action` to take ownership of the entire container would
break the current usage pattern:

```rust
root_bc.sim_next_action();
root_bc.sim_next_action();
```

It would also make recursive processing of `battle_ctns` more cumbersome.
The surgical design is therefore:

```text
BattleContainer::sim_next_action(&mut self)
    -> self.battle_state.take()
    -> BattleState::sim_action(self, action)
    -> move deterministic state into BattleContainer
    -> retain cloning only for branching outcomes
```

This preserves the current container API while allowing the healing path and
other deterministic paths to transfer ownership without cloning.
