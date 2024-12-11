#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![allow(incomplete_features)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::config::Config;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_time::{Duration, Instant, Ticker};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

fn scale_brightness(value: u8) -> u8 {
    // Scale to 10% brightness
    ((value as u16 * 25) / 255) as u8
}

const NUM_LEDS: usize = 22;
const ROW_SIZES: &[usize] = &[6, 5, 4, 3, 2, 1, 1]; // Last 1 is the star

fn get_row_start_index(row: usize) -> usize {
    ROW_SIZES.iter().take(row).sum()
}

async fn rows_animation<'d, P: embassy_rp::pio::Instance, const S: usize>(
    ws2812: &mut PioWs2812<'d, P, S, NUM_LEDS>,
) {
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(300));
    let green = RGB8::new(0, scale_brightness(255), 0);
    let star_color = RGB8::new(scale_brightness(255), scale_brightness(255), 0);

    // Light up rows from bottom to top, 3 times
    for _ in 0..3 {
        // Clear all LEDs
        for led in data.iter_mut() {
            *led = RGB8::default();
        }
        ws2812.write(&data).await;
        ticker.next().await;

        // Light each row
        for row in 0..ROW_SIZES.len() - 1 {
            // Exclude star
            let start_idx = get_row_start_index(row);
            for i in 0..ROW_SIZES[row] {
                data[start_idx + i] = green;
            }
            ws2812.write(&data).await;
            ticker.next().await;
        }

        // Make star twinkle
        for _ in 0..3 {
            // Star on
            data[NUM_LEDS - 1] = star_color;
            ws2812.write(&data).await;
            ticker.next().await;

            // Star off
            data[NUM_LEDS - 1] = RGB8::default();
            ws2812.write(&data).await;
            ticker.next().await;
        }
    }
}

async fn twinkle_animation<'d, P: embassy_rp::pio::Instance, const S: usize>(
    ws2812: &mut PioWs2812<'d, P, S, NUM_LEDS>,
) {
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(100));
    let green = RGB8::new(0, scale_brightness(255), 0);
    let star_color = RGB8::new(scale_brightness(255), scale_brightness(255), 0);

    // Twinkle for about 5 seconds
    for _ in 0..50 {
        let time = Instant::now().as_micros();

        // Update tree LEDs (excluding star)
        for i in 0..NUM_LEDS - 1 {
            let random_val = ((time + i as u64 * 7919) % 5) as u8;
            data[i] = if random_val == 0 {
                green
            } else {
                RGB8::default()
            };
        }

        // Star always twinkles with a different pattern
        let star_random = (time % 3) as u8;
        data[NUM_LEDS - 1] = if star_random == 0 {
            star_color
        } else {
            RGB8::default()
        };

        ws2812.write(&data).await;
        ticker.next().await;
    }
}

async fn sparkle_animation<'d, P: embassy_rp::pio::Instance, const S: usize>(
    ws2812: &mut PioWs2812<'d, P, S, NUM_LEDS>,
) {
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(50));
    let star_color = RGB8::new(scale_brightness(255), scale_brightness(220), 0); // Warm yellow for star
    let colors: [RGB8; 4] = [
        RGB8::new(scale_brightness(255), 0, 0), // Red
        RGB8::new(0, scale_brightness(255), 0), // Green
        RGB8::new(0, 0, scale_brightness(255)), // Blue
        RGB8::new(
            scale_brightness(255),
            scale_brightness(255),
            scale_brightness(255),
        ), // White
    ];

    // Keep star lit
    data[NUM_LEDS - 1] = star_color;

    // Store target colors for each LED
    let mut target_colors: [RGB8; NUM_LEDS - 1] = [RGB8::default(); NUM_LEDS - 1];
    // Store brightness factors for dimming effect (0-100%)
    let mut brightness_factors: [u8; NUM_LEDS - 1] = [100; NUM_LEDS - 1];
    // Store timestamps for when LEDs started dimming
    let mut dim_start_times: [u64; NUM_LEDS - 1] = [0; NUM_LEDS - 1];

    // Sparkle for about 5 seconds
    for _ in 0..500 {
        let time = Instant::now().as_micros();

        // Update each LED individually
        for i in 0..NUM_LEDS - 1 {
            let random_val = ((time + i as u64 * 7919) % 10) as u8;
            let color_index = (time + i as u64 * 1234) % 4;

            // Randomly start dimming effect
            let dim_random = ((time + i as u64 * 13331) % 50) as u8;
            if dim_random == 0 && brightness_factors[i] == 100 {
                brightness_factors[i] = 20; // Dim to 20%
                dim_start_times[i] = time;
            }

            // Gradually restore brightness after 200ms
            if brightness_factors[i] < 100 && time > dim_start_times[i] + 200_000 {
                brightness_factors[i] = brightness_factors[i].saturating_add(5);
            }

            if random_val == 0 {
                // Set new target color
                target_colors[i] = colors[color_index as usize];
            }

            // Transition current color towards target color
            if data[i] != target_colors[i] {
                if data[i].r < target_colors[i].r {
                    data[i].r = data[i].r.saturating_add(5);
                } else if data[i].r > target_colors[i].r {
                    data[i].r = data[i].r.saturating_sub(5);
                }
                if data[i].g < target_colors[i].g {
                    data[i].g = data[i].g.saturating_add(5);
                } else if data[i].g > target_colors[i].g {
                    data[i].g = data[i].g.saturating_sub(5);
                }
                if data[i].b < target_colors[i].b {
                    data[i].b = data[i].b.saturating_add(5);
                } else if data[i].b > target_colors[i].b {
                    data[i].b = data[i].b.saturating_sub(5);
                }
            }

            // Apply brightness factor
            data[i].r = (data[i].r as u16 * brightness_factors[i] as u16 / 100) as u8;
            data[i].g = (data[i].g as u16 * brightness_factors[i] as u16 / 100) as u8;
            data[i].b = (data[i].b as u16 * brightness_factors[i] as u16 / 100) as u8;
        }

        ws2812.write(&data).await;
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn run_main_strip(mut ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>) {
    loop {
        sparkle_animation(&mut ws2812).await;
        rows_animation(&mut ws2812).await;
        twinkle_animation(&mut ws2812).await;
    }
}

#[embassy_executor::task]
async fn run_sky_stars(mut ws2812: PioWs2812<'static, PIO0, 1, NUM_LEDS>) {
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(100));
    let star_color = RGB8::new(
        scale_brightness(255),
        scale_brightness(255),
        scale_brightness(255),
    );

    loop {
        let time = Instant::now().as_micros();

        // Random twinkle pattern for sky stars
        for i in 0..NUM_LEDS {
            let random_val = ((time + i as u64 * 7919) % 5) as u8;
            data[i] = if random_val == 0 {
                star_color
            } else {
                RGB8::default()
            };
        }

        ws2812.write(&data).await;
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = Config::default();
    let p = embassy_rp::init(config);

    let Pio {
        mut common,
        sm0,
        sm1,
        ..
    } = Pio::new(p.PIO0, Irqs);

    // Initialize the first LED strip on PIN_16 (Christmas tree)
    let program1 = PioWs2812Program::new(&mut common);
    let ws2812_1 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16, &program1);

    // Initialize the second LED strip on PIN_17 (Sky stars)
    let program2 = PioWs2812Program::new(&mut common);
    let ws2812_2 = PioWs2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_17, &program2);

    // Spawn both LED strip tasks
    spawner.spawn(run_main_strip(ws2812_1)).unwrap();
    spawner.spawn(run_sky_stars(ws2812_2)).unwrap();

    loop {
        embassy_time::Timer::after(Duration::from_secs(1)).await;
    }
}
