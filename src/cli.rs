use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "cpuminer_stratum_v2",
    version,
    about = "A CPU miner for Stratum V2 Protocol"
)]
pub struct CliArgs {
    #[arg(
        short,
        long,
        default_value = "localhost:3333",
        help = "Address of the mining server. <host:port>"
    )]
    pub address: String,

    #[arg(
        short,
        long,
        default_value_t = 10,
        help = "Timeout in seconds for the mining server connection."
    )]
    pub timeout: u64,

    #[arg(
        long,
        default_value = "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72",
        help = "Public key for the miner, used for authentication."
    )]
    pub public_key: String,
}
