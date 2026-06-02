# TODO

Here's the TODO list

## Goal
So the goal is this
**Input**:
    A bunch of Pokemon+stats, items, mechanics etc.
**Output:**
    A bunch of teams that have a high win-rate and the routes taken.

But a smaller goal is
**Input:**
    2-6 pokemon on 2 teams

I'm going to take the idea of getting this Pokemon as an input
Then I need to build the battle simulator which will be the hard bit


## Today
- Create a Garchomp with stats, no item, with Draco Meteor
- 2nd pokemon will be Bisharp
- Create a battle stat, then calculate the battle-event of attacking
- Output the pokemon state

For like Sand Force, it needs to check the battle state. So smth like:
1. Garchomp does Draco Meteor
2. Ability is checked, has a move modifier
3. Ability checks the Battle state for Weather
4. Ability modifies the move strength
5. Move continues the stuff

The battle logic is gonna be complicated, so I'm going to build this step by step. I think I'll group abilities & traits to understand what affects what.


Expanding the current test case:
1. Add Draco to the action queue-
2. Pop the action queue, evaluate the move
3. Check all conditions that will prevent the move from occurring (choice, encore, ability block)
4. Check modifications to the move, target, power, etc.

I want to get a list of moves so I have a generic case somewhat.
It's frustating



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