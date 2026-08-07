
use crate::BattleState;
use crate::battle::BattleAction;
use crate::math::{PkmnRational, get_random_int};


// Contains Battle State 
pub struct BattleContainer<'battle> {
    pub battle_ctns: Vec<BattleContainer<'battle>>,
    pub battle_state: Option<BattleState<'battle>>,
    pub pct_chance: PkmnRational,
    pub message: String,
}

impl<'battle> BattleContainer<'battle> {

    pub fn new() -> BattleContainer<'battle> {
        BattleContainer {
            battle_ctns: vec![],
            battle_state: None,
            pct_chance: PkmnRational::ONE(),
            message: String::new()
        }
    }

    pub fn simple(battle_state:BattleState, 
        pct_chance:PkmnRational) -> BattleContainer {
            BattleContainer {
                battle_ctns: vec![],
                battle_state: Some(battle_state),
                pct_chance,
                message: String::new()
            }
        }

    /// processes all states and modifies to the next state
    // TODO: Send hashes to dedup
    pub fn sim_next_action(&mut self) {

        // let mut new_states:Vec<BattleContainer> = vec![];
        if self.battle_state.is_some() {
            // TODO: I need to pop the queue to do this, has to be mut
            let b_state = self.battle_state.as_mut().unwrap();

            let new_bcs:Vec<BattleContainer<'battle>> = b_state.sim_action();

            if new_bcs.len() == 1 {
                // Need to iterate & consume the vector
                let b_ctn = new_bcs.into_iter().next().unwrap();
                self.battle_state = b_ctn.battle_state;
            } else {
                // if zero, end (or panic)
                self.battle_ctns = new_bcs;
                self.battle_state = None
            }
        } else {
            // NOTE: Should not have battle_state & containers
            // process internal states
            for battle_ctn in &mut self.battle_ctns {
                // TODO: Fix later
                // new_states.extend(battle_ctn.sim_next_action());
            }
        }
        
        // return new_states
    }

}

pub struct BattleProcessor<'battle> {
    pub battle_ctns:Vec<BattleContainer<'battle>>,
    pub teams:String, // TODO: Implement static teams
    
    /// number of iterations that have occurred
    pub iter_num:u64,
    pub keep_one_universe:bool,
    pub state_hashes:String, // TODO: Keep hashes that have been processed
}

impl<'battle> BattleProcessor<'battle> {

    pub fn new() -> BattleProcessor<'battle>{
        BattleProcessor {
            battle_ctns: vec![],
            teams: "".to_string(),
            iter_num: 0,
            keep_one_universe: true,
            state_hashes: "".to_ascii_lowercase()
        }
    }

    /// Collapse container by making a random value and choosing a state to return
    fn collapse(bc:BattleContainer<'battle>, 
        seed:Option<PkmnRational>) -> BattleContainer<'battle> {
        
        if bc.battle_ctns.len() == 0 {
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
        for eval_bc in &bc.battle_ctns {
            let mut check_val = eval_bc.pct_chance - curr_val;

            // TODO: Fix comparison operator
            if check_val.float() < rand_rat.float() {
                return BattleProcessor::collapse(bc, Some(rand_rat))
            }
            check_val += eval_bc.pct_chance
        }
        
        return bc
    }

    /// Process all battleStates to the next iteration
    pub fn process_all_states_by_one(&mut self) {

        // let mut next_states:Vec<BattleContainer> = vec![];
        // Process each state
        // TODO: Process until state terminates
        for battle_ctn in &mut self.battle_ctns {
            battle_ctn.sim_next_action();
        }

        // battle_ctn.sim_next_action();

        // TODO: After completion, remove fainted / duplicaties

        // self.battle_ctns.clear();
        // self.battle_ctns.extend(next_states);
    }
}
