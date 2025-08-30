mod routes;
mod ui;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, prelude::Peripherals},
    http::{
        server::{Configuration as HttpCfg, EspHttpServer},
        Method::Get,
    },
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crate::ui::Ui;

enum ShowTime {
    DHT11,
    Requests,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    sleep(Duration::from_secs(2));

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let pins = peripherals.pins;

    let mut display = Ui::new(peripherals.i2c0, pins.gpio5, pins.gpio4);

    // Inicializa DHT11 no pino GPIO16
    let mut sensor = PinDriver::input_output_od(pins.gpio16).unwrap();

    // Contadores compartilhados
    let request_count = Arc::new(Mutex::new(0u32));
    let last_params = Arc::new(Mutex::new(String::from("Nenhum")));

    // Display inicial
    display.update_req(0, "Nenhum")?;

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

    let mut show_time = ShowTime::DHT11;

    loop {
        match show_time {
            ShowTime::DHT11 => {
                let [humidity, temperature] = esp_idf_dht::read(&mut sensor)
                    .map_err(|e| anyhow::anyhow!("Failed to read DHT11 sensor: {:?}", e))?;

                display.show_dht(temperature, humidity)?;

                show_time = ShowTime::Requests
            }
            ShowTime::Requests => {
                let count = *request_count.lock().unwrap();
                let params = last_params.lock().unwrap().clone();

                display.update_req(count, &params)?;

                show_time = ShowTime::DHT11
            }
        }

        sleep(Duration::from_secs(2));
    }
}
