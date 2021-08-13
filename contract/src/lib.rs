mod debugging;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Deserialize,
    log, Balance, Promise,
};
use near_sdk::{env, near_bindgen, PublicKey};
use near_sdk::{json_types::Base58PublicKey, AccountId};
use near_sdk::collections::LookupMap;

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AnswerDirection {
    Across,
    Down,
}

/// The origin (0,0) starts at the top left side of the square
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CoordinatePair {
    x: u8,
    y: u8,
}

// {"num": 1, "start": {"x": 19, "y": 31}, "direction": "Across", "length": 8}
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair,
    direction: AnswerDirection,
    length: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum PuzzleStatus {
    Unsolved,
    Solved { solver_pk: PublicKey },
    Claimed { memo: String },
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,
    reward: Balance,
    creator: AccountId,
    /// Use the CoordinatePair assuming the origin is (0, 0) in the top left side of the puzzle.
    dimensions: CoordinatePair,
    answer: Vec<Answer>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Crossword {
    puzzles: LookupMap<PublicKey, Puzzle>,
}

#[near_bindgen]
impl Crossword {
    pub fn submit_solution(&mut self, solver_pk: Base58PublicKey) {
        let answer_pk = env::signer_account_pk();
        log!("aloha you gave me this pk: {:?}", answer_pk);
        // check to see if the answer_pk from signer is in the puzzles
        let mut puzzle = self
            .puzzles
            .get(&answer_pk)
            .expect("ERR_NOT_CORRECT_ANSWER");

        // Check if the puzzle is already solved. If it's not solved, make batch action of
        // removing that public key and adding the user's public key
        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved => PuzzleStatus::Solved {
                solver_pk: solver_pk.clone().into(),
            },
            _ => {
                env::panic(b"ERR_PUZZLE_SOLVED");
            }
        };

        // Reinsert the puzzle back in after we modified the status:
        self.puzzles.insert(&answer_pk, &puzzle);

        log!(
            "Puzzle with pk {:?} solved, solver pk: {}",
            answer_pk,
            String::from(&solver_pk)
        );

        // Add new function call access key for claim_reward
        Promise::new(env::current_account_id()).add_access_key(
            solver_pk.into(),
            250000000000000000000000,
            env::current_account_id(),
            b"claim_reward".to_vec(),
        );

        // Delete old function call key
        Promise::new(env::current_account_id()).delete_key(answer_pk);
    }

    pub fn claim_reward(&mut self, crossword_pk: Base58PublicKey, receiver_acc_id: String, memo: String) {
        let signer_pk = env::signer_account_pk();
        /* check to see if signer_pk is in the puzzles keys */
        let mut puzzle = self
            .puzzles
            .get(&crossword_pk.0)
            .expect("Not a correct public key to solve puzzle");

        /* check if puzzle is already solved and set `Claimed` status */
        puzzle.status = match puzzle.status {
            PuzzleStatus::Solved { solver_pk: _ } => PuzzleStatus::Claimed {
                memo: memo.clone().into(),
            },
            _ => {
                env::panic(b"puzzle should have `Solved` status to be claimed");
            }
        };

        // Reinsert the puzzle back in after we modified the status:
        self.puzzles.insert(&crossword_pk.0, &puzzle);

        Promise::new(receiver_acc_id.clone()).transfer(puzzle.reward);

        log!(
            "Puzzle with pk: {:?} claimed, receiver: {}, memo: {}, reward claimed: {}",
            crossword_pk,
            receiver_acc_id,
            memo,
            puzzle.reward
        );

        /* delete function call key*/
        Promise::new(env::current_account_id()).delete_key(signer_pk);
    }

    /// Puzzle creator provides:
    /// `answer_pk` - a public key generated from crossword answer (seed phrase)
    /// `dimensions` - the shape of the puzzle, lengthwise (`x`) and high (`y`)
    /// `answers` - the answers for this puzzle
    /// Call with NEAR CLI like so:
    /// `near call $NEAR_ACCT new_puzzle '{"answer_pk": "ed25519:psA2GvARwAbsAZXPs6c6mLLZppK1j1YcspGY2gqq72a", "dimensions": {"x": 19, "y": 13}, "answers": [{"num": 1, "start": {"x": 19, "y": 31}, "direction": "Across", "length": 8}]}' --accountId $NEAR_ACCT`
    #[payable]
    pub fn new_puzzle(&mut self, answer_pk: Base58PublicKey, dimensions: CoordinatePair, answers: Vec<Answer>) {
        let value_transferred = env::attached_deposit();
        let creator = env::predecessor_account_id();
        let answer_pk = PublicKey::from(answer_pk);
        let existing = self.puzzles.insert(
            &answer_pk,
            &Puzzle {
                status: PuzzleStatus::Unsolved,
                reward: value_transferred,
                creator,
                dimensions,
                answer: answers
            },
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        Promise::new(env::current_account_id()).add_access_key(
            answer_pk,
            250000000000000000000000,
            env::current_account_id(),
            b"submit_solution".to_vec(),
        );
    }
}

impl Default for Crossword {
    fn default() -> Self {
        Self { puzzles: LookupMap::new(b"c") }
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
