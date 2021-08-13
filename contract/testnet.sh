#!/bin/bash

./build.sh

export NEAR_ACCT=xword.demo.testnet
near delete $NEAR_ACCT demo.testnet
near create-account $NEAR_ACCT --masterAccount demo.testnet
near deploy $NEAR_ACCT --wasmFile /Users/mike/near/near-crossword/contract/res/crossword.wasm
echo "Keys before:"
near keys $NEAR_ACCT
near call $NEAR_ACCT new_puzzle '{
  "answer_pk": "ed25519:psA2GvARwAbsAZXPs6c6mLLZppK1j1YcspGY2gqq72a",
  "dimensions": {
   "x": 19,
   "y": 13
  },
  "answers": [
   {
     "num": 1,
     "start": {
       "x": 1,
       "y": 2
     },
     "direction": "Across",
     "length": 8
   },
   {
     "num": 1,
     "start": {
       "x": 1,
       "y": 2
     },
     "direction": "Down",
     "length": 10
   },
   {
     "num": 2,
     "start": {
       "x": 0,
       "y": 7
     },
     "direction": "Across",
     "length": 9
   },
   {
     "num": 3,
     "start": {
       "x": 7,
       "y": 4
     },
     "direction": "Down",
     "length": 7
   },
   {
     "num": 4,
     "start": {
       "x": 5,
       "y": 5
     },
     "direction": "Across",
     "length": 11
   },
   {
     "num": 5,
     "start": {
       "x": 7,
       "y": 10
     },
     "direction": "Across",
     "length": 3
   },
   {
     "num": 6,
     "start": {
       "x": 14,
       "y": 1
     },
     "direction": "Down",
     "length": 10
   },
   {
     "num": 7,
     "start": {
       "x": 12,
       "y": 2
     },
     "direction": "Across",
     "length": 4
   },
   {
     "num": 8,
     "start": {
       "x": 11,
       "y": 8
     },
     "direction": "Across",
     "length": 4
   },
   {
     "num": 8,
     "start": {
       "x": 11,
       "y": 8
     },
     "direction": "Down",
     "length": 3
   }
  ]
}' --accountId mike.testnet

echo "Keys after"
near keys $NEAR_ACCT
near view $NEAR_ACCT debug_get_puzzle '{"pk": "ed25519:psA2GvARwAbsAZXPs6c6mLLZppK1j1YcspGY2gqq72a"}'