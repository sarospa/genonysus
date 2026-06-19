mod cpu;

pub const CART_SIZE: usize = 0x400000;
pub const CPU_RAM_SIZE: usize = 0x10000;
pub const CPU_RAM_START: usize = 0xFF0000;
pub const CPU_RAM_END: usize = CPU_RAM_START + CPU_RAM_SIZE;
pub const CPU_ADDRESS_SPACE: usize = 0xFFFFFF;

trait Motorola68KBus {
	fn read_u8(&self, address: u32) -> u8;
	fn write_u8(&mut self, address: u32, data: u8);
	
	fn read_u16(&self, address: u32) -> u16 {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		((self.read_u8(address) as u16) << 8)
			+ (self.read_u8(address + 1) as u16)
	}
	
	fn read_u32(&self, address: u32) -> u32 {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		((self.read_u8(address) as u32) << 24)
			+ ((self.read_u8(address + 1) as u32) << 16)
			+ ((self.read_u8(address + 2) as u32) << 8)
			+ (self.read_u8(address + 3) as u32)
	}
	
	fn write_u16(&mut self, address: u32, data: u16) {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		self.write_u8(address, ((data & 0xFF00) >> 8) as u8);
		self.write_u8(address, (data & 0x00FF) as u8);
	}
	
	fn write_u32(&mut self, address:u32, data: u32) {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		self.write_u8(address, ((data & 0xFF000000) >> 24) as u8);
		self.write_u8(address, ((data & 0x00FF0000) >> 16) as u8);
		self.write_u8(address, ((data & 0x0000FF00) >> 8) as u8);
		self.write_u8(address, (data & 0x000000FF) as u8);
	}
	
}

pub struct Genesis {
	cpu: Option<cpu::CPU>,
	cpu_ram: Vec<u8>,
	cart_memory: Vec<u8>,
	controller1: Controller,
	controller1_control: u8,
	controller2: Controller,
	controller2_control: u8,
	cycles: u64
}
impl Genesis {
	pub fn new(rom: &Vec<u8>) -> Genesis {
		let mut cart_memory = vec![0; CART_SIZE];
		for i in 0..Ord::min(CART_SIZE, rom.len()) {
			cart_memory[i] = rom[i];
		};
		
		let mut genesis = Genesis {
			cpu: None,
			cpu_ram: vec![0; CPU_RAM_SIZE],
			cart_memory: cart_memory,
			controller1: Controller::Unplugged,
			controller1_control: 0,
			controller2: Controller::Unplugged,
			controller2_control: 0,
			cycles: 0,
		};
		let cpu = cpu::CPU::new(&genesis);
		genesis.cpu = Some(cpu);
		genesis
	}
	
	pub fn advance_cycle(&mut self) {
		self.cycles += 1;
		if self.cycles % 7 == 0 {
			let mut cpu = self.cpu.take().unwrap();
			cpu.advance_cycle(self);
			self.cpu = Some(cpu);
		}
	}
}
impl Motorola68KBus for Genesis {
	fn read_u8(&self, address: u32) -> u8 {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		match address_index {
			0..CART_SIZE => self.cart_memory[address_index],
			CPU_RAM_START..CPU_RAM_END => self.cpu_ram[address_index - CPU_RAM_START],
			0xA10002..=0xA10003 => self.controller1.read(),
			0xA10004..=0xA10005 => self.controller2.read(),
			0xA10008..=0xA10009 => self.controller1_control,
			0xA1000A..=0xA1000B => self.controller2_control,
			0xA1000C..=0xA1000D => 0, // Expansion port control
			_ => {
				println!("Address {:#08X} located in unimplemented memory region.", address);
				panic!("accessed unimplemented CPU memory");
			}
		}
	}
	
	fn write_u8(&mut self, address: u32, data: u8) {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		match address_index {
			0..CART_SIZE => (), // Don't overwrite the cart ROM!
			CPU_RAM_START..CPU_RAM_END => self.cpu_ram[address_index - CPU_RAM_START] = data,
			0xA10002..=0xA10003 => self.controller1.write(),
			0xA10004..=0xA10005 => self.controller2.write(),
			0xA10008..=0xA10009 => self.controller1_control = data,
			0xA1000A..=0xA1000B => self.controller2_control = data,
			0xA1000C..=0xA1000D => (), // Expansion port control
			_ => {
				panic!("Address {:#08X} located in unimplemented memory region.", address);
			}
		}
	}
}


#[derive(Debug)]
pub enum Controller {
	Unplugged,
	ThreeButton,
	SixButton,
}

impl Controller {
	pub fn read(&self) -> u8 {
		match self {
			Controller::Unplugged => 0,
			Controller::ThreeButton => panic!("Three button controller not implemented."),
			Controller::SixButton => panic!("Six button controller not implemented."),
		}
	}
	
	pub fn write(&mut self) -> () {
		match self {
			Controller::Unplugged => (),
			Controller::ThreeButton => panic!("Three button controller not implemented."),
			Controller::SixButton => panic!("Six button controller not implemented."),
		}
	}
}
