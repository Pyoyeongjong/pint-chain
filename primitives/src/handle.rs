use std::{fmt::Debug, net::SocketAddr};
use crate::{block::{Block, Header, Payload, PayloadHeader}, transaction::{Recovered, SignedTransaction}};

pub trait Handle: Send + Sync + std::fmt::Debug{
    type Msg: Send + Sync;

    fn send(&self, msg: Self::Msg);
}

#[derive(Debug)]
pub enum NetworkHandleMessage {
    PeerConnectionTest{
        peer: SocketAddr
    },
    NewTransaction(SignedTransaction),
    NewPayload(Block),
    BroadcastBlock(Block),
    UpdateData
}

impl NetworkHandleMessage {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::PeerConnectionTest { peer: _ } => {
                let msg_type = 0x00 as u8;
                let payload_length = 0x00 as u8;
                let protocol_version = 0x00 as u8;

                let raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw
            }
            Self::NewTransaction(signed) => {
                let msg_type = 0x01 as u8;
                let payload_length = 0x41 as u8; // 65
                let protocol_version = 0x00 as u8;
                let mut data = signed.encode();

                let mut raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw.append(&mut data);
                raw
            }
            Self::NewPayload(block) => {

                let msg_type = 0x02 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = block.encode();
                let payload_length: u8 = data.len() as u8;

                let mut raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw.append(&mut data);
                raw
            }
            Self::BroadcastBlock(block) => {
                let msg_type = 0x03 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = block.encode();
                let payload_length: u8 = data.len() as u8;
                let mut raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw.append(&mut data);
                raw
            }
            Self::UpdateData => {
                let msg_type = 0x04 as u8;
                let payload_length = 0x00 as u8;
                let protocol_version = 0x00 as u8;

                let raw: Vec<u8> = vec![msg_type, payload_length, protocol_version];
                raw
            }
        }
    }

    // First Byte: Message Type
    // Second Byte: Payload Length
    // Third Byte: Protocol Version
    // remains: Data
    pub fn decode(buf: &[u8], addr: SocketAddr) -> Option<NetworkHandleMessage>{
        if buf.len() < 3 {
            return None;
        }

        let msg_type = buf[0];
        let payload_length = buf[1] as usize;
        let protocol_version = buf[2];

        if buf.len() < 3 + payload_length {
            return None;
        }

        if protocol_version > 0 {
            return None;
        }

        let data = &buf[3..];

        match msg_type {
            0x00 => Some(NetworkHandleMessage::PeerConnectionTest{peer: addr}),
            0x01 => {
                let signed = match SignedTransaction::decode(&data.to_vec()) {
                    Ok((signed,_)) => signed,
                    Err(e) => {
                        return None
                    }
                };
                Some(NetworkHandleMessage::NewTransaction(signed))
            }
            _ => {
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum ConsensusHandleMessage {
    ImportBlock(Block),
    NewTransaction(Recovered),
}

#[derive(Debug)]
pub enum PayloadBuilderHandleMessage {
    BuildPayload,
    Stop,
}
#[derive(Debug)]
pub enum PayloadBuilderResultMessage {
    Payload(Payload)
}

#[derive(Debug)]
pub enum MinerHandleMessage {
    NewPayload(PayloadHeader),
}

pub enum MinerResultMessage {
    MiningSuccess(Header),
}