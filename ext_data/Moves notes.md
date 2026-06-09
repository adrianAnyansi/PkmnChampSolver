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

## Types
- Generic moves; type + power + accuracy
- Status moves (type + accuracy but no power)
- Special (move does a lot of things)

I plan to have no difference between an effect occurring, so I'm going to break up certain categories for easier reading

Last check was Hex

## Conditions
- Fails if not first move
- Fails if not <Pokemon>

## Move Effects
- Multi-hit move
    - Dragon Darts with smart targeting
- Multi-turn
    - Dive/Fly/Dig
    - Hyper Beam/Electro Shot
- Protect Status (1/3 on 2nd consceutive attempt, fails on 3rd)
    - Addt effect if hit*
- Power changes
    - Acrobatcis (no item)
    - History
        - Boost occurred
        - User took damage/moved last
        - party/team state
    - Based on non-standard stats/health
### Condition
- Was berry eaten this battle
### Latent Effect
- Ignores Accuracy
    - Changes based on battle/weather
- Increased Crit
- Increased Priority
- Ignores Stats

## Special (This shit is wack yo)
- Copies last a move (Copycat is very complicated)
- Curse (basically 2 different moves depending on user type)
- Disable (disables a move, making that move unable to be used)
    - Encore
- Electrify - Changes the type of the move used by an opponent
- Destiny Bond
- Defog (removes all statuses, self-evasiveness -1 and field effects)
- OHKO - separate accuracy check, always kills
- Fling - power/effect depends on item
- Forest Curse -  adds the Grass type (YES ADDS IT)
- Future Sight - attacks that spot with the calculated dmg on attack after 2 turns
- Helping Hand - boosts an ally attack by 50%

## Secondary Effects (any non-power effect)
- Stat changes (boost and drops)
- Causes a flinch 30% (affects turn order)
- Inflicts a permanent status
    - Dire Claw has multiple
- Changes field status (shields or terrain)
- Recharging
- Recoil / Recovery
- Affects Held Item
- Changes/Affects user type
- Swaps Pokemon
- User Faints
- Change Ability

## Special
- Move changes turn order (After You)
- Swap pokemon (ally switch)
- Checks if Pokemon is performing a move (before completing)
- Baton Pass
- Lost 1/2 hp and boost stats (does this count as self damage?)
    - Note move fails when health cannot be consumed
- Counters a move
- Destiny bond

## Latent Effect
- Fist
- Sound
- Ignores Protect/Status/more