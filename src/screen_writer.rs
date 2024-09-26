use embedded_graphics::primitives::{Arc, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use thiserror::Error;

use display_interface::DisplayError;
use ssd1306::prelude::*;
use ssd1306::{mode::BufferedGraphicsMode, Ssd1306};

use embedded_graphics::mono_font::{MonoFont, MonoTextStyleBuilder};
use embedded_graphics::prelude::*;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

pub struct ScreenWriter<DI, DS: DisplaySize> {
    display: Ssd1306<DI, DS, BufferedGraphicsMode<DS>>,
}

#[derive(Error, Debug)]
pub enum ScreenWriterError {
    #[error("Display initialization error: {:?}", _0)]
    Init(DisplayError),
    #[error("Error clearing display: {:?}", _0)]
    Clear(DisplayError),
    #[error("Error flushing display: {:?}", _0)]
    Flush(DisplayError),
    #[error("Error writing to screen: {:?}", _0)]
    Write(DisplayError),
}

type R<T> = Result<T, ScreenWriterError>;

impl<DI, DS> ScreenWriter<DI, DS>
where
    DI: WriteOnlyDataCommand, /* i2c interface*/
    DS: DisplaySize,
{
    pub fn new(
        i2c_interface: DI,
        size: DS,
        rotation: DisplayRotation,
    ) -> Result<Self, ScreenWriterError> {
        let mut ssd1306 = Ssd1306::new(i2c_interface, size, rotation).into_buffered_graphics_mode();
        ssd1306.init().map_err(ScreenWriterError::Init)?;
        ssd1306
            .clear(BinaryColor::Off)
            .map_err(ScreenWriterError::Clear)?;

        Ok(Self { display: ssd1306 })
    }

    pub fn write_text(
        &mut self,
        text: &str,
        pos: Point,
        font: &MonoFont,
    ) -> Result<(), ScreenWriterError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(font)
            .text_color(BinaryColor::On)
            .build();

        let text = Text::new(text, pos, text_style);
        text.draw(&mut self.display)
            .map_err(ScreenWriterError::Write)?;
        Ok(())
    }

    pub fn write_line(&mut self, start: Point, end: Point) -> R<()> {
        let line =
            Line::new(start, end).into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
        line.draw(&mut self.display)
            .map_err(ScreenWriterError::Write)?;
        Ok(())
    }

    pub fn write_box(&mut self, top_left: Point, size: Size) -> R<()> {
        let rect =
            Rectangle::new(top_left, size).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
        rect.draw(&mut self.display)
            .map_err(ScreenWriterError::Write)?;
        Ok(())
    }

    pub fn write_loading_icon(&mut self, top_left: Point, size: u32, offset: u16) -> R<()> {
        let arc = Arc::new(top_left, size, (offset as f32 * 36.0).deg(), 245.0.deg())
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
        arc.draw(&mut self.display)
            .map_err(ScreenWriterError::Write)?;

        Ok(())
    }

    pub fn flush(&mut self) -> R<()> {
        self.display.flush().map_err(ScreenWriterError::Flush)
    }

    pub fn clear(&mut self) -> R<()> {
        self.display
            .clear(BinaryColor::Off)
            .map_err(ScreenWriterError::Clear)
    }
}
