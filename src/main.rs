use image::{ImageBuffer, Rgb, RgbImage};
use image::GenericImageView;
use eframe::{egui, epi};

const NULL_BLOCK_CHAR: char = '\u{1A}';

fn get_color_chunk(url: &str, index: usize) -> (u8, u8, u8) {
    let chunk = url
        .chars()
        .skip(index)
        .take(3)
        .chain(std::iter::repeat(NULL_BLOCK_CHAR))
        .take(3)
        .collect::<Vec<_>>();
    (chunk[0] as u8, chunk[1] as u8, chunk[2] as u8)
}

fn get_chunk_from_color(color: [u8; 3]) -> String {
    format!("{}{}{}", color[0] as char, color[1] as char, color[2] as char)
}

fn draw_rectangle(img: &mut RgbImage, color: Rgb<u8>, x: u32, y: u32, size: u32) {
    for dy in 0..size {
        for dx in 0..size {
            let px_x = x + dx;
            let px_y = y + dy;
            if px_x < img.width() && px_y < img.height() {
                *img.get_pixel_mut(px_x, px_y) = color;
            }
        }
    }
}

fn create_snaking_qr_grid(url: &str, cell_size: u32) -> RgbImage {
    let url_length = url.len();
    let required_cells = (url_length + 2) / 3 + 3;
    let grid_size = (required_cells as f64).sqrt().ceil() as usize;
    let img_size = grid_size as u32 * cell_size;
    let mut img = ImageBuffer::new(img_size, img_size);

    let corner_colors = [
        (0, 0, Rgb([0, 255, 0])),
        (0, grid_size - 1, Rgb([255, 0, 0])),
        (grid_size - 1, 0, Rgb([255, 255, 0])),
    ];
    let mut coordinates = Vec::new();
    for row in (0..grid_size).rev() {
        if (grid_size - 1 - row) % 2 == 0 {
            coordinates.extend((0..grid_size).rev().map(|col| (row, col)));
        } else {
            coordinates.extend((0..grid_size).map(|col| (row, col)));
        }
    }

    let mut index = 0;
    for (row, col) in coordinates {
        let color = if let Some(&(_, _, color)) = corner_colors
            .iter()
            .find(|&&(r, c, _)| r == row && c == col)
        {
            color
        } else if index < url.len() {
            let chunk = get_color_chunk(url, index);
            index += 3;
            Rgb([chunk.0, chunk.1, chunk.2])
        } else {
            Rgb([NULL_BLOCK_CHAR as u8, NULL_BLOCK_CHAR as u8, NULL_BLOCK_CHAR as u8])
        };

        draw_rectangle(&mut img, color, col as u32 * cell_size, row as u32 * cell_size, cell_size);
    }

    img
}

fn decode_snaking_qr_grid(image_path: &str, cell_size: u32) -> String {
    let img = image::open(image_path).expect("Failed to open image");
    let img_size = img.width();
    let grid_size = img_size / cell_size;

    let mut coordinates = Vec::new();
    for row in (0..grid_size).rev() {
        if (grid_size - 1 - row) % 2 == 0 {
            for col in (0..grid_size).rev() {
                coordinates.push((row, col));
            }
        } else {
 for col in 0..grid_size {
                coordinates.push((row, col));
            }
        }
    }

    let mut url = String::new();
    for (row, col) in coordinates {
        let pixel = img.get_pixel(col * cell_size, row * cell_size);
        let color = pixel.0;
        if color[0] != NULL_BLOCK_CHAR as u8 {
            url.push(color[0] as char);
            url.push(color[1] as char);
            url.push(color[2] as char);
        }
    }

    url
}

#[derive(Default)]
struct CQRApp {
    url: String,
    cell_size: u32,
}

impl epi::App for CQRApp {
    fn name(&self) -> &str {
        "CQR Generator"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter URL:");
            ui.text_edit_singleline(&mut self.url);
            ui.label("Cell Size:");
            ui.add(egui::Slider::new(&mut self.cell_size, 2..=100).text("Cell Size"));
            if ui.button("Generate QR").clicked() {
                let img = create_snaking_qr_grid(&self.url, self.cell_size);
                let _path = "null_block_qr_grid.png";
                let _path = "null_block_qr_grid.png";
                img.save(_path).unwrap();
                ui.label("QR Code generated!");
            }
            if ui.button("Decode QR").clicked() {
                let decoded_url = decode_snaking_qr_grid("null_block_qr_grid.png", self.cell_size);
                ui.label(format!("Decoded URL: {}", decoded_url));
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(CQRApp::default()), options);
}
