mod ui;

use crate::ui::Ui;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::http::server::{Configuration as HttpCfg, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches(); // p/ patches do IDF
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop: esp_idf_svc::eventloop::EspEventLoop<esp_idf_svc::eventloop::System> =
        EspSystemEventLoop::take()?;
    let nvs: esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault> =
        EspDefaultNvsPartition::take()?;

    let mut display = Ui::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    );

    // Contadores compartilhados
    let request_count = Arc::new(Mutex::new(0u32));
    let last_params = Arc::new(Mutex::new(String::from("Nenhum")));

    // Display inicial
    display.update(0, "Nenhum")?;

    // --- Wi-Fi Station ---
    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "CONECTE-SE SE FOR CAPAZ".try_into().unwrap(),
        password: "jose.258".try_into().unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;

    // --- HTTP Server ---
    let count_clone = Arc::clone(&request_count);
    let params_clone = Arc::clone(&last_params);

    let mut server = EspHttpServer::new(&HttpCfg::default())?;
    server.fn_handler(
        "/",
        esp_idf_svc::http::Method::Get,
        move |req| -> Result<(), anyhow::Error> {
            // Incrementa contador
            let mut count = count_clone.lock().unwrap();
            *count += 1;
            let current_count = *count;
            drop(count);

            // Extrai parâmetros
            let params = req.uri().split('?').nth(1).unwrap_or("Nenhum").to_string();

            // Atualiza último parâmetro
            {
                let mut last = params_clone.lock().unwrap();
                *last = params.clone();
            }

            println!("Request #{}: {}", current_count, params);

            let mut resp = req.into_ok_response()?;
            resp.write(format!("Request #{} - Params: {}", current_count, params).as_bytes())?;
            Ok(())
        },
    )?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let count = *request_count.lock().unwrap();
        let params = last_params.lock().unwrap().clone();

        if let Err(e) = display.update(count, &params) {
            println!("Erro ao atualizar display: {:?}", e);
        }
    }
}
