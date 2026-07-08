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
	interrupt_vectors: [u32; 8],
}

impl CPU {
	pub fn new(bus: &mut dyn Motorola68KBus) -> CPU {
		CPU {
			program_counter: bus.read_u32(4),
			status_register: 0,
			d: [0; 8],
			a: [0, 0, 0, 0, 0, 0, 0],
			usp: 0,
			ssp: bus.read_u32(0),
			cycles: 0,
			countdown: 0,
			interrupt_vectors: [bus.read_u32(0x60), bus.read_u32(0x64), bus.read_u32(0x68), bus.read_u32(0x6C),
				bus.read_u32(0x70), bus.read_u32(0x74), bus.read_u32(0x78), bus.read_u32(0x7C)],
		}
	}
	
	pub fn advance_cycle(&mut self, bus: &mut dyn Motorola68KBus) {
		if self.countdown == 0 {
			match bus.acknowledge_interrupt() {
				Some(level) => {
					let mask = self.get_i();
					if mask < level || level == 7 {
						self.push(bus, Data::Long(self.program_counter));
						self.push(bus, Data::Word(self.status_register));
						self.set_i(6);
						self.program_counter = self.interrupt_vectors[level as usize];
						self.countdown += 72;
						self.cycles += 72;
					}
					else {
						self.run_opcode(bus);
					}
				},
				None => {
					self.run_opcode(bus);
				}
			}
		}
		self.countdown -= 1;
		self.cycles += 1;
	}
	
	fn read(bus: &mut dyn Motorola68KBus, address: u32, size: Size) -> Data {
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
	
	fn write_register(&mut self, reg: Register, data: Data) {
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
	
	fn push(&mut self, bus: &mut dyn Motorola68KBus, data: Data) {
		self.write_with_mode(bus, AddrMode::AddressWithPredec(7), data, true);
	}
	
	fn pop(&mut self, bus: &mut dyn Motorola68KBus, size: Size) -> Data {
		self.read_with_mode(bus, AddrMode::AddressWithPostinc(7), size, true)
	}

	fn pop_u16(&mut self, bus: &mut dyn Motorola68KBus) -> u16 {
		if let Data::Word(data) =  self.pop(bus, Size::Word) {
			data
		}
		else {
			panic!("Incorrect size in pop u16");
		}
	}

	fn pop_u32(&mut self, bus: &mut dyn Motorola68KBus) -> u32 {
		if let Data::Long(data) =  self.pop(bus, Size::Long) {
			data
		}
		else {
			panic!("Incorrect size in pop u16");
		}
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
	fn read_with_mode(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, size: Size, advance: bool) -> Data {
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
					self.program_counter += Ord::max(size.width(), 2);
				};
				data
			},
		}
	}
	
	fn read_with_mode_u8(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u8 {
		let Data::Byte(data) = self.read_with_mode(bus, addr_mode, Size::Byte, advance)
		else {
			panic!("read_with_mode_u8 returned a non-byte value.");
		};
		return data;
	}
	
	fn read_with_mode_u16(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u16 {
		let Data::Word(data) = self.read_with_mode(bus, addr_mode, Size::Word, advance)
		else {
			panic!("read_with_mode_u16 returned a non-word value.");
		};
		return data;
	}
	
	fn read_with_mode_u32(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, advance: bool) -> u32 {
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
				self.write_register(Register::new(reg as usize, false), data)
			},
			AddrMode::AddressReg(reg) => {
				self.write_register(Register::new(reg as usize, true), data)
			},
			_ => panic!("Unimplemented write addressing mode {:?}.", addr_mode),
		}
	}
	
	fn calc_addr(&mut self, bus: &mut dyn Motorola68KBus, addr_mode: AddrMode, size: Size, advance: bool) -> u32 {
		let address = match addr_mode {
			AddrMode::Address(reg) => {
				self.read_register_u32(Register::new(reg, true))
			},
			AddrMode::AddressWithPostinc(reg) => {
				let register = Register::new(reg, true);
				let address = self.read_register_u32(register);
				if advance {
					// Stack pointer always increments by a minimum of 2 bytes
					if reg == 7 && size == Size::Byte {
						self.write_register(register, Data::Long(address.wrapping_add(2)))
					} else {
						self.write_register(register, Data::Long(address.wrapping_add(size.width())));
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
						self.write_register(register, Data::Long(address.wrapping_sub(2)))
					} else {
						self.write_register(register, Data::Long(address.wrapping_sub(size.width())));
					};
				};
				address.wrapping_sub(size.width())
			},
			AddrMode::AddressWithDisp(reg) => {
				let disp: i32 = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
				let address = self.read_register_u32(Register::new(reg, true));
				if advance {
					self.program_counter += 2;
				};
				address.wrapping_add_signed(disp)
			},
			AddrMode::AddressWithIndex(reg) => {
				let extension_word = bus.read_u16(self.program_counter);
				let index: i32 = CPU::sign_extend_u8((extension_word & 0xFF) as u8);
				let address = self.read_register_u32(Register::new(reg, true));
				let mode = (extension_word & 0x8000) >> 15;
				let reg_2 = (extension_word & 0x7000) >> 12;
				let size = if ((extension_word & 0x0800) >> 11) == 1 { Size::Long } else { Size::Word };
				let reg_data = self.read_register(Register::new(reg_2 as usize, mode == 1), size).sign_extend();
				if advance {
					self.program_counter += 2;
				};
				match reg_data {
					Data::Long(d) => address.wrapping_add_signed(index).wrapping_add(d),
					_ => panic!("Invalid size in Address With Index"),
				}
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
		};
		address & 0x00FFFFFF
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

	fn get_i(&self) -> u16 {
		(self.status_register & 0b0000011100000000) >> 8
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

	fn handle_result_flags(&mut self, data: Data) {
		self.set_n(data.is_negative());
		self.set_z(data.is_zero());
		self.set_v(false);
		self.set_c(false);
	}

	fn handle_add_flags(&mut self, dest: Data, source: Data, result: Data) {
		self.set_x(result < dest || result < source);
		self.set_n(result.is_negative());
		self.set_z(result.is_zero());
		self.set_v(dest.is_negative() == source.is_negative() && dest.is_negative() != result.is_negative());
		self.set_c(result < dest || result < source);
	}

	fn handle_sub_flags(&mut self, dest: Data, source: Data, result: Data) {
		self.set_x(result > dest || result > source);
		self.set_n(result.is_negative());
		self.set_z(result.is_zero());
		self.set_v(dest.is_negative() != source.is_negative() && dest.is_negative() != result.is_negative());
		self.set_c(result > dest || result > source);
	}

	fn handle_compare_flags(&mut self, dest: Data, source: Data, result: Data) {
		self.set_n(result.is_negative());
		self.set_z(result.is_zero());
		self.set_v(dest.is_negative() != source.is_negative() && dest.is_negative() != result.is_negative());
		self.set_c(result > dest || result > source);
	}
	
	#[bitmatch]
	pub fn run_opcode(&mut self, bus: &mut dyn Motorola68KBus) -> () {
		#[cfg(feature = "trace")]
		print!("{:#010X} {:#010} ", self.program_counter, self.cycles);
		let opcode = self.decode_opcode(bus);
		self.program_counter += 2;
		let mut cycles = 0;
		match opcode {
			Opcode::OrI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let data = imm | data;
				self.handle_result_flags(data);
				self.write_with_mode(bus, addr_mode, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("ORI {},{} = {}", imm, addr_mode, data);
			},
			Opcode::AndI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let data = imm & data;
				self.handle_result_flags(data);
				self.write_with_mode(bus, addr_mode, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("ANDI #{imm},{addr_mode} = {data}");
			},
			Opcode::SubI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let new_data = data - imm;
				self.handle_sub_flags(data, imm, new_data);
				self.write_with_mode(bus, addr_mode, new_data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("SUBI #{imm},{addr_mode} = {new_data}");
			},
			Opcode::AddI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let new_data = data + imm;
				self.handle_add_flags(data, imm, new_data);
				self.write_with_mode(bus, addr_mode, new_data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("ADDI #{imm},{addr_mode} = {new_data}");
			},
			Opcode::EorI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, false);
				let data = imm ^ data;
				self.handle_result_flags(data);
				self.write_with_mode(bus, addr_mode, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("EORI {},{} = {}", imm, addr_mode, data);
			},
			Opcode::CmpI { size, addr_mode } => {
				let imm = self.read_with_mode(bus, AddrMode::Immediate, size, true);
				let data = self.read_with_mode(bus, addr_mode, size, true);
				let new_data = data - imm;
				self.handle_compare_flags(data, imm, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("CMPI #imm,{addr_mode}");
			},
			Opcode::Btst { addr_mode } => {
				let size = match addr_mode {
					AddrMode::DataReg(_) => Size::Long,
					_ => Size::Byte,
				};
				let data = self.read_with_mode(bus, addr_mode, size, true);
				let bit_select = self.read_with_mode(bus, AddrMode::Immediate, Size::Word, true);
				match (data, bit_select) {
					(Data::Byte(d), Data::Word(s)) => self.set_z(((d >> (s % 8)) & 0b1) == 0),
					(Data::Long(d), Data::Word(s)) => self.set_z(((d >> (s % 32)) & 0b1) == 0),
					_ => panic!("Incorrect data sizes in BTST"),
				};
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("BTST #{bit_select},{addr_mode}");
			}
			Opcode::MoveA { size, dest, source } => {
				let data = self.read_with_mode(bus, source, size, true);
				self.write_register(Register::from_areg(dest), data.sign_extend());
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("MOVEA {},A{} = {}", source, dest.get(), data);
			},
			Opcode::Move { size, dest, source } => {
				let data = self.read_with_mode(bus, source, size, true);
				self.handle_result_flags(data);
				self.write_with_mode(bus, dest, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("MOVE {},{} = {}", source, dest, data);
			},
			Opcode::MoveToSr { addr_mode } => {
				self.status_register = self.read_with_mode_u16(bus, addr_mode, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, Size::Word);
				#[cfg(feature = "trace")]
				println!("MOVE {},SR = {:#06X}", addr_mode, self.status_register);
			},
			Opcode::MoveFromSr { addr_mode } => {
				self.write_with_mode(bus, addr_mode, Data::Word(self.status_register), true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, Size::Word);
				#[cfg(feature = "trace")]
				println!("MOVE SR,{} = {:#06X}", addr_mode, self.status_register);
			}
			Opcode::Clr { size, addr_mode } => {
				// CLR generates a read before writing to the effective address
				let _ = self.read_with_mode(bus, addr_mode, size, false);
				let data = match size {
					Size::Byte => Data::Byte(0),
					Size::Word => Data::Word(0),
					Size::Long => Data::Long(0),
				};
				self.set_n(false);
				self.set_z(true);
				self.set_v(false);
				self.set_c(false);
				self.write_with_mode(bus, addr_mode, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("CLR {}", addr_mode);
			},
			Opcode::Ext { size, dest } => {
				let register = Register::from_dreg(dest);
				let data = match size {
					Size::Word => {
						Data::Word(((self.read_register_u8(register) as i8) as i16) as u16)
					},
					Size::Long => {
						self.read_register(register, Size::Word).sign_extend()
					},
					_ => panic!("Incorrect size in EXT"),
				};
				self.handle_result_flags(data);
				self.write_register(register, data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("CLR {dest}");
			},
			Opcode::Swap { dest } => {
				let register = Register::from_dreg(dest);
				let data = self.read_register_u32(register);
				let new_data = Data::Long((data << 16) | (data >> 16));
				self.handle_result_flags(new_data);
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("SWAP {register} = {new_data}");
			},
			Opcode::Pea { addr_mode } => {
				let address = self.calc_addr(bus, addr_mode, Size::Long, true);
				self.push(bus, Data::Long(address));
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("PEA {}", addr_mode);
			}
			Opcode::Tst { size, addr_mode } => {
				let data = self.read_with_mode(bus, addr_mode, size, true);
				let zero = match size {
					Size::Byte => Data::Byte(0),
					Size::Word => Data::Word(0),
					Size::Long => Data::Long(0),
				};
				self.handle_compare_flags(data, zero, data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("TST {} = CCR {:#04X}", addr_mode, self.status_register);
			},
			Opcode::Link { frame_pointer } => {
				let register = Register::from_areg(frame_pointer);
				let data = self.read_register(register, Size::Long);
				self.push(bus, data);
				let stack_register = Register::new(7, true);
				let stack_pointer = Data::Long(self.read_register_u32(stack_register));
				self.write_register(register, stack_pointer);
				let disp = self.read_with_mode(bus, AddrMode::Immediate, Size::Word, true).sign_extend();
				self.write_register(stack_register, stack_pointer + disp);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("LINK {frame_pointer}");
			},
			Opcode::Unlnk { frame_pointer } => {
				let register = Register::from_areg(frame_pointer);
				let stack_pointer = self.read_register(register, Size::Long);
				let stack_register = Register::new(7, true);
				self.write_register(stack_register, stack_pointer);
				let data = self.pop(bus, Size::Long);
				self.write_register(register, data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("UNLNK {frame_pointer}");
			}
			Opcode::MoveUsp { dir, a } => {
				match dir {
					MoveDirection::RegToMem => {
						let data = self.read_register_u32(Register::from_areg(a));
						self.usp = data;
						#[cfg(feature = "trace")]
						println!("MOVE {},USP", a);
					}
					MoveDirection::MemToReg => {
						self.write_register(Register::from_areg(a), Data::Long(self.usp));
						#[cfg(feature = "trace")]
						println!("MOVE USP,{}", a);
					}
				};
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
			},
			Opcode::Nop => {
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("NOP");
			}
			Opcode::Not { size, addr_mode } => {
				let data = self.read_with_mode(bus, addr_mode, size, false) ^ Data::max(size);
				self.handle_result_flags(data);
				self.write_with_mode(bus, addr_mode, data, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("NOT {addr_mode} = {data}");
			}
			Opcode::Rte => {
				self.status_register = self.pop_u16(bus);
				self.program_counter = self.pop_u32(bus);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("RTE");
			}
			Opcode::Rts => {
				let pc = self.pop(bus, Size::Long);
				match pc {
					Data::Long(d) => self.program_counter = d,
					_ => panic!("Incorrect data size in RTS"),
				};
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("RTS");
			}
			Opcode::Jsr { addr_mode } => {
				let jump_pc = self.calc_addr(bus, addr_mode, Size::Long, true);
				self.push(bus, Data::Long(self.program_counter));
				self.program_counter = jump_pc;
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("JSR {}", addr_mode);
			},
			Opcode::Jmp { addr_mode } => {
				self.program_counter = self.calc_addr(bus, addr_mode, Size::Long, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("JMP {}", addr_mode);
			},
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
				let mut reg_count = 0;
				// MOVEM has a special case handling of address modes
				// If it's predec or postinc, it advances on each write or read as normal
				// If it's anything else, it calculates the address, and then increments it
				// for each register in order to read/write all of them in a line of memory.
				// This sucks.
				let mut address = match addr_mode {
					AddrMode::AddressWithPredec(_)
						| AddrMode::AddressWithPostinc(_) => None,
					_ => Some(self.calc_addr(bus, addr_mode, size, true)),
				};
				match dir {
					MoveDirection::RegToMem => {
						for i in (0..16).rev() {
							if reg_flags[i] {
								let data = self.read_register(reg_list[i], size);
								if let AddrMode::AddressWithPredec(_) = addr_mode {
									self.write_with_mode(bus, addr_mode, data, true);
								}
								else {
									CPU::write(bus, address.unwrap(), data);
									address = Some(address.unwrap() + size.width());
								}
								reg_count += 1;
							}
						};
						#[cfg(feature = "trace")]
						println!("MOVEM {:#06X},{}", reg_bits, addr_mode);
					},
					MoveDirection::MemToReg => {
						for i in (0..16).rev() {
							if reg_flags[i] {
								let data = if let AddrMode::AddressWithPostinc(_) = addr_mode {
									self.read_with_mode(bus, addr_mode, size, true)
								}
								else {
									let d = CPU::read(bus, address.unwrap(), size);
									address = Some(address.unwrap() + size.width());
									d
								};
								self.write_register(reg_list[i], data);
								reg_count += 1;
							}
						};
						#[cfg(feature = "trace")]
						println!("MOVEM {},{:#06X}", addr_mode, reg_bits);
					}
				};
				cycles += calc_opcode_cycles(opcode, None, None, Some(reg_count), None);
			},
			Opcode::Lea { dest, addr_mode } => {
				let address = self.calc_addr(bus, addr_mode, Size::Long, true);
				self.write_register(Register::new(dest.get(), true), Data::Long(address));
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("LEA {},A{} = {:#010X}", addr_mode, dest.get(), address);
			},
			Opcode::AddQ { data, size, addr_mode } => {
				let size = match addr_mode {
					AddrMode::AddressReg(_) => Size::Long,
					_ => size,
				};
				let value = self.read_with_mode(bus, addr_mode, size, false);
				let data = match size {
					Size::Byte => Data::Byte(data),
					Size::Word => Data::Word(data as u16),
					Size::Long => Data::Long(data as u32),
				};
				let new_value = value + data;
				if let AddrMode::AddressReg(_) = addr_mode { }
				else {
					self.handle_add_flags(value, data, new_value);
				};
				self.write_with_mode(bus, addr_mode, new_value, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("ADDQ #{data},{addr_mode}");
			},
			Opcode::SubQ { data, size, addr_mode } => {
				let size = match addr_mode {
					AddrMode::AddressReg(_) => Size::Long,
					_ => size,
				};
				let value = self.read_with_mode(bus, addr_mode, size, false);
				let data = match size {
					Size::Byte => Data::Byte(data),
					Size::Word => Data::Word(data as u16),
					Size::Long => Data::Long(data as u32),
				};
				let new_value = value - data;
				if let AddrMode::AddressReg(_) = addr_mode { }
				else {
					self.handle_sub_flags(value, data, new_value);
				};
				self.write_with_mode(bus, addr_mode, new_value, true);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
				#[cfg(feature = "trace")]
				println!("SUBQ #{data},{addr_mode}");
			}
			Opcode::DBcc { cond, loop_down } => {
				let mut branch_taken = false;
				let disp = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
				let mut loop_count = self.read_register_u16(Register::from_dreg(loop_down));
				if !cond.check(self.get_ccr_flags()) {
					loop_count = loop_count.wrapping_add_signed(-1);
					self.write_register(Register::from_dreg(loop_down), Data::Word(loop_count));
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
				cycles += calc_opcode_cycles(opcode, Some(branch_taken), Some(loop_count == 0xFFFF), None, None);
				#[cfg(feature = "trace")]
				println!("DB{cond},{loop_down} = PC {:#010X}, {loop_down} = {loop_count:#06X}", self.program_counter);
			},
			Opcode::Bsr { disp } => {
				let (displacement, return_address) = if disp == 0 {
					let long_disp = CPU::sign_extend_u16(bus.read_u16(self.program_counter));
					(long_disp, self.program_counter + 2)
				}
				else {
					(disp, self.program_counter)
				};
				self.push(bus, Data::Long(return_address));
				self.program_counter = self.program_counter.wrapping_add_signed(displacement);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("BSR = PC {:#010X}", self.program_counter);
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
				cycles += calc_opcode_cycles(opcode, Some(branch_taken), None, None, None);
				if cond == Condition::True {
					#[cfg(feature = "trace")]
					println!("BRA = PC {:#010X}", self.program_counter);
				}
				else {
					#[cfg(feature = "trace")]
					println!("B{cond} = PC {:#010X}", self.program_counter);
				}
			},
			Opcode::MoveQ { dest, data } => {
				self.set_n((data & 0x80) == 0x80);
				self.set_z(data == 0);
				self.set_v(false);
				self.set_c(false);
				self.write_register(Register::from_dreg(dest), Data::Long(CPU::sign_extend_u8(data) as u32));
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				let d = dest.get();
				#[cfg(feature = "trace")]
				println!("MOVEQ = D{} {:#010X}", d, self.d[d]);
			},
			Opcode::DivU { dest, source } => {
				let register = Register::from_dreg(dest);
				let dest_data = self.read_register_u32(register);
				let source_data = self.read_with_mode_u16(bus, source, true) as u32;
				if source_data == 0 {
					panic!("Divide by zero exception unimplemented.");
				}
				if (dest_data >> 16) > source_data {
					self.set_v(true);
					#[cfg(feature = "trace")]
					println!("DIVU {source},{dest} = Overflow");
				}
				else {
					self.set_v(false);
					let quotient = dest_data / source_data;
					let remainder = dest_data % source_data;
					self.set_n((quotient & 0x8000) == 0x8000);
					self.set_z(quotient == 0);
					let new_data = Data::Long(quotient + (remainder << 16));
					self.write_register(register, new_data);
					#[cfg(feature = "trace")]
					println!("DIVU {source},{dest} = {new_data}");
				}
				let cycle_args = ((dest_data as u64) << 32) + (source_data as u64);
				cycles += calc_opcode_cycles(opcode, None, None, Some(cycle_args), None)
					+ self.calc_addr_cycles(source, Size::Word);
			},
			Opcode::Or { reg, dir, size, addr_mode } => {
				let register = Register::from_dreg(reg);
				let ea_data = self.read_with_mode(bus, addr_mode, size, dir == BinOpDirection::ToReg);
				let reg_data = self.read_register(register, size);
				let new_data = ea_data | reg_data;
				match dir {
					BinOpDirection::ToEA => {
						self.write_with_mode(bus, addr_mode, new_data, true);
						#[cfg(feature = "trace")]
						println!("OR {reg},{addr_mode} = {new_data}");
					},
					BinOpDirection::ToReg => {
						self.write_register(register, new_data);
						#[cfg(feature = "trace")]
						println!("OR {addr_mode},{reg} = {new_data}");
					},
				};
				self.handle_result_flags(new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
			},
			Opcode::Sub { reg, dir, size, addr_mode } => {
				let register = Register::from_dreg(reg);
				let ea_data = self.read_with_mode(bus, addr_mode, size, dir == BinOpDirection::ToReg);
				let reg_data = self.read_register(register, size);
				let source = match dir {
					BinOpDirection::ToReg => ea_data,
					BinOpDirection::ToEA => reg_data,
				};
				let dest = match dir {
					BinOpDirection::ToReg => reg_data,
					BinOpDirection::ToEA => ea_data,
				};
				let new_data = dest - source;
				match dir {
					BinOpDirection::ToReg => {
						self.write_register(register, new_data);
						#[cfg(feature = "trace")]
						println!("SUB {addr_mode},{register} = {new_data}");
					},
					BinOpDirection::ToEA => {
						self.write_with_mode(bus, addr_mode, new_data, true);
						#[cfg(feature = "trace")]
						println!("SUB {register},{addr_mode} = {new_data}");
					}
				};
				self.handle_sub_flags(dest, source, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
			}
			Opcode::SubA { dest, size, source } => {
				let register = Register::from_areg(dest);
				let ea_data = self.read_with_mode(bus, source, size, true).sign_extend();
				let reg_data = self.read_register(register, Size::Long);
				let new_data = reg_data - ea_data;
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("SUBA {source},{dest} = {new_data}");
			},
			Opcode::Cmp { dest, size, source } => {
				let source_data = self.read_with_mode(bus, source, size, true);
				let dest_data = self.read_register(Register::from_dreg(dest), size);
				let new_data = dest_data - source_data;
				self.handle_compare_flags(dest_data, source_data, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(source, size);
				#[cfg(feature = "trace")]
				println!("CMP {source},{dest}");
			},
			Opcode::CmpA { dest, size, source } => {
				let source_data = self.read_with_mode(bus, source, size, true).sign_extend();
				let dest_data = self.read_register(Register::from_areg(dest), size).sign_extend();
				let new_data = dest_data - source_data;
				self.handle_compare_flags(dest_data, source_data, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(source, size);
				#[cfg(feature = "trace")]
				println!("CMPA {source},{dest}");
			},
			Opcode::MulU { dest, source } => {
				let register = Register::from_dreg(dest);
				let source_data = self.read_with_mode_u16(bus, source, true);
				let dest_data = self.read_register_u16(register);
				let new_data = (source_data as u32) * (dest_data as u32);
				let mut calc_count = 0; // Cycle count increases based on number of 1s in EA
				for n in 0..16 {
					calc_count += (source_data >> n) & 1;
				}
				self.write_register(register, Data::Long(new_data));
				cycles += calc_opcode_cycles(opcode, None, None, Some(calc_count as u64), None);
				#[cfg(feature = "trace")]
				println!("MULU {source},{dest} = {new_data:#010X}");
			}
			Opcode::And { reg, dir, size, addr_mode } => {
				let register = Register::from_dreg(reg);
				let ea_data = self.read_with_mode(bus, addr_mode, size, dir == BinOpDirection::ToReg);
				let reg_data = self.read_register(register, size);
				let new_data = ea_data & reg_data;
				match dir {
					BinOpDirection::ToEA => {
						self.write_with_mode(bus, addr_mode, new_data, true);
						#[cfg(feature = "trace")]
						println!("AND {reg},{addr_mode} = {new_data}");
					},
					BinOpDirection::ToReg => {
						self.write_register(register, new_data);
						#[cfg(feature = "trace")]
						println!("AND {addr_mode},{reg} = {new_data}");
					},
				};
				self.handle_result_flags(new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
			},
			Opcode::Add { reg, dir, size, addr_mode } => {
				let register = Register::from_dreg(reg);
				let ea_data = self.read_with_mode(bus, addr_mode, size, dir == BinOpDirection::ToReg);
				let reg_data = self.read_register(register, size);
				let (source, dest) = match dir {
					BinOpDirection::ToReg => (ea_data, reg_data),
					BinOpDirection::ToEA => (reg_data, ea_data),
				};
				let new_data = dest + source;
				match dir {
					BinOpDirection::ToReg => {
						self.write_register(register, new_data);
						#[cfg(feature = "trace")]
						println!("ADD {addr_mode},{register} = {new_data}");
					},
					BinOpDirection::ToEA => {
						self.write_with_mode(bus, addr_mode, new_data, true);
						#[cfg(feature = "trace")]
						println!("ADD {register},{addr_mode} = {new_data}");
					}
				};
				self.handle_add_flags(dest, source, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(addr_mode, size);
			},
			Opcode::AddX { dest, size, source } => {
				let dest_data = self.read_with_mode(bus, dest, size, false);
				let source_data = self.read_with_mode(bus, dest, size, true);
				let x = if self.get_ccr_flags().x { Data::from_size(size, 1) } else { Data::from_size(size, 0) };
				let new_data = dest_data + source_data + x;
				self.write_with_mode(bus, dest, new_data, true);
				self.handle_add_flags(dest_data, source_data, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None) + self.calc_addr_cycles(source, size);
				#[cfg(feature = "trace")]
				println!("ADDX {source},{dest} = {new_data}");
			}
			Opcode::AddA { dest, size, source } => {
				let register = Register::from_areg(dest);
				let ea_data = self.read_with_mode(bus, source, size, true).sign_extend();
				let reg_data = self.read_register(register, Size::Long);
				let new_data = reg_data + ea_data;
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, None);
				#[cfg(feature = "trace")]
				println!("ADDA {source},{dest} = {new_data}");
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
					Size::Byte => Data::Byte(wide_data as u8),
					Size::Word => Data::Word(wide_data as u16),
					Size::Long => Data::Long(wide_data as u32),
				};
				self.set_z(new_data.is_zero());
				self.set_n(new_data.is_negative());
				self.set_v(false);
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, Some(rotation as u64));
				#[cfg(feature = "trace")]
				println!("LS{dir} {mode}{rot},{reg}");
			},
			Opcode::RoXdToD { rot, dir, size, mode, reg } => {
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
				let x = if self.get_ccr_flags().x { 1 } else { 0 };
				if rotation > 0 {
					match dir {
						RotateDirection::Right => {
							let extension_bit = (wide_data >> (rotation - 1)) & 0x1 == 0x1;
							wide_data = (wide_data >> rotation)
								+ (wide_data << ((size.width() * 8 + 1) - (rotation as u32)))
								+ (x << ((size.width() * 8 + 1) - (rotation as u32)));
							self.set_c(extension_bit);
							self.set_x(extension_bit);
						},
						RotateDirection::Left => {
							let extension_bit = (wide_data >> (size.width() * 8 - (rotation as u32))) & 0x1 == 0x1;
							wide_data = (wide_data << rotation)
								+ (wide_data >> ((size.width() * 8 + 1) - (rotation as u32)))
								+ (x >> ((size.width() * 8 + 1) - (rotation as u32)));
							let carry_bit = (wide_data & 0x1) == 0x1;
							self.set_c(extension_bit);
							self.set_x(extension_bit);
						}
					}
				}
				else {
					self.set_c(self.get_ccr_flags().x);
				}
				let new_data = match size {
					Size::Byte => Data::Byte(wide_data as u8),
					Size::Word => Data::Word(wide_data as u16),
					Size::Long => Data::Long(wide_data as u32),
				};
				self.set_z(new_data.is_zero());
				self.set_n(new_data.is_negative());
				self.set_v(false);
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, Some(rotation as u64));
				#[cfg(feature = "trace")]
				println!("ROX{dir} {mode}{rot},{reg}");
			},
			Opcode::RodToD { rot, dir, size, mode, reg } => {
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
							wide_data = (wide_data >> rotation) + (wide_data << ((size.width() * 8) - (rotation as u32)));
							let carry_bit = ((wide_data >> ((size.width() * 8) - 1)) & 0x1) == 0x1;
							self.set_c(carry_bit);
						},
						RotateDirection::Left => {
							wide_data = (wide_data << rotation) + (wide_data >> ((size.width() * 8) - (rotation as u32)));
							let carry_bit = (wide_data & 0x1) == 0x1;
							self.set_c(carry_bit);
						}
					}
				}
				else {
					self.set_c(false);
				}
				let new_data = match size {
					Size::Byte => Data::Byte(wide_data as u8),
					Size::Word => Data::Word(wide_data as u16),
					Size::Long => Data::Long(wide_data as u32),
				};
				self.set_z(new_data.is_zero());
				self.set_n(new_data.is_negative());
				self.set_v(false);
				self.write_register(register, new_data);
				cycles += calc_opcode_cycles(opcode, None, None, None, Some(rotation as u64));
				#[cfg(feature = "trace")]
				println!("RO{dir} {mode}{rot},{reg}");
			},
			_ => panic!("{opcode} unimplemented."),
		}
		if cycles == 0 {
			panic!("{opcode} failed to advance cycles.");
		}
		self.countdown += cycles;
	}

	#[bitmatch]
	fn decode_opcode(&self, bus: &mut dyn Motorola68KBus) -> Opcode {
		let opcode = bus.read_u16(self.program_counter);
		#[cfg(feature = "trace")]
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
				Opcode::Bsr {
					disp: CPU::sign_extend_u8(d as u8)
				}
			}
			// Bcc
			"0110_cccc_dddd_dddd" => {
				Opcode::Bcc {
					cond: Condition::new(c),
					disp: CPU::sign_extend_u8(d as u8)
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
			// SUBA
			"1001_aaas_11mm_mxxx" => {
				Opcode::SubA {
					dest: AReg::new(a),
					size: Size::from_bit(s == 1),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
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
			// SUB
			"1001_dddr_ssmm_mxxx" => {
				Opcode::Sub {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
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
			// ADDA
			"1101_aaas_11mm_mxxx" => {
				Opcode::AddA {
					dest: AReg::new(a),
					size: Size::from_bit(s == 1),
					source: self.decode_addressing_mode(((m << 3) + x) as u8),
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
			// ADD
			"1101_dddr_ssmm_mxxx" => {
				Opcode::Add {
					reg: DReg::new(d),
					dir: BinOpDirection::new(r == 1),
					size: Size::from_low_bits(s),
					addr_mode: self.decode_addressing_mode(((m << 3) + x) as u8),
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
	pub fn test_reset(&mut self, bus: &mut dyn Motorola68KBus) {
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