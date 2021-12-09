use borsh::{BorshDeserialize, BorshSerialize};
use chrono::{DateTime, Utc};
use near_network_primitives::types::{PeerInfo, ReasonForBan};
use near_primitives::time::Clock;
use near_primitives::utils::{from_timestamp, to_timestamp};

/// Status of the known peers.
#[derive(BorshSerialize, BorshDeserialize, Eq, PartialEq, Debug, Clone)]
pub enum KnownPeerStatus {
    Unknown,
    NotConnected,
    Connected,
    Banned(ReasonForBan, u64),
}

impl KnownPeerStatus {
    pub fn is_banned(&self) -> bool {
        match self {
            KnownPeerStatus::Banned(_, _) => true,
            _ => false,
        }
    }
}

/// Information node stores about known peers.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct KnownPeerState {
    pub peer_info: PeerInfo,
    pub status: KnownPeerStatus,
    pub first_seen: u64,
    pub last_seen: u64,
}

impl KnownPeerState {
    pub(crate) fn new(peer_info: PeerInfo) -> Self {
        KnownPeerState {
            peer_info,
            status: KnownPeerStatus::Unknown,
            first_seen: to_timestamp(Clock::utc()),
            last_seen: to_timestamp(Clock::utc()),
        }
    }

    pub(crate) fn last_seen(&self) -> DateTime<Utc> {
        from_timestamp(self.last_seen)
    }
}

impl TryFrom<Vec<u8>> for KnownPeerState {
    type Error = Box<dyn std::error::Error>;

    fn try_from(bytes: Vec<u8>) -> Result<KnownPeerState, Self::Error> {
        KnownPeerState::try_from_slice(&bytes).map_err(|err| err.into())
    }
}
