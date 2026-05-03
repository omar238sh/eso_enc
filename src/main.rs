use std::{error::Error, thread::sleep, time::Duration};

use esp32_nimble::{uuid128, BLEDevice, NimbleProperties};

mod cmdash;
mod control;
fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let ble = BLEDevice::take();
    let server = ble.get_server();

    let service_uuid = uuid128!("ec7e994a-166d-467d-b749-ada22691bdec");
    let rx_characteristic_uuid = uuid128!("d99e7e56-c326-4b83-ba73-c28f11b3ec9a");
    let service = server.create_service(service_uuid);

    let rx = service.lock().create_characteristic(
        rx_characteristic_uuid,
        NimbleProperties::READ | NimbleProperties::WRITE | NimbleProperties::NOTIFY,
    );

    let adv = ble.get_advertising();
    let mut config = esp32_nimble::BLEAdvertisementData::new();

    adv.lock()
        .set_data(config.name("ESP32 Control").add_service_uuid(service_uuid))?;

    adv.lock().start()?;
    rx.lock().on_write(|data| {
        let message = String::from_utf8_lossy(data.recv_data());
        println!("{message}",);
    });
    rx.lock().on_read(|b, _| {
        b.set_value(b"My name is ESP32 Control");
    });
    loop {
        sleep(Duration::from_millis(50));
    }
}
