use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use network::builder::NetworkConfig;
use node::{builder::LaunchContext, configs::BlockConfig};
use primitives::{transaction::SignedTransaction, types::Address};
use tokio::signal;
use transaction_pool::identifier::TransactionOrigin;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    address: IpAddr,

    #[arg(short, long, default_value_t = 33333)]
    port: u16,

    #[arg(short, long, default_value_t = 8888)]
    rpc_port: u16,

    #[arg(short, long, default_value_t = String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c"))] // pint
    miner_address: String,

    #[arg(short, long, default_value_t = false)]
    boot_node: bool,

    #[arg(short, long, default_value_t = false)]
    in_memory_db: bool,

    #[arg(short, long, default_value_t = String::from("boot_node"))]                                
    name: String,
}

// 28dcb1338b900419cd613a8fb273ae36e7ec2b1d pint
// 0534501c34f5a0f3fa43dc5d78e619be7edfa21a chain
// 08041f667c366ee714d6cbefe2a8477ad7488f10 apple
// b2aaaf07a29937c3b833dca1c9659d98a9569070 banana
// 28dcb1338b900419cd613a8fb273ae36e7ec2b1c
#[tokio::main]
async fn main() {
    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    let args = Args::parse(); 
    println!("({}) Try to launch PintChain Node.", args.name);

    let miner_address = Address::from_hex(args.miner_address).expect("Wrong miner address! Node is shut.");

    let mut network_config = NetworkConfig::new(args.address, args.port, args.rpc_port);
    network_config.boot_node.is_boot_node = args.boot_node;
    let block_config: BlockConfig = BlockConfig::new(miner_address);
    let launch_context = LaunchContext::new(network_config.clone(), block_config, args.in_memory_db);


    let node = match launch_context.launch().await {
        Ok(node ) => node,
        Err(err) => {
            eprintln!("Failed to launch PintChain Node. {:?}", err);
            return;
        }
    };

    println!("[ Name: {} ] PintChain Node launcing Ok.", args.name);

    if args.boot_node {
        // test code! initial transactions
        // pint apple fee 10 value 1000 nonce 0
        let tx = "0000000000000000000000000000000008041f667c366ee714d6cbefe2a8477ad7488f100000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e8e124cac1252a8595c4da5e4d810d231a68571e8b590da337c17a67980e9452ef4e4dbd0a4b7312bd778b5a28dde2e73d152c07a56c5cb246d84f2d6f6d5631aa00";
        let data = hex::decode(tx).unwrap();
        let (signed, _) = SignedTransaction::decode(&data).unwrap();

        if let Err(_e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
            eprintln!("Tx1 add failed");
        }

        // pint banana fee 10 value 1000 nonce 1
        let tx = "00000000000000000000000000000001b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e8c1f3d993c37465ba08cf75eecddb01b214f84d77be915543c47374ae22d4cc6b78354616140743272fd536194a866ad0bd3c6d2d3f4531ee52d3c6bad99b5d1a01";
        let data = hex::decode(tx).unwrap();
        let (signed, _) = SignedTransaction::decode(&data).unwrap();

        if let Err(_e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
            eprintln!("Tx2 add failed");
        }

        // chain banana fee 5 value 1000 nonce 0
        let tx = "00000000000000000000000000000000b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000003e806cc9be9a58dbba4fa5459512c6d5c3d100bbfcb71cfffb669037243babb0c8678077cba676a8eb659f35a148b551dfadaef085cccbac97729c5a743cab9eec901";
        let data = hex::decode(tx).unwrap();
        let (signed, _) = SignedTransaction::decode(&data).unwrap();

        if let Err(_e) = node.pool.add_transaction(TransactionOrigin::External, signed.into_recovered().unwrap()) {
            eprintln!("Tx3 add failed");
        }

        node.pool.print_pool();
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
