
use crate::BattleState;
use crate::battle::BattleAction;
use crate::math::{PkmnRational, get_random_int};


// Contains Battle State 
pub struct BattleContainer<'battle> {
    pub battle_ctns: Vec<BattleContainer<'battle>>,
    pub battle_state: Option<BattleState<'battle>>,
    pub pct_chance: PkmnRational,
    
    pub message: String,
    pub name: String
}


impl<'battle> BattleContainer<'battle> {

    pub fn new() -> BattleContainer<'battle> {
        BattleContainer {
            battle_ctns: vec![],
            battle_state: None,
            pct_chance: PkmnRational::ONE(),
            message: String::new(),
            name: format!("BattleCtn ")
        }
    }

    pub fn simple(battle_state:BattleState, 
        pct_chance:PkmnRational) -> BattleContainer {
            BattleContainer {
                battle_ctns: vec![],
                battle_state: Some(battle_state),
                pct_chance,
                message: String::new(),
                name: format!("BattleCtn ")
            }
        }

    pub fn one(battle_state:BattleState,
        message:Option<String>) -> BattleContainer {
            BattleContainer { 
                battle_ctns: vec![], 
                battle_state: Some(battle_state), 
                pct_chance: PkmnRational::ONE(), 
                message: message.unwrap_or(String::new()),
                name: format!("BattleCtn::ONE")
            }
        }

    /// Is this the main container 
    pub fn is_root(&self) -> bool {
        return self.pct_chance == PkmnRational::ONE()
    }

    /// Return true if contains battle_state
    pub fn is_leaf(&self) -> bool {
        return self.battle_ctns.len() == 0 
            && self.battle_state.is_some()
    }

    /// Return count of all battlecount
    pub fn count_tree(&self) -> i32 {
        if self.is_leaf() {
            return 1
        }
        self.battle_ctns.iter().map(
            |bc| bc.count_tree()
        ).sum()
    }

    pub fn add_msg(mut self, new_str:String) -> Self {
        self.message.push_str(&new_str);
        self
    }

    /// If root object, use rng to select a state,
    /// Then promote that container/state to the main root
    pub fn collapse(&mut self) {

        if !self.is_root() {
            return // only run from root
        }

        if self.is_leaf() {
            return // no collapse needed
        }
        
        // Result should be a leaf
        let prob_set:Vec<PkmnRational> = self.battle_ctns.iter().map(|eval_bc| eval_bc.pct_chance).collect();

        // Final pick
        let chosen_idx = PkmnRational::PickFrom(prob_set);
        let picked_parent_bc = self.battle_ctns.remove(chosen_idx);
        let picked_bc = picked_parent_bc.pick_owned();

        
        self.battle_ctns.clear(); // clear all other states
        // transfer data from the picked
        self.message = picked_bc.message;
        self.battle_state = picked_bc.battle_state;
        // name remain the same
        // pct chance remains the same

    }

    /// Pick a battle container at random
    pub fn pick(&self) -> &BattleContainer<'battle> {

        if self.is_leaf() {
            return self
        }

        // NOTE: Picking a new rng seed instead carry -> minus -> multiply
        let rand_rat = PkmnRational::new(
                get_random_int(1, 10_000) as i32,
                 10_000);

        let mut curr_val = PkmnRational::ZERO();
        for eval_bc in &self.battle_ctns {

            // TODO: Fix comparison operator
            if rand_rat.float() < (eval_bc.pct_chance - curr_val).float() {
                return eval_bc.pick();
            }
            curr_val += eval_bc.pct_chance
        }
        // As fail-safe (since for-loop doesnt know every ratio is covered)
        return self.battle_ctns.last().as_ref().unwrap().pick()
    }

    /// Pick a battle container at random, consuming and returning ownership
    pub fn pick_owned(self) -> BattleContainer<'battle> {

        let BattleContainer {
            battle_ctns,
            battle_state,
            pct_chance,
            message,
            name,
        } = self;

        if battle_ctns.len() == 0 {
            return BattleContainer {
                battle_ctns,
                battle_state,
                pct_chance,
                message,
                name,
            }
        }

        // NOTE: Picking a new rng seed instead carry -> minus -> multiply
        let rand_rat = PkmnRational::new(
                get_random_int(1, 10_000) as i32,
                 10_000);

        let mut curr_val = PkmnRational::ZERO();
        let mut last_bc:Option<BattleContainer<'battle>> = None;
        for eval_bc in battle_ctns.into_iter() {

            // TODO: Fix comparison operator
            if rand_rat.float() < (eval_bc.pct_chance - curr_val).float() {
                return eval_bc.pick_owned();
            }
            curr_val += eval_bc.pct_chance;
            last_bc = Some(eval_bc);
        }

        // As fail-safe (since for-loop doesnt know every ratio is covered)
        return last_bc.unwrap().pick_owned()
    }

    /// processes all states and modifies to the next state
    // TODO: Send hashes to dedup
    pub fn sim_next_action(&mut self) {
        
        self.message = String::new(); // Reset message since a simulation moves the bc forward

        if self.battle_state.is_some() {
            
            let b_state = self.battle_state.as_mut().unwrap();
            
            // pop action outside of the simulation step
            let action = match b_state.action_queue.pop_front() {
                Some(action) => action,
                None => return
            };
            b_state.action_num += 1;
            
            let new_bcs:Vec<BattleContainer<'battle>> = b_state.sim_action(action);

            if new_bcs.len() == 0 {
                // if zero, no change to state
                self.message = "No state change occurred from action*, review".to_string();
                return 
            }
            else if new_bcs.len() == 1 {
                // Need to iterate & consume the vector
                let b_ctn = new_bcs.into_iter().next().unwrap();
                self.battle_state = b_ctn.battle_state;
                self.message = b_ctn.message;
                self.name = b_ctn.name;
            } else {
                // consume all bcs and override it
                self.battle_ctns = new_bcs;
                self.battle_state = None
            }
        } else {
            // recurse and process
            for battle_ctn in &mut self.battle_ctns {
                battle_ctn.sim_next_action(); // Recurse, review later
                // TODO: Fix later
            }
        }

        // collapse?
        
    }

    /// Returns true if all BattleStates have no more actions
    pub fn completed (&self) -> bool {
        // TODO: I should determine this during the sim_next_action instead
        // of reiterating the ctns
        if self.battle_state.is_some() {
            return self.battle_state.as_ref().unwrap().action_queue.len() == 0
        } else {
            return self.battle_ctns.iter().all(
                |bc| { bc.completed() }
            )
        }
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
    #[deprecated(note="Incomplete and does not get the right value")]
    fn collapse_ctn(bc:BattleContainer<'battle>, 
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
                // TODO: Obvious bug usingg the bc here, thats an infinite
                // return BattleProcessor::collapse_ctn(bc, Some(rand_rat))
            }
            check_val += eval_bc.pct_chance
        }
        
        return bc
    }

    // TODO: Flatten the battle_ctns by hash, moving that % to the other
    // location ()

    /// Process all battleStates to the next iteration
    pub fn process_all_states_by_one_turn(&mut self, collapse_on_action:bool) {

        // Process each state
        const MAX_ACTIONS:u32 = 100;
        let mut completed:bool = false;
        // TODO: Process until state terminates
        for battle_ctn in &mut self.battle_ctns {
            for _idx in 0..MAX_ACTIONS {
                battle_ctn.sim_next_action();
                if collapse_on_action {
                    let full_count = battle_ctn.count_tree();
                    if full_count > 1 {
                        println!("[-/-] Collapsing {full_count} bcs")
                    }
                    battle_ctn.collapse();
                    let bc_msg = &battle_ctn.message;
                    // TODO: Need a generic BC print-out, this is optimized for collapse
                    let collsped_bs = battle_ctn.battle_state.as_ref().unwrap();
                    println!("Turn[{}/{}]: {}",
                        collsped_bs.turn_num, collsped_bs.action_num,  bc_msg);
                }

                if battle_ctn.completed() {
                    battle_ctn.battle_state.as_mut().unwrap().turn_num += 1;
                    completed = true;
                    break;
                }
            } // end of action/turn iteration
            if !completed {
                println!("Too many iterations, review this-");
            }
            // Update the turn count
            battle_ctn.battle_state.as_mut().unwrap().turn_num += 1;
        }

        // TODO: After completion, remove fainted / duplicaties

    }
}
