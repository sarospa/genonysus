mod cpu;

pub struct Genesis {
	cpu: cpu::CPU,
	cycles: u64
}
impl Genesis {
	pub fn new(rom: &Vec<u8>) -> Genesis {
		Genesis {
			cpu: cpu::CPU::new(rom),
			cycles: 0,
		}
	}
	
	pub fn advance_cycle(&mut self) {
		self.cycles += 1;
		if self.cycles % 7 == 0 {
			self.cpu.advance_cycle();
		}
	}
}