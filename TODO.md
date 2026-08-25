# TODO
Just moves from 2 teams, just 2 teams focus

I need to finish all the moves, and I need better testing cause I'm worrying things won't work correctly-

Collapse works, going to think about the teams and move choice later, still mechanics I'm missing

## Todo
damage calc is still invalid on lower end*
    - might need a better way to do this
Then speed / priority calculations
Then implementing teams, switch/faint effects and draft selection stuff?

## Thoughts

With messages on battle state, event can be made, but later

 ---
 After all actions are done, turn end is inited, do a co routine where end-of-turn actions occur
 Going to set a flag so code doesn't accidentally skip turns without checking things
 I want a volatile "battle state" to clear, i mean well technically there is no internal pokemon state outside HP (so far)



## Today

Lets do turns.
Then I'll do Solar Beam, and then Stomping logic

Solar Beam can't be done cause I have no turn end for charging right now
Stomping Tantrum next... failed move time, neat
    This also requires turn end, ok lets just do this imo
Then Fake Out - (speed priority)
Throat Chop needs a data field for turns remaining
Matcha Gotcha drain needed

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