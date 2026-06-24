# TODO

Here's the TODO list

Also I'm thinking of shelving this project since it's getting pretty big in scope- I don't quite know how it's possible it will be
I've been thinking about the optimal plan for an actual battle- and that's also very difficult to understand

I also didn't figure out how to do funciton pointers- I just need to link the enum to the pointer and it SUCKSSSSSSSSSSS


## Goal
So the goal is this
**Input**:
    A bunch of Pokemon+stats, items, mechanics etc.
**Output:**
    A bunch of teams that have a high win-rate and the routes taken.
    Probably sent to file

I'm aiming to meet the smaller goal of sending 2 teams and getting all permutations

## The multi-universe problem
In order to explore all possible interactions, I need to explore all leads & moves & interactions from battle. This is a hard problem.
My solution was that any func that creates outcomes (damage effect, etc) will entire contain a variable amount of damage or effects.
### Actually doing this
I don't want to implement this until the core engine is done, but I need the functions to be possible
### Choosing
Its kind of simple to choose a pokemon/move/swap/etc and create a universe for each.
### Effects
For effects, I need to return all effects/ranges to the queue and the battle engine needs to know each one. Lets give some examples
- Move can status, miss, roll, kill, stat, etc.
    Each effect should come with a % chance of occuring, but when there are multiple outcomes how
    - Move can miss, return damage acc%. Includes the range of damage
        - This wont work for multi-hit that fall-out 2nd effects... maybe. If I process dmg effects each, it could work
    - 2nd effects need to be linked to hits, thats hard to know

It might be better to go piece by piece cause I can't figure out the entire sequence of information required
I should target the smaller issue of team vs team full path instead of the netire metagame search- so leave this for later

## General
Here's the thing. I do want to get all ranges when it comes to doing calculations. Currently I just have performing 1 move, but performing the entire battle will require all the ranges & the branching- which I straight up don't want to do.
It's hard enough to consider the ability/item/move resolving right not.
My plan is to just have some random moves trigger, and then return the result

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
Do the 2nd effect of sp_atk drop on hit
Then Impl Kowtow Cleave, accuracy modifier and the Rough Skin calculation



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