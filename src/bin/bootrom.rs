use gb_chloro::Device;

fn main() {
    let mut dev = Device::new();
    dev.load_boot();
    dev.dump_state();
}
