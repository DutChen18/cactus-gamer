use eframe::egui;
use std::error::Error;
use std::time::Duration;

const WIDTH: usize = 8;
const HEIGHT: usize = 4;
const CELL_SIZE: egui::Vec2 = egui::Vec2::new(64.0, 64.0);
const VENDOR_ID: u16 = 0x16C0;
const PRODUCT_ID: u16 = 0x05DC;
const MANUFACTURER_STRING: &str = "dutchen18@gmail.com";
const PRODUCT_STRING: &str = "Lights";

fn find_device() -> Result<Option<rusb::DeviceHandle<rusb::GlobalContext>>, Box<dyn Error>> {
    for device in rusb::devices()?.iter() {
        let device_desc = device.device_descriptor()?;

        if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
            let device_handle = device.open()?;

            if device_handle.read_manufacturer_string_ascii(&device_desc)? == MANUFACTURER_STRING
                && device_handle.read_product_string_ascii(&device_desc)? == PRODUCT_STRING
            {
                return Ok(Some(device_handle));
            }
        }
    }

    Ok(None)
}

fn main() -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(egui::Vec2::new(
            WIDTH as f32 * CELL_SIZE.x,
            HEIGHT as f32 * CELL_SIZE.y + 20.0,
        )),
        ..Default::default()
    };

    let device = find_device()?.unwrap();
    let mut rgb = [0; WIDTH * HEIGHT * 3];
    let mut edit_rgb = [0.0; 3];

    device.read_control(
        rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        ),
        0,
        0,
        0,
        &mut rgb,
        Duration::from_secs_f32(1.0),
    )?;

    eframe::run_simple_native(
        "haha brrr",
        native_options,
        move |ctx: &egui::Context, _frame: &mut eframe::Frame| {
            let input = ctx.input(Clone::clone);

            for file in input.raw.dropped_files {
                println!("{:?}", file);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                let painter = ui.painter();

                let response = ui.interact(
                    egui::Rect::EVERYTHING,
                    egui::Id::from("grid"),
                    egui::Sense::click_and_drag(),
                );

                if let Some(pos) = response.interact_pointer_pos() {
                    let x = (pos.x / CELL_SIZE.x) as usize;
                    let y = (pos.y / CELL_SIZE.y) as usize;
                    let i = y * WIDTH + x;

                    if i < WIDTH * HEIGHT {
                        rgb[i * 3 + 0] = (edit_rgb[1] * 255.0) as u8;
                        rgb[i * 3 + 1] = (edit_rgb[0] * 255.0) as u8;
                        rgb[i * 3 + 2] = (edit_rgb[2] * 255.0) as u8;
                    }

                    device
                        .write_control(
                            rusb::request_type(
                                rusb::Direction::Out,
                                rusb::RequestType::Vendor,
                                rusb::Recipient::Device,
                            ),
                            0,
                            0,
                            0,
                            &mut rgb,
                            Duration::from_secs_f32(1.0),
                        )
                        .unwrap();
                }

                for i in 0..WIDTH * HEIGHT {
                    let x = i % WIDTH;
                    let y = i / WIDTH;
                    let min = egui::Pos2::new(x as f32 * CELL_SIZE.x, y as f32 * CELL_SIZE.y);
                    let rgb = &rgb[i * 3..];

                    painter.rect_filled(
                        egui::Rect::from_min_size(min, CELL_SIZE),
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgb(rgb[1], rgb[0], rgb[2]),
                    );
                }
            });

            egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                ui.color_edit_button_rgb(&mut edit_rgb);
            });
        },
    )?;

    Ok(())
}
