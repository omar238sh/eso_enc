use std::{
    error::Error,
    sync::{mpsc::sync_channel, Arc, Mutex},
    thread::{self},
};

use encrypt::{decrypt::DecryptSession, EncryptedMessage};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, peripherals::Peripherals},
    nvs::EspDefaultNvsPartition,
    wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi},
};
use generic_ec::{curves::Ed25519, SecretScalar};
use rand_core::OsRng;
use std::net::UdpSocket;

const AP_SSID: &str = "ESP32-Control";
const AP_PASS: &str = "12345678";
const UDP_PORT: u16 = 4210;
const CMD_GET_PUBKEY: &[u8] = b"GET_PUBKEY";

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: AP_SSID.try_into().unwrap(),
        password: AP_PASS.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        max_connections: 4,
        ..Default::default()
    }))?;

    wifi.start()?;
    println!("AP started — SSID: {AP_SSID}  PORT: {UDP_PORT}");

    type E = Ed25519;
    let decrypt_session: Arc<Mutex<DecryptSession<E>>> = Arc::new(Mutex::new(DecryptSession::new(
        SecretScalar::random(&mut OsRng),
    )));

    let (tx, rc) = sync_channel::<(EncryptedMessage<E>, std::net::SocketAddr)>(8);
    let session_worker = Arc::clone(&decrypt_session);
    let socket_worker = UdpSocket::bind(format!("0.0.0.0:{}", UDP_PORT + 1))?; // worker reply socket
    let mut driver = PinDriver::output(peripherals.pins.gpio2).unwrap();
    thread::Builder::new()
        .stack_size(20192)
        .spawn(move || loop {
            match rc.recv() {
                Ok((encrypted_message, peer)) => {
                    let decrypted = session_worker.lock().unwrap().decrypt(encrypted_message);
                    let msg = decrypted.to_string();
                    println!("Decrypted from {peer}: {msg}");
                    if msg == String::from("led") {
                        driver.set_high().unwrap();
                    } else if msg == String::from("close_led") {
                        driver.set_low().unwrap();
                    }
                    let _ = socket_worker.send_to(b"OK", peer);
                }
                Err(e) => {
                    println!("Channel closed: {e}");
                    break;
                }
            }
        })
        .unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{UDP_PORT}"))?;
    println!("Listening on UDP 0.0.0.0:{UDP_PORT}");

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, peer)) => {
                let data = &buf[..len];

                if data.is_empty() {
                    println!("Empty packet from {peer}, ignoring");
                    continue;
                }

                if data == CMD_GET_PUBKEY {
                    let public_key = decrypt_session.lock().unwrap().public_key();
                    let key_bytes: Vec<u8> = public_key.to_bytes(true).as_bytes().into();
                    let hex_key = hex::encode(&key_bytes);
                    match socket.send_to(hex_key.as_bytes(), peer) {
                        Ok(_) => println!("Public key sent to {peer}: {hex_key}"),
                        Err(e) => println!("Send error: {e}"),
                    }
                    continue;
                }

                match String::from_utf8(data.to_vec()) {
                    Ok(encoded) => {
                        println!("Received from {peer} ({len} bytes): {encoded}");
                        match EncryptedMessage::from_hex(&encoded) {
                            Ok(msg) => {
                                if tx.try_send((msg, peer)).is_err() {
                                    println!("Channel full, dropping message from {peer}");
                                }
                            }
                            Err(e) => println!("Decode error from {peer}: {e:?}"),
                        }
                    }
                    Err(e) => println!("UTF-8 error from {peer}: {e}"),
                }
            }
            Err(e) => println!("recv_from error: {e}"),
        }
    }
}
