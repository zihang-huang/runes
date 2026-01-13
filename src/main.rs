pub mod cpu;
pub mod ppu;
pub mod bus;
pub mod opcodes;
pub mod ui;
pub mod cartridge;
pub mod renderer;

use cpu::CPU;
use ui::ui;
use cartridge::Cartridge;

use std::env;

fn main() {
    #[cfg(target_os = "linux")]
    {
        if env::var("WINIT_UNIX_BACKEND").is_err()
            && env::var("WAYLAND_DISPLAY").is_ok()
            && env::var("DISPLAY").is_ok()
        {
            // Avoid Wayland protocol issues by defaulting to X11 when available.
            env::set_var("WINIT_UNIX_BACKEND", "x11");
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: runes <path-to-rom>");
        return;
    }

    let cartridge_path = &args[1];
    let cpu = CPU::new(Cartridge::new(cartridge_path).unwrap());
    ui(cpu).unwrap();
}
