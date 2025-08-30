use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Size,
    mono_font::{ascii::FONT_6X13, ascii::FONT_7X13_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use esp_idf_svc::hal::{
    i2c::{I2cConfig, I2cDriver},
    units::Hertz,
};
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

pub struct Ui<'a> {
    pub display: Ssd1306<
        I2CInterface<I2cDriver<'a>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
}

impl<'a> Ui<'a> {
    pub fn new(
        gpi2c: esp_idf_svc::hal::i2c::I2C0,
        gpio5: esp_idf_svc::hal::gpio::Gpio5,
        gpio4: esp_idf_svc::hal::gpio::Gpio4,
    ) -> Self {
        let i2c_config = I2cConfig::new()
            .baudrate(Hertz(400_000))
            .sda_enable_pullup(true)
            .scl_enable_pullup(true);

        let i2c = I2cDriver::new(
            gpi2c,
            gpio5, // SDA - OLED_SDA conforme esquema
            gpio4, // SCL - OLED_SCL conforme esquema
            &i2c_config,
        )
        .unwrap();

        let interface = I2CDisplayInterface::new(i2c);

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        let _ = display.init();

        Ui { display }
    }

    pub fn show_dht(&mut self, temperature: f32, humidity: f32) -> anyhow::Result<()> {
        DrawTarget::clear(&mut self.display, BinaryColor::Off)
            .map_err(|e| anyhow::anyhow!("Display clear error: {:?}", e))?;

        Rectangle::new(Point::new(0, 0), Size::new(128, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Border draw error: {:?}", e))?;

        let text_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        let text_style_subtitle = MonoTextStyle::new(&FONT_6X13, BinaryColor::On);

        Text::with_baseline("DHT11 Sensor", Point::new(8, 3), text_style, Baseline::Top)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

        Rectangle::new(Point::new(5, 16), Size::new(118, 1))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Line draw error: {:?}", e))?;

        Text::with_baseline(
            &format!("Temperature: {}C", temperature),
            Point::new(8, 22),
            text_style_subtitle,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

        Text::with_baseline(
            &format!("Humidity: {}%", humidity),
            Point::new(8, 40),
            text_style_subtitle,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

        self.display
            .flush()
            .map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))
    }

    pub fn update_req(&mut self, count: u32, params: &str) -> anyhow::Result<()> {
        DrawTarget::clear(&mut self.display, BinaryColor::Off)
            .map_err(|e| anyhow::anyhow!("Display clear error: {:?}", e))?;

        Rectangle::new(Point::new(0, 0), Size::new(128, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Border draw error: {:?}", e))?;

        Rectangle::new(Point::new(5, 16), Size::new(118, 1))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Line draw error: {:?}", e))?;

        let text_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        let text_style_subtitle = MonoTextStyle::new(&FONT_6X13, BinaryColor::On);

        Text::with_baseline(
            &format!("Requests: {}", count),
            Point::new(8, 3),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

        Text::with_baseline(
            "Ultimo params:",
            Point::new(8, 22),
            text_style_subtitle,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

        // Quebra texto longo em múltiplas linhas
        let max_chars = 18; // Aprox. 18 caracteres por linha com FONT_6X10
        if params.len() > max_chars {
            let (first_line, second_line) = params.split_at(std::cmp::min(max_chars, params.len()));

            Text::with_baseline(
                first_line,
                Point::new(8, 35),
                text_style_subtitle,
                Baseline::Top,
            )
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;

            if !second_line.is_empty() {
                Text::with_baseline(
                    &second_line[..std::cmp::min(max_chars, second_line.len())],
                    Point::new(8, 46),
                    text_style_subtitle,
                    Baseline::Top,
                )
                .draw(&mut self.display)
                .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;
            }
        } else {
            Text::with_baseline(
                params,
                Point::new(8, 43),
                text_style_subtitle,
                Baseline::Top,
            )
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Display draw error: {:?}", e))?;
        }

        self.display
            .flush()
            .map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))
    }
}
