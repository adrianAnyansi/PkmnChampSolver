
use crate::BattleState;
use crate::math::{PkmnRational, get_random_int};


// Contains Battle State 
pub struct BattleContainer<'battle> {
    pub battle_ctns: Vec<BattleContainer<'battle>>,
    pub battle_state: Option<BattleState<'battle>>,
    pub pct_chance: PkmnRational
}

impl<'battle> BattleContainer<'battle> {

    pub fn new() -> BattleContainer<'battle> {
        BattleContainer {
            battle_ctns: vec![],
            battle_state: None,
            pct_chance: PkmnRational::ONE()
        }
    }

    pub fn simple(battle_state:BattleState, 
        pct_chance:PkmnRational) -> BattleContainer {
            BattleContainer {
                battle_ctns: vec![],
                battle_state: Some(battle_state),
                pct_chance
            }
        }

    // processes all states and returns the updated container of all consequences
    // TODO: Send hashes to dedup
    pub fn sim_next_action(&mut self) -> Vec<BattleContainer<'battle>> {

        let mut new_states:Vec<BattleContainer> = vec![];
        if self.battle_state.is_some() {
            // TODO: I need to pop the queue to do this, has to be mut
            let b_state = self.battle_state.as_mut().unwrap();

            let new_bcs:Vec<BattleContainer<'battle>> = b_state.sim_action();
            
            // TODO: Need to fold the pct with a new bc
            new_states.extend(new_bcs);
        }
        
        // NOTE: Should not have battle_state & containers
        // process internal states
        for battle_ctn in &mut self.battle_ctns {
            new_states.extend(battle_ctn.sim_next_action());
        }
        
        return new_states
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

        let mut next_states:Vec<BattleContainer> = vec![];
        // Process each state
        for battle_ctn in &mut self.battle_ctns {
            next_states.extend(battle_ctn.sim_next_action());
        }

        // TODO: After completion, remove fainted / duplicaties

        self.battle_ctns.clear();
        self.battle_ctns.extend(next_states);
    }
}
