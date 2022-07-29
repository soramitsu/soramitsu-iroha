#![allow(missing_docs, clippy::pedantic, clippy::restriction)]

use iroha::samples::get_config;
use iroha_core::genesis::{
    GenesisNetwork, GenesisNetworkTrait, RawGenesisBlock, RawGenesisBlockBuilder,
};
use iroha_data_model::prelude::*;
use test_network::{get_key_pair, Peer as TestPeer, PeerBuilder, TestRuntime};
use tokio::runtime::Runtime;

fn main() {
    fn generate_genesis(num_domains: u32) -> RawGenesisBlock {
        let mut builder = RawGenesisBlockBuilder::new();

        let key_pair = get_key_pair();
        for i in 0_u32..num_domains {
            builder = builder
                .domain(format!("wonderland-{}", i).parse().expect("Valid"))
                .with_account(
                    format!("Alice-{}", i).parse().expect("Valid"),
                    key_pair.public_key().clone(),
                )
                .with_asset(
                    format!("xor-{}", i).parse().expect("Valid"),
                    AssetValueType::Quantity,
                )
                .finish_domain();
        }

        builder.build()
    }
    let mut peer = <TestPeer>::new().expect("Failed to create peer");
    let configuration = get_config(
        std::iter::once(peer.id.clone()).collect(),
        Some(get_key_pair()),
    );
    let rt = Runtime::test();
    let genesis = GenesisNetwork::from_configuration(
        true,
        generate_genesis(1_000_000_u32),
        &Some(configuration.genesis.clone()),
        &configuration.sumeragi.transaction_limits,
    )
    .expect("genesis creation failed");

    let builder = PeerBuilder::new()
        .with_into_genesis(genesis)
        .with_configuration(configuration);

    // This only submits the genesis. It doesn't check if the accounts
    // are created, because that check is 1) not needed for what the
    // test is actually for, 2) incredibly slow, making this sort of
    // test impractical, 3) very likely to overflow memory on systems
    // with less than 16GiB of free memory.
    rt.block_on(builder.start_with_peer(&mut peer));
}
