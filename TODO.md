# TODO (Overview)
Just moves from 2 teams, just 2 teams focus

I need to finish all the moves, and I need better testing cause I'm worrying things won't work correctly-

## Today
--- 
Throat Chop skip flag
Heal effect enum
Then Rage Powder logic


## Moves to implement
Fake Out - Flinch, turn 0
Flare Blitz - Recoil after hit
Throat Chop (add flag)
Matcha Gotcha - Heal after hit
Rage Powder - Center of attention
Wide Guard - Special Protect
Light Screen - room effect + damage calc

## Current Thoughts
---
Should do a review on messaging, its fragmented right now
It would be nice to have very good logging on effects and things that happen with strings, as when the logic gets more complicated, it will be impossible to track

Also want to separate sim_ & exec_ functions,
Currently logic is BS -> BC<...BS>, but BC is a 1-1 so messaging can sit there. Lets go bottom to top actually-
exec does BS -> BS, but I need BC anyways, so I have sim_ calling exec_
exec_ modifies the sent BS, but also needs to add a message so that must be a return value. Also since I want to use a function for BC gen, gotta encapsulate it too. So it can return a vec<string>

The root BS does not need to be stored?

Some sim_ functions need a base_clone since some actions will always occur (i.e move miss still triggers turnsActive, etc). Throwing away this clone would be a waste, and I will never reuse the OG again (in the case where I'm making a decision, i.e sim move 1,2,3,4. Since I will be editing the action_queue, these are no longer equivalent states. I want to save it but its just not possible)
I would need to make base_clones anyways, lets not overthink it much



## Future todos and cleanup
---
- damage calc is invalid on lower end, review
- speed / priority (going to use a [priority, speed, action] queue)
- team implementation, switch/faint


# Long Goal
Need easier pokemon & move creation, going to make a central library to contain all
Then more testing with spread moves
Then I'll implment abilities

Create something that checks all pokemon and selects a random move
    - Also need to add additional actions (mega, switching)

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