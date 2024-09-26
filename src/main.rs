use clap::Parser;
use clap_num::maybe_hex;
use tracing::info;

use std::time::Duration;

use rppal::i2c::I2c;

use ssd1306::{prelude::*, size::DisplaySize128x64, I2CDisplayInterface};

use embedded_graphics::prelude::Point;
use embedded_graphics::{mono_font::iso_8859_1::*, prelude::Size};

mod screen_writer;
use screen_writer::ScreenWriter;

mod system;

const FRAME_DELAY: Duration = Duration::from_millis(50);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'b', long, default_value_t = 1)]
    i2c_bus: u8,
    #[arg(short = 'a', long, value_parser = maybe_hex::<u16>, default_value = "0x3C")]
    i2c_address: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let i2c = I2c::with_bus(args.i2c_bus)
        .and_then(|mut i2c| {
            i2c.set_slave_address(args.i2c_address)?;
            Ok(i2c)
        })
        .unwrap(); // Panics if fails

    let interface = I2CDisplayInterface::new(i2c);
    let mut writer = ScreenWriter::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)?;
    let mut sys = system::SysInfo::new();

    info!("Starting Raspi-Monitor");
    let mut frame: u16 = 0;
    let mut cycle = 0;
    let mut page = 0;
    sys.refresh();
    loop {
        if frame
            % (sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis() / FRAME_DELAY.as_millis()) as u16
            == 0
        {
            sys.refresh();
        }
        writer.clear()?;
        let res = match page {
            0 => render_page_1(&sys, &mut writer, frame),
            1 => render_page_2(&sys, &mut writer, frame),
            _ => unreachable!(),
        };
        match res {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error rendering page: {:?}", e);
            }
        }

        frame += 1;
        if frame == 10 {
            frame = 0;
            cycle += 1;
        }
        if cycle == 2 {
            cycle = 0;
            page = (page + 1) % 2;
        }
        writer.flush()?;

        std::thread::sleep(FRAME_DELAY);
    }
}

fn render_page_1(
    sys: &system::SysInfo,
    writer: &mut ScreenWriter<I2CInterface<I2c>, DisplaySize128x64>,
    frame: u16,
) -> anyhow::Result<()> {
    // Hostname
    // CPU, Mem, Temp
    writer.write_loading_icon(Point::new(128 - 9, 0), 9, frame)?;
    writer.write_text(
        // system::get_hostname()?.as_str(),
        system::SysInfo::hostname().as_str(),
        Point::new(0, 9),
        &FONT_7X14_BOLD,
    )?;
    writer.write_line(Point::new(0, 12), Point::new(128, 12))?;

    render_system_monitor(sys, writer)?;
    Ok(())
}

fn render_page_2(
    sys: &system::SysInfo,
    writer: &mut ScreenWriter<I2CInterface<I2c>, DisplaySize128x64>,
    frame: u16,
) -> anyhow::Result<()> {
    // Hostname
    // CPU, Mem, Temp
    writer.write_loading_icon(Point::new(128 - 9, 0), 9, frame)?;
    writer.write_text(
        // system::get_hostname()?.as_str(),
        sys.ip_addr().to_string().as_str(),
        Point::new(0, 9),
        &FONT_7X14_BOLD,
    )?;
    writer.write_line(Point::new(0, 12), Point::new(128, 12))?;

    render_system_monitor(sys, writer)?;
    Ok(())
}

fn render_system_monitor(
    sys: &system::SysInfo,
    writer: &mut ScreenWriter<I2CInterface<I2c>, DisplaySize128x64>,
) -> anyhow::Result<()> {
    render_bar_graph(
        writer,
        Point::new(0, 19),
        "CPU",
        |v| format!("{:>3.0}%", v),
        sys.cpu_usage(),
        100.0,
    )?;

    render_bar_graph(
        writer,
        Point::new(0, 29),
        "Mem",
        |v| format!("{:>3.0}%", v),
        sys.memory_usage(),
        100.0,
    )?;

    match sys.root_disk_usage() {
        Some(usage) => {
            render_bar_graph(
                writer,
                Point::new(0, 39),
                "Disk",
                |v| format!("{:>3.0}%", v),
                usage,
                100.0,
            )?;
        }
        None => {
            render_bar_graph(
                writer,
                Point::new(0, 39),
                "Disk",
                |_| "N/A".to_string(),
                0.0,
                100.0,
            )?;
        }
    }

    writer.write_text(
        format!("Up: {}", system::SysInfo::uptime()).as_str(),
        Point::new(0, 58),
        &FONT_6X10,
    )?;

    writer.write_text(
        format!("C: {:>2.0}°C", sys.cpu_temp()).as_str(),
        Point::new(84, 58),
        &FONT_6X10,
    )?;
    Ok(())
}

fn render_bar_graph(
    writer: &mut ScreenWriter<I2CInterface<I2c>, DisplaySize128x64>,
    top_left: Point,
    label: &str,
    value_format_func: impl Fn(f32) -> String,
    value: f32,
    max_value: f32,
) -> anyhow::Result<()> {
    writer.write_text(label, Point::new(top_left.x, top_left.y + 5), &FONT_6X10)?;
    writer.write_box(
        Point::new(top_left.x + 26, top_left.y),
        Size::new(((value / max_value) * 74.0) as u32, 4), // CPU throttles at 80°C, and throttles further at 85°C
    )?;
    writer.write_text(
        value_format_func(value).as_str(),
        Point::new(104, top_left.y + 5),
        &FONT_6X10,
    )?;
    Ok(())
}
