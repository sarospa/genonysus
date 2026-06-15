#![allow(dead_code)]

use std::env;
use std::fs;

mod genesis;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("Please specify a ROM.");
		return;
	}
    let rom: Vec<u8> = match fs::read(&args[1]){
		Ok(vec) => vec,
		Err(_) => {
			println!("{} is not a valid ROM.", args[1]);
			return;
		}
	};
	
	let mut console = genesis::Genesis::new(&rom);
	
	loop {
		console.advance_cycle();
	}
}