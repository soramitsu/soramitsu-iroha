//! Iroha - A simple, enterprise-grade decentralized ledger.

pub mod block;
pub mod block_sync;
pub mod genesis;
pub mod kura;
pub mod modules;
pub mod queue;
pub mod smartcontracts;
pub mod sumeragi;
pub mod tx;
pub mod wsv;

use std::time::Duration;

use iroha_data_model::prelude::*;
use parity_scale_codec::{Decode, Encode};
use tokio::sync::broadcast;

use crate::{
    block_sync::message::VersionedMessage as BlockSyncMessage, prelude::*,
    sumeragi::message::VersionedMessage as SumeragiMessage,
};

/// The interval at which sumeragi checks if there are tx in the `queue`.
pub const TX_RETRIEVAL_INTERVAL: Duration = Duration::from_millis(100);

/// Specialized type of Iroha Network
pub type IrohaNetwork = iroha_p2p::Network<NetworkMessage>;

/// Ids of peers.
pub type PeersIds = dashmap::DashSet<<Peer as Identifiable>::Id>;

/// Provides an API to work with collection of key([`DomainId`]) - value([`Domain`]) pairs.
pub type DomainsMap = dashmap::DashMap<<Domain as Identifiable>::Id, Domain>;

/// `RolesMap` provides an API to work with collection of key(`PeerId`) - value(`Role`) pairs.
pub type RolesMap = dashmap::DashMap<<Role as Identifiable>::Id, Role>;

/// Type of `Sender<Event>` which should be used for channels of `Event` messages.
pub type EventsSender = broadcast::Sender<Event>;
/// Type of `Receiver<Event>` which should be used for channels of `Event` messages.
pub type EventsReceiver = broadcast::Receiver<Event>;

/// The network message
#[derive(Clone, Debug, Encode, Decode, iroha_actor::Message)]
pub enum NetworkMessage {
    /// Blockchain message
    SumeragiMessage(Box<SumeragiMessage>),
    /// Block sync message
    BlockSync(Box<BlockSyncMessage>),
    /// Health check message
    Health,
}

/// Check to see if the given item was included in the blockchain.
pub trait IsInBlockchain {
    /// Checks if this item has already been committed or rejected.
    fn is_in_blockchain(&self, wsv: &WorldStateView) -> bool;
}

pub mod prelude {
    //! Re-exports important traits and types. Meant to be glob imported when using `Iroha`.

    #[doc(inline)]
    pub use iroha_crypto::{Algorithm, Hash, KeyPair, PrivateKey, PublicKey};

    #[doc(inline)]
    pub use crate::{
        block::{
            CommittedBlock, PendingBlock, ValidBlock, VersionedCommittedBlock, VersionedValidBlock,
            DEFAULT_CONSENSUS_ESTIMATION_MS,
        },
        smartcontracts::permissions::{
            builder::Validator as ValidatorBuilder, combinators::AllowAll,
        },
        smartcontracts::ValidQuery,
        tx::{
            AcceptedTransaction, ValidTransaction, VersionedAcceptedTransaction,
            VersionedValidTransaction,
        },
        wsv::{World, WorldStateView},
        IsInBlockchain,
    };
}
