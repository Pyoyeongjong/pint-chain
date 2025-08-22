use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use network::builder::NetworkConfig;
use node::builder::LaunchContext;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    address: IpAddr,

    #[arg(short, long, default_value_t = 9557)]
    port: u16,

    #[arg(short, long, default_value_t = 9558)]
    rpc_port: u16,
}

#[tokio::main]
async fn main() {
    println!("PintChain Node Launching starts.");

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    
    let args = Args::parse();
    let network_config = NetworkConfig::new(args.address, args.port, args.rpc_port);
    let launch_context = LaunchContext::new(network_config);

    let node = match launch_context.launch().await {
        Ok(node ) => node,
        Err(err) => {
            eprintln!("Failed to launch PintChain Node. {:?}", err);
            return;
        }
    };

    // dbg!(node);

    // Starts RPC Server

}
