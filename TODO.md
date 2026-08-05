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

## Today

Then implement the move simulation* logic
    For move simulation, there can be 3 targets this time
    and since there are separate accuracy (depending on ablities, state, etc, need function for this)

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