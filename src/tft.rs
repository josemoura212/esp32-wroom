use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565, prelude::WebColors};
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{AnyIOPin, Gpio13, Gpio14, Gpio26, Gpio27, Output, PinDriver},
    spi::{
        config::{Config as DevConfig, DriverConfig, Duplex},
        SpiDeviceDriver, SpiDriver,
    },
    units::Hertz,
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{Orientation, Rotation},
    Builder,
};

// Tipo concreto do display, mantido como alias para esconder a verbosidade.
type St7789Display = mipidsi::Display<
    SpiInterface<
        'static,
        SpiDeviceDriver<'static, &'static SpiDriver<'static>>,
        PinDriver<'static, Gpio27, Output>,
    >,
    ST7789,
    PinDriver<'static, Gpio26, Output>,
>;

#[allow(unused)]
pub struct Ui {
    pub display: St7789Display,
}

#[allow(unused)]
impl Ui {
    // Mapeamento de pinos do TFT nesta placa:
    // SCLK=GPIO14, MOSI=GPIO13, RST=GPIO26, DC=GPIO27
    pub fn new(
        spi2: esp_idf_svc::hal::spi::SPI2,
        sclk: Gpio14,
        mosi: Gpio13,
        rst: Gpio26,
        dc: Gpio27,
    ) -> Self {
        // 1) SPI host (sem MISO)
        let driver: &'static SpiDriver = Box::leak(Box::new(
            SpiDriver::new(
                spi2,
                sclk,
                mosi,
                Option::<AnyIOPin>::None, // sem MISO
                &DriverConfig::new(),
            )
            .expect("SPI driver init failed"),
        ));

        // 2) Dispositivo SPI em half‑duplex com clock moderado
        let dev_cfg = DevConfig::new()
            .duplex(Duplex::Half)
            .baudrate(Hertz(6_000_000));
        let spi_dev =
            SpiDeviceDriver::new(driver, Option::<AnyIOPin>::None, &dev_cfg).expect("SPI dev");

        // 3) Pinos de controle do display
        let dc = PinDriver::output(dc).expect("DC pin");
        let rst = PinDriver::output(rst).expect("RST pin");

        // 4) Interface SPI para o ST7789 (buffer pequeno)
        let buf: &'static mut [u8; 512] = Box::leak(Box::new([0u8; 512]));
        let di = SpiInterface::new(spi_dev, dc, buf);

        // 5) Inicialização do ST7789 no formato mais simples
        let mut display = Builder::new(ST7789, di)
            .display_size(240, 240)
            // Muitos ST7789 240x240 têm framebuffer 240x320 com offset de 80 linhas
            .display_offset(0, 80)
            .orientation(Orientation::new().rotate(Rotation::Deg0))
            .reset_pin(rst)
            .init(&mut Ets {})
            .expect("ST7789 init");

        // Cor inicial para validar que o TFT ligou.
        DrawTarget::clear(&mut display, Rgb565::CSS_BLUE).ok();

        Ui { display }
    }
}
