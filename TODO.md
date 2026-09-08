# TODO (Overview)
Just moves from 2 teams, just 2 teams focus

I need to finish all the moves, and I need better testing cause I'm worrying things won't work correctly-

## Today
Onto Stomping Tantrum
    The test for this is annoying
Fake Out
    Still not implementing priority, just want the move to fail on not turn 1
    Actually does

## Moves to implement
Fake Out - Flinch, turn 0
Flare Blitz - Recoil after hit
Throat Chop (add flag)
Matcha Gotcha - Heal after hit
Rage Powder - Center of attention
Wide Guard - Special Protect
Light Screen - room effect + damage calc

## Current Thoughts
For fake out, I need to do the turns_active counter (which triggers abilities)
    Slow Start, toxic, perish?
    Natural Cure/Regenator only work if >0
    Baton Pass (perserves this counter)

Thinking about the generic state stuff
volatile status can work independently, so I can do just 0-1 state
for multiple targets, i need the power_set for it to work correctly
for multiple stats, this is never a case (at least as an AI says)


## Future todos and cleanup
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