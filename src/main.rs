mod routes;
mod ui;
// mod wifi;

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
use std::sync::{Arc, Mutex};
use std::{thread::sleep, time::Duration};

use crate::ui::Ui;

#[derive(Copy, Clone)]
enum ShowTime {
    DHT11,
    Requests,
}

fn main() {
    if let Err(e) = app() {
        log::error!("Application error: {e:?}");
        loop {
            sleep(Duration::from_secs(1));
        }
    }
}

fn app() -> anyhow::Result<()> {
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
    let ssid_str = option_env!("WIFI_SSID").unwrap();
    let password_str = option_env!("WIFI_PASSWORD").unwrap();

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid_str.try_into().unwrap(),
        password: password_str.try_into().unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;

    // --- HTTP Server ---
    let count_clone = Arc::clone(&request_count);
    let params_clone = Arc::clone(&last_params);

    // Estado compartilhado para alternar telas a partir do handler
    let ui_state = Arc::new(Mutex::new(ShowTime::DHT11));
    let ui_state_for_handler = Arc::clone(&ui_state);

    let mut server = EspHttpServer::new(&HttpCfg::default())?;
    server.fn_handler("/", Get, move |req| {
        if let Ok(mut st) = ui_state_for_handler.lock() {
            *st = ShowTime::Requests;
        }
        routes::init_routes(req, Arc::clone(&count_clone), Arc::clone(&params_clone))
    })?;

    loop {
        let show_time = *ui_state.lock().unwrap();
        match show_time {
            ShowTime::DHT11 => {
                for _ in 1..=3 {
                    match esp_idf_dht::read(&mut sensor) {
                        Ok([humidity, temperature]) => {
                            if let Err(e) = display.show_dht(temperature, humidity) {
                                log::warn!("Display show_dht error: {:?}", e);
                            }
                            break;
                        }
                        Err(_) => sleep(Duration::from_millis(2200)),
                    }
                }
            }
            ShowTime::Requests => {
                let count = *request_count.lock().unwrap();
                let params = last_params.lock().unwrap().clone();

                if let Err(e) = display.update_req(count, &params) {
                    log::warn!("Display update_req error: {:?}", e);
                }

                if let Ok(mut st) = ui_state.lock() {
                    sleep(Duration::from_secs(10));
                    *st = ShowTime::DHT11;
                }
            }
        }

        sleep(Duration::from_secs(1));
    }
}
