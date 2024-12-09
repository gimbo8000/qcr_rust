use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::ProgressBar;
use std::io::{self, Write};

fn get_color_chunk(url: &str, index: usize) -> (u8, u8, u8) {
    let chunk = url.chars().skip(index).take(3).chain(std::iter::repeat('\u{1A}')).take(3).collect::<Vec<_>>();
    (chunk[0] as u8, chunk[1] as u8, chunk[2] as u8)
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

fn create_snaking_qr_grid(url: &str) -> RgbImage {
    let url_length = url.len();
    let required_cells = (url_length + 2) / 3 + 3;
    let grid_size = (required_cells as f64).sqrt().ceil() as usize;
    let cell_size = 2;
    let img_size = (grid_size * cell_size) as u32;
    let mut img = ImageBuffer::new(img_size, img_size);

    let corner_colors = [(0, 0, Rgb([0, 255, 0])), (0, grid_size - 1, Rgb([255, 0, 0])), (grid_size - 1, 0, Rgb([255, 255, 0]))];
    let mut coordinates = Vec::new();
    for row in (0..grid_size).rev() {
        if (grid_size - 1 - row) % 2 == 0 {
            coordinates.extend((0..grid_size).rev().map(|col| (row, col)));
        } else {
            coordinates.extend((0..grid_size).map(|col| (row, col)));
        }
    }

    let pb = ProgressBar::new(coordinates.len() as u64);
    let mut index = 0;

    for (row, col) in coordinates {
        let color = if let Some(&(_, _, color)) = corner_colors.iter().find(|&&(r, c, _)| r == row && c == col) {
            color
        } else if index < url.len() {
            let chunk = get_color_chunk(url, index);
            index += 3;
            Rgb([chunk.0, chunk.1, chunk.2])
        } else {
            Rgb([26, 26, 26])
        };

        draw_rectangle(&mut img, color, (col * cell_size) as u32, (row * cell_size) as u32, cell_size as u32);
        pb.inc(1);
    }

    pb.finish_with_message("QR Grid generation complete");
    img
}

fn main() {
    print!("Enter a URL: ");
    io::stdout().flush().unwrap();
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    let grid_image = create_snaking_qr_grid(url.trim());
    grid_image.save("null_block_qr_grid.png").unwrap();
    println!("Image saved as null_block_qr_grid.png");
}
