# General Overview

The goal of this project is simple
**Input**
    A bunch of Pokemon, Mechanics, everything required to define a battle, tournament
**Output**
    A bunch of teams with high win-rate and routes taken in a file format.

This cretes a general purpose solver for Pokemon, hopefully, or at least something that can do higher level analysis than a damage calculator.

# Ranges
Most things in Pokemon and randomness in number generation, so I need a special class that contains the low-high range numbers.
This will auto-split to other universes whenever any consequence (fainting, ability/item trigger) occurs in <100% of the range.

Similarily this works for stats, which then extends to how natures are calculated. 

Due to speed & damage calcs work, I need some way to retrospec certain values and lock certain things in, for moves/abilities/natures. That needs to be a separate system.
Call it Single Stat/Ability Resolver


# Multi-universe simulations
In order to explore all interactions, I need to create a separate state for each action/effect. 

Here's how that will work-
[BattleState, pct-chance]
Any time the state would split, duplicate states are made with the pct-chance of that action. For that reason, all actions (move resolving, chance resolving, etc) returns a list of BattleStates, or rather a list of BattleContainers where the state can nest.

If I just want to simulate a battle, the in-between resolver will roll the pct chances and resolve to just 1. For full-flow diagrams, each and every state gets simulated separately.

In cases where there are multiple indepedent events (usually on multiple targets), it makes sense to spread (do multiple at once in an action) rather than in-sequence.

Therefore I'm changing the logic for non-fail cases* to just return a battlestate instead of containers 
