use std::ops::Add;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Sub;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Size {
	Byte,
	Word,
	Long
}

impl Size {
	pub fn from_low_bits(bits: u16) -> Size {
		match bits {
			0b00 => Size::Byte,
			0b01 => Size::Word,
			0b10 => Size::Long,
			_ => panic!("{:#b} is not a valid size.", bits),
		}
	}
	
	pub fn from_bit(bit: bool) -> Size {
		match bit {
			false => Size::Word,
			true => Size::Long,
		}
	}
	
	pub fn from_high_bits(bits: u16) -> Size {
		match bits {
			0b01 => Size::Byte,
			0b11 => Size::Word,
			0b10 => Size::Long,
			_ => panic!("{:#b} is not a valid size.", bits),
		}
	}
	
	pub fn length(&self) -> u32 {
		match self {
			Size::Byte => 1,
			Size::Word => 2,
			Size::Long => 4,
		}
	}
	
	pub fn from_data(data: Data) -> Size {
		match data {
			Data::Byte(_) => Size::Byte,
			Data::Word(_) => Size::Word,
			Data::Long(_) => Size::Long,
		}
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum Data {
	Byte(u8),
	Word(u16),
	Long(u32)
}
impl Data {
	pub fn sign_extend(self) -> Data {
		match self {
			Data::Byte(d) => Data::Long(((d as i8) as i32) as u32),
			Data::Word(d) => Data::Long(((d as i16) as i32) as u32),
			Data::Long(d) => Data::Long(d)
		}
	}
	
	pub fn is_negative(self) -> bool {
		match self {
			Data::Byte(d) => d & 0x80 == 0x80,
			Data::Word(d) => d & 0x8000 == 0x8000,
			Data::Long(d) => d & 0x80000000 == 0x80000000
		}
	}
	
	pub fn is_zero(self) -> bool {
		match self {
			Data::Byte(d) => d == 0,
			Data::Word(d) => d == 0,
			Data::Long(d) => d == 0,
		}
	}
	
	pub fn max(size: Size) -> Data {
		match size {
			Size::Byte => Data::Byte(0xFF),
			Size::Word => Data::Word(0xFFFF),
			Size::Long => Data::Long(0xFFFFFFFF),
		}
	}
}
impl fmt::Display for Data {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Data::Byte(v) => write!(f, "{:#04X}", v),
			Data::Word(v) => write!(f, "{:#06X}", v),
			Data::Long(v) => write!(f, "{:#010X}", v),
		}
	}
}
impl Add for Data {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		match (self, other) {
			(Data::Byte(a), Data::Byte(b)) => Data::Byte(a.wrapping_add(b)),
			(Data::Word(a), Data::Word(b)) => Data::Word(a.wrapping_add(b)),
			(Data::Long(a), Data::Long(b)) => Data::Long(a.wrapping_add(b)),
			_ => panic!("Mismatched data sizes in add operation."),
		}
	}
}
impl Sub for Data {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		match (self, other) {
			(Data::Byte(a), Data::Byte(b)) => Data::Byte(a.wrapping_sub(b)),
			(Data::Word(a), Data::Word(b)) => Data::Word(a.wrapping_sub(b)),
			(Data::Long(a), Data::Long(b)) => Data::Long(a.wrapping_sub(b)),
			_ => panic!("Mismatched data sizes in subtract operation."),
		}
	}
}
impl BitAnd for Data {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		match (self, other) {
			(Data::Byte(a), Data::Byte(b)) => Data::Byte(a & b),
			(Data::Word(a), Data::Word(b)) => Data::Word(a & b),
			(Data::Long(a), Data::Long(b)) => Data::Long(a & b),
			_ => panic!("Mismatched data sizes in bitwise and operation."),
		}
	}
}
impl BitOr for Data {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		match (self, other) {
			(Data::Byte(a), Data::Byte(b)) => Data::Byte(a | b),
			(Data::Word(a), Data::Word(b)) => Data::Word(a | b),
			(Data::Long(a), Data::Long(b)) => Data::Long(a | b),
			_ => panic!("Mismatched data sizes in bitwise or operation."),
		}
	}
}
impl BitXor for Data {
	type Output = Self;

	fn bitxor(self, other: Self) -> Self {
		match (self, other) {
			(Data::Byte(a), Data::Byte(b)) => Data::Byte(a ^ b),
			(Data::Word(a), Data::Word(b)) => Data::Word(a ^ b),
			(Data::Long(a), Data::Long(b)) => Data::Long(a ^ b),
			_ => panic!("Mismatched data sizes in bitwise xor operation."),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AReg { i: usize }
impl AReg {
	pub fn new(reg: u16) -> AReg {
		if reg >= 8 {
			panic!("{} is not a valid A register.", reg);
		}
		AReg { i: reg as usize }
	}
	
	pub fn get(&self) -> usize {
		self.i
	}
}
impl fmt::Display for AReg {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "A{}", self.i)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct DReg { i: usize }
impl DReg {
	pub fn new(reg: u16) -> DReg {
		if reg >= 8 {
			panic!("{} is not a valid D register.", reg);
		}
		DReg { i: reg as usize }
	}
	
	pub fn get(&self) -> usize {
		self.i
	}
}
impl fmt::Display for DReg {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "D{}", self.i)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
	A(AReg),
	D(DReg),
}
impl Register {
	pub fn new(reg: usize, is_a: bool) -> Register {
		match is_a {
			true => Register::A(AReg::new(reg as u16)),
			false => Register::D(DReg::new(reg as u16)),
		}
	}
	
	pub fn from_u16(reg: u16, is_a: bool) -> Register {
		Register::new(reg as usize, is_a)
	}
	
	pub fn from_areg(a: AReg) -> Register {
		Register::A(AReg::new(a.get() as u16))
	}
	
	pub fn from_dreg(d: DReg) -> Register {
		Register::D(DReg::new(d.get() as u16))
	}
}
impl fmt::Display for Register {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Register::A(a) => write!(f, "{}", a),
			Register::D(d) => write!(f, "{}", d),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Vector { i: u16 }
impl Vector {
	pub fn new(vector: u16) -> Vector {
		if vector >= 16 {
			panic!("{} is not a valid vector.", vector);
		}
		Vector { i: vector }
	}
}

#[derive(Clone, Copy)]
pub enum RotateDirection {
	Left,
	Right,
}
impl RotateDirection {
	pub fn new(bit: bool) -> RotateDirection {
		match bit {
			false => RotateDirection::Right,
			true => RotateDirection::Left,
		}
	}
}
impl fmt::Display for RotateDirection {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			RotateDirection::Right => write!(f, "R"),
			RotateDirection::Left => write!(f, "L"),
		}
	}
}

#[derive(Clone, Copy)]
pub enum RotateMode {
	Immediate,
	Register,
}
impl RotateMode {
	pub fn new(bit: bool) -> RotateMode {
		match bit {
			false => RotateMode::Immediate,
			true => RotateMode::Register,
		}
	}
}
impl fmt::Display for RotateMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			RotateMode::Immediate => write!(f, "#"),
			RotateMode::Register => write!(f, "D"),
		}
	}
}

#[derive(Clone, Copy)]
pub enum MoveDirection {
	RegToMem,
	MemToReg,
}
impl MoveDirection {
	pub fn new(bit: bool, is_movep: bool) -> MoveDirection {
		match (bit, is_movep) {
			(false, false) => MoveDirection::RegToMem,
			(true, false) => MoveDirection::MemToReg,
			(false, true) => MoveDirection::MemToReg,
			(true, true) => MoveDirection::RegToMem,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinOpDirection {
	ToReg,
	ToEA,
}
impl BinOpDirection {
	pub fn new(bit: bool) -> BinOpDirection {
		match bit {
			false => BinOpDirection::ToReg,
			true => BinOpDirection::ToEA,
		}
	}
}

#[derive(Clone, Copy)]
pub enum Condition {
	True,
	False,
	Higher,
	LowerOrSame,
	CarryClear,
	CarrySet,
	NotEqual,
	Equal,
	OverflowClear,
	OverflowSet,
	Plus,
	Minus,
	GreaterOrEqual,
	LessThan,
	GreaterThan,
	LessOrEqual,
}
impl Condition {
	pub fn new(bits: u16) -> Condition {
		match bits {
			0b0000 => Condition::True,
			0b0001 => Condition::False,
			0b0010 => Condition::Higher,
			0b0011 => Condition::LowerOrSame,
			0b0100 => Condition::CarryClear,
			0b0101 => Condition::CarrySet,
			0b0110 => Condition::NotEqual,
			0b0111 => Condition::Equal,
			0b1000 => Condition::OverflowClear,
			0b1001 => Condition::OverflowSet,
			0b1010 => Condition::Plus,
			0b1011 => Condition::Minus,
			0b1100 => Condition::GreaterOrEqual,
			0b1101 => Condition::LessThan,
			0b1110 => Condition::GreaterThan,
			0b1111 => Condition::LessOrEqual,
			_ => panic!("{:#b} is not a valid branch condition.", bits),
		}
	}
	
	pub fn check(&self, f: Flags) -> bool {
		match self {
			Condition::True => true,
			Condition::False => false,
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
		}
	}
}
impl fmt::Display for Condition {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Condition::True => write!(f, "T"),
			Condition::False => write!(f, "F"),
			Condition::Higher => write!(f, "HI" ),
			Condition::LowerOrSame => write!(f, "LS" ),
			Condition::CarryClear => write!(f, "CC"),
			Condition::CarrySet => write!(f, "CS"),
			Condition::NotEqual => write!(f, "NE"),
			Condition::Equal => write!(f, "EQ"),
			Condition::OverflowClear => write!(f, "VC"),
			Condition::OverflowSet => write!(f, "VS"),
			Condition::Plus => write!(f, "PL"),
			Condition::Minus => write!(f, "MI"),
			Condition::GreaterOrEqual => write!(f, "GE"), 
			Condition::LessThan => write!(f, "LT"),
			Condition::GreaterThan => write!(f, "GT"),
			Condition::LessOrEqual => write!(f, "LE"),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum AddrMode {
	DataReg(usize),
	AddressReg(usize),
	Address(usize),
	AddressWithPostinc(usize),
	AddressWithPredec(usize),
	AddressWithDisp(usize),
	AddressWithIndex(usize),
	PCWithDisp,
	PCWithIndex,
	AbsoluteShort,
	AbsoluteLong,
	Immediate
}
impl fmt::Display for AddrMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			AddrMode::DataReg(reg) => write!(f, "D{reg}"),
			AddrMode::AddressReg(reg) => write!(f, "A{reg}"),
			AddrMode::Address(reg) => write!(f, "(A{reg})"),
			AddrMode::AddressWithPostinc(reg) => write!(f, "(A{reg})+"),
			AddrMode::AddressWithPredec(reg) => write!(f, "-(A{reg})"),
			AddrMode::AddressWithDisp(reg) => write!(f, "(d16,A{reg})"),
			AddrMode::AddressWithIndex(reg) => write!(f, "(d8,A{reg},Xn)"),
			AddrMode::PCWithDisp => write!(f, "(d16,PC)"),
			AddrMode::PCWithIndex => write!(f, "(d8,PC,Xn"),
			AddrMode::AbsoluteShort => write!(f, "(xxx).W"),
			AddrMode::AbsoluteLong => write!(f, "(xxx).L"),
			AddrMode::Immediate => write!(f, "#imm"),
		}
	}
}

#[derive(Clone, Copy)]
pub enum Opcode {
	OrIToCcr,
	OrIToSr,
	OrI { size: Size, addr_mode: AddrMode},
	AndIToCcr,
	AndIToSr,
	AndI { size: Size, addr_mode: AddrMode},
	SubI { size: Size, addr_mode: AddrMode},
	AddI { size: Size, addr_mode: AddrMode},
	EorIToCcr,
	EorIToSr,
	EorI { size: Size, addr_mode: AddrMode},
	CmpI { size: Size, addr_mode: AddrMode},
	Btst { addr_mode: AddrMode },
	Bchg { addr_mode: AddrMode },
	Bclr { addr_mode: AddrMode },
	Bset { addr_mode: AddrMode },
	BtstFromD { source: DReg, addr_mode: AddrMode },
	BchgFromD { source: DReg, addr_mode: AddrMode },
	BclrFromD { source: DReg, addr_mode: AddrMode },
	BsetFromD { source: DReg, addr_mode: AddrMode },
	MoveP { source: DReg, dir: MoveDirection, size: Size, dest: AReg },
	MoveA { size: Size, dest: AReg, source: AddrMode},
	Move { size: Size, dest: AddrMode, source: AddrMode },
	MoveFromSr { addr_mode: AddrMode },
	MoveToCcr { addr_mode: AddrMode },
	MoveToSr { addr_mode: AddrMode },
	NegX { size: Size, addr_mode: AddrMode},
	Clr { size: Size, addr_mode: AddrMode},
	Neg { size: Size, addr_mode: AddrMode},
	Not { size: Size, addr_mode: AddrMode},
	Ext { size: Size, dest: DReg },
	Nbcd { addr_mode: AddrMode },
	Swap { dest: DReg },
	Pea { addr_mode: AddrMode },
	Illegal,
	Tas { addr_mode: AddrMode },
	Tst { size: Size, addr_mode: AddrMode },
	Trap { vector: Vector },
	Link { frame_pointer: AReg },
	Unlnk { frame_pointer: AReg },
	MoveUsp { dir: MoveDirection, a: AReg },
	Reset,
	Nop,
	Stop,
	Rte,
	Rts,
	TrapV,
	Rtr,
	Jsr { addr_mode: AddrMode },
	Jmp { addr_mode: AddrMode },
	MoveM { dir: MoveDirection, size: Size, addr_mode: AddrMode },
	Lea { dest: AReg, addr_mode: AddrMode },
	Chk { source: DReg, addr_mode: AddrMode },
	AddQ { data: u8, size: Size, addr_mode: AddrMode },
	SubQ { data: u8, size: Size, addr_mode: AddrMode },
	Scc { cond: Condition, addr_mode: AddrMode },
	DBcc { cond: Condition, loop_down: DReg },
	Bsr { disp: i32 },
	Bcc { cond: Condition, disp: i32 },
	MoveQ { dest: DReg, data: u8 },
	DivU { dest: DReg, source: AddrMode },
	DivS { dest: DReg, source: AddrMode },
	Sbcd { dest: AddrMode, source: AddrMode },
	Or { reg: DReg, dir: BinOpDirection, size: Size, addr_mode: AddrMode },
	Sub { reg: DReg, dir: BinOpDirection, size: Size, addr_mode: AddrMode },
	SubX { dest: AddrMode, size: Size, source: AddrMode },
	SubA { dest: AReg, size: Size, source: AddrMode },
	Eor { dest: DReg, size: Size, source: AddrMode },
	CmpM { dest: AReg, size: Size, source: AReg },
	Cmp { dest: DReg, size: Size, source: AddrMode },
	CmpA { dest: AReg, size: Size, source: AddrMode },
	MulU { dest: DReg, source: AddrMode },
	MulS { dest: DReg, source: AddrMode },
	Abcd { dest: AddrMode, source: AddrMode },
	Exg { first: Register, second: Register },
	And { reg: DReg, dir: BinOpDirection, size: Size, addr_mode: AddrMode },
	Add { reg: DReg, dir: BinOpDirection, size: Size, addr_mode: AddrMode },
	AddX { dest: AddrMode, size: Size, source: AddrMode },
	AddA { dest: AReg, size: Size, source: AddrMode },
	Asd { dir: RotateDirection, addr_mode: AddrMode },
	Lsd { dir: RotateDirection, addr_mode: AddrMode },
	RoXd { dir: RotateDirection, addr_mode: AddrMode },
	Rod { dir: RotateDirection, addr_mode: AddrMode },
	AsdToD { rot: u8, dir: RotateDirection, size: Size, mode: RotateMode, reg: DReg },
	LsdToD { rot: u8, dir: RotateDirection, size: Size, mode: RotateMode, reg: DReg },
	RoXdToD { rot: u8, dir: RotateDirection, size: Size, mode: RotateMode, reg: DReg },
	RodToD { rot: u8, dir: RotateDirection, size: Size, mode: RotateMode, reg: DReg },
}

impl fmt::Display for Opcode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Opcode::OrIToCcr { .. } => write!(f, "ORI to CCR"),
			Opcode::OrIToSr { .. } => write!(f, "ORI to SR"),
			Opcode::OrI { .. } => write!(f, "ORI"),
			Opcode::AndIToCcr { .. } => write!(f, "ANDI to CCR"),
			Opcode::AndIToSr { .. } => write!(f, "ANDI to SR"),
			Opcode::AndI { .. } => write!(f, "ANDI"),
			Opcode::SubI { .. } => write!(f, "SUBI"),
			Opcode::AddI { .. } => write!(f, "ADDI"),
			Opcode::EorIToCcr { .. } => write!(f, "EORI to CCR"),
			Opcode::EorIToSr { .. } => write!(f, "EORI to SR"),
			Opcode::EorI { .. } => write!(f, "EORI"),
			Opcode::CmpI { .. } => write!(f, "CMPI"),
			Opcode::Btst { .. } => write!(f, "BTST"),
			Opcode::Bchg { .. } => write!(f, "BCHG"),
			Opcode::Bclr { .. } => write!(f, "BCLR"),
			Opcode::Bset { .. } => write!(f, "BSET"),
			Opcode::BtstFromD { .. } => write!(f, "BTST from D"),
			Opcode::BchgFromD { .. } => write!(f, "BCHG from D"),
			Opcode::BclrFromD { .. } => write!(f, "BCLR from D"),
			Opcode::BsetFromD { .. } => write!(f, "BSET from D"),
			Opcode::MoveP { .. } => write!(f, "MOVEP"),
			Opcode::MoveA { .. } => write!(f, "MOVEA"),
			Opcode::Move { .. } => write!(f, "MOVE"),
			Opcode::MoveFromSr { .. } => write!(f, "MOVE from SR"),
			Opcode::MoveToCcr { .. } => write!(f, "MOVE to CCR"),
			Opcode::MoveToSr { .. } => write!(f, "MOVE to SR"),
			Opcode::NegX { .. } => write!(f, "NEGX"),
			Opcode::Clr { .. } => write!(f, "CLR"),
			Opcode::Neg { .. } => write!(f, "NEG"),
			Opcode::Not { .. } => write!(f, "NOT"),
			Opcode::Ext { .. } => write!(f, "EXT"),
			Opcode::Nbcd { .. } => write!(f, "NBCD"),
			Opcode::Swap { .. } => write!(f, "SWAP"),
			Opcode::Pea { .. } => write!(f, "PEA"),
			Opcode::Illegal { .. } => write!(f, "ILLEGAL"),
			Opcode::Tas { .. } => write!(f, "TAS"),
			Opcode::Tst { .. } => write!(f, "TST"),
			Opcode::Trap { .. } => write!(f, "TRAP"),
			Opcode::Link { .. } => write!(f, "LINK"),
			Opcode::Unlnk { .. } => write!(f, "UNLNK"),
			Opcode::MoveUsp { .. } => write!(f, "MOVE USP"),
			Opcode::Reset { .. } => write!(f, "RESET"),
			Opcode::Nop { .. } => write!(f, "NOP"),
			Opcode::Stop { .. } => write!(f, "STOP"),
			Opcode::Rte { .. } => write!(f, "RTE"),
			Opcode::Rts { .. } => write!(f, "RTS"),
			Opcode::TrapV { .. } => write!(f, "TRAPV"),
			Opcode::Rtr { .. } => write!(f, "RTR"),
			Opcode::Jsr { .. } => write!(f, "JSR"),
			Opcode::Jmp { .. } => write!(f, "JMP"),
			Opcode::MoveM { .. } => write!(f, "MOVEM"),
			Opcode::Lea { .. } => write!(f, "LEA"),
			Opcode::Chk { .. } => write!(f, "CHK"),
			Opcode::AddQ { .. } => write!(f, "ADDQ"),
			Opcode::SubQ { .. } => write!(f, "SUBQ"),
			Opcode::Scc { .. } => write!(f, "Scc"),
			Opcode::DBcc { .. } => write!(f, "DBcc"),
			Opcode::Bsr { .. } => write!(f, "BSR"),
			Opcode::Bcc { .. } => write!(f, "Bcc"),
			Opcode::MoveQ { .. } => write!(f, "MOVEQ"),
			Opcode::DivU { .. } => write!(f, "DIVU"),
			Opcode::DivS { .. } => write!(f, "DIVS"),
			Opcode::Sbcd { .. } => write!(f, "SBCD"),
			Opcode::Or { .. } => write!(f, "OR"),
			Opcode::Sub { .. } => write!(f, "SUB"),
			Opcode::SubX { .. } => write!(f, "SUBX"),
			Opcode::SubA { .. } => write!(f, "SUBA"),
			Opcode::Eor { .. } => write!(f, "EOR"),
			Opcode::CmpM { .. } => write!(f, "CMPM"),
			Opcode::Cmp { .. } => write!(f, "CMP"),
			Opcode::CmpA { .. } => write!(f, "CMPA"),
			Opcode::MulU { .. } => write!(f, "MULU"),
			Opcode::MulS { .. } => write!(f, "MULS"),
			Opcode::Abcd { .. } => write!(f, "ABCD"),
			Opcode::Exg { .. } => write!(f, "EXG"),
			Opcode::And { .. } => write!(f, "AND"),
			Opcode::Add { .. } => write!(f, "ADD"),
			Opcode::AddX { .. } => write!(f, "ADDX"),
			Opcode::AddA { .. } => write!(f, "ADDA"),
			Opcode::Asd { .. } => write!(f, "ASd"),
			Opcode::Lsd { .. } => write!(f, "LSd"),
			Opcode::RoXd { .. } => write!(f, "ROXd"),
			Opcode::Rod { .. } => write!(f, "ROd"),
			Opcode::AsdToD { .. } => write!(f, "ASd to D"),
			Opcode::LsdToD { .. } => write!(f, "LSd to D"),
			Opcode::RoXdToD { .. } => write!(f, "ROXd to D"),
			Opcode::RodToD { .. } => write!(f, "ROd to D"),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Flags {
	pub x: bool,
	pub n: bool,
	pub z: bool,
	pub v: bool,
	pub c: bool,
}

pub fn calc_opcode_cycles(opcode: Opcode, branch_taken: Option<bool>, counter_expired: Option<bool>, special_count: Option<u64>, rot: Option<u64>) -> u64 {
	match opcode {
		Opcode::OrI { size, addr_mode }
			| Opcode::SubI { size, addr_mode }
			| Opcode::AddI { size, addr_mode }
			| Opcode::EorI { size, addr_mode } => {
			match addr_mode {
				AddrMode::DataReg(_) => {
					match size {
						Size::Long => 16,
						_ => 8,
					}
				},
				_ => {
					match size {
						Size::Long => 20,
						_ => 12,
					}
				}
			}
		},
		Opcode::AndI { size, addr_mode } => {
			match addr_mode {
				AddrMode::DataReg(_) => {
					match size {
						Size::Long => 14,
						_ => 8,
					}
				},
				_ => {
					match size {
						Size::Long => 20,
						_ => 12
					}
				}
			}
		},
		Opcode::CmpI { size, addr_mode } => {
			match addr_mode {
				AddrMode::DataReg(_) => {
					match size {
						Size::Long => 14,
						_ => 8,
					}
				},
				_ => {
					match size {
						Size::Long => 12,
						_ => 8,
					}
				}
			}
		},
		Opcode::Btst { addr_mode } => {
			match addr_mode {
				AddrMode::DataReg(_) => 10,
				_ => 8,
			}
		},
		Opcode::Bchg { .. } => 12,
		Opcode::Bclr { addr_mode } => {
			match addr_mode {
				AddrMode::DataReg(_) => 14,
				_ => 12,
			}
		},
		Opcode::Bset { .. } => 12,
		Opcode::BtstFromD { addr_mode, .. } => {
			match addr_mode {
				AddrMode::DataReg(_) => 6,
				_ => 4,
			}
		},
		Opcode::BchgFromD { .. } => 8,
		Opcode::BclrFromD { addr_mode, .. } => {
			match addr_mode {
				AddrMode::DataReg(_) => 10,
				_ => 8,
			}
		},
		Opcode::BsetFromD { .. } => 8,
		Opcode::MoveA { size, source, .. } => {
			match size {
				Size::Long => {
					match source {
						AddrMode::DataReg(_) | AddrMode::AddressReg(_) => 4,
						AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::Immediate => 12,
						AddrMode::AddressWithPredec(_) => 14,
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 16,
						AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => 18,
						AddrMode::AbsoluteLong => 20,
					}
				},
				_ => {
					match source {
						AddrMode::DataReg(_) | AddrMode::AddressReg(_) => 4,
						AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::Immediate => 8,
						AddrMode::AddressWithPredec(_) => 10,
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 12,
						AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => 14,
						AddrMode::AbsoluteLong => 16,
					}
				}
			}
		}
		Opcode::Move { size, dest, source } => {
			match size {
				Size::Long => {
					match source {
						AddrMode::DataReg(_) | AddrMode::AddressReg(_) => {
							match dest {
								AddrMode::DataReg(_) => 4,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 12,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 16,
								AddrMode::AddressWithIndex(_) => 18,
								AddrMode::AbsoluteLong => 20,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::Immediate => {
							match dest {
								AddrMode::DataReg(_) => 12,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 20,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 24,
								AddrMode::AddressWithIndex(_) => 26,
								AddrMode::AbsoluteLong => 28,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithPredec(_) => {
							match dest {
								AddrMode::DataReg(_) => 14,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 22,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 26,
								AddrMode::AddressWithIndex(_) => 28,
								AddrMode::AbsoluteLong => 30,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => {
							match dest {
								AddrMode::DataReg(_) => 16,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 24,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 28,
								AddrMode::AddressWithIndex(_) => 30,
								AddrMode::AbsoluteLong => 32,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => {
							match dest {
								AddrMode::DataReg(_) => 18,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 26,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 30,
								AddrMode::AddressWithIndex(_) => 32,
								AddrMode::AbsoluteLong => 34,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AbsoluteLong => {
							match dest {
								AddrMode::DataReg(_) => 20,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 28,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 32,
								AddrMode::AddressWithIndex(_) => 34,
								AddrMode::AbsoluteLong => 36,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
					}
				},
				_ => {
					match source {
						AddrMode::DataReg(_) | AddrMode::AddressReg(_) => {
							match dest {
								AddrMode::DataReg(_) => 4,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 8,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 12,
								AddrMode::AddressWithIndex(_) => 14,
								AddrMode::AbsoluteLong => 16,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::Immediate => {
							match dest {
								AddrMode::DataReg(_) => 8,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 12,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 16,
								AddrMode::AddressWithIndex(_) => 18,
								AddrMode::AbsoluteLong => 20,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithPredec(_) => {
							match dest {
								AddrMode::DataReg(_) => 10,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 14,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 18,
								AddrMode::AddressWithIndex(_) => 20,
								AddrMode::AbsoluteLong => 22,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => {
							match dest {
								AddrMode::DataReg(_) => 12,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 16,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 20,
								AddrMode::AddressWithIndex(_) => 22,
								AddrMode::AbsoluteLong => 24,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => {
							match dest {
								AddrMode::DataReg(_) => 14,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 18,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 22,
								AddrMode::AddressWithIndex(_) => 24,
								AddrMode::AbsoluteLong => 26,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
						AddrMode::AbsoluteLong => {
							match dest {
								AddrMode::DataReg(_) => 16,
								AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) | AddrMode::AddressWithPredec(_) => 20,
								AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 24,
								AddrMode::AddressWithIndex(_) => 26,
								AddrMode::AbsoluteLong => 28,
								_ => panic!("Invalid addressing mode {dest} in opcode {opcode}"),
							}
						},
					}
				}
			}
		},
		Opcode::MoveToSr { .. } => 12,
		Opcode::Clr { size, addr_mode }
			| Opcode::Neg { size, addr_mode }
			| Opcode::NegX { size, addr_mode }
			| Opcode::Not { size, addr_mode } => {
			match size {
				Size::Long => {
					match addr_mode {
						AddrMode::DataReg(_) => 6,
						_ => 12,
					}
				},
				_ => {
					match addr_mode {
						AddrMode::DataReg(_) => 4,
						_ => 6,
					}
				}
			}
		},
		Opcode::Swap { .. } => 4,
		Opcode::Pea { addr_mode } => {
			match addr_mode {
				AddrMode::Address(_) => 12,
				AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 16,
				AddrMode::AddressWithIndex(_) | AddrMode::AbsoluteLong | AddrMode::PCWithIndex => 20,
				_ => panic!("Invalid addressing mode {addr_mode} in opcode {opcode}"),
			}
		},
		Opcode::Tst { .. } => 4,
		Opcode::MoveUsp { .. } => 4,
		Opcode::Rts => 16,
		Opcode::Jsr { addr_mode } => {
			match addr_mode {
				AddrMode::Address(_) => 16,
				AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 18,
				AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => 22,
				AddrMode::AbsoluteLong => 20,
				_ => panic!("Invalid addressing mode {addr_mode} in opcode {opcode}"),
			}
		},
		Opcode::Jmp { addr_mode } => {
			match addr_mode {
				AddrMode::Address(_) => 8,
				AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 10,
				AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => 14,
				AddrMode::AbsoluteLong => 12,
				_ => panic!("Invalid addressing mode {addr_mode} in opcode {opcode}"),
			}
		},
		Opcode::MoveM { dir, addr_mode, size } => {
			let reg_cycles = match size {
				Size::Long => 8 * special_count.unwrap(),
				_ => 4 * special_count.unwrap(),
			};
			match dir {
				MoveDirection::MemToReg => {
					match addr_mode {
						AddrMode::Address(_) | AddrMode::AddressWithPostinc(_) => 12 + reg_cycles,
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 16 + reg_cycles,
						AddrMode::AddressWithIndex(_) | AddrMode::PCWithIndex => 18 + reg_cycles,
						AddrMode::AbsoluteLong => 20 + reg_cycles,
						_ => panic!("Invalid address mode {addr_mode} on {opcode}.")
					}
				},
				MoveDirection::RegToMem => {
					match addr_mode {
						AddrMode::Address(_) | AddrMode::AddressWithPredec(_) => 8 + reg_cycles,
						AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort => 12 + reg_cycles,
						AddrMode::AddressWithIndex(_) => 14 + reg_cycles,
						AddrMode::AbsoluteLong => 16 + reg_cycles,
						_ => panic!("Invalid address mode {addr_mode} on {opcode}.")
					}
				}
			}
		},
		Opcode::Lea { addr_mode, .. } => {
			match addr_mode {
				AddrMode::Address(_) => 4,
				AddrMode::AddressWithDisp(_) | AddrMode::AbsoluteShort | AddrMode::PCWithDisp => 8,
				AddrMode::AddressWithIndex(_) | AddrMode::AbsoluteLong | AddrMode::PCWithIndex => 12,
				_ => panic!("Invalid address mode {addr_mode} on {opcode}.")
			}
		},
		Opcode::AddQ { size, addr_mode, .. } => {
			match size {
				Size::Long => {
					match addr_mode {
						AddrMode::DataReg(_) => 8,
						AddrMode::AddressReg(_) => 8,
						_ => 12,
					}
				},
				_ => {
					match addr_mode {
						AddrMode::DataReg(_) => 4,
						AddrMode::AddressReg(_) => 4,
						_ => 8,
					}
				}
			}
		},
		Opcode::SubQ { size, addr_mode, .. } => {
			match size {
				Size::Long => {
					match addr_mode {
						AddrMode::DataReg(_) => 8,
						AddrMode::AddressReg(_) => 8,
						_ => 12,
					}
				},
				_ => {
					match addr_mode {
						AddrMode::DataReg(_) => 4,
						AddrMode::AddressReg(_) => 8,
						_ => 8,
					}
				}
			}
		}
		Opcode::DBcc { .. } => {
			let branch_taken = branch_taken.unwrap();
			let counter_expired = counter_expired.unwrap();
			if branch_taken { 10 }
			else {
				if counter_expired { 14 }
				else { 12 }
			}
		},
		Opcode::Bsr { .. } => 18,
		Opcode::Bcc { disp, .. } => {
			let branch_taken = branch_taken.unwrap();
			if branch_taken { 10 }
			else {
				if disp == 0 { 12 }
				else { 8 }
			}
		},
		Opcode::MoveQ { .. } => 4,
		Opcode::Or { dir, size, addr_mode, .. }
			| Opcode::Sub { dir, size, addr_mode, .. }
			| Opcode::And { dir, size, addr_mode, .. }
			| Opcode::Add { dir, size, addr_mode, .. } => {
			match size {
				Size::Long => {
					match dir {
						BinOpDirection::ToReg => {
							match addr_mode {
								AddrMode::AddressReg(_) | AddrMode::DataReg(_) | AddrMode::Immediate => 8,
								_ => 6,
							}
						},
						BinOpDirection::ToEA => 12,
					}
				},
				_ => {
					match dir {
						BinOpDirection::ToReg => 4,
						BinOpDirection::ToEA => 8,
					}
				},
			}
		},
		Opcode::SubA { size, source, .. }
			| Opcode::AddA { size, source, .. } => {
			match size {
				Size::Long => {
					match source {
						AddrMode::AddressReg(_) | AddrMode::DataReg(_) | AddrMode::Immediate => 8,
						_ => 6,
					}
				},
				_ => 8,
			}
		},
		Opcode::MulU { .. } => 38 + (2 * special_count.unwrap()),
		Opcode::And { dir, size, addr_mode, .. } => {
			match size {
				Size::Long => {
					match dir {
						BinOpDirection::ToReg => {
							match addr_mode {
								AddrMode::DataReg(_) | AddrMode::Immediate => 8,
								_ => 4,
							}
						},
						BinOpDirection::ToEA => 12,
					}
				},
				_ => {
					match dir {
						BinOpDirection::ToReg => 4,
						BinOpDirection::ToEA => 8,
					}
				}
			}
		},
		Opcode::LsdToD { size, .. } => {
			let rotation = rot.unwrap();
			match size {
				Size::Long => 8 + (rotation * 2),
				_ => 6 + (rotation * 2),
			}
		}
		_ => panic!("Cycle calculation for {opcode} unimplemented."),
	}
}