extern crate minifb;

mod vm;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use vm::VM;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    let mut args = std::env::args();
    let xname = args.next().unwrap();

    let rom_file = match args.next() {
        Some(file) => file,
        None => {
            eprintln!("Usage:\n\t{} <rom> [BG Color] [FG Color]", xname);
            std::process::exit(1);
        }
    };

    let mut window = Window::new(
        &{
            let wname = match rom_file.find('.') {
                Some(i) => rom_file[..i].to_string(),
                None => rom_file.clone(),
            };

            match wname.rfind('/') {
                Some(i) => wname[i + 1..].to_string(),
                None => wname,
            }
        },
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: true,
            resize: true,
            scale: Scale::X16,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut vm = VM::new(&rom_file);

    let color = [
        args.next()
            .as_ref()
            .map(|color| u32::from_str_radix(color, 16).unwrap_or(0x333333))
            .unwrap_or(0x333333),
        args.next()
            .as_ref()
            .map(|color| u32::from_str_radix(color, 16).unwrap_or(0x333333))
            .unwrap_or(0xEEEEEE),
    ];

    // 600 Hz
    window.limit_update_rate(Some(std::time::Duration::from_nanos(1_666_667)));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut wsize = (WIDTH, HEIGHT);

    let mut ticks = 0;

    loop {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        ticks += 1;

        if ticks == 10 {
            vm.tick();
            ticks = 0;
        }

        vm.update_keys([
            window.is_key_down(Key::V) && window.is_key_down(Key::Space),
            window.is_key_down(Key::E),
            window.is_key_down(Key::R),
            window.is_key_down(Key::T),
            window.is_key_down(Key::D),
            window.is_key_down(Key::F),
            window.is_key_down(Key::G),
            window.is_key_down(Key::C) && !window.is_key_down(Key::Space),
            window.is_key_down(Key::V) && !window.is_key_down(Key::Space),
            window.is_key_down(Key::B) && !window.is_key_down(Key::Space),
            window.is_key_down(Key::C) && window.is_key_down(Key::Space),
            window.is_key_down(Key::B) && window.is_key_down(Key::Space),
            window.is_key_down(Key::Y),
            window.is_key_down(Key::H),
            window.is_key_down(Key::N) && !window.is_key_down(Key::Space),
            window.is_key_down(Key::N) && window.is_key_down(Key::Space),
        ]);

        vm.cycle();

        let mut draw = vm.is_drawing();

        let size = window.get_size();

        if size != wsize {
            wsize = size;
            draw = true;
        }

        if draw {
            let pixels = vm.get_fb();

            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    buffer[x + WIDTH * y] = color[(pixels[y] >> (63 - x) & 1) as usize];
                }
            }

            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        } else {
            window.update();
        }
    }
}
