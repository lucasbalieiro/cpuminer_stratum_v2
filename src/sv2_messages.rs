#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Protocol {
    MiningProtocol = 0,
    JobDeclarationProtocol = 1,
    TemplateDistributionProtocol = 2,
}

#[repr(u8)]
pub enum MessageType {
    SetupConnection = 0,
    SetupConnectionSuccess = 1,
    SetupConnectionError = 2,
}

// https://stratumprotocol.org/specification/03-Protocol-Overview/#36-common-protocol-messages
pub struct SetupConnection {
    pub protocol: Protocol,
    pub min_version: u16, // the current minimum version of the protocol supported by the client
    pub max_version: u16, // the current maximum version of the protocol supported by the client
    pub flags: u32, //Flags indicating optional protocol features the client supports. Each protocol from protocol field as its own values/flags.
    pub endpoint_host: String, // hostname or IP address of the server
    pub endpoint_port: u16, // port number of the server
    //Device information section
    pub vendor: String,
    pub hardware_version: String,
    pub firmware: String,
    pub device_id: String, // unique identifier for the device as defined by the vendor
}

impl SetupConnection {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.protocol as u8);
        bytes.extend_from_slice(&self.min_version.to_le_bytes());
        bytes.extend_from_slice(&self.max_version.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend(self.endpoint_host.as_bytes());
        bytes.push(0); // null terminator for the string
        bytes.extend_from_slice(&self.endpoint_port.to_le_bytes());
        bytes.extend(self.vendor.as_bytes());
        bytes.push(0); // null terminator for the string
        bytes.extend(self.hardware_version.as_bytes());
        bytes.push(0); // null terminator for the string
        bytes.extend(self.firmware.as_bytes());
        bytes.push(0); // null terminator for the string
        bytes.extend(self.device_id.as_bytes());
        bytes.push(0); // null terminator for the string
        bytes
    }

    /// Frames a message according to https://stratumprotocol.org/specification/03-Protocol-Overview/#32-framing
    /// extension_type: Unique identifier of the extension (u16, little-endian)
    /// msg_type: Unique identifier of the protocol message (u8)
    /// payload: Message-specific payload (bytes)
    pub fn frame_message(extension_type: u16, msg_type: MessageType, payload: &[u8]) -> Vec<u8> {
        let mut framed = Vec::with_capacity(6 + payload.len());
        // extension_type (2 bytes, little-endian)
        framed.extend_from_slice(&extension_type.to_le_bytes());
        // msg_type (1 byte)
        framed.push(msg_type as u8);
        // msg_length (3 bytes, little-endian)
        let msg_length = payload.len();
        let msg_length_bytes = [
            (msg_length & 0xFF) as u8,
            ((msg_length >> 8) & 0xFF) as u8,
            ((msg_length >> 16) & 0xFF) as u8,
        ];
        framed.extend_from_slice(&msg_length_bytes);
        // payload
        framed.extend_from_slice(payload);
        framed
    }
}
