use std::{
    io::Write,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::sv2_messages::{MessageType, Protocol, SetupConnection};

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

        // TODO: I NEED to study this later cause i think im missing something part of the processes
        // Im doing the handshake, but Im not using this anymore in the connection process
        // and getting this error: Shutting down noise stream reader! AeadError(Error) from pleblottery server
        // ##############################################################################
        let mut noise_codec = Self::perform_noise_handshake(&mut server_connection, public_key);
        tracing::info!("Noise handshake completed with mining server");

        Self::send_setup_connection_message(&mut server_connection, &mut noise_codec);
        // ##############################################################################
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

    fn send_setup_connection_message(
        server_connection: &mut TcpStream,
        noise_codec: &mut noise_sv2::NoiseCodec,
    ) {
        let setup_connection_message = SetupConnection {
            protocol: Protocol::MiningProtocol,
            min_version: 2,
            max_version: 2,
            flags: 0, // TODO: need to understand what flags are needed here
            endpoint_host: server_connection.peer_addr().unwrap().ip().to_string(),
            endpoint_port: server_connection.peer_addr().unwrap().port(),
            vendor: "cpuminer_stratum_v2".to_string(),
            hardware_version: "HWv1.0".to_string(),
            firmware: "FWv1.0".to_string(),
            device_id: "balieiro_dev".to_string(),
        };

        let mut framed_message = SetupConnection::frame_message(
            0, // extension_type for mining protocol
            MessageType::SetupConnection,
            &setup_connection_message.to_bytes(),
        );

        // Encrypt the framed message using the noise codec
        noise_codec
            .encrypt(&mut framed_message)
            .expect("Failed to encrypt setup connection message");

        server_connection
            .write_all(&framed_message)
            .expect("Failed to send setup connection message");
        tracing::info!("Setup connection message sent to mining server");
    }
}
