use gb_chloro::Device;

fn main() {
    let mut dev = Device::new();
    dev.load_cartrige("roms/pokemon.gbc")
        .expect("Cannot open ROM");
    dev.dump_cartrige_header();
    // dev.dump_memory();
    dev.run();
}
