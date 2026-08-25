
# Mechanic thoughts

There's so many issues where I need to consistently think about how best to implement mechanics so I'm putting it in a note


# Protection
- Move history: To know how many protects in a row occurred
- Flag for all moves that count for protection
- Logic for accuracy changes

Ok so the protect counter always needs to be accessed, and specifically on hit
I have to reset the counter on every executed move that isnt a protect move
I can't process this chain

For any move
- Reset counter to 0
For protect move
- If failed, reset to 0
- If success, increase counter

This implies a default reset always, which disables on protect move success
However there is no default action, it makes sense to specifically have functions on miss, but again the DEFAULT move is reset to 0
So both are necessary.

Since protect is treated as a promoted hit_action, there's no avenue for missed action and none would scale for causes like high-jump-kick or etc.
I'm adding a miss action and that will contain reset logic?
Or maybe override the null case for protect specifically since its the only time that this matters currently... ok i'm going with that now

All these issues are because of protect_counter and not the 2nd effect logic, so thats the exception.

Ok so now
1. hit action Protect adds BattleAction::Protect
2. This will call to sim_protect (I can add miss case in sim_protect)
3. Add the clones and states


# Charging moves

Doing multi turn logic is confusing, trying to figure this out since there are a lot of cases.

Moves like Electro Shot, Meteor Beam, Skull Bash have stat changes on turn 1. Other moves do not
Similar 2 turn moves also need this (turn 1 do this, turn 2 do this)
I was considering putting a BREAKER in hit_actions for turn 1, turn 2 but flags are for both, so i can make the assumption but hell-

Trying to wrap together all multi-turn moves while keeping Pokemon functionality ends up having me just port the engine to keep the same functionality.

I.E for Rampaging moves, need a counter + locked move
For 2 turn, need charing state and separator
For Dragon Darts, need a custom move action generated after the move
For multi-hit, roll a counter and then do N damage 

I'm going to stop overthinking and just do the 2 turn


charging needs to use a base cloned state, so things with an early exit is possible.
Sim_move currently makes a bunch of effects but doesn't directly edit the clone. For protect, it also sends an action, so going to do a thing.
This makes sense since an effect can be operated by many things, but for charge which only works on the pokemon its odd. 

Basically I can do 
Charge Effect -> If skipped, then put the move thing
See https://bulbapedia.bulbagarden.net/wiki/Category:Moves_with_a_charging_turn for all things

## Separate script
Oh yea I forgot, I was thinking of doing a separate move which would cover most things? Ok just ignoring this idea ngl

# Stomping Tantrum
If last turn the move missed, failed to affect or was prevented by an effect, power is doubled.
So I need to know if a move missed as well as history?

# Last Resort
- Move history will work for this

# Direct Damage
- Some moves like One-hit-KO or Mirror Coat, etc do not do damage calcs

# Rage Fist 
- Needs a counter on how many times the pokemon was hit

# Solar Beam
- I'm thinking of putting a Charging flag and then triggering the previous move when the next turn occurs.
But that needs to be cancelled when the 

# Weather Ball
This needs to check the weather, then edit the power & type of the move.
Clone the move and use that new value

# Fake Out
Check if move_history is empty, otherwise prevent.
(Again, if move failed, you still cant redo Fake Out right? right?)

# Parting Shot
I have the stats, the switch out needs to be a new action
However I don't have teams so I'm gonna ignore this

# Throat Chop
Add status, the moves it affects don't exist

# Matcha Gotcha
Recovery needs to be triggered from DamageEffect and come back to user
Not gonna think about the different situations recovery stuff gets changed

# Rage Powder
center_of_attention status
Need to run a field effect that redirects all moves towards it

# Trick Room
No. Not until speed is done