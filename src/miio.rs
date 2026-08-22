use aes::{
    Aes128,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray},
};
use md5::{Digest, Md5};
use serde_json::{Value, json};
use std::{
    io,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{net::UdpSocket, time::timeout};

const PORT: u16 = 54321;
const MAGIC: [u8; 2] = [0x21, 0x31];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PowerReading {
    pub watts: f64,
}

#[derive(Debug)]
pub enum MiioError {
    Io(io::Error),
    Timeout,
    Protocol(String),
    Crypto(String),
}
impl std::fmt::Display for MiioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "network: {e}"),
            Self::Timeout => write!(f, "device timeout"),
            Self::Protocol(e) => write!(f, "protocol: {e}"),
            Self::Crypto(e) => write!(f, "crypto: {e}"),
        }
    }
}
impl std::error::Error for MiioError {}
impl From<io::Error> for MiioError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub async fn read_power(
    ip: IpAddr,
    token_hex: &str,
    _model: &str,
    siid: u32,
    piid: u32,
) -> Result<PowerReading, MiioError> {
    let token = hex::decode(token_hex.trim())
        .map_err(|_| MiioError::Crypto("token must be hexadecimal".into()))?;
    if token.len() != 16 {
        return Err(MiioError::Crypto("token must contain 16 bytes".into()));
    }
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(SocketAddr::new(ip, PORT)).await?;
    let hello = hello_packet();
    socket.send(&hello).await?;
    let mut response = [0u8; 2048];
    let size = timeout(Duration::from_secs(2), socket.recv(&mut response))
        .await
        .map_err(|_| MiioError::Timeout)??;
    if size < 32 || response[0..2] != MAGIC {
        return Err(MiioError::Protocol("invalid hello response".into()));
    }
    let mut device_id = [0u8; 4];
    device_id.copy_from_slice(&response[8..12]);
    let device_timestamp = u32::from_be_bytes(
        response[12..16]
            .try_into()
            .map_err(|_| MiioError::Protocol("hello timestamp missing".into()))?,
    );
    let payload = json!({
        "id": 1,
        "method": "get_properties",
        "params": [{"did": "prop", "siid": siid, "piid": piid}]
    });
    let packet = encode_request(
        &device_id,
        device_timestamp,
        &token,
        payload.to_string().as_bytes(),
    )?;
    socket.send(&packet).await?;
    let size = timeout(Duration::from_secs(3), socket.recv(&mut response))
        .await
        .map_err(|_| MiioError::Timeout)??;
    let body = decode_response(&response[..size], &device_id, &token)?;
    let value = body
        .get("result")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("value"))
        .and_then(Value::as_f64)
        .or_else(|| {
            body.get("result")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_i64)
                .map(|v| v as f64)
        })
        .ok_or_else(|| MiioError::Protocol("power property missing from response".into()))?;
    Ok(PowerReading { watts: value })
}

fn md5_bytes(parts: &[&[u8]]) -> [u8; 16] {
    let mut hasher = Md5::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

fn hello_packet() -> [u8; 32] {
    let mut packet = [0xff; 32];
    packet[0..2].copy_from_slice(&MAGIC);
    packet[2..4].copy_from_slice(&(32u16.to_be_bytes()));
    packet
}

fn encode_request(
    device_id: &[u8; 4],
    timestamp: u32,
    token: &[u8],
    payload: &[u8],
) -> Result<Vec<u8>, MiioError> {
    let key = md5_bytes(&[token]);
    let iv = md5_bytes(&[&key, token]);
    let encrypted = encrypt_cbc(payload, &key, &iv)?;
    let length = 32usize
        .checked_add(encrypted.len())
        .ok_or_else(|| MiioError::Protocol("packet too large".into()))?;
    let mut packet = vec![0u8; length];
    packet[0..2].copy_from_slice(&MAGIC);
    packet[2..4].copy_from_slice(&(length as u16).to_be_bytes());
    packet[8..12].copy_from_slice(device_id);
    packet[12..16].copy_from_slice(&timestamp.to_be_bytes());
    packet[32..].copy_from_slice(&encrypted);
    let checksum = md5_bytes(&[&packet[0..16], token, &packet[32..]]);
    packet[16..32].copy_from_slice(&checksum);
    Ok(packet)
}

fn decode_response(packet: &[u8], device_id: &[u8; 4], token: &[u8]) -> Result<Value, MiioError> {
    if packet.len() < 32 || packet[0..2] != MAGIC || &packet[8..12] != device_id {
        return Err(MiioError::Protocol("invalid response header".into()));
    }
    let checksum = md5_bytes(&[&packet[0..16], token, &packet[32..]]);
    if packet[16..32] != checksum {
        return Err(MiioError::Protocol("response checksum mismatch".into()));
    }
    let key = md5_bytes(&[token]);
    let iv = md5_bytes(&[&key, token]);
    let decrypted = decrypt_cbc(&packet[32..], &key, &iv)?;
    serde_json::from_slice(&decrypted)
        .map_err(|e| MiioError::Protocol(format!("invalid JSON: {e}")))
}

fn encrypt_cbc(input: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, MiioError> {
    let pad = 16 - (input.len() % 16);
    let mut data = input.to_vec();
    data.extend(std::iter::repeat_n(pad as u8, pad));
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut previous = *iv;
    let mut output = Vec::with_capacity(data.len());
    for chunk in data.chunks_exact_mut(16) {
        for i in 0..16 {
            chunk[i] ^= previous[i];
        }
        let block = GenericArray::from_mut_slice(chunk);
        cipher.encrypt_block(block);
        previous.copy_from_slice(block);
        output.extend_from_slice(block);
    }
    Ok(output)
}
fn decrypt_cbc(input: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, MiioError> {
    if input.is_empty() || input.len() % 16 != 0 {
        return Err(MiioError::Crypto("ciphertext is not block aligned".into()));
    }
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut previous = *iv;
    let mut output = Vec::with_capacity(input.len());
    for chunk in input.chunks_exact(16) {
        let mut block = *GenericArray::from_slice(chunk);
        cipher.decrypt_block(&mut block);
        for i in 0..16 {
            block[i] ^= previous[i];
        }
        previous.copy_from_slice(chunk);
        output.extend_from_slice(&block);
    }
    let pad = *output
        .last()
        .ok_or_else(|| MiioError::Crypto("empty plaintext".into()))? as usize;
    if !(1..=16).contains(&pad)
        || output.len() < pad
        || !output[output.len() - pad..]
            .iter()
            .all(|b| *b as usize == pad)
    {
        return Err(MiioError::Crypto("invalid padding".into()));
    }
    output.truncate(output.len() - pad);
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cbc_round_trip() {
        let key = [7u8; 16];
        let iv = [3u8; 16];
        let plain = b"hello miio";
        let enc = encrypt_cbc(plain, &key, &iv).unwrap();
        assert_eq!(decrypt_cbc(&enc, &key, &iv).unwrap(), plain);
    }

    #[test]
    fn hello_uses_miio_discovery_fill_bytes() {
        let hello = hello_packet();
        assert_eq!(&hello[..4], &[0x21, 0x31, 0x00, 0x20]);
        assert!(hello[4..].iter().all(|byte| *byte == 0xff));
    }

    #[test]
    fn request_uses_device_timestamp() {
        let packet = encode_request(&[1, 2, 3, 4], 0x1234_5678, &[7; 16], b"{}").unwrap();
        assert_eq!(&packet[12..16], &[0x12, 0x34, 0x56, 0x78]);
    }
}
