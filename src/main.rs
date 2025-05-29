use clap::Parser;
use cpuminer_stratum_v2::{cli::CliArgs, miner::BlockHeader, stratum_v2};

pub trait WorkProvider {
    fn get_work(&self) -> BlockHeader;
}

pub struct LocalWorkProvider;

fn main() {
    let cli_args = CliArgs::parse();
    tracing_subscriber::fmt::init();
    tracing::info!("Starting cpuminer_stratum_v2 with args: {:?}", cli_args);

    stratum_v2::StratumV2Client::connect(
        &cli_args.address,
        &cli_args.timeout,
        &cli_args.public_key,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpuminer_stratum_v2::miner::Miner;

    impl WorkProvider for LocalWorkProvider {
        fn get_work(&self) -> BlockHeader {
            BlockHeader {
                version: u8::from_str_radix("00000001", 16).unwrap(),
                previous_block_hash: hex::decode(
                    "6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                merkle_root: hex::decode(
                    "982051fd1e4ba744bbbe680e1fee14677ba1a3c3540bf7b1cdb606e857233e0e",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                timestamp: u32::from_str_radix("4966bc61", 16).unwrap(),
                bits: u32::from_str_radix("1d00ffff", 16).unwrap(),
                nonce: u32::from_str_radix("9962e301", 16).unwrap(),
            }
        }
    }

    #[test]
    fn test_local_work_provider_mining() {
        let work_provider = LocalWorkProvider;
        let block = work_provider.get_work();
        // This will likely not find a valid nonce quickly due to high difficulty,
        // but we can at least check that mining runs and returns a result type.
        // For a real test, use a lower difficulty bits value.
        let (_nonce, _block_hash) = Miner::mine(block);
    }

    // Placeholder for future Stratum V2 integration test
    // #[test]
    // fn test_stratum_v2_connection() {
    //     // TODO: Implement connection and job retrieval from Stratum V2 server
    // }
}
