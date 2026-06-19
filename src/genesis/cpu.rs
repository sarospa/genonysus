#![allow(dead_code)]
#![allow(unused)]

use bitmatch::bitmatch;

mod cpu_support;
use cpu_support::*;
use crate::genesis::Motorola68KBus;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CPU {
	program_counter: u32,
	status_register: u16,
	d: [u32; 8],
	a: [u32; 7],
	usp: u32,
	ssp: u32,
	cycles: u64,
	countdown: u64,
}

impl CPU {
	pub fn new(bus: &dyn Motorola68KBus) -> CPU {	
		CPU {
			program_counter: bus.read_u32(4),
			status_register: 0,
			d: [0; 8],
			a: [0, 0, 0, 0, 0, 0, 0],
			usp: 0,
			ssp: bus.read_u32(0),
			cycles: 0,
			countdown: 0,
		}
	}
	
	pub fn advance_cycle(&mut self, bus: &mut dyn Motorola68KBus) {
		if self.countdown == 0 {
			self.run_opcode(bus);
		}
		self.countdown -= 1;
		self.cycles += 1;
	}
	
	fn read(bus: &dyn Motorola68KBus, address: u32, size: Size) -> Data {
		match size {
			Size::Byte => Data::Byte(bus.read_u8(address)),
			Size::Word => Data::Word(bus.read_u16(address)),
			Size::Long => Data::Long(bus.read_u32(address)),
		}
	}
	
	fn read_register(&self, reg: Register, size: Size) -> Data {
		let data = match reg {
			Register::A(a) => {
				if a.get() < 7 {
					self.a[a.get()]
				}
				else if self.status_register & 0x2000 == 0x2000 {
					self.ssp
				}
				else {
					self.usp
				}
			},
			Register::D(d) => self.d[d.get()],
		};
		match size {
			Size::Byte => Data::Byte(data as u8),
			Size::Word => Data::Word(data as u16),
			Size::Long => Data::Long(data),
		}
	}
	
	fn read_register_u8(&self, reg: Register) -> u8 {
		match self.read_register(reg, Size::Byte) {
			Data::Byte(d) => d,
			_ => panic!("Incorrect data size in read_register_u8"),
		}
	}
	
	fn read_register_u16(&self, reg: Register) -> u16 {
		match self.read_register(reg, Size::Word) {
			Data::Word(d) => d,
			_ => panic!("Incorrect data size in read_register_u16"),
		}
	}
	
	fn read_register_u32(&self, reg: Register) -> u32 {
		match self.read_register(reg, Size::Long) {
			Data::Long(d) => d,
			_ => panic!("Incorrect data size in read_register_u32"),
		}
	}
	
	fn write(bus: &mut dyn Motorola68KBus, address: u32, data: Data) {
		match data {
			Data::Byte(d) => bus.write_u8(address, d),
			Data::Word(d) => bus.write_u16(address, d),
			Data::Long(d) => bus.write_u32(address, d),
		}
	}
	
	fn write_register(&mut self, data: Data, reg: Register) {
		let reg_value = match reg {
			Register::A(a) => {
				if a.get() < 7 {
					self.a[a.get()]
				}
				else if self.status_register & 0x2000 == 0x2000 {
					self.ssp
				}
				else {
					self.usp
				}
			},
			Register::D(d) => self.d[d.get()],
		};
		let value = match data {
			Data::Byte(v) => (reg_value & 0xFFFFFF00) | (v as u32),
			Data::Word(v) => (reg_value & 0xFFFF0000) | (v as u32),
			Data::Long(v) => v,
		};
		match reg {
			Register::A(a) => {
				if a.get() < 7 {
					self.a[a.get()] = value
				}
				else if self.status_register & 0x2000 == 0x2000 {
					self.ssp = value
				}
				else {
					self.usp = value
				}
			},
			Register::D(d) => self.d[d.get()] = value,
		};
	}
	
	fn decode_addressing_mode(&self, addressing_bits: u8) -> AddrMode {
		let reg_bits = (addressing_bits & 0b111) as usize;
		match addressing_bits {
			0b000000..=0b000111 => AddrMode::DataReg(reg_bits),
			0b001000..=0b001111 => AddrMode::AddressReg(reg_bits),
			0b010000..=0b010111 => AddrMode::Address(reg_bits),
			0b011000..=0b011111 => AddrMode::AddressWithPostinc(reg_bits),
			0b100000..=0b100111 => AddrMode::AddressWithPredec(reg_bits),
			0b101000..=0b101111 => AddrMode::AddressWithDisp(reg_bits),
			0b110000..=0b110111 => AddrMode::AddressWithIndex(reg_bits),
			0b111010 => AddrMode::PCWithDisp,
			0b111011 => AddrMode::PCWithIndex,
			0b111000 => AddrMode::AbsoluteShort,
			0b111001 => AddrMode::AbsoluteLong,
			0b111100 => AddrMode::Immediate,
			_ => panic!("{:#08b} is not a valid addressing mode.", addressing_bits),
		}
	}
	
	// Assumes the program counter has advanced to the extension word, and advances it past it if applicable.
	fn read_with_mode(&mut self, bus: &dyn Motorola68KBus, addr_mode: AddrMode, size: Size, advance: bool) -> Data {
		match addr_mode {
			AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_)
			| AddrMode::AddressWithDisp(_) | AddrMode::AddressWithIndex(_)
			| AddrMode::PCWithDisp | AddrMode::PCWithIndex | AddrMode::AbsoluteShort
			| AddrMode::AbsoluteLong => {
				let address = self.calc_addr(bus, addr_mode, size, advance);
				CPU::read(bus, address, size)
			},
			AddrMode::DataReg(reg) => {
				self.read_register(Register::new(reg as usize, false), size)
			},
			AddrMode::AddressReg(reg) => {
				self.read_register(Register::new(reg as usize, true), size)
			}
			AddrMode::Immediate => {
				let data = match size {
					Size::Byte => CPU::read(bus, self.program_counter + 1, Size::Byte), // data is stored in the low byte of extension word
					Size::Word => CPU::read(bus, self.program_counter, Size::Word),
					Size::Long => CPU::read(bus, self.program_counter, Size::Long),
				};
				if advance {
					self.program_counter += Ord::max(size.length(), 2);
				};
				data
			},
		}
	}
	
	fn read_with_mode_u8(&mut self, bus: &dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u8 {
		let Data::Byte(data) = self.read_with_mode(bus, addr_mode, Size::Byte, advance)
		else {
			panic!("read_with_mode_u8 returned a non-byte value.");
		};
		return data;
	}
	
	fn read_with_mode_u16(&mut self, bus: &dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u16 {
		let Data::Word(data) = self.read_with_mode(bus, addr_mode, Size::Word, advance)
		else {
			panic!("read_with_mode_u16 returned a non-word value.");
		};
		return data;
	}
	
	fn read_with_mode_u32(&mut self, bus: &dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u32 {
		let Data::Long(data) = self.read_with_mode(bus, addr_mode, Size::Long, advance)
		else {
			panic!("read_with_mode_u32 returned a non-long value.");
		};
		return data;
	}
	
	fn write_with_mode(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, data: Data, advance: bool) {
		match addr_mode {
			AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_)
			| AddrMode::AddressWithDisp(_) | AddrMode::AddressWithIndex(_)
			| AddrMode::PCWithDisp | AddrMode::PCWithIndex | AddrMode::AbsoluteShort
			| AddrMode::AbsoluteLong => {
				let address = self.calc_addr(bus, addr_mode, Size::from_data(data), advance);
				CPU::write(bus, address, data);
			},
			AddrMode::DataReg(reg) => {
				self.write_register(data, Register::new(reg as usize, false))
			},
			AddrMode::AddressReg(reg) => {
				self.write_register(data, Register::new(reg as usize, true))
			},
			_ => panic!("Unimplemented write addressing mode {:?}.", addr_mode),
		}
	}
	
	fn calc_addr(&mut self, bus: &dyn Motorola68KBus, addr_mode: AddrMode, size: Size, advance: bool) -> u32 {
		match addr_mode {
			AddrMode::Address(reg) => {
				self.read_register_u32(Register::new(reg, true))
			},
			AddrMode::AddressWithPostinc(reg) => {
				let register = Register::new(reg, true);
				let address = self.read_register_u32(register);
				if advance {
					// Stack pointer always increments by a minimum of 2 bytes
					if reg == 7 && size == Size::Byte {
						self.write_register(Data::Long(address + 2), register)
					}
					else {
						self.write_register(Data::Long(address + size.length()), register);
					}
				}
				address
			},
			AddrMode::AddressWithPredec(reg) => {
				let register = Register::new(reg, true);
				let address = self.read_register_u32(register);
				if advance {
					// Stack pointer always decrements by a minimum of 2 bytes
					if reg == 7 && size == Size::Byte {
						self.write_register(Data::Long(address - 2), register)
					}
					else {
						self.write_register(Data::Long(address - size.length()), register);
					};
				};
				address - size.length()
			},
			AddrMode::AddressWithDisp(reg) => {
				let disp: i32 = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
				let address = self.read_register_u32(Register::new(reg, true));
				if advance {
					self.program_counter += 2;
				};
				address.wrapping_add_signed(disp)
			},
			AddrMode::PCWithDisp => {
				let disp: i32 = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
				let address = self.program_counter.wrapping_add_signed(disp);
				if advance {
					self.program_counter += 2;
				};
				address
			},
			AddrMode::AbsoluteShort => {
				let address = 0u32.wrapping_add_signed(CPU::sign_extend_u16(bus.read_u16(self.program_counter)));
				if advance {
					self.program_counter += 2;
				};
				address
			},
			AddrMode::AbsoluteLong => {
				let address = bus.read_u32(self.program_counter);
				if advance {
					self.program_counter += 4;
				};
				address
			},
			_ => panic!("Unimplemented address calculation addressing mode {:?}.", addr_mode),
		}
	}
	
	fn calc_addr_cycles(&mut self, addr_mode: AddrMode, size: Size) -> u64 {
		match addr_mode {
			AddrMode::DataReg(_) | AddrMode::AddressReg(_) => {
				0
			},
			AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::Immediate => {
				match size {
					Size::Long => 8,
					_ => 4,
				}
			},
			AddrMode::AddressWithPredec(_) => {
				match size {
					Size::Long => 10,
					_ => 6,
				}
			},
			AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => {
				match size {
					Size::Long => 12,
					_ => 8,
				}
			},
			AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => {
				match size {
					Size::Long => 14,
					_ => 10,
				}
			},
			AddrMode::AbsoluteLong => {
				match size {
					Size::Long => 16,
					_ => 12,
				}
			}
		}
	}
	
	// SR helper functions. SR structure is T_S__III ___XNZVC.
	
	fn set_x(&mut self, val: bool) {
		let x = if val { 0b00010000 } else { 0 };
		self.status_register = (self.status_register & 0b1111111111101111) | x;
	}
	
	fn set_n(&mut self, val: bool) {
		let n = if val { 0b00001000 } else { 0 };
		self.status_register = (self.status_register & 0b1111111111110111) | n;
	}
	
	fn set_z(&mut self, val: bool) {
		let z = if val { 0b00000100 } else { 0 };
		self.status_register = (self.status_register & 0b1111111111111011) | z;
	}
	
	fn set_v(&mut self, val: bool) {
		let v = if val { 0b00000010 } else { 0 };
		self.status_register = (self.status_register & 0b1111111111111101) | v;
	}
	
	fn set_c(&mut self, val: bool) {
		let c = if val { 0b00000001 } else { 0 };
		self.status_register = (self.status_register & 0b1111111111111110) | c;
	}
	
	fn set_i(&mut self, val: u8) {
		let i = (val as u16) << 8;
		self.status_register = (self.status_register & 0b1111100011111111) | i;
	}
	
	fn get_ccr_flags(&self) -> Flags {
		Flags {
			x: self.status_register & 0b0000000000010000 == 0b00010000,
			n: self.status_register & 0b0000000000001000 == 0b00001000,
			z: self.status_register & 0b0000000000000100 == 0b00000100,
			v: self.status_register & 0b0000000000000010 == 0b00000010,
			c: self.status_register & 0b0000000000000001 == 0b00000001,
		}
	}
	
	#[bitmatch]
	pub fn run_opcode(&mut self, bus: &mut dyn Motorola68KBus) -> () {
		print!("{:#010X} {:#010} ", self.program_counter, self.cycles);
		let opcode = self.decode_opcode(bus);
		self.program_counter += 2;
		match opcode {
			Opcode::AndI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let data = match (imm, data) {
					(Data::Byte(a), Data::Byte(b)) => Data::Byte(a & b),
					(Data::Word(a), Data::Word(b)) => Data::Word(a & b),
					(Data::Long(a), Data::Long(b)) => Data::Long(a & b),
					_ => panic!("Non-matching data in ANDI"),
				};
				match data {
					Data::Byte(d) => {
						self.set_n((d & 0x80) == 0x80);
						self.set_z(d == 0);
					},
					Data::Word(d) => {
						self.set_n((d & 0x8000) == 0x8000);
						self.set_z(d == 0);
					},
					Data::Long(d) => {
						self.set_n((d & 0x80000000) == 0x80000000);
						self.set_z(d == 0);
					},
				};
				self.write_with_mode(bus, addr_mode, data, true);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				let addr_cycles = self.calc_addr_cycles(addr_mode, size);
				self.countdown += addr_cycles + instr_cycles;
				println!("ANDI {},{} = {}", imm, addr_mode, data);
			}
			Opcode::MoveA { size, dest, source } => {
				let data = self.read_with_mode(bus, source, size, true);
				self.write_register(data.sign_extend(), Register::from_areg(dest));
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
				println!("MOVEA {},A{} = {}", source, dest.get(), data);
			}
			Opcode::Move { size, dest, source } => {
				let data = self.read_with_mode(bus, source, size, true);
				self.set_v(false);
				self.set_c(false);
				match data {
					Data::Byte(d) => {
						self.set_n((d & 0x80) == 0x80);
						self.set_z(d == 0);
					},
					Data::Word(d) => {
						self.set_n((d & 0x8000) == 0x8000);
						self.set_z(d == 0);
					},
					Data::Long(d) => {
						self.set_n((d & 0x80000000) == 0x80000000);
						self.set_z(d == 0);
					},
				};
				self.write_with_mode(bus, dest, data, true);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
				println!("MOVE {},{} = {}", source, dest, data);
			},
			Opcode::MoveToSr { addr_mode } => {
				self.status_register = self.read_with_mode_u16(bus, addr_mode, true);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				let addr_cycles = self.calc_addr_cycles(addr_mode, Size::Word);
				self.countdown += addr_cycles + instr_cycles;
				println!("MOVE {},SR = {:#06X}", addr_mode, self.status_register);
			},
			Opcode::Tst { size, addr_mode } => {
				let data = self.read_with_mode(bus, addr_mode, size, true);
				self.set_v(false);
				self.set_c(false);
				match data {
					Data::Byte(val) => {
						self.set_n((val as i8) < 0);
						self.set_z((val as i8) == 0);
					}
					Data::Word(val) => {
						self.set_n((val as i16) < 0);
						self.set_z((val as i16) == 0);
					}
					Data::Long(val) => {
						self.set_n((val as i32) < 0);
						self.set_z((val as i32) == 0);
					}
				}
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				let addr_cycles = self.calc_addr_cycles(addr_mode, Size::Word);
				self.countdown += addr_cycles + instr_cycles;
				println!("TST {} = CCR {:#04X}", addr_mode, self.status_register);
			},
			Opcode::MoveUsp { dir, a } => {
				match dir {
					MoveDirection::RegToMem => {
						let data = self.read_register_u32(Register::from_areg(a));
						self.usp = data;
						println!("MOVE {},USP", a);
					}
					MoveDirection::MemToReg => {
						self.write_register(Data::Long(self.usp), Register::from_areg(a));
						println!("MOVE USP,{}", a);
					}
				};
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
			},
			Opcode::Jmp { addr_mode } => {
				self.program_counter = self.calc_addr(bus, addr_mode, Size::Long, true);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
				println!("JMP {}", addr_mode);
			}
			Opcode::MoveM { dir, size, addr_mode } => {
				let reg_bits = bus.read_u16(self.program_counter);
				self.program_counter += 2;
				let reg_list = match addr_mode {
					AddrMode::AddressWithPredec(_) => {
						[Register::new(0, false)
						,Register::new(1, false)
						,Register::new(2, false)
						,Register::new(3, false)
						,Register::new(4, false)
						,Register::new(5, false)
						,Register::new(6, false)
						,Register::new(7, false)
						,Register::new(0, true)
						,Register::new(1, true)
						,Register::new(2, true)
						,Register::new(3, true)
						,Register::new(4, true)
						,Register::new(5, true)
						,Register::new(6, true)
						,Register::new(7, true)]
					},
					_ => {
						[Register::new(7, true)
						,Register::new(6, true)
						,Register::new(5, true)
						,Register::new(4, true)
						,Register::new(3, true)
						,Register::new(2, true)
						,Register::new(1, true)
						,Register::new(0, true)
						,Register::new(7, false)
						,Register::new(6, false)
						,Register::new(5, false)
						,Register::new(4, false)
						,Register::new(3, false)
						,Register::new(2, false)
						,Register::new(1, false)
						,Register::new(0, false)]
					}
				};
				let reg_flags = #[bitmatch] match reg_bits {
					"abcd_efgh_ijkl_mnop" => {
						[a == 1, b == 1, c == 1, d == 1, e == 1, f == 1, g == 1, h == 1
						, i == 1, j == 1, k == 1, l == 1, m == 1, n == 1, o == 1, p == 1]
					}
				};
				let mut address = self.calc_addr(bus, addr_mode, size, true);
				let mut reg_count = 0;
				match dir {
					MoveDirection::RegToMem => {
						for i in 15..=0 {
							if reg_flags[i] {
								if let AddrMode::AddressWithPredec(_) = addr_mode {
									address -= size.length();
								};
								CPU::write(bus, address, self.read_register(reg_list[i], size));
								if let AddrMode::AddressWithPredec(_) = addr_mode { }
								else {
									address += size.length();
								};
								reg_count += 1;
							}
						};
						if let AddrMode::AddressWithPredec(reg) = addr_mode {
							self.write_register(Data::Long(address), Register::new(reg, true));
						};
						println!("MOVEM {:#06X},{}", reg_bits, addr_mode);
					},
					MoveDirection::MemToReg => {
						for i in 15..=0 {
							if reg_flags[i] {
								self.write_register(CPU::read(bus, address, size), reg_list[i]);
								address += size.length();
								reg_count += 1;
							}
						};
						if let AddrMode::AddressWithPostinc(reg) = addr_mode {
							self.write_register(Data::Long(address), Register::new(reg, true));
						};
						println!("MOVEM {},{:#06X}", addr_mode, reg_bits);
					}
				};
				let instr_cycles = calc_opcode_cycles(opcode, None, None, Some(reg_count), None);
				self.countdown += instr_cycles;
			},
			Opcode::Lea { dest, addr_mode } => {
				let address = self.calc_addr(bus, addr_mode, Size::Long, true);
				self.write_register(Data::Long(address), Register::new(dest.get(), true));
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
				println!("LEA {},A{} = {:#010X}", addr_mode, dest.get(), address);
			},
			Opcode::AddQ { data, size, addr_mode } => {
				let size = match addr_mode {
					AddrMode::AddressReg(_) => Size::Long,
					_ => size,
				};
				let value = self.read_with_mode(bus, addr_mode, size, false);
				let new_value = match value {
					Data::Byte(d) => Data::Byte(d.wrapping_add(data)),
					Data::Word(d) => Data::Word(d.wrapping_add(data.into())),
					Data::Long(d) => Data::Long(d.wrapping_add(data.into())),
				};
				if let AddrMode::AddressReg(_) = addr_mode { }
				else {
					self.set_x(value.is_negative() && !new_value.is_negative());
					self.set_n(new_value.is_negative());
					self.set_z(new_value.is_zero());
					self.set_v(!value.is_negative() && new_value.is_negative());
					self.set_c(value.is_negative() && !new_value.is_negative());
				};
				self.write_with_mode(bus, addr_mode, new_value, true);
				println!("ADDQ #{data},{addr_mode}");
			},
			Opcode::SubQ { data, size, addr_mode } => {
				let size = match addr_mode {
					AddrMode::AddressReg(_) => Size::Long,
					_ => size,
				};
				let value = self.read_with_mode(bus, addr_mode, size, false);
				let new_value = match value {
					Data::Byte(d) => Data::Byte(d.wrapping_sub(data)),
					Data::Word(d) => Data::Word(d.wrapping_sub(data.into())),
					Data::Long(d) => Data::Long(d.wrapping_sub(data.into())),
				};
				if let AddrMode::AddressReg(_) = addr_mode { }
				else {
					self.set_x(value.is_negative() && !new_value.is_negative());
					self.set_n(new_value.is_negative());
					self.set_z(new_value.is_zero());
					self.set_v(!value.is_negative() && new_value.is_negative());
					self.set_c(value.is_negative() && !new_value.is_negative());
				};
				self.write_with_mode(bus, addr_mode, new_value, true);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				let addr_cycles = self.calc_addr_cycles(addr_mode, size);
				self.countdown += addr_cycles + instr_cycles;
				println!("SUBQ #{data},{addr_mode}");
			}
			Opcode::DBcc { cond, loop_down } => {
				let mut branch_taken = false;
				let disp = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
				let mut loop_count = self.read_register_u16(Register::from_dreg(loop_down));
				if !cond.check(self.get_ccr_flags()) {
					loop_count = loop_count.wrapping_add_signed(-1);
					self.write_register(Data::Word(loop_count), Register::from_dreg(loop_down));
					if loop_count & 0x8000 == 0 { // If loop count is positive
						self.program_counter = self.program_counter.wrapping_add_signed(disp);
						branch_taken = true;
					}
					else {
						self.program_counter += 2;
					}
				}
				else {
					self.program_counter += 2;
				}
				let instr_cycles = calc_opcode_cycles(opcode, Some(branch_taken), Some(loop_count == 0xFFFF), None, None);
				self.countdown += instr_cycles;
				println!("DB{cond},{loop_down} = PC {:#010X}, {loop_down} = {loop_count:#06X}", self.program_counter);
			}
			Opcode::Bcc { cond, disp } => {
				let mut branch_taken = false;
				let displacement = if disp == 0 {
					let long_disp = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
					long_disp
				}
				else {
					disp
				};
				if cond.check(self.get_ccr_flags()) {
					self.program_counter = self.program_counter.wrapping_add_signed(displacement);
					branch_taken = true;
				}
				else if disp == 0 {
					self.program_counter += 2;
				}
				let instr_cycles = calc_opcode_cycles(opcode, Some(branch_taken), None, None, None);
				self.countdown += instr_cycles;
				println!("B{cond} = PC {:#010X}", self.program_counter);
			},
			Opcode::MoveQ { dest, data } => {
				self.set_n((data & 0x80) == 0x80);
				self.set_z(data == 0);
				self.set_v(false);
				self.set_c(false);
				self.write_register(Data::Long(CPU::sign_extend_u8(data) as u32), Register::from_dreg(dest));
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, None);
				self.countdown += instr_cycles;
				let d = dest.get();
				println!("MOVEQ = D{} {:#010X}", d, self.d[d]);
			},
			Opcode::LsdToD { rot, dir, size, mode, reg } => {
				let rotation = match mode {
					RotateMode::Immediate => if rot == 0 { 8 } else { rot },
					RotateMode::Register => self.read_register_u8(Register::new(rot as usize, false)) % 64,
				};
				let register = Register::from_dreg(reg);
				let data = self.read_register(register, size);
				// Rust REALLY does not like shifting past the size of the integer.
				// I don't love handling this way but since the shift is limited to 63 bits,
				// it's probably easiest to just temporarily cast to a u64.
				let mut wide_data = match data {
					Data::Byte(d) => d as u64,
					Data::Word(d) => d as u64,
					Data::Long(d) => d as u64,
				};
				if rotation > 0 {
					match dir {
						RotateDirection::Right => {
							wide_data = wide_data >> (rotation - 1);
							let carry_bit = (wide_data & 0x1) == 0x1;
							self.set_c(carry_bit);
							self.set_x(carry_bit);
							wide_data = wide_data >> 1;
						},
						RotateDirection::Left => {
							wide_data = wide_data << (rotation - 1);
							let bitmask = match size {
								Size::Byte => 0x80,
								Size::Word => 0x8000,
								Size::Long => 0x80000000,
							};
							let carry_bit = (wide_data & bitmask) == bitmask;
							self.set_c(carry_bit);
							self.set_x(carry_bit);
							wide_data = wide_data << 1;
						}
					}
				}
				else {
					self.set_c(false);
				}
				let new_data = match size {
					Size::Byte => {
						let new_val = wide_data as u8;
						self.set_z(new_val == 0);
						self.set_n((new_val & 0x80) == 0x80);
						Data::Byte(new_val)
					},
					Size::Word => {
						let new_val = wide_data as u16;
						self.set_z(new_val == 0);
						self.set_n((new_val & 0x8000) == 0x8000);
						Data::Word(new_val)
					},
					Size::Long => {
						let new_val = wide_data as u32;
						self.set_z(new_val == 0);
						self.set_n((new_val & 0x80000000) == 0x80000000);
						Data::Long(new_val)
					},
				};
				self.set_v(false);
				self.write_register(new_data, register);
				let instr_cycles = calc_opcode_cycles(opcode, None, None, None, Some(rotation as u64));
				self.countdown += instr_cycles;
				println!("LS{dir} {mode}{rot},{reg}");
			},
			_ => panic!("{opcode} unimplemented."),
		}
		if self.countdown == 0 {
			panic!("{opcode} failed to advance cycles.");
		}
	}
	
	#[bitmatch]
	fn decode_opcode(&self, bus: &dyn Motorola68KBus) -> Opcode {
		let opcode = bus.read_u16(self.program_counter);
		print!("{:#06X} ", opcode);
		#[bitmatch]
		match opcode {
			// ORI to CCR
			"0000_0000_0011_1100" => {
				Opcode::OrIToCcr
			},
			// ORI to SR
			"0000_0000_0111_1100" => {
				Opcode::OrIToSr
			},
			// ORI
			"0000_0000_ssmm_mxxx" => {
				Opcode::OrI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// ANDI to CCR
			"0000_0010_0011_1100" => {
				Opcode::AndIToCcr
			},
			// ANDI to SR
			"0000_0010_0111_1100" => {
				Opcode::AndIToSr
			},
			// ANDI
			"0000_0010_ssmm_mxxx" => {
				Opcode::AndI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// SUBI
			"0000_0100_ssmm_mxxx" => {
				Opcode::SubI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// ADDI
			"0000_0110_ssmm_mxxx" => {
				Opcode::AddI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// EORI to CCR
			"0000_1010_0011_1100" => {
				Opcode::EorIToCcr
			},
			// EORI to SR
			"0000_1010_0111_1100" => {
				Opcode::EorIToSr
			},
			// EORI
			"0000_1010_ssmm_mxxx" => {
				Opcode::EorI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// CMPI
			"0000_1100_ssmm_mxxx" => {
				Opcode::CmpI {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BTST
			"0000_1000_00mm_mxxx" => {
				Opcode::Btst {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BCHG
			"0000_1000_01mm_mxxx" => {
				Opcode::Bchg {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BCLR
			"0000_1000_10mm_mxxx" => {
				Opcode::Bclr {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BSET
			"0000_1000_11mm_mxxx" => {
				Opcode::Bset {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BTST from D Register
			"0000_ddd1_00mm_mxxx" => {
				Opcode::BtstFromD {
					source: DReg::new(d),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BCHG from D Register
			"0000_ddd1_01mm_mxxx" => {
				Opcode::BchgFromD {
					source: DReg::new(d),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BCLR from D Register
			"0000_ddd1_10mm_mxxx" => {
				Opcode::BclrFromD {
					source: DReg::new(d),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BSET from D Register
			"0000_ddd1_11_mm_mxxx" => {
				Opcode::BsetFromD {
					source: DReg::new(d),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVEP
			"0000_ddd1_rs00_1aaa" => {
				Opcode::MoveP {
					source: DReg::new(d),
					dir: MoveDirection::new(r == 1, true),
					size: Size::from_bit(s == 1),
					dest: AReg::new(a)
				}
			},
			// MOVEA
			"00ss_aaa0_01mm_mxxx" => {
				Opcode::MoveA {
					size: Size::from_high_bits(s),
					dest: AReg::new(a),
					source: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVE
			"00ss_yyyn_nnmmm_xxx" => {
				Opcode::Move {
					size: Size::from_high_bits(s),
					dest: self.decode_addressing_mode(((n << 3) + y) as u8),
					source: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVE from SR
			"0100_0000_11mm_mxxx" => {
				Opcode::MoveFromSr {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVE to CCR
			"0100_0100_11mm_mxxx" => {
				Opcode::MoveToCcr {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVE to SR
			"0100_0110_11mm_mxxx" => {
				Opcode::MoveToSr {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// NEGX
			"0100_0000_ssmm_mxxx" => {
				Opcode::NegX {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// CLR
			"0100_0010_ssmm_mxxx" => {
				Opcode::Clr {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// NEG
			"0100_0100_ssmm_mxxx" => {
				Opcode::Neg {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// NOT
			"0100_0110_ssmm_mxxx" => {
				Opcode::Not {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// EXT
			"0100_1000_1s00_0ddd" => {
				Opcode::Ext {
					size: Size::from_bit(s == 1),
					dest: DReg::new(d)
				}
			},
			// NBCD
			"0100_1000_00mm_mxxx" => {
				Opcode::Nbcd {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// SWAP
			"0100_1000_0100_0ddd" => {
				Opcode::Swap {
					dest: DReg::new(d)
				}
			},
			// PEA
			"0100_1000_01mm_mxxx" => {
				Opcode::Pea {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// ILLEGAL
			"0100_1010_1111_1100" => {
				Opcode::Illegal
			},
			// TAS
			"0100_1010_11mm_mxxx" => {
				Opcode::Tas {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// TST
			"0100_1010_ssmm_mxxx" => {
				Opcode::Tst {
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// TRAP
			"0100_1110_0100_vvvv" => {
				Opcode::Trap {
					vector: Vector::new(v)
				}
			},
			// LINK
			"0100_1110_0101_0aaa" => {
				Opcode::Link {
					frame_pointer: AReg::new(a)
				}
			},
			// UNLNK
			"0100_1110_0101_1aaa" => {
				Opcode::Unlnk {
					frame_pointer: AReg::new(a)
				}
			},
			// MOVE USP
			"0100_1110_0110_daaa" => {
				Opcode::MoveUsp {
					dir: MoveDirection::new(d == 1, false),
					a: AReg::new(a)
				}
			},
			// RESET
			"0100_1110_0111_0000" => {
				Opcode::Reset
			},
			// NOP
			"0100_1110_0111_0001" =>  {
				Opcode::Nop
			},
			// STOP
			"0100_1110_0111_0010" => {
				Opcode::Stop
			},
			// RTE
			"0100_1110_0111_0011" => {
				Opcode::Rte
			},
			// RTS
			"0100_1110_0111_0101" => {
				Opcode::Rts
			},
			// TRAPV
			"0100_1110_0111_0110" => {
				Opcode::TrapV
			},
			// RTR
			"0100_1110_0111_0111" => {
				Opcode::Rtr
			},
			// JSR
			"0100_1110_10mm_mxxx" => {
				Opcode::Jsr {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// JMP
			"0100_1110_11mm_mxxx" => {
				Opcode::Jmp {
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// MOVEM
			"0100_1d00_1smm_mxxx" => {
				Opcode::MoveM {
					dir: MoveDirection::new(d == 1, false),
					size: Size::from_bit(s == 1),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// LEA
			"0100_aaa1_11mm_mxxx" =>  {
				Opcode::Lea {
					dest: AReg::new(a),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// CHK
			"0100_ddd1_10mm_mxxx" => {
				Opcode::Chk {
					source: DReg::new(d),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// DBcc
			"0101_cccc_1100_1ddd" => {
				Opcode::DBcc {
					cond: Condition::new(c),
					loop_down: DReg::new(d),
				}
			},
			// Scc
			"0101_cccc_11mm_mxxx" => {
				Opcode::Scc {
					cond: Condition::new(c),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// ADDQ
			"0101_ddd0_ssmm_mxxx" => {
				Opcode::AddQ {
					data: if d == 0 { 8 } else { d } as u8,
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// SUBQ
			"0101_ddd1_ssmm_mxxx" => {
				Opcode::SubQ {
					data: if d == 0 { 8 } else { d } as u8,
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8)
				}
			},
			// BSR
			"0110_0001_dddd_dddd" => {
				Opcode::BSR {
					disp: CPU::sign_extend_u16(d)
				}
			}
			// Bcc
			"0110_cccc_dddd_dddd" => {
				Opcode::Bcc {
					cond: Condition::new(c),
					disp: CPU::sign_extend_u16(d)
				}
			},
			// MOVEQ
			"0111_ddd0_vvvv_vvvv" => {
				Opcode::MoveQ {
					dest: DReg::new(d),
					data: v as u8,
				}
			},
			// DIVU
			"1000_ddd0_11mm_mxxx" => {
				Opcode::DivU {
					dest: DReg::new(d),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// DIVS
			"1000_ddd1_11mm_mxxx" => {
				Opcode::DivS {
					dest: DReg::new(d),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// SBCD
			"1000_xxx1_0000_myyy" => {
				Opcode::Sbcd {
					dest: if m == 0 {
						AddrMode::DataReg(x as usize)
					}
					else {
						AddrMode::AddressWithPredec(x as usize)
					},
					source: if m == 0 {
						AddrMode::DataReg(y as usize)
					}
					else {
						AddrMode::AddressWithPredec(y as usize)
					},
				}
			},
			// OR
			"1000_dddr_ssmm_mxxx" => {
				Opcode::Or {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// SUB
			"1001_dddr_ssmm_mxxx" => {
				Opcode::Sub {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// SUBX
			"1001_xxx1_ss00_myyy" => {
				Opcode::SubX {
					dest: if m == 0 {
						AddrMode::DataReg(x as usize)
					}
					else {
						AddrMode::AddressWithPredec(x as usize)
					},
					size: Size::from_low_bits(s),
					source: if m == 0 {
						AddrMode::DataReg(y as usize)
					}
					else {
						AddrMode::AddressWithPredec(y as usize)
					},
				}
			},
			// SUBA
			"1001_aaas_11mm_mxxx" => {
				Opcode::SubA {
					dest: AReg::new(a),
					size: Size::from_bit(s == 1),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// EOR
			"1011_ddd1_ssmm_mxxx" => {
				Opcode::Eor {
					dest: DReg::new(d),
					size: Size::from_low_bits(s),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// CMPM
			"1011_aaa1_ss00_1bbb" => {
				Opcode::CmpM {
					dest: AReg::new(a),
					size: Size::from_low_bits(s),
					source: AReg::new(b),
				}
			},
			// CMP
			"1011_ddd0_ssmm_mxxx" => {
				Opcode::Cmp {
					dest: DReg::new(d),
					size: Size::from_low_bits(s),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// CMPA
			"1011_aaas_11mm_mxxx" => {
				Opcode::CmpA {
					dest: AReg::new(a),
					size: Size::from_bit(s == 1),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// MULU
			"1100_ddd0_11mm_mxxx" => {
				Opcode::MulU {
					dest: DReg::new(d),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// MULS
			"1100_ddd1_11mm_mxxx" => {
				Opcode::MulS {
					dest: DReg::new(d),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ABCD
			"1100_xxx1_0000_oyyy" => {
				Opcode::Abcd {
					dest: if o == 0 {
						AddrMode::DataReg(x as usize)
					}
					else {
						AddrMode::AddressWithPredec(x as usize)
					},
					source: if o == 0 {
						AddrMode::DataReg(y as usize)
					}
					else {
						AddrMode::AddressWithPredec(y as usize)
					},
				}
			},
			// EXG
			"1100_xxx1_mm00_myyy" => {
				Opcode::Exg {
					first: Register::new(x as usize, m & 0x1 == 0x1),
					second: Register::new(y as usize, m & 0x1 == (m & 0x2 >> 1)),
				}
			},
			// AND
			"1100_dddr_ssmm_mxxx" => {
				Opcode::And {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ADD
			"1101_dddr_ssmm_mxxx" => {
				Opcode::Add {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ADDX
			"1101_yyy1_ss00_mxxx" => {
				Opcode::AddX {
					dest: if m == 0 {
						AddrMode::DataReg(x as usize)
					}
					else {
						AddrMode::AddressWithPredec(x as usize)
					},
					size: Size::from_low_bits(s),
					source: if m == 0 {
						AddrMode::DataReg(y as usize)
					}
					else {
						AddrMode::AddressWithPredec(y as usize)
					},
				}
			},
			// ADDA
			"1101_aaas_11mm_mxxx" => {
				Opcode::AddA {
					dest: AReg::new(a),
					size: Size::from_bit(s == 1),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ASd
			"1110_000d_11mm_mxxx" => {
				Opcode::Asd {
					dir: RotateDirection::new(d == 1),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// LSd
			"1110_001d_11mm_mxxx" => {
				Opcode::Lsd {
					dir: RotateDirection::new(d == 1),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ROXd
			"1110_010d_11mm_mxxx" => {
				Opcode::RoXd {
					dir: RotateDirection::new(d == 1),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ROd
			"1110_011d_11mm_mxxx" => {
				Opcode::Rod {
					dir: RotateDirection::new(d == 1),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
				}
			},
			// ASd to D register
			"1110_rrri_ssm0_0ddd" => {
				Opcode::AsdToD {
					rot: r as u8,
					dir: RotateDirection::new(i == 1),
					size: Size::from_low_bits(s),
					mode: RotateMode::new(m == 1),
					reg: DReg::new(d),
				}
			},
			// LSd to D register
			"1110_rrri_ssm0_1ddd" => {
				Opcode::LsdToD {
					rot: r as u8,
					dir: RotateDirection::new(i == 1),
					size: Size::from_low_bits(s),
					mode: RotateMode::new(m == 1),
					reg: DReg::new(d),
				}
			},
			// ROXd to D register
			"1110_rrri_ssm1_0ddd" => {
				Opcode::RoXdToD {
					rot: r as u8,
					dir: RotateDirection::new(i == 1),
					size: Size::from_low_bits(s),
					mode: RotateMode::new(m == 1),
					reg: DReg::new(d),
				}
			},
			// ROd to D register
			"1110_rrri_ssm1_1ddd" => {
				Opcode::RodToD {
					rot: r as u8,
					dir: RotateDirection::new(i == 1),
					size: Size::from_low_bits(s),
					mode: RotateMode::new(m == 1),
					reg: DReg::new(d),
				}
			},
			_ => Opcode::Illegal
		}
	}
	
	fn load_u32_at(data: &Vec<u8>, index: usize) -> u32 {
	((data[index] as u32) << 24)
		+ ((data[index + 1] as u32) << 16)
		+ ((data[index + 2] as u32) << 8)
		+ data[index + 3] as u32
	}
	
	// Convert an unsigned 16-bit value a 32-bit version of its signed representation.
	// Moved into a helper function since it's a bit unintuitive.
	fn sign_extend_u16(x: u16) -> i32 {
		(x as i16) as i32
	}
	
	fn sign_extend_u8(x: u8) -> i32 {
		(x as i8) as i32
	}
	
	// Put the cpu in a clean state to start a test.
	#[cfg(test)]
	pub fn test_reset(&mut self, bus: &dyn Motorola68KBus) {
		self.program_counter = bus.read_u32(4);
		self.status_register = 0x2000;
		self.a = [0; 7];
		self.d = [0; 8];
		self.usp = 0;
		self.ssp = bus.read_u32(0);
		self.cycles = 0;
		self.countdown = 0;
	}
}