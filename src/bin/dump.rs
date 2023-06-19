use gb_chloro::Device;

fn main() {
    let mut dev = Device::new();
    dev.load_from_file("test_roms/bully/bully.gb")
        .expect("Could not open file");
    dev.dump_memory();
}
