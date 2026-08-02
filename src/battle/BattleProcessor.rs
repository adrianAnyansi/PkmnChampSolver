
use rand;
use rand::rngs::ThreadRng;
use crate::BattleState;
use crate::math::{PkmnRational, get_random_int};


// Contains Battle State 
pub struct BattleContainer<'battle> {
    pub battleCtns: Vec<BattleContainer<'battle>>,
    pub battle_state: Option<BattleState<'battle>>,
    pub pct_chance: PkmnRational
}

pub struct BattleProcessor<'battle> {

    pub battle_state_vec:Vec<BattleContainer<'battle>>,
    pub iter:u64,

}

impl<'battle> BattleProcessor<'battle> {

    // Collapse container by making a random value and choosing a state to return
    fn collapse(bc:BattleContainer<'battle>, 
        seed:Option<PkmnRational>) -> BattleContainer<'battle> {
        
        if bc.battleCtns.len() == 0 {
            return bc
        }

        let rand_rat:PkmnRational;
        if seed.is_some() {
            rand_rat = seed.unwrap();
        } else {
            rand_rat = PkmnRational::new(
                get_random_int(1, 10_000) as i32,
                 10_000);
        }
        
        // NOTE: If higher resolution, throw an error
        let curr_val = PkmnRational::ZERO();
        for eval_bc in &bc.battleCtns {
            let mut check_val = eval_bc.pct_chance - curr_val;

            // TODO: Fix comparison operator
            if check_val.float() < rand_rat.float() {
                return BattleProcessor::collapse(bc, Some(rand_rat))
            }
            check_val += eval_bc.pct_chance
        }
        
        return bc
    }
}
