mod routes;
mod ui;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    http::{
        server::{Configuration as HttpCfg, EspHttpServer},
        Method::Get,
    },
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use std::sync::{Arc, Mutex};

use crate::ui::Ui;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches(); // p/ patches do IDF
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

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
    server.fn_handler("/", Get, move |req| {
        routes::init_routes(req, Arc::clone(&count_clone), Arc::clone(&params_clone))
    })?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let count = *request_count.lock().unwrap();
        let params = last_params.lock().unwrap().clone();

        if let Err(e) = display.update(count, &params) {
            println!("Erro ao atualizar display: {:?}", e);
        }
    }
}
