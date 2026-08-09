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

Working on a generic N targets for an event
Might delay until Im better at Rust

---


---

I have the BC message* and understanding the state stuff
So battle processor exists to hold multiple universes with different origins
also to hold hashes
i.e battle container can simulate 1 battle -> end but not multiple origins despite overlaps being possible. 
BC collapse could be on BC but battle processor wants the collapse, I never want to collapse when processing the BC, I want the flatten (to merge similar hashes) but battle_processor is the one that should also do this*

Issue is that I have a root_bc because sim_next_action has to contain itself- which defeats the purpose of processor root? idk.
Yeah collapse should be on the root, and a root means its % is ONE. Make that a property and its easier to reason about, not to mention the logic of collapse requires a 0..1 range, ah its the mutable/immutable problem again.

I want collapse to mutate the root so I don't need to change the vector when collapsing, so its a different function.
 

 ---
 Move message
 1. Poke used N!
    On cancel, say why target was cancelled
2. On damage, say N took N damage
    On cancel/edit, change the message
    

## Today

Add messages to each action use for debugging
Create something that loops until all actions are complete

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