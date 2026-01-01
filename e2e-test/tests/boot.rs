use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use e2e_test::{
    common::{create_key_pairs, create_signed},
    process::{NodeConfig, launch_test_node},
    rpc_client::{get_tx_from_rpc, send_tx_to_rpc},
};
use primitives::{
    transaction::Transaction,
    types::{Address, U256},
};

#[tokio::test]
async fn e2e_single_node_basic() {
    let boot_node = NodeConfig {
        name: String::from("Boot_node"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33333,
        rpc_port: 8888,
        miner_address: String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c"),
        boot_node: true,
    };
    let _ = launch_test_node(boot_node).await;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let (key_pint, _addr_pint) = create_key_pairs("pint".as_bytes());
    let (_key_apple, addr_apple) = create_key_pairs("banana".as_bytes());

    let tx = Transaction {
        chain_id: 0,
        nonce: 0,
        to: Address::from_byte(addr_apple.try_into().unwrap()),
        fee: 5,
        value: U256::from(1000),
    };

    let signed = create_signed(key_pint, tx);

    let _ = send_tx_to_rpc(signed.clone(), "http://127.0.0.1:8888")
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(15)).await;
    let encoded_hash = hex::encode(signed.hash.0.as_slice());
    let res = get_tx_from_rpc(encoded_hash, "http://127.0.0.1:8888").await;

    match res {
        Ok(tx) => {
            let tx_hash = tx.encode_for_signing();
            assert_eq!(tx_hash, signed.hash);
        }
        Err(e) => {
            println!("no tx\n {:?}", e);
        }
    }
}

#[tokio::test]
async fn e2e_multi_node_basic() {
    let boot_node = NodeConfig {
        name: String::from("Boot_node"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33333,
        rpc_port: 8888,
        miner_address: String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c"),
        boot_node: true,
    };
    let _ = launch_test_node(boot_node).await;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let node_a = NodeConfig {
        name: String::from("Node_A"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33334,
        rpc_port: 8889,
        miner_address: String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b20"),
        boot_node: false,
    };
    let _ = launch_test_node(node_a).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
}
