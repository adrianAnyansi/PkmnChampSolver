
# Mechanic thoughts

There's so many issues where I need to consistently think about how best to implement mechanics so I'm putting it in a note


# Protection
- Move history: To know how many protects in a row occurred
- Flag for all moves that count for protection
- Logic for accuracy changes

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