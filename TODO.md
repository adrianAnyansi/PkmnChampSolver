# TODO

Reducing this file so it's just current tasks
Goal is to keep the task small and focused

I'm going to pick 2 teams from the last pokemon champions tourney
Then simulate the whole match (without universes*)

Moves/Pokes are implemented
Next is the Container/processor
damage calc is still invalid on lower end*
    - might need a better way to do this
Then speed / priority calculations
Then implementing teams and draft selection stuff?

## Thoughts

Working on a generic N targets for an event
    Message made this more difficult, I think it should go into the battle state as a temp variable

 ---
The best way to impl moves/etc is to do tests, but I need better helpers for making battles/pokemon rn

Want to work on bitflags to compare move things quickly
Need to implement Switching and Fainting so I can work on turn mechanics and end of battle
Speed order at some point of course

## Today

In order to make Protect, heres the plan
1. Set protect flag on user
2. Use move_history to keep track of moves (good for last resort, multiple moves & etc)
3. Move fails if Pokemon is last (not used)

Also need to think about 2nd effects on block, like Spiky Shield or Baneful. So the hit is blocked but it still counts as an effect trigger

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