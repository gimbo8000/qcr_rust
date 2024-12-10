
use std::fs::File;
use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::ProgressBar;
use std::io::{self, BufWriter, Write};
use image::GenericImageView;

const NULL_BLOCK_CHAR: char = '\u{1A}';
fn get_color_chunk(url: &str, index: usize) -> (u8, u8, u8) {
    let chunk = url.chars().skip(index).take(3).chain(std::iter::repeat('\u{1A}')).take(3).collect::<Vec<_>>();
    (chunk[0] as u8, chunk[1] as u8, chunk[2] as u8)
}
fn get_chunk_from_color(color: [u8; 3]) -> String {
    let r = color[0] as char;
    let g = color[1] as char;
    let b = color[2] as char;
    format!("{}{}{}", r, g, b)
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

        draw_rectangle(&mut img, color, col as u32 * cell_size, row as u32 * cell_size, cell_size);
        pb.inc(1);
    }

    pb.finish_with_message("QR Grid generation complete");
    img
}
fn decode_snaking_qr_grid(image_path: &str) -> String {
    let img = image::open(image_path).expect("Failed to open image");
    let img_size = img.width();
    let cell_size = 2;
    let grid_size = img_size / cell_size;

    // Generate snaking coordinates
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

    // Corner positions to skip
    let corners = vec![(0, 0), (0, grid_size - 1), (grid_size - 1, 0)];
    let mut decoded_url = String::new();

    for (row, col) in coordinates {
        if !corners.contains(&(row, col)) {
            let x = col * cell_size + cell_size / 2;
            let y = row * cell_size + cell_size / 2;
            let color = img.get_pixel(x, y);
            let r = color[0];
            let g = color[1];
            let b = color[2];
            let chunk = get_chunk_from_color([r, g, b]);

            // Stop processing if NULL_BLOCK is encountered
            for char in chunk.chars() {
                if char == NULL_BLOCK_CHAR {
                    return decoded_url;
                }
                decoded_url.push(char);
            }
        }
    }

    decoded_url
}

fn main() {
    println!("Choose an option:");
    println!("1: encode");
    println!("2: decode");
    println!("3: Exit");

    let mut input = String::new();

    // Read user input
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    // Match the input and call the corresponding function
    match input.trim() {
        "1" => execute_first_action(),
        "2" => execute_second_action(),
        "3" => {
            println!("Exiting the program.");
            return;
        }
        _ => println!("Invalid choice! Please type 1, 2, or 3."),
    }
}

fn execute_first_action() {
    print!("Enter a URL: ");
    io::stdout().flush().unwrap();
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    print!("Enter cell size: ");
    io::stdout().flush().unwrap();
    let mut cell_size_input = String::new();
    io::stdin().read_line(&mut cell_size_input).unwrap();
    let grid_image = create_snaking_qr_grid(url.trim(), cell_size_input.trim().parse().unwrap());
    grid_image.save("null_block_qr_grid.png").unwrap();
    println!("Image saved as null_block_qr_grid.png");
}

fn execute_second_action() {
    let image_path = "null_block_qr_grid.png";
    let decoded_url = decode_snaking_qr_grid(image_path);
    let file = File::create("ABC.txt").expect("Unable to create file");
    let mut writer = BufWriter::new(file);
    writer.write_all(decoded_url.as_bytes()).expect("Unable to write data");
    println!("Written successfully");
}