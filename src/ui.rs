use eframe::egui;
use std::time::{Duration, Instant};
use crate::cpu::CPU;
use crate::ppu::SYSTEM_PALLETE;
use egui_dock::{DockArea, NodeIndex, Style, Tree};

use crate::opcodes::references;
use crate::renderer;

pub fn ui(cpu: CPU) -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(1920.0, 1080.0)),
        ..Default::default()
    };

    eframe::run_native(
        "runes", 
        options, 
        Box::new(|_cc| Box::<RunesApp>::new(RunesApp::new(cpu))))
}

struct RunesContext {
    cpu: CPU,
    page_cpu: u16,
    page_rom: u16,

    chr_rom_texture: Option<egui::TextureHandle>,
    frame_texture: Option<egui::TextureHandle>,
    running: bool,
    chr_rom_dirty: bool,
}

impl egui_dock::TabViewer for RunesContext {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "CPU Memory Inspector" => self.cpu_memory_inspector(ui),
            "Game" => self.game(ui),
            "CPU Register Inspector" => self.cpu_register_inspector(ui),
            "CPU Debug Inspector" => self.cpu_debug_inspector(ui),
            "ROM Memory Inspector" => self.rom_memory_inspector(ui),
            "ROM Header Inspector" => self.rom_header_inspector(ui),
            "CHR ROM Inspector" => self.chr_rom_inspector(ui),
            _ => {}
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    
}

impl RunesContext {
    fn run_frame(&mut self) -> bool {
        const FRAME_CYCLES: u32 = 341 * 262;
        for _ in 0..FRAME_CYCLES {
            self.cpu.clock();
            if self.cpu.bus.ppu.frame_complete {
                self.cpu.bus.ppu.frame_complete = false;
                return true;
            }
        }
        false
    }

    fn run_for_budget(&mut self, budget: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < budget {
            self.cpu.clock();
            if self.cpu.bus.ppu.frame_complete {
                self.cpu.bus.ppu.frame_complete = false;
                return true;
            }
        }
        false
    }

    fn step_instruction(&mut self) {
        loop {
            self.cpu.clock();
            if self.cpu.complete() {
                break;
            }
        }
    }

    fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.bus.ppu.reset();
    }

    fn update_controller_state(&mut self, ctx: &egui::Context) {
        let state = ctx.input(|i| {
            let mut state = 0u8;
            if i.key_down(egui::Key::Z) {
                state |= 1 << 0;
            }
            if i.key_down(egui::Key::X) {
                state |= 1 << 1;
            }
            if i.key_down(egui::Key::Tab) {
                state |= 1 << 2;
            }
            if i.key_down(egui::Key::Enter) {
                state |= 1 << 3;
            }
            if i.key_down(egui::Key::ArrowUp) || i.key_down(egui::Key::W) {
                state |= 1 << 4;
            }
            if i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::S) {
                state |= 1 << 5;
            }
            if i.key_down(egui::Key::ArrowLeft) || i.key_down(egui::Key::A) {
                state |= 1 << 6;
            }
            if i.key_down(egui::Key::ArrowRight) || i.key_down(egui::Key::D) {
                state |= 1 << 7;
            }
            state
        });

        self.cpu.bus.set_controller_state(0, state);
        self.cpu.bus.set_controller_state(1, 0);
    }

    fn update_frame_texture(&mut self, ctx: &egui::Context) {
        let image = egui::ColorImage::from_rgb(
            [256, 240],
            &self.cpu.bus.ppu.frame_buffer,
        );

        if let Some(texture) = &mut self.frame_texture {
            texture.set(image, Default::default());
        } else {
            self.frame_texture = Some(ctx.load_texture("ppu-frame", image, Default::default()));
        }
    }

    fn cpu_memory_inspector(&mut self, ui: &mut egui::Ui) {
        // change style to monospace
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

        // page selector
        ui.horizontal(|ui| {
            ui.label("Page: ");
            ui.add(egui::DragValue::new(&mut self.page_cpu).speed(1.0).clamp_range(0..=0x07));
        });

        for addr in 0..=15 {
            ui.horizontal(|ui| {
                ui.label(format!("{:02X}{:2X}0", self.page_cpu, addr));
                ui.separator();
                for i in 0..=15 {
                    // format as hex
                    // only print when read from page 8000 ~ 8010
                    ui.label(format!("{:02X}", self.cpu.bus.cpu_vram[(self.page_cpu << 8 | addr << 4 | i) as usize]));
                }
            });
        }
    }

    fn rom_memory_inspector(&mut self, ui: &mut egui::Ui) {
        // change style to monospace
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

        // page selector
        ui.horizontal(|ui| {
            ui.label("Page: ");
            ui.add(egui::DragValue::new(&mut self.page_rom).speed(1.0).clamp_range(0x80..=0xFF));
        });

        for addr in 0..=15 {
            ui.horizontal(|ui| {
                ui.label(format!("{:02X}{:2X}0", self.page_rom, addr));
                ui.separator();
                for i in 0..=15 {
                    // format as hex
                    // only print when read from page 8000 ~ 8010
                    ui.label(format!("{:02X}", self.cpu.bus.read_prg_rom(self.page_rom << 8 | addr << 4 | i)));
                }
            });
        }
    }

    fn cpu_register_inspector(&mut self, ui: &mut egui::Ui) {
        // change style to monospace
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

        ui.horizontal(|ui| {
            ui.label("A: ");
            ui.label(format!("{:02X}", self.cpu.accumulator));
        });

        ui.horizontal(|ui| {
            ui.label("X: ");
            ui.label(format!("{:02X}", self.cpu.x_register));
        });

        ui.horizontal(|ui| {
            ui.label("Y: ");
            ui.label(format!("{:02X}", self.cpu.y_register));
        });

        ui.horizontal(|ui| {
            ui.label("PC: ");
            ui.label(format!("{:04X}", self.cpu.program_counter));
        });

        ui.horizontal(|ui| {
            ui.label("SP: ");
            ui.label(format!("{:02X}", self.cpu.stack_pointer));
        });

        ui.horizontal(|ui| {
            ui.label("Status: ");
            ui.label(format!("{:08b}", self.cpu.status));
        });

    }

    fn cpu_debug_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Opcode {}", references::INSTRUCTION_LOOKUP[self.cpu.opcode as usize]));       
        ui.label(format!("Cycles: {:?}", self.cpu.cycles));
    }

    fn rom_header_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("PRG ROM Size: {}", self.cpu.bus.cartridge.header.prg_rom_size));
        ui.label(format!("CHR ROM Size: {}", self.cpu.bus.cartridge.header.chr_rom_size));
        ui.label(format!("Mapper: {}", (self.cpu.bus.cartridge.header.mapper_2 & 0xF0) | (self.cpu.bus.cartridge.header.mapper_1 >> 4)));
    }

    fn chr_rom_inspector(&mut self, ui: &mut egui::Ui) {
        if self.chr_rom_texture.is_none() || self.chr_rom_dirty {
            let width = 256;
            let height = 240;
            let mut renderer = renderer::PPURenderer::new_custom_size(width, height);

            let mut tile_y = 0;
            let mut tile_x = 0;

            for tile_n in 0..255 {
                if tile_n != 0 && tile_n % 20 == 0 {
                    tile_y += 10;
                    tile_x = 0;
                }
                // load tiles into texture
                let tile = &self.cpu.bus.ppu.chr_rom[tile_n * 16..=tile_n * 16 + 15];

                for tile_index_y in 0..=7 {
                    let mut upper = tile[tile_index_y];
                    let mut lower = tile[tile_index_y + 8];

                    for tile_index_x in (0..=7).rev() {
                        let color = (1 & upper) << 1 | (1 & lower);
                        upper >>= 1;
                        lower >>= 1;

                        let rgb = match color {
                            0 => SYSTEM_PALLETE[0x01],
                            1 => SYSTEM_PALLETE[0x23],
                            2 => SYSTEM_PALLETE[0x30],
                            3 => SYSTEM_PALLETE[0x3F],
                            _ => panic!("Invalid color value"),
                        };

                        renderer.set_pixel(tile_x + tile_index_x, tile_y + tile_index_y, rgb);
                    }
                }

                tile_x += 10;
            }

            let image = renderer.get_color_image();
            if let Some(texture) = &mut self.chr_rom_texture {
                texture.set(image, Default::default());
            } else {
                self.chr_rom_texture = Some(ui.ctx().load_texture(
                    "chr-rom-texture",
                    image,
                    Default::default(),
                ));
            }

            self.chr_rom_dirty = false;
        }

        if let Some(texture) = &self.chr_rom_texture {
            let size = ui.available_size().min_elem();
            ui.image(texture, [size, size]);
        }
    }

    fn game(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(if self.running { "Running" } else { "Paused" });
            ui.separator();
            ui.label("Space: Run/Pause");
            ui.label("N: Step");
            ui.label("F: Frame");
            ui.label("R: Reset");
        });

        if let Some(texture) = &self.frame_texture {
            let available = ui.available_size();
            let scale = (available.x / 256.0).min(available.y / 240.0);
            let size = egui::Vec2::new(256.0 * scale, 240.0 * scale);
            ui.image(texture, size);
        } else {
            ui.label("Framebuffer not ready yet.");
        }
    }
}

struct RunesApp {
    context: RunesContext,
    tree: Tree<String>
}


impl RunesApp {
    fn new(mut cpu: CPU) -> Self {
        cpu.reset();
        cpu.bus.ppu.reset();
        let mut tree = Tree::new(vec!["Game".to_owned()]);

        let [game_node_index , cpu_memory_inspector_node_index] = tree.split_right(NodeIndex::root(), 0.78 ,vec!["CPU Memory Inspector".to_owned()]);

        tree.split_left(game_node_index, 0.3, vec!["CHR ROM Inspector".to_owned()]);

        let [_ , rom_memory_inspector_node_index] = tree.split_below(cpu_memory_inspector_node_index, 0.38, vec!["ROM Memory Inspector".to_owned(), "ROM Header Inspector".to_owned()]);
        let [_ , cpu_register_inspector_node_index] = tree.split_below(rom_memory_inspector_node_index, 0.7, vec!["CPU Register Inspector".to_owned()]);


        tree.split_right(cpu_register_inspector_node_index, 0.5, vec!["CPU Debug Inspector".to_owned()]);

        Self {
            context: RunesContext {
                cpu,
                page_cpu: 0,
                page_rom: 0x80,
                chr_rom_texture: None,
                frame_texture: None,
                running: false,
                chr_rom_dirty: true,
            },
            tree
        }
    }
}

impl eframe::App for RunesApp { 
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.context.update_controller_state(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.context.running = !self.context.running;
        }

        let mut frame_dirty = false;
        let mut frame_complete = false;

        if ctx.input(|i| i.key_pressed(egui::Key::N)) {
            self.context.step_instruction();
            frame_dirty = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            frame_complete = self.context.run_frame();
            frame_dirty = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.context.reset();
            frame_dirty = true;
        }

        if self.context.running {
            frame_complete |= self.context.run_for_budget(Duration::from_millis(4));
            ctx.request_repaint();
        }

        if frame_dirty || frame_complete {
            self.context.update_frame_texture(ctx);
            if self.context.cpu.bus.cartridge.chr_is_ram {
                self.context.chr_rom_dirty = true;
            }
        }

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.context);
    }
}
