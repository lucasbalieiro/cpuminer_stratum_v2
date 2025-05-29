use std::{net::ToSocketAddrs, time::Duration};

pub struct StratumV2Client;

impl StratumV2Client {
    pub fn connect(mining_server_address: &str, timeout: &u64, public_key: &str) {
        let addr = mining_server_address
            .to_socket_addrs()
            .expect("Invalid mining server address")
            .next()
            .expect("No valid address provided for mining server");

        tracing::info!(
            "Attempting to connect to mining server at {}",
            mining_server_address
        );

        let server_connection =
            std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(*timeout));

        let mut server_connection = match server_connection {
            Ok(stream) => {
                tracing::info!(
                    "Successfully connected to mining server at {}",
                    mining_server_address
                );
                stream
            }
            Err(e) => {
                tracing::error!("Failed to connect to mining server: {}", e);
                panic!(
                    "Could not connect to mining server at {}: {}",
                    mining_server_address, e
                );
            }
        };

        let _ = Self::perform_noise_handshake(&mut server_connection, public_key);

        tracing::info!("Noise handshake completed with mining server");
    }

    fn perform_noise_handshake(
        server_connection: &mut std::net::TcpStream,
        public_key: &str,
    ) -> noise_sv2::NoiseCodec {
        use bs58::decode;
        use noise_sv2::Initiator;
        use secp256k1::XOnlyPublicKey;
        use std::io::{Read, Write};

        // Parse the responder's (server's) public key from base58 to XOnlyPublicKey
        let public_key = decode(public_key)
            .with_check(None)
            .into_vec()
            .expect("Failed to decode public key from base58");
        let public_key = XOnlyPublicKey::from_slice(&public_key[2..])
            .expect("Failed to convert public key to XOnlyPublicKey");
        let mut initiator = Initiator::new(Some(public_key));

        // Step 0: initiator sends first handshake message
        let first_message = initiator
            .step_0()
            .expect("Initiator failed first step of handshake");
        server_connection
            .write_all(&first_message)
            .expect("Failed to send handshake message");

        // Step 1: initiator receives server's response
        let mut response = [0u8; 234];
        server_connection
            .read(&mut response)
            .expect("Failed to read handshake response");
        let mut second_message = [0u8; 234];
        second_message.copy_from_slice(&response[..234]);

        initiator
            .step_2(second_message)
            .expect("Initiator failed third step of handshake")
    }
}
