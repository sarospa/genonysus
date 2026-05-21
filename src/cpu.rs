#![allow(dead_code)]
#![allow(unused)]

use bitmatch::bitmatch;

mod cpu_support;
use cpu_support::*;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CPU {
	pub cart_memory: Vec<u8>,
	pub ram: Vec<u8>,
	pub program_counter: u32,
	pub status_register: u16,
	pub d: [u32; 8],
	pub a: [u32; 8],
	pub controller1: Controller,
	pub controller1_control: u8,
	pub controller2: Controller,
	pub controller2_control: u8,
}

impl CPU {
	pub fn new(rom: &Vec<u8>) -> CPU {
		let mut cart_memory = vec![0; CART_SIZE];
		for i in 0..Ord::min(CART_SIZE, rom.len()) {
			cart_memory[i] = rom[i];
		}
		
		CPU {
			cart_memory: cart_memory,
			ram: vec![0; RAM_SIZE],
			program_counter: CPU::load_u32_at(&rom, 4),
			status_register: 0,
			d: [0; 8],
			a: [0, 0, 0, 0, 0, 0, 0, CPU::load_u32_at(&rom, 0)],
			controller1: Controller::Unplugged,
			controller1_control: 0,
			controller2: Controller::Unplugged,
			controller2_control: 0,
		}
	}
	
	fn read_u8(&self, address: u32) -> u8 {
		let address_index = (address as usize) & ADDRESS_SPACE;
		match address_index {
			0..CART_SIZE => self.cart_memory[address_index],
			RAM_START..RAM_END => self.cart_memory[address_index - RAM_START],
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
	
	fn read(&self, address: u32, size: &Size) -> Data {
		match size {
			Size::Byte => Data::Byte(self.read_u8(address)),
			Size::Word => Data::Word(self.read_u16(address)),
			Size::Long => Data::Long(self.read_u32(address)),
		}
	}
	
	fn read_register(&self, size: &Size, reg: &Register) -> Data {
		let data = match reg {
			Register::A(a) => self.a[a.get()],
			Register::D(d) => self.d[d.get()],
		};
		match size {
			Size::Byte => Data::Byte(data as u8),
			Size::Word => Data::Word(data as u16),
			Size::Long => Data::Long(data),
		}
	}
	
	fn read_register_u8(&self, reg: &Register) -> u8 {
		match reg {
			Register::A(a) => self.a[a.get()] as u8,
			Register::D(d) => self.d[d.get()] as u8,
		}
	}
	
	fn read_register_u16(&self, reg: &Register) -> u16 {
		match reg {
			Register::A(a) => self.a[a.get()] as u16,
			Register::D(d) => self.d[d.get()] as u16,
		}
	}
	
	fn read_register_u32(&self, reg: &Register) -> u32 {
		match reg {
			Register::A(a) => self.a[a.get()],
			Register::D(d) => self.d[d.get()],
		}
	}
	
	fn write_u8(&mut self, address: u32, data: u8) {
		let address_index = (address as usize) & ADDRESS_SPACE;
		match address_index {
			0..CART_SIZE => self.cart_memory[address_index] = data,
			RAM_START..RAM_END => self.cart_memory[address_index - RAM_START] = data,
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
	
	fn write(&mut self, address: u32, data: &Data) {
		match data {
			Data::Byte(d) => self.write_u8(address, *d),
			Data::Word(d) => self.write_u16(address, *d),
			Data::Long(d) => self.write_u32(address, *d),
		}
	}
	
	fn write_register(&mut self, data: &Data, reg: &Register) {
		let reg_value = match reg {
			Register::A(a) => self.a[a.get()],
			Register::D(d) => self.d[d.get()],
		};
		let value = match *data {
			Data::Byte(v) => (reg_value & 0xFFFFFF00) | (v as u32),
			Data::Word(v) => (reg_value & 0xFFFF0000) | (v as u32),
			Data::Long(v) => v,
		};
		match reg {
			Register::A(a) => self.a[a.get()] = value,
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
	fn read_with_mode(&mut self, addr_mode: &AddrMode, size: &Size) -> Data {
		match addr_mode {
			AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_)
			| AddrMode::AddressWithDisp(_) | AddrMode::AddressWithIndex(_)
			| AddrMode::PCWithDisp | AddrMode::PCWithIndex | AddrMode::AbsoluteShort
			| AddrMode::AbsoluteLong => {
				let address = self.calc_addr(&addr_mode, &size);
				self.read(address, &size)
			},
			AddrMode::DataReg(reg) => {
				self.read_register(&size, &Register::new(*reg as usize, false))
			},
			AddrMode::AddressReg(reg) => {
				self.read_register(&size, &Register::new(*reg as usize, true))
			}
			AddrMode::Immediate => {
				let data = match size {
					Size::Byte => self.read(self.program_counter + 1, &Size::Byte), // data is stored in the low byte of extension word
					Size::Word => self.read(self.program_counter, &Size::Word),
					Size::Long => self.read(self.program_counter, &Size::Long),
				};
				data
			},
			_ => panic!("Unimplemented read addressing mode {:?}.", addr_mode),
		}
	}
	
	fn read_with_mode_u8(&mut self, addr_mode: &AddrMode) -> u8 {
		let Data::Byte(data) = self.read_with_mode(addr_mode, &Size::Byte)
		else {
			panic!("read_with_mode_u8 returned a non-byte value.");
		};
		return data;
	}
	
	fn read_with_mode_u16(&mut self, addr_mode: &AddrMode) -> u16 {
		let Data::Word(data) = self.read_with_mode(addr_mode, &Size::Word)
		else {
			panic!("read_with_mode_u16 returned a non-word value.");
		};
		return data;
	}
	
	fn read_with_mode_u32(&mut self, addr_mode: &AddrMode) -> u32 {
		let Data::Long(data) = self.read_with_mode(addr_mode, &Size::Long)
		else {
			panic!("read_with_mode_u32 returned a non-long value.");
		};
		return data;
	}
	
	fn write_with_mode(&mut self, addr_mode: &AddrMode, data: &Data) {
		match addr_mode {
			AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_)
			| AddrMode::AddressWithDisp(_) | AddrMode::AddressWithIndex(_)
			| AddrMode::PCWithDisp | AddrMode::PCWithIndex | AddrMode::AbsoluteShort
			| AddrMode::AbsoluteLong => {
				let address = self.calc_addr(&addr_mode, &Size::from_data(&data));
				self.write(address, &data);
			},
			AddrMode::DataReg(reg) => {
				self.write_register(&data, &Register::new(*reg as usize, false))
			},
			AddrMode::AddressReg(reg) => {
				self.write_register(&data, &Register::new(*reg as usize, true))
			}
			_ => panic!("Unimplemented write addressing mode {:?}.", addr_mode),
		}
	}
	
	fn calc_addr(&self, addr_mode: &AddrMode, size: &Size) -> u32 {
		match addr_mode {
			AddrMode::Address(reg) => {
				self.read_register_u32(&Register::new(*reg, true))
			},
			AddrMode::AddressWithPostinc(reg) => {
				let reg = Register::new(*reg, true);
				let address = self.read_register_u32(&reg);
				address
			},
			AddrMode::AddressWithPredec(reg) => {
				let reg = Register::new(*reg, true);
				let address = self.read_register_u32(&reg);
				address - size.length()
			},
			AddrMode::AddressWithDisp(reg) => {
				let disp: i32 = CPU::sign_extend_u16(self.read_u16(self.program_counter));
				let address = self.read_register_u32(&Register::new(*reg, true));
				address.wrapping_add_signed(disp)
			},
			AddrMode::PCWithDisp => {
				let disp: i32 = CPU::sign_extend_u16(self.read_u16(self.program_counter));
				self.program_counter.wrapping_add_signed(disp)
			},
			AddrMode::AbsoluteShort => {
				let address = 0u32.wrapping_add_signed(CPU::sign_extend_u16(self.read_u16(self.program_counter)));
				address
			},
			AddrMode::AbsoluteLong => {
				let address = self.read_u32(self.program_counter);
				address
			},
			_ => panic!("Unimplemented address calculation addressing mode {:?}.", addr_mode),
		}
	}
	
	// Carry out all side effects of the address mode, like advancing the program counter or altering a register
	fn advance_with_mode(&mut self, addr_mode: &AddrMode, size: &Size) {
		match addr_mode {
			AddrMode::DataReg(_) | AddrMode::AddressReg(_) | AddrMode::Address(_) => {
				
			},
			AddrMode::AddressWithPostinc(reg) => {
				let reg = Register::new(*reg, true);
				let address = self.read_register_u32(&reg);
				self.write_register(&Data::Long(address + size.length()), &reg);
			},
			AddrMode::AddressWithPredec(reg) => {
				let reg = Register::new(*reg, true);
				let address = self.read_register_u32(&reg);
				self.write_register(&Data::Long(address - size.length()), &reg);
			},
			AddrMode::AddressWithDisp(_) | AddrMode::PCWithDisp | AddrMode::AbsoluteShort => {
				self.program_counter += 2;
			},
			AddrMode::AbsoluteLong => {
				self.program_counter += 4;
			},
			AddrMode::Immediate => {
				self.program_counter += Ord::max(size.length(), 2);
			},
			_ => panic!("Unimplemented addressing mode advance {:?}.", addr_mode),
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
	fn decode_opcode(&self) -> Opcode {
		let opcode = self.read_u16(self.program_counter);
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
			// Bcc
			"0110_bbbb_dddd_dddd" => {
				Opcode::Bcc {
					cond: Condition::new(b),
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
			_ => panic!("Encountered unimplemented opcode {:#04X} located at {:#08X}", opcode, self.program_counter),
		}
	}
	
	#[bitmatch]
	pub fn run_opcode(&mut self) -> () {
		let opcode = self.decode_opcode();
		print!("{:#010X} ", self.program_counter);
		self.program_counter += 2;
		match opcode {
			Opcode::AndI { size, addr_mode } => {
				let imm = self.read_with_mode(&AddrMode::Immediate, &size);
				self.advance_with_mode(&AddrMode::Immediate, &size);
				let data = self.read_with_mode(&addr_mode, &size);
				let data = match (&imm, data) {
					(Data::Byte(a), Data::Byte(b)) => Data::Byte(a & b),
					(Data::Word(a), Data::Word(b)) => Data::Word(a & b),
					(Data::Long(a), Data::Long(b)) => Data::Long(a & b),
					_ => panic!("Non-matching data in ANDI"),
				};
				self.write_with_mode(&addr_mode, &data);
				self.advance_with_mode(&addr_mode, &size);
				println!("ANDI {},{} = {}", imm, addr_mode, data);
			}
			Opcode::MoveA { size, dest, source } => {
				let data = self.read_with_mode(&source, &size);
				self.advance_with_mode(&source, &size);
				self.write_register(&data.sign_extend(), &Register::from_areg(&dest));
				println!("MOVEA {},A{} = {}", source, dest.get(), data);
			}
			Opcode::Move { size, dest, source } => {
				let data = self.read_with_mode(&source, &size);
				self.advance_with_mode(&source, &size);
				self.write_with_mode(&dest, &data);
				self.advance_with_mode(&dest, &Size::from_data(&data));
				println!("MOVE {},{} = {}", source, dest, data);
			},
			Opcode::MoveToSr { addr_mode } => {
				self.status_register = self.read_with_mode_u16(&addr_mode);
				self.advance_with_mode(&addr_mode, &Size::Word);
				println!("MOVE {},SR = {:#06X}", addr_mode, self.status_register);
			},
			Opcode::Tst { size, addr_mode } => {
				let data = self.read_with_mode(&addr_mode, &size);
				self.advance_with_mode(&addr_mode, &size);
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
				println!("TST {} = CCR {:#04X}", addr_mode, self.status_register);
			},
			Opcode::MoveM { dir, size, addr_mode } => {
				let reg_bits = self.read_u16(self.program_counter);
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
				let mut address = self.calc_addr(&addr_mode, &size);
				self.advance_with_mode(&addr_mode, &size);
				match dir {
					MoveDirection::RegToMem => {
						for i in 15..=0 {
							if reg_flags[i] {
								if let AddrMode::AddressWithPredec(_) = addr_mode {
									address -= size.length();
								};
								self.write(address, &self.read_register(&size, &reg_list[i]));
								if let AddrMode::AddressWithPredec(_) = addr_mode { }
								else {
									address += size.length();
								};
							}
						};
						if let AddrMode::AddressWithPredec(reg) = addr_mode {
							self.write_register(&Data::Long(address), &Register::new(reg, true));
						};
						println!("MOVEM {:#06X},{}", reg_bits, addr_mode);
					},
					MoveDirection::MemToReg => {
						for i in 15..=0 {
							if reg_flags[i] {
								let data = self.read(address, &size);
								self.write_register(&self.read(address, &size), &reg_list[i]);
								address += size.length();
							}
						};
						if let AddrMode::AddressWithPostinc(reg) = addr_mode {
							self.write_register(&Data::Long(address), &Register::new(reg, true));
						};
						println!("MOVEM {},{:#06X}", addr_mode, reg_bits);
					}
				};
			},
			Opcode::Lea { dest, addr_mode } => {
				let address = self.calc_addr(&addr_mode, &Size::Long);
				self.advance_with_mode(&addr_mode, &Size::Long);
				self.write_register(&Data::Long(address), &Register::new(dest.get(), true));
				println!("LEA {},A{} = {:#010X}", addr_mode, dest.get(), address);
			},
			Opcode::Bcc { cond, disp } => {
				let f = self.get_ccr_flags();
				let disp = if disp == 0 {
					let long_disp = CPU::sign_extend_u16(self.read_u16(self.program_counter));
					self.program_counter += 2;
					long_disp
				}
				else {
					disp
				};
				let branch = match cond {
					Condition::True => true,
					Condition::False => panic!("BSR unimplemented."),
					Condition::Higher => !f.c && !f.z,
					Condition::LowerOrSame => f.c || f.z,
					Condition::CarryClear => !f.c,
					Condition::CarrySet => f.c,
					Condition::NotEqual => !f.z,
					Condition::Equal => f.z,
					Condition::OverflowClear => !f.v,
					Condition::OverflowSet => f.v,
					Condition::Plus => !f.n,
					Condition::Minus => f.n,
					Condition::GreaterOrEqual => (f.n && f.v) || (!f.n && !f.v), 
					Condition::LessThan => (f.n && !f.v) || (!f.n && f.v),
					Condition::GreaterThan => (f.n && f.v && !f.z) || (!f.n && !f.v && !f.z),
					Condition::LessOrEqual => f.z || (f.n && !f.v) || (!f.n && f.v),
				};
				if branch {
					self.program_counter = self.program_counter.wrapping_add_signed(disp);
				};
				println!("B{cond} = PC {:#010X}", self.program_counter);
			},
			Opcode::MoveQ { dest, data } => {
				self.write_register(&Data::Long(CPU::sign_extend_u8(data) as u32), &Register::from_dreg(&dest));
				let d = dest.get();
				println!("MOVEQ = D{} {:#010X}", d, self.d[d]);
			},
			_ => panic!("{opcode} unimplemented."),
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
	
	// Helper function for manually writing 68K test code.
	#[cfg(test)]
	pub fn write_rom(&mut self, data: u8) {
		self.cart_memory[self.program_counter as usize] = data;
		self.program_counter += 1;
	}
	
	// Put the cpu in a clean state to start a test.
	#[cfg(test)]
	pub fn test_reset(&mut self) {
		self.program_counter = CPU::load_u32_at(&self.cart_memory, 4);
		self.status_register = 0x2000;
		self.a = [0; 8];
		self.d = [0; 8];
		self.ram = vec![0; RAM_SIZE];
	}
}