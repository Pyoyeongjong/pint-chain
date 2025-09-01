use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use network::builder::NetworkConfig;
use node::{builder::LaunchContext, configs::BlockConfig};
use primitives::{handle::ConsensusHandleMessage, transaction::SignedTransaction, types::Address};
use tokio::signal;
use transaction_pool::identifier::TransactionOrigin;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    address: IpAddr,

    #[arg(short, long, default_value_t = 30303)]
    port: u16,

    #[arg(short, long, default_value_t = 8545)]
    rpc_port: u16,

    #[arg(short, long, default_value_t = String::from("0101010101010101010101010101010101010101"))]
    miner_address: String,

    #[arg(short, long, default_value_t = true)]
    boot_node: bool,
}

#[tokio::main]
async fn main() {
    println!("PintChain Node Launching starts.");

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    let args = Args::parse();
    let miner_address = Address::from_hex(args.miner_address).expect("Wrong miner address! Node is shut.");

    let network_config = NetworkConfig::new(args.address, args.port, args.rpc_port);
    let block_config = BlockConfig::new(miner_address);
    let launch_context = LaunchContext::new(network_config.clone(), block_config);

    let node = match launch_context.launch().await {
        Ok(node ) => node,
        Err(err) => {
            eprintln!("Failed to launch PintChain Node. {:?}", err);
            return;
        }
    };

    // dbg!(node);
    println!("PintChain Node Launcing Ok.");

    // test code!
    // pint apple fee 10/ value 1000 nonce 0
    let tx = "0000000000000000000000000000000008041f667c366ee714d6cbefe2a8477ad7488f100000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e8f85a817ce9e8ea5613f45724da453c3373eac386865cc1c6d692a9b0bc5663d024072de9971987f43853ec23edd2ee1fdc11241704a20476f79d5dc8124a779101";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
        eprintln!("Tx1 add failed");
    }

    // pint banana fee 10/ value 1000 nonce 1
    let tx = "00000000000000000000000000000001b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e800cb4bb629218b69617e717922e3e6829f378f10c427d0520c5c09650f48e6080786124947aba090cf93b9354df89571f0a25a9cad4204c5b7949e5d64d1303501";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
        eprintln!("Tx2 add failed");
    }

    // chain banana fee 10/ value 1000
    let tx = "00000000000000000000000000000000b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000003e8da9360adcd9911fe5e2be9656155bfe54fe38ca730bc6f23a38ce55e807cfb817f056cc6a54439ed119bdc246a384e016766b40512ae6df13c1de820a20f66d001";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
        eprintln!("Tx3 add failed");
    }


    // Starts RPC Server
    // Graceful shutdown
    tokio::select! {
        _ = node.run_rpc(network_config) => {},
        _ = signal::ctrl_c() => {
            println!("Ctrl_C: Gracefully shutdown Node..")
        }
    }

}
