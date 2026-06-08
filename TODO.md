# TODO

Here's the TODO list

## Goal
So the goal is this
**Input**:
    A bunch of Pokemon+stats, items, mechanics etc.
**Output:**
    A bunch of teams that have a high win-rate and the routes taken.
    Probably sent to file

I'm aiming to meet the smaller goal of sending 2 teams and getting all permutations


## TODO
I want to finish the current poke abilities and moves to flesh out how scripting will work
Will likely have an immediate event queue for things

### Abilities
Sand-Stream setting up weather
Rough Skin doing additional damage
Add Basculegion for Adapatability, Swift Swim & Mold Breaker?
Add Sneasler for Unburden

### Moves
Secondary effects
- Flinch
- Stat changes
- Priority changes (Fake Out, Aqua Jet)
- Hyper Beam/secondary turn stuff
- Status changes
- Protect
- Sandstorm (pure status move)


- Active Pokemon should combine the trained/base stats to 1 object to be easy
- Moves json, and start working on scripting
- MoveResults thought
    - Faint
    - Damage
- Start thinking about specific triggers for abilities and etc

## Today



## Tasks

1. I want to read a file and output a Pokemon struct that represents a pokemon accurately
2. I want to put 2 pokemon into a battle simulator, and get the ranges of every move used against each pokemon (ignoring speed/etc)
3. I want to save the state to a hash properly & quickly
4. Calculate the battle states


## Spread
One thing I'm trying to architect is that I need to spread the moves/natures/etc during the battle, but I want the simple Pokemon so the actual simulator is using 1 thing-

Ok lets use an example to think:

I'm calcing a Garchomp Draco Meteor
    - The Move has a damage roll
    - The Nature affects the damage roll
    - EVs affect damage roll
    - All this applies to Enemy as well
    - The user could swap to another type
    - User could protect or etc
    - Ability changes

I see there are damage roll stuff- but the others basically change the state before the damage roll occurs.
So the spread contains
    - Damage roll min-max
    - Nature & EV min-max on the stat

These are separate things (damage & stat min-max)
The spread for non-math stat is too fixed the state to work in this way.
I'll need to build a range class/struct that math can verigy and return