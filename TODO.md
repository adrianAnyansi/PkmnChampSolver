# TODO (Overview)

Goal - all <12 pokemon, moves and abilities done
Then a timed simulation and optimization
I'm strudging forward

## Backlog/Put aside
Turn 0 doesnt matter until trainer choice and turn mechanics work

## Today
--- 

Want to either do Drought/Sandstorm, but that relies on DamageEffect and end of turn generation, as well as tracking weather turns (weather memory)

Also could do Sturdy due to damage calculation, which includes ability memory

Also I can do Fairy Aura, which is just a move modifier

## Ability notes
For chlorophyll, I can put a flag to say "Check this on weather change"
But whenever speed gets calculated, I gotta pull the modifier.
So OnWeatherChange + SpeedModifier means engine has to recalc speed order.

So it would be great to make an event that says "speed change occurred". That itself could be a MoveEffect that I can generate on change, but requires memory, which I don't want as it complicates things

Also realised I cant check speed prior to running things, because onWeatherChange has already changed the speed.

AI says the ability sets a dirty flag basically- this is similar to what I was thinking about raising an event. So the speed check can remain stateless, but onWeather contains the previous -> new state and knows whether a change occurred.
Going forward with that... 

Forgot that in order to test this, I need to set weather & I need to set weather turns too, I also don't quite have turn logic 100% so I need another test for that

---

Ok lets think- for chrolophyll it can activate onEnter same as weather change. For retrieving the value & multiplier, i have the damage modifier method to use, but something like unburden needs a flag thats reset on certain values/situations.
For example, Unburden gets a boost thats applied after base but before stat mult, but I don't want a constant check of ability speed every time- any other stat which should sit on the pokemon (like grass seed)


## Ability memory

Here's the solution to stuff like Disguise, Unburden, etc
In ActivePokemon I'll have votalite + non volatile memory
This way the ability can store memory without needing to worry about location/per ability memory

However 


## Moves to implement
Trick Room - Speed order stuff (not doing this btw)
Light Screen - room effect + damage calc
Heavy Slam - Weight for base power
Knock Off - Item mechanic

## Abilities to implement
Chlorophyll - Speed mechanic with instant change
Flower Veil - Protect status
Unburden - Item mechanic
Gale Wings - Speed mechanic
Sturdy - OnDamage from full, leave at 1HP, 
Levitate - Ignore Ground moves

Sand Stream - OnEnter, Start Sandstorm
Drought - OnEnter, start Sun

Sand Force - MoveModifier In Sandstorm, boost moves
Fairy Aura - Fairy moves do more, Dragon? does less

## Current Thoughts
---

## TrainedPokemon vs Active vs Inactive
TrainedPokemon needs all its stats.
Active needs to change:
    * Abilities
    - Items
    - Moves? (Copycat)
    * Stats* (can be swapped or overridden?)


## Delay/Post moves
How to delay/ make things go to the next turn?
I have a way to work with charge, but I cant do Future Sight or things that need to save data

## Other concepts

Thinking about some bigger concepts
There needs to be an effective stat method, that also accounts for battle status (boosts from abilities, items, more)

TrainedPokemon can lose their item, ability, type- I need even more classes great- best to treat them completely separately

Need to put the overall control of trainer choice into a note and thoughts 

Just focusing on things to do 1 by 1
- Moves (dependant on other things)
- Abilties 
- Items
- Switch Action & Fainting
- Beginning / End of battle
- Speed mechanics
- Formes/Mega evolution

## Messaging thoughts

Should do a review on messaging, its fragmented right now
It would be nice to have very good logging on effects and things that happen with strings, as when the logic gets more complicated, it will be impossible to track

Also want to separate sim_ & exec_ functions,
Currently logic is BS -> BC<...BS>, but BC is a 1-1 so messaging can sit there. Lets go bottom to top actually-
exec does BS -> BS, but I need BC anyways, so I have sim_ calling exec_
exec_ modifies the sent BS, but also needs to add a message so that must be a return value. Also since I want to use a function for BC gen, gotta encapsulate it too. So it can return a vec<string>

The root BS does not need to be stored?

Some sim_ functions need a base_clone since some actions will always occur (i.e move miss still triggers turnsActive, etc). Throwing away this clone would be a waste, and I will never reuse the OG again (in the case where I'm making a decision, i.e sim move 1,2,3,4. Since I will be editing the action_queue, these are no longer equivalent states. I want to save it but its just not possible)
I would need to make base_clones anyways, lets not overthink it much


NOTE: I need to think about source for certain actions
some status/stat are blocked only if opposing, so a generic source object will be helpful and give me redundancy in the future


## Future todos and cleanup
---
- damage calc is invalid on lower end, review
- speed / priority (going to use a [priority, speed, action] queue)
- team implementation, switch/faint


# Long Goal
Libraries
    Need a library get moves, pokemon, etc on demand. 
    I plan on making an interface that will create and return things as a simulation lifetime.

Create something that checks all pokemon and selects a random move
    - Also need to add additional actions (mega, switching)
    Will call it "TrainerChoice" or something

Adding the condition that match ends when all pokemon on one side are dead

Then going to validate the damage calc with stats and then natures


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