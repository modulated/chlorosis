use chlorosis_core::{Device};

fn main() {
	let f = std::env::args().nth(1);
	let mut dev = Device::default();
	if let Some(f) = f {
		dev.load_cartrige(f).expect("Could not load ROM file");
	}
	// dev.dump_cartrige_header();	
	dev.run();
}

