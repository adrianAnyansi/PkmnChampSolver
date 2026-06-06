# Moves descriptions

Future thought- make all pokemon evaluate if they can submit an attack or change they move/action to be automatic (certain attacks/checks) still occur during the turn order

Ok lets do the example again again

- Add Draco Meteor with target

## Simulation thoughts
So here's some things I'm considering...
For programming reasons any probablastic effect needs to be sent up- basically pulled out of the battle state completely.
There are obvious shorthand - % miss rolls become split, damage rolls.
But what about statuses, basically a generic splitter needs to be envisioned. I'm gonna do that later though- it requires thought
But remember that probablistic stuff needs to be returned

I thought bout it on a drive
- return either a vec of battlestates or the range for the battlestate internally instead of modifiying the root one
- wait im gonna have to hash the state anyways, damn


## Primary
- Generic moves; type + power + accuracy
- Status moves
- Move ignores accuracy check
    - Accuracy depends on weather/battle
- Move has increased crit ratio
- Increased priority
- Protect status with addt effect
    - Reduce stats or perform damage (Baneful Bunker/Spiky Shield)
- Power/Hits depends changes based on condition
    - Acrobatics (not holding an item, static battle state)
    - If a boost occurred this turn (history)
    - User took damage/moved last
    - on party/team/fainted team
    - depends on Defense (separate stat)
- Based on History
    - Did the pokemon eat a berry this battle
- Multiple attacks
- Two-Turn moves
- Ignores stat changes

## Secondary Effects
- Stat changes (boost and drops)
- Causes a flinch 30% (affects turn order)
- Inflicts a permanent status
- Changes field status (shields or terrain)
- Recharging
- Recoil / Recovery
- Affects Held Item
- Changes/Affects user type
- Swaps Pokemon

## Special
- Move changes turn order (After You)
- Swap pokemon (ally switch)
- Checks if Pokemon is performing a move (before completing)
- Baton Pass
- Lost 1/2 hp and boost stats (does this count as self damage?)
- Mimics a move*
- Counters a move
- Curse (based on user Type)
- Destiny bond

## Latent Effect
- Fist
- Sound
- Ignores Protect/Status/more