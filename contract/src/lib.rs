use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log, Balance, Promise,
};
use near_sdk::{env, near_bindgen, PublicKey};
use near_sdk::{json_types::Base58PublicKey, AccountId};
use std::collections::HashMap;

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize)]
pub enum PuzzleStatus {
    Unsolved,
    Solved { solver_pk: PublicKey },
    Claimed { memo: String },
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Puzzle {
    status: PuzzleStatus,
    value: Balance,
    creator: AccountId,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Crossword {
    puzzles: HashMap<PublicKey, Puzzle>,
}

#[near_bindgen]
impl Crossword {
    pub fn submit_solution(&mut self, solver_pk: Base58PublicKey) {
        let answer_pk = env::signer_account_pk();
        /* check to see if the answer_pk from signer is in the puzzles */
        let puzzle = self
            .puzzles
            .get_mut(&answer_pk)
            .expect("Not a correct public key to solve puzzle");

        /* check if the puzzle is already solved, if it's not solved - make batch action of
        removing that public key and adding the user's public key */
        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved => PuzzleStatus::Solved {
                // TODO: why to do this? What it gives us? Why not just set PuzzleStatus::Solved?
                solver_pk: solver_pk.clone().into(),
            },
            _ => {
                env::panic(b"puzzle is already solved");
            }
        };

        log!("Puzzle solved, solever pk: {}", String::from(&solver_pk));

        /* add new function call key for claim_reward */
        Promise::new(env::current_account_id()).add_access_key(
            solver_pk.into(),
            250000000000000000000000,
            env::current_account_id(),
            b"claim_reward".to_vec(),
        );

        /* delete old funciton call key*/
        Promise::new(env::current_account_id()).delete_key(answer_pk);
    }
    pub fn claim_reward(&mut self, _reciever_acc_id: String) {
        // TODO: is it ok to have reciever_acc_id for now?
        // TODO: what puzzle are we solving?

        // TODO:
        // TODO: delete function call key
    }

    // Puzzle creator provides `key` that's the answer
    #[payable]
    pub fn new_puzzle(&mut self, key: Base58PublicKey) {
        let value_transfered = env::attached_deposit();
        let creator = env::predecessor_account_id();
        let key = PublicKey::from(key);
        let existing = self.puzzles.insert(
            key.clone(),
            Puzzle {
                status: PuzzleStatus::Unsolved,
                value: value_transfered,
                creator,
            },
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        Promise::new(env::current_account_id()).add_access_key(
            key,
            250000000000000000000000,
            env::current_account_id(),
            // * Strange API for it to be cs names
            b"submit_solution".to_vec(),
        );
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use near_sdk::MockedBlockchain;
    // use near_sdk::{testing_env, VMContext};

    // // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    // fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
    //     VMContext {
    //         current_account_id: "alice_near".to_string(),
    //         signer_account_id: "bob_near".to_string(),
    //         signer_account_pk: vec![0, 1, 2],
    //         predecessor_account_id: "carol_near".to_string(),
    //         input,
    //         block_index: 0,
    //         block_timestamp: 0,
    //         account_balance: 0,
    //         account_locked_balance: 0,
    //         storage_usage: 0,
    //         attached_deposit: 0,
    //         prepaid_gas: 10u64.pow(18),
    //         random_seed: vec![0, 1, 2],
    //         is_view,
    //         output_data_receivers: vec![],
    //         epoch_height: 19,
    //     }
    // }
}
