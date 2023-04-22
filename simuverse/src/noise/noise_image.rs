use super::SimplexNoise;

static ZOOM: u32 = 8;

pub fn create_noise(width: usize, height: usize) -> Vec<u8> {
    let noise = SimplexNoise::new();

    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for x in 0..width {
        for y in 0..height {
            let value = noise
                .step(
                    ((x as f64 - (width as f64 / 2.0)) / ZOOM as f64),
                    ((y as f64 - (height as f64 / 2.0)) / ZOOM as f64),
                )
                .abs();
            pixels.push(((value * 0.5 + 0.5) * 255.) as u8);
            pixels.push(((value * 0.5 + 0.5) * 255.) as u8);
            pixels.push(((value * 0.5 + 0.5) * 255.) as u8);
            pixels.push(255. as u8);
        }
    }
    pixels
}
