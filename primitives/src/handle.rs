use std::{fmt::Debug, net::{IpAddr, Ipv4Addr, SocketAddr}};
use crate::{block::{Block, Header, Payload, PayloadHeader}, error::DecodeError, transaction::{Recovered, SignedTransaction}};

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
    RequestDataResponse(IpAddr, u16),
    RequestData,
    RequestDataResponseFinished,
    HandShake(u64, IpAddr, u16),
    Hello(u64, IpAddr, u16),
    RemovePeer(u64),
    BroadcastTransaction(SignedTransaction),
}

impl NetworkHandleMessage {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::PeerConnectionTest { peer: _ } => {
                let msg_type = 0x00 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = 0x00 as usize;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw
            }
            Self::NewTransaction(signed) => {
                let msg_type = 0x01 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = signed.encode();
                let payload_length = data.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
            Self::NewPayload(block) => {

                let msg_type = 0x02 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = block.encode();
                let payload_length= data.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
            Self::BroadcastBlock(block) => {
                let msg_type = 0x03 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = block.encode();
                let payload_length= data.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
            Self::RequestDataResponse(ip_addr, port) => {
                let msg_type = 0x04 as u8;
                let protocol_version = 0x00 as u8;
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let mut port = port.to_be_bytes().to_vec();
                let payload_length = ip_addr.len() + port.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut ip_addr);
                raw.append(&mut port);
                // ??? why should it be here ???
                dbg!(raw.len());
                raw
            }
            Self::RequestData => {
                let msg_type = 0x05 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = 0x00 as usize;
                let mut raw =vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw
            }
            Self::RequestDataResponseFinished => {
                let msg_type = 0x06 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = 0x00 as usize;
                let mut raw = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw
            }
            Self::HandShake(pid ,ip_addr, port) => {
                let msg_type = 0x07 as u8;
                let protocol_version = 0x00 as u8;
                let pid = pid.to_be_bytes();
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = pid.len() + ip_addr.len() + port.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&pid);
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            Self::Hello(pid, ip_addr, port) => {
                let msg_type = 0x08 as u8;
                let protocol_version = 0x00 as u8;
                let pid = pid.to_be_bytes();
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = pid.len() + ip_addr.len() + port.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&pid);
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            Self::RemovePeer(_pid) => {
                let raw = Vec::new();
                raw
            }
            Self::BroadcastTransaction(signed) => {
                let msg_type = 0x10 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = signed.encode();
                let payload_length= data.len();

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
        }
    }

    // First Byte: Message Type
    // Second Byte: Payload Length
    // Third Byte: Protocol Version
    // remains: Data
    pub fn decode(buf: &[u8], addr: SocketAddr) -> Result<Option<NetworkHandleMessage>, DecodeError>{
        if buf.len() < 3 {
            println!("Here1");
            return Ok(None);
        }

        let msg_type = buf[0];
        let protocol_version = buf[1];
        let mut payload_len_raw = [0u8; 8];
        payload_len_raw.copy_from_slice(&buf[2..10]);
        let payload_length = usize::from_be_bytes(payload_len_raw);

        if buf.len() < 10 + payload_length {
            eprintln!("Too short raw data.");
            return Ok(None);
        }

        if protocol_version > 0 {
            println!("Not proper protocol version.");
            return Ok(None);
        }

        let data = &buf[10..];

        match msg_type {
            // PeerConnectionTest
            0x00 => Ok(Some(NetworkHandleMessage::PeerConnectionTest{peer: addr})),
            // NewTransaction
            0x01 => {
                let (signed, _) = SignedTransaction::decode(&data.to_vec())?;
                Ok(Some(NetworkHandleMessage::NewTransaction(signed)))
            }
            // NewPayload
            0x02 => {
                let block = Block::decode(&data.to_vec())?;
                Ok(Some(NetworkHandleMessage::NewPayload(block)))
            }
            // RequestDataResponse
            0x04 => {
                if data.len() < 6 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                let ip_addr = IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[4..6]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                Ok(Some(NetworkHandleMessage::RequestDataResponse(ip_addr, port)))

            }
            // Handshake
            0x07 => {
                if data.len() < 14 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let pid = usize::from_be_bytes(arr);
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[8..12]);
                let ip_addr = IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[12..14]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                Ok(Some(NetworkHandleMessage::HandShake(pid as u64, ip_addr, port)))
            }
            // Hello
            0x08 => {
                if data.len() < 14 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let pid = usize::from_be_bytes(arr);
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[8..12]);
                let ip_addr = IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[12..14]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                Ok(Some(NetworkHandleMessage::Hello(pid as u64, ip_addr, port)))
            }
            _ => {
                println!("Here4");
                Ok(None)
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
    Payload(Payload),
    PoolIsEmpty,
}

#[derive(Debug)]
pub enum MinerHandleMessage {
    NewPayload(PayloadHeader),
}

#[derive(Debug)]
pub enum MinerResultMessage {
    MiningSuccess(Header),
}