# TODO (Overview)

Goal - all <12 pokemon, moves and abilities done
Then a timed simulation and optimization
I'm strudging forward

## Today
--- 

Implementing the send_out phase of turn 0
So abilities can be tied to enter/exit

Then I'll do parting shot, and the logic for that


## Moves to implement
Parting Shot - Return after stats
Trick Room - Speed order stuff
Light Screen - room effect + damage calc
Heavy Slam - Weight for base power
Knock Off - Item mechanic

## Abilities to implement
Chlorophyll - Speed mechanic with instant change
Blaze - Move thing
Rough Skin - OnDamage, do more damage (check source)
Intimidate  - OnEnter, change statistics
Flower Veil - Protect status
Hospitality - OnEnter, give ally 1/8? healing
Unburden - Item mechanic
Gale Wings - Speed mechanic
Sturdy - OnDamage from full, leave at 1HP, 
Levitate - Ignore Ground moves
Sand Stream - OnEnter, Start Sandstorm
Drought - OnEnter, start Sun
Fairy Aura - Fairy moves do more, Dragon? does less
Sand Force - MoveModifier In Sandstorm, boost moves

## Current Thoughts
---

## Switch actions and etc
Future actions probably need their own struct, avoiding this logic right now
Speed should be stable, but I'll do testing
Speed-tie will be outside the battle state, no way around it

Switch in means no action at super priority (including no mega/etc)
thinking about trainer choice later

I need to choose teams & leads, preferably at the same time...
My idea of trainer choice is to give BC all the possible options (like an action)- 

Select Team (front_team), first 2 trigger the 

Switch(Target BattlePosition, possible team indexes)
Move(Source poke, moves, targets on field*)

The brain (intelligent trainer choicer) needs to know about the best simulation (considering the entire team), lets slot that in later, i just need 4

Think of starting a battle as "choose list of 4" and generating 4 send_out there. So like a choice for 4 pokemon
Team choice can take external params, so thats fine.

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