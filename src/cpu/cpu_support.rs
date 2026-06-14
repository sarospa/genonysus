use std::fmt;

pub const CART_SIZE: usize = 0x400000;
pub const RAM_SIZE: usize = 0x10000;
pub const RAM_START: usize = 0xFF0000;
pub const RAM_END: usize = RAM_START + RAM_SIZE;
pub const ADDRESS_SPACE: usize = 0xFFFFFF;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub struct Vector { i: u16 }
impl Vector {
	pub fn new(vector: u16) -> Vector {
		if vector >= 16 {
			panic!("{} is not a valid vector.", vector);
		}
		Vector { i: vector }
	}
}

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
	BSR { disp: i32 },
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
			Opcode::BSR { .. } => write!(f, "BSR"),
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
