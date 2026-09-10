# Readme

## What is this solver?
This is a project designed to solve a battle in Pokemon Champions, by simulating all possible battle states and creating percent chance of any actions occurring.
This goes above a typical Pokemon damage calculator which simulates a single move and simulates multiple actions.

## How does this work?
This is still in heavy development, but imagine solving a game of tic-tac-toe or Connect 4 by simulating every possible move each player can do. This is an optimized version of this with a Pokemon battle engine in Rust that keeps track of random events.

## Goal
This intends to help discover optimal 100% win paths and discover new teams, counter strategies and optimal play for Pokemon competitive players.

Eventually this will also include retrospection (guessing spreads from damage calculations) and optimization (simulating games against virtual opponents) to make plans and better teams.

# Current State

## Battle
- [x] Pokemon Natures
- [x] Pokemon Stats
- [-] Damage Calculation (inaccurate for lower-bound)
- [ ] Abilities
- [ ] Items
- [ ] Speed order
- [-] Basic Moves
- [ ] Moves with unique effects
- [ ] Switch pokemon
- [ ] Team mechanics (things that interact with teams)
- [ ] Mega evolution
- [ ] Terassilation (dunno spelling bleh)

## Meta Solver
- [ ] Move/Basic AI simulation
- [ ] Stat Optimization
- [ ] Meta simulation/adapation
- [ ] Generational AI algo to build/optimize teams


# Future work

The battle engine is still being built. Currently working on a very small subset of moves/pokemon/abilities to test the engine.
After speed/team states are implemented, a full simulation of all pokemon leads/states will be simulated, then continuing to add more pokemon, moves, abilities until meeting current parity.