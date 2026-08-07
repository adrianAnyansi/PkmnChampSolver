# TODO

Reducing this file so it's just current tasks
Goal is to keep the task small and focused

I'm going to pick 2 teams from the last pokemon champions tourney
Then simulate the whole match (without universes*)

Moves/Pokes are implemented
Next is the Container/processor
Then verify the damage calculation
Then speed / priority calculations
Then implementing teams and draft selection stuff?

## Thoughts
Very slowly understanding the lifetimes & more
I need to write all the simulation logic for everything and its hard

Would like a function to do:
    battleState + actions with pct = vec<battleState>
Gotta think about how to do this cleanly

---
When doing status/Stat/more, I need to think about targetting
- Stat modifier needs a target but no source/dest (usually? Assuming the targetting* is blocked on the move/ability level)
- Status needs accuracy, status, target which it doesn't have, gonna make a struct so to put that on the move/ability etc.

For Effects not sure if I should join/split targets or effects.
multiple targets just means I need to write a loop like I do with moves, but moves NEED to know about multiple targets

During the BattleAction creation I can split this, its just good for storing data easily

Technically, no status effect can target the move user other than the opponent; going to have a generic MoveEffect/BattleEffect where theres always effect, %, target possible.
If I find cases where this differs, handle it then.

With all this, now I want to do sludge-bomb, do the follow-up and print this out.

---
When building a prob map, there is always a unique action to be applied and number of targets to apply it to-

Think of it as N independent events, where 0-N can be applied at any time. 
i.e 01010
Since there's 1 action and multiple targets, the target is the decider so the prob function needs to also take the list of targets joined with the combination. This honestly sucks to write & reason not even joking

If I were to split every interaction, so status & stat & general does not ever do multiple targets at once, I'd never need to worry about the probability set ever- 
but I would need to nest things so i.e the BC with the null case & the hit case both need to simulate the next case, so I'd need to collapse and verify both cases are the same and sum them during iteration.
I've also been writing this with the assumption that status/stat changes are completely indep, but there are cases when stat_change can trigger abilities or items

If I create a way to have spread moves perform independently

## Today

Finish the move + accuracy test
Finish the status + stat test
Create a flow that picks at random one of the above and prints the message that occurred

Trying to make a impl that is easy to build multiple probabilities and put in the action queue (dupl)
    - Finish the move accuracy impl logic (after effects from moves gotta also come from this)
    - move effects


Write test for the move/type logic
Need a string for the output/changes per state?

Going to implement the container and have it continue to pick random attacks

Adding the condition that match ends when all pokemon on one side are dead

Then going to validate the damage calc with stats and then natures

# Battle State
Thinking through battle state mutating itself vs processing on the state
No just do the safe thing, multiple return and optimize after

I should go action by action-
the turn thing is sort of a simulation thing


ok to copy/clone the state, gotta fix the ActivePokemon thing
basically need a TrainedPokemon intermediate so I can cache the pokemon stats-
or do I?
Trained stats are the only thing that remains independent between states, and since I might shift them for 

# Teams to implement
## ARSAL PURI
Venusaur @ Focus Sash  
Ability: Chlorophyll  
 Timid Nature  
- Sleep Powder  
- Sludge Bomb  
- Earth Power  
- Protect  

Charizard @ Charizardite Y  
Ability: Blaze  
 Modest Nature  
- Heat Wave  
- Solar Beam  
- Weather Ball  
- Protect  

Garchomp @ Choice Scarf  
Ability: Rough Skin  
 Adamant Nature  
- Earthquake  
- Rock Slide  
- Stomping Tantrum  
- Dragon Claw  

Incineroar @ Sitrus Berry  
Ability: Intimidate  
 Careful Nature  
- Fake Out  
- Flare Blitz  
- Parting Shot  
- Throat Chop  

Floette-Eternal @ Floettite  
Ability: Flower Veil  
 Modest Nature  
- Moonblast  
- Dazzling Gleam  
- Calm Mind  
- Protect  

Sinistcha @ Kasib Berry  
Ability: Hospitality  
 Relaxed Nature  
- Matcha Gotcha  
- Rage Powder  
- Trick Room  
- Protect  



## WOLFE GLICK
Sneasler @ White Herb  
Ability: Unburden  
 Jolly Nature  
- Protect  
- Close Combat  
- Dire Claw  
- Fake Out  

Sinistcha @ Sitrus Berry  
Ability: Hospitality  
 Relaxed Nature  
- Rage Powder  
- Trick Room  
- Matcha Gotcha  
- Protect  

Talonflame @ Sharp Beak  
Ability: Gale Wings  
 Jolly Nature  
- Protect  
- Brave Bird  
- Flare Blitz  
- Swords Dance  

Steelix @ Steelixite  
Ability: Sturdy  
 Brave Nature  
- Protect  
- Heavy Slam  
- High Horsepower  
- Wide Guard  

Rotom-Wash @ Leftovers  
Ability: Levitate  
 Bold Nature  
- Will-O-Wisp  
- Thunderbolt  
- Hydro Pump  
- Light Screen  

Tyranitar @ Tyranitarite  
Ability: Sand Stream  
 Jolly Nature  
- Protect  
- Rock Slide  
- Knock Off  
- Dragon Dance  