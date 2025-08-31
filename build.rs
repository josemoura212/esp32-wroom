fn main() {
    // Exporta variáveis de ambiente do ESP-IDF para a compilação
    embuild::espidf::sysenv::output();

    // Recarrega quando .env muda
    println!("cargo:rerun-if-changed=.env");

    // Carrega .env (se existir) e reexporta WIFI_* como rustc-env
    let _ = dotenvy::from_filename(".env");
    if let Ok(ssid) = std::env::var("WIFI_SSID") {
        println!("cargo:rustc-env=WIFI_SSID={}", ssid);
    }
    if let Ok(pass) = std::env::var("WIFI_PASSWORD") {
        println!("cargo:rustc-env=WIFI_PASSWORD={}", pass);
    }
}
