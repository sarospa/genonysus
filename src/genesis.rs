use std::collections::VecDeque;
use bitmatch::bitmatch;

mod cpu;
mod vdp;
mod external;
use external::RealExternal;
use crate::genesis::external::External;

pub const CART_SIZE: usize = 0x400000;
pub const CPU_RAM_SIZE: usize = 0x10000;
pub const CPU_RAM_START: usize = 0xFF0000;
pub const CPU_RAM_END: usize = CPU_RAM_START + CPU_RAM_SIZE;
pub const CPU_ADDRESS_SPACE: usize = 0xFFFFFF;

#[derive(PartialEq, Eq)]
struct QueuedWrite {
	address: u32,
	data: u8,
}

#[derive(PartialEq, Eq)]
enum BusLock {
	CPU,
	VDP,
}

trait Motorola68KBus {
	fn read_u8(&mut self, address: u32) -> u8;
	fn write_u8(&mut self, address: u32, data: u8);

	fn assert_interrupt(&mut self, level: u16);

	fn acknowledge_interrupt(&mut self) -> Option<u16>;

	fn read_u16(&mut self, address: u32) -> u16 {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		((self.read_u8(address) as u16) << 8)
			+ (self.read_u8(address + 1) as u16)
	}
	
	fn read_u32(&mut self, address: u32) -> u32 {
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
		self.write_u8(address + 1, (data & 0x00FF) as u8);
	}
	
	fn write_u32(&mut self, address:u32, data: u32) {
		if address % 2 == 1 {
			panic!("Illegal access across word boundary at {:#08X}.", address);
		}
		self.write_u8(address, ((data & 0xFF000000) >> 24) as u8);
		self.write_u8(address + 1, ((data & 0x00FF0000) >> 16) as u8);
		self.write_u8(address + 2, ((data & 0x0000FF00) >> 8) as u8);
		self.write_u8(address + 3, (data & 0x000000FF) as u8);
	}

	fn expose_vdp_state(&mut self) -> &mut VDPState;

	fn expose_io(&mut self) -> &mut dyn External;

	fn ref_io(&self) -> &dyn External;
}

pub struct Genesis {
	cpu: cpu::CPU,
	vdp: vdp::VDP,
	cycles: u64,
	bus: GenesisBus,
}
impl Genesis {
	pub fn new(rom: &Vec<u8>) -> Genesis {
		let mut bus = GenesisBus::new(rom);
		Genesis {
			cpu: cpu::CPU::new(&mut bus),
			vdp: vdp::VDP::new(),
			cycles: 0,
			bus: bus,
		}
	}
	
	pub fn advance_cycle(&mut self) {
		self.cycles += 1;
		self.bus.cycles = self.cycles;
		if self.cycles % 7 == 0 && self.bus.bus_lock == BusLock::CPU {
			if self.bus.cpu_write_queue.len() == 0 {
				self.cpu.advance_cycle(&mut self.bus);
			}
			else {
				// Test front of queue, and if it fails, take it out and put it back in front
				let queued_write = self.bus.cpu_write_queue.pop_front().unwrap();
				self.bus.write_u8(queued_write.address, queued_write.data);
				if self.bus.write_failed {
					let failed_write = self.bus.cpu_write_queue.pop_back().unwrap();
					self.bus.cpu_write_queue.push_front(failed_write);
				}
			}
		}
		if self.bus.vdp_state.countdown == 0 {
			self.vdp.advance_cycle(&mut self.bus);
		}
		self.bus.vdp_state.countdown -= 1;
	}

	pub fn open(&self) -> bool {
		self.bus.io.open()
	}
}

struct GenesisBus {
	cpu_ram: Vec<u8>,
	cart_memory: Vec<u8>,
	interrupt_level: u16,
	interrupt_set: bool,
	controller1: Controller,
	controller1_control: u8,
	controller2: Controller,
	controller2_control: u8,
	vdp_control_select: Option<u8>,
	vdp_data_write_upper: Option<u8>,
	vdp_state: VDPState,
	bus_lock: BusLock,
	cpu_write_queue: VecDeque<QueuedWrite>,
	write_failed: bool,
	cycles: u64,
	io: RealExternal,
}
impl GenesisBus {
	fn new(rom: &Vec<u8>) -> GenesisBus {
		let mut cart_memory = vec![0; CART_SIZE];
		for i in 0..Ord::min(CART_SIZE, rom.len()) {
			cart_memory[i] = rom[i];
		};

		GenesisBus {
			cpu_ram: vec![0; CPU_RAM_SIZE],
			cart_memory: cart_memory,
			interrupt_level: 0,
			interrupt_set: false,
			controller1: Controller::ThreeButton { control: 0 },
			controller1_control: 0,
			controller2: Controller::Unplugged,
			controller2_control: 0,
			vdp_control_select: None,
			vdp_data_write_upper: None,
			vdp_state: VDPState::new(),
			bus_lock: BusLock::CPU,
			cpu_write_queue: VecDeque::new(),
			write_failed: false,
			cycles: 0,
			io: RealExternal::new(),
		}
	}
}
impl Motorola68KBus for GenesisBus {
	fn read_u8(&mut self, address: u32) -> u8 {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		match address_index {
			0..CART_SIZE => self.cart_memory[address_index],
			CPU_RAM_START..CPU_RAM_END => self.cpu_ram[address_index - CPU_RAM_START],
			0xA00000..=0xA0FFFF => 0, // Z80 address space (stub)
			0xA10000..=0xA10001 => 0, // Version register
			0xA10002..=0xA10003 => self.controller1.read(self.ref_io()),
			0xA10004..=0xA10005 => self.controller2.read(self.ref_io()),
			0xA10006..=0xA10007 => 0, // Expansion port data
			0xA10008..=0xA10009 => self.controller1_control,
			0xA1000A..=0xA1000B => self.controller2_control,
			0xA1000C..=0xA1000D => 0, // Expansion port control
			0xA11100..=0xA11101 => 0, // Z80 Bus Request Register (stub)
			0xA11200..=0xA11201 => 0, // Z80 Reset Register (stub)
			0xA14000..=0xA14003 => 0, // TMSS Register
			0xC00000 | 0xC00002 => self.vdp_state.external_read_upper(),
			0xC00001 | 0xC00003 => self.vdp_state.external_read_lower(),
			0xC00004 | 0xC00006 => self.vdp_state.read_register_upper(),
			0xC00005 | 0xC00007 => self.vdp_state.read_register_lower(),
			0xC00008 => self.vdp_state.get_v_counter(),
			0xC00009 => self.vdp_state.get_h_counter(),
			_ => {
				panic!("Attempt to read from address {:#08X} located in unimplemented memory region.", address);
			}
		}
	}

	fn write_u8(&mut self, address: u32, data: u8) {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		self.write_failed = false;
		if address_index != 0xC00005 && address_index != 0xC00007 {
			self.vdp_control_select = None;
		}
		if address_index != 0xC00001 && address_index != 0xC00003 {
			self.vdp_data_write_upper = None;
		}
		match address_index {
			0..CART_SIZE => (), // Don't overwrite the cart ROM!
			CPU_RAM_START..CPU_RAM_END => self.cpu_ram[address_index - CPU_RAM_START] = data,
			0xA00000..=0xA0FFFF => (), // Z80 address space (stub)
			0xA10000..=0xA10001 => (), // Version register
			0xA10002..=0xA10003 => self.controller1.write(data, self.cycles),
			0xA10004..=0xA10005 => self.controller2.write(data, self.cycles),
			0xA10006..=0xA10007 => (), // Expansion port data
			0xA10008..=0xA10009 => self.controller1_control = data,
			0xA1000A..=0xA1000B => self.controller2_control = data,
			0xA1000C..=0xA1000D => (), // Expansion port control
			0xA11100..=0xA11101 => (), // Z80 Bus Request Register (stub)
			0xA11200..=0xA11201 => (), // Z80 Reset Register (stub)
			0xA14000..=0xA14003 => (), // TMSS Register
			0xC00000 | 0xC00002 => self.vdp_data_write_upper = Some(data),
			0xC00001 | 0xC00003 => {
				if let Some(upper) = self.vdp_data_write_upper {
					let result = self.vdp_state.external_write(((upper as u16) << 8) + (data as u16));
					if !result {
						self.cpu_write_queue.push_back(QueuedWrite {
							address: address - 1,
							data: upper,
						});
						self.write_failed = true;
						self.cpu_write_queue.push_back(QueuedWrite {
							address: address,
							data: data,
						});
						self.write_failed = true;
					}
				}
			}
			0xC00004 | 0xC00006 => self.vdp_control_select = Some(data),
			0xC00005 | 0xC00007 => {
				if let Some(select) = self.vdp_control_select {
					self.vdp_state.write_register(((select as u16) << 8) + (data as u16));
				}
			},
			0xC00011 => (), // PSG Control Register (stub)
			_ => {
				panic!("Attempt to write to address {:#08X} located in unimplemented memory region.", address);
			}
		};
	}

	fn assert_interrupt(&mut self, level: u16) {
		self.interrupt_level = level;
		self.interrupt_set = true;
	}

	fn acknowledge_interrupt(&mut self) -> Option<u16> {
		if self.interrupt_set {
			self.interrupt_set = false;
			Some(self.interrupt_level)
		}
		else {
			None
		}
	}

	fn expose_vdp_state(&mut self) -> &mut VDPState {
		&mut self.vdp_state
	}

	fn expose_io(&mut self) -> &mut dyn External {
		&mut self.io
	}

	fn ref_io(&self) -> &dyn External {
		&self.io
	}
}

#[derive(Debug)]
pub enum Controller {
	Unplugged,
	ThreeButton { control: u8 },
	SixButton { control: u8, state: u8, cycles: u64},
}

impl Controller {
	pub fn read(&self, io: &dyn External) -> u8 {
		match self {
			Controller::Unplugged => 0,
			Controller::ThreeButton { control } => {
				let buttons = io.button_array(); // A B C X Y Z Up Down Left Right Start Mode
				let control_bit = *control & 0x40;
				if control_bit == 0x40 {
					let c_bit = if buttons[2] { 0 } else { 0x20 };
					let b_bit = if buttons[1] { 0 } else { 0x10 };
					let right_bit = if buttons[9] { 0 } else { 0x08 };
					let left_bit = if buttons[8] { 0 } else { 0x04 };
					let down_bit = if buttons[7] { 0 } else { 0x02 };
					let up_bit = if buttons[6] { 0 } else { 0x01 };
					control_bit | c_bit | b_bit | right_bit | left_bit | down_bit | up_bit
				}
				else {
					let start_bit = if buttons[10] { 0 } else { 0x20 };
					let a_bit = if buttons[0] { 0 } else { 0x10 };
					let down_bit = if buttons[7] { 0 } else { 0x02 };
					let up_bit = if buttons[6] { 0 } else { 0x01 };
					control_bit | start_bit | a_bit | down_bit | up_bit
				}
			},
			Controller::SixButton { .. } => panic!("Six button controller not implemented."),
		}
	}
	
	pub fn write(&mut self, data: u8, cur_cycles: u64) -> () {
		match self {
			Controller::Unplugged => (),
			Controller::ThreeButton { control } => *control = data,
			Controller::SixButton { control, state, cycles } => {
				// Reset to three button behavior after ~1.5ms
				if cur_cycles - *cycles > 11505 {
					*state = (*control & 0x40) >> 6;
				}
				*cycles = cur_cycles;
				if (*control & 0x40) != (data & 0x40) {
					*state = (*state + 1) % 8;
				}
				*control = data;
			},
		}
	}
}

pub const VRAM_SIZE: usize = 0x10000;
pub const CRAM_SIZE: usize = 128;
pub const VSRAM_SIZE: usize = 80;

struct VDPState {
	h_counter: u16,
	h_phase: u8,
	h_blank: bool,
	access_slot: u16,
	slot_ready: bool,
	v_counter: u16,
	v_phase: u8,
	v_blank: bool,
	dot_counter: u16,
	vram: Vec<u8>,
	color_ram: Vec<u8>,
	vscroll_ram: Vec<u8>,
	dma_active: bool,
	fill_active: bool,
	queue: [FifoSlot; 4],
	queue_end: usize,
	queue_start: usize,
	queue_size: u8,
	data_command: bool,
	pub clock_speed: u8,
	pub countdown: u8,
	left_blank: bool,
	palette_select: bool,
	h_interrupt_enable: bool,
	hv_counter_latch: bool,
	display_disable: bool,
	vram_128k: bool,
	display_enable: bool,
	v_interrupt_enable: bool,
	dma_enable: bool,
	v_resolution: bool,
	video_mode: bool,
	plane_a_address: u32,
	window_address: u32,
	plane_b_address: u32,
	sprite_table_address: u32,
	background_palette: u16,
	background_color: u16,
	sms_h_scroll: u16,
	sms_v_scroll: u16,
	h_interrupt_counter: u16,
	h_interrupt_countdown: u16,
	ext_interrupt_enable: bool,
	column_scroll: bool,
	h_scroll: HScroll,
	h32_mode: bool,
	shadow_highlight_mode: bool,
	interlace_mode: InterlaceMode,
	h_scroll_address: u32,
	auto_increment: u16,
	plane_height: u16,
	plane_width: u16,
	window_right: bool,
	window_h: u16,
	window_down: bool,
	window_v: u16,
	dma_length: u16,
	dma_source: u32,
	dma_type: DmaType,
	data_address: u16,
	data_code: DataCode,
	data_type_bits: u16,
	v_interrupt_triggered: bool,
	scanline_sprite_overflow: bool,
	sprite_overlap: bool,
	odd_frame: bool,
}
impl VDPState {
	fn new() -> VDPState {
		let empty_fifo = FifoSlot {
			code: DataCode::VramRead,
			address: 0,
			data: 0,
			half_complete: false,
		};
		VDPState {
			h_counter: 0,
			h_phase: 1,
			h_blank: false,
			access_slot: 0,
			slot_ready: true,
			v_counter: 0,
			v_phase: 1,
			v_blank: false,
			dot_counter: 0,
			vram: vec!(0u8; VRAM_SIZE),
			color_ram: vec!(0u8; CRAM_SIZE),
			vscroll_ram: vec!(0u8; VSRAM_SIZE),
			dma_active: false,
			fill_active: false,
			queue: [empty_fifo, empty_fifo, empty_fifo, empty_fifo],
			queue_end: 0,
			queue_start: 0,
			queue_size: 0,
			data_command: false,
			clock_speed: 10,
			countdown: 0,
			left_blank: false,
			palette_select: false,
			h_interrupt_enable: false,
			hv_counter_latch: false,
			display_disable: false,
			vram_128k: false,
			display_enable: false,
			v_interrupt_enable: false,
			dma_enable: false,
			v_resolution: false,
			video_mode: false,
			plane_a_address: 0,
			window_address: 0,
			plane_b_address: 0,
			sprite_table_address: 0,
			background_palette: 0,
			background_color: 0,
			sms_h_scroll: 0,
			sms_v_scroll: 0,
			h_interrupt_counter: 0,
			h_interrupt_countdown: 0,
			ext_interrupt_enable: false,
			column_scroll: false,
			h_scroll: HScroll::Fullscreen,
			h32_mode: false,
			shadow_highlight_mode: false,
			interlace_mode: InterlaceMode::NoInterlace,
			h_scroll_address: 0,
			auto_increment: 0,
			plane_height: 256,
			plane_width: 256,
			window_right: false,
			window_h: 0,
			window_down: false,
			window_v: 0,
			dma_length: 0,
			dma_source: 0,
			dma_type: DmaType::Cpu,
			data_address: 0,
			data_code: DataCode::VramRead,
			data_type_bits: 0,
			v_interrupt_triggered: false,
			scanline_sprite_overflow: false,
			sprite_overlap: false,
			odd_frame: false,
		}
	}

	pub fn get_v_counter(&self) -> u8 {
		(self.v_counter & 0xFF) as u8
	}

	pub fn get_h_counter(&self) -> u8 {
		(self.h_counter >> 1) as u8
	}

	#[bitmatch]
	pub fn write_register(&mut self, data: u16) {
		if data & 0xF000 != 0 {
			self.data_command = false;
		}
		#[bitmatch]
		match data {
			"1000_0000_??cd_?fgh" => {
				self.left_blank = c == 1;
				self.h_interrupt_enable = d == 1;
				self.palette_select = f == 1;
				self.hv_counter_latch = g == 1;
				self.display_disable = h == 1;
			},
			"1000_0001_abcd_ef??" => {
				self.vram_128k = a == 1;
				self.display_enable = b == 1;
				self.v_interrupt_enable = c == 1;
				self.dma_enable = d == 1;
				self.v_resolution = e == 1;
				self.video_mode = f == 1;
			},
			"1000_0010_?aaa_a???" => {
				self.plane_a_address = (a as u32) << 13;
			},
			"1000_0011_?www_www?" => {
				self.window_address = (w as u32) << 11;
			},
			"1000_0100_????_bbbb" => {
				self.plane_b_address = (b as u32) << 13;
			},
			"1000_0101_ssss_ssss" => {
				self.sprite_table_address = (s as u32) << 9;
			},
			// Something about the sprite data in 128K RAM mode, finding contradictory info about it
			"1000_0110_????_????" => {
			},
			"1000_0111_??pp_cccc" => {
				self.background_palette = p;
				self.background_color = c;
			}
			"1000_1000_hhhh_hhhh" => {
				self.sms_h_scroll = h;
			},
			"1000_1001_vvvv_vvvv" => {
				self.sms_v_scroll = v;
			},
			"1000_1010_hhhh_hhhh" => {
				self.h_interrupt_counter = h;
			},
			"1000_1011_????ivhh" => {
				self.ext_interrupt_enable = i == 1;
				self.column_scroll = v == 1;
				self.h_scroll = match h {
					0b00 => HScroll::Fullscreen,
					0b10 => HScroll::Row,
					0b11 => HScroll::Line,
					_ => panic!("Invalid hscroll mode {:#04b}.", h),
				}
			},
			"1000_1100_h???_sii?" => {
				let previous_mode = self.h32_mode;
				self.h32_mode = h == 0;
				if self.h32_mode && !previous_mode {
					self.clock_speed = 10;
				}
				if !self.h32_mode && previous_mode {
					self.clock_speed = 8;
				}
				self.shadow_highlight_mode = s == 1;
				self.interlace_mode = match i {
					0x00 => InterlaceMode::NoInterlace,
					0x01 => InterlaceMode::NormalInterlace,
					0x11 => InterlaceMode::DoubleInterlace,
					_ => panic!("Invalid interlace mode {:#04b}.", i),
				}
			},
			"1000_1101_?hhh_hhhh" => {
				self.h_scroll_address = (h as u32) << 10;
			}
			// Something about plane tables in 128K RAM mode, finding contradictory info about it
			"1000_1110_????_????" => {
			},
			"1000_1111_iiii_iiii" => {
				self.auto_increment = i;
			},
			"1001_0000_??hh_??ww" => {
				let height = match h {
					0x00 => 256,
					0x01 => 512,
					0x11 => 1024,
					_ => panic!("Invalid plane height setting {:#04b}", h),
				};
				let width = match w {
					0x00 => 256,
					0x01 => 512,
					0x11 => 1024,
					_ => panic!("Invalid plane width setting {:#04b}", h),
				};
				if h * w >= 0x2000 { panic!("Plane area exceeds 0x2000 pixels."); }
				self.plane_height = height;
				self.plane_width = width;
			},
			"1001_0001_r??h_hhhh" => {
				self.window_right = r == 1;
				self.window_h = h;
			},
			"1001_0010_d??v_vvvv" => {
				self.window_down = d == 1;
				self.window_v = v;
			},
			"1001_0011_dddd_dddd" => {
				self.dma_length = (self.dma_length & 0xFF00) | d;
			}
			"1001_0100_dddd_dddd" => {
				self.dma_length = (self.dma_length & 0x00FF) | (d << 8);
			},
			"1001_0101_aaaa_aaaa" => {
				self.dma_source = (self.dma_source & 0xFFFF00) | (a as u32);
			},
			"1001_0110_aaaa_aaaa" => {
				self.dma_source = (self.dma_source & 0xFF00FF) | ((a as u32) << 8);
			},
			"1001_0111_ttaa_aaaa" => {
				self.dma_source = (self.dma_source & 0x00FFFF) | ((a as u32) << 16);
				self.dma_type = match t {
					0b10 => DmaType::Fill,
					0b11 => DmaType::Copy,
					_ => {
						self.dma_source = (self.dma_source & 0x3FFFFF) | (((t as u32) & 0x1) << 22);
						DmaType::Cpu
					}
				};
			},
			"0000_0000_cccc_00aa" if self.data_command => {
				self.data_type_bits = self.data_type_bits | (c << 2);
				self.data_address = self.data_address | (a << 14);
				self.data_code = match self.data_type_bits {
					0b000000 => DataCode::VramRead,
					0b000001 => DataCode::VramWrite,
					0b100001 => DataCode::VramDma,
					0b001000 => DataCode::CramRead,
					0b000011 => DataCode::CramWrite,
					0b100011 => DataCode::CramDma,
					0b000100 => DataCode::VsramRead,
					0b000101 => DataCode::VsramWrite,
					0b100101 => DataCode::VsramDma,
					0b001100 => DataCode::EightBitRead,
					_ => panic!("Invalid VDP data type."),
				};
				self.dma_active = (self.data_type_bits & 0b100000) == 0b100000;
				self.data_command = false;
			},
			"ccaa_aaaa_aaaa_aaaa" => {
				self.data_type_bits = c;
				self.data_address = a;
				self.data_command = true;
			},
			_ => panic!("Unimplemented VDP control write {data:#06X}."),
		}
	}

	pub fn read_register_upper(&self) -> u8 {
		let bit_1 = if self.queue_size == 0 { 0b10 }
		else { 0 };
		let bit_0 = if self.queue_size == 4 { 0b1 }
		else { 0 };
		0b00110100 | bit_1 | bit_0
	}

	pub fn read_register_lower(&self) -> u8 {
		let bit_7 = if self.v_interrupt_triggered { 0b10000000 }
		else { 0 };
		let bit_6 = if self.scanline_sprite_overflow {0b1000000 }
		else { 0 };
		let bit_5 = if self.sprite_overlap { 0b100000 }
		else { 0 };
		let bit_4 = if self.odd_frame { 0b10000 }
		else { 0 };
		let bit_3 = if self.v_blank { 0b1000 }
		else { 0 };
		let bit_2 = if self.h_blank { 0b100 }
		else { 0 };
		let bit_1 = if self.dma_active { 0b10 }
		else { 0 };
		let bit_0 = if self.v_resolution { 0b1 }
		else { 0 };
		bit_7 | bit_6 | bit_5 | bit_4 | bit_3 | bit_2 | bit_1 | bit_0
	}

	pub fn external_write(&mut self, data: u16) -> bool {
		if self.dma_active && self.dma_type == DmaType::Fill {
			self.fill_active = true;
		}
		if self.queue_size < 4 {
			self.queue[self.queue_end] = FifoSlot {
				code: self.data_code,
				address: self.data_address,
				data: data,
				half_complete: false,
			};
			self.data_address = match self.data_code {
				DataCode::VramWrite | DataCode::VramDma => self.data_address.wrapping_add(self.auto_increment),
				DataCode::CramWrite | DataCode::CramDma =>
					(self.data_address + self.auto_increment) % (CRAM_SIZE as u16),
				DataCode::VsramWrite | DataCode::VsramDma =>
					(self.data_address + self.auto_increment) % (VSRAM_SIZE as u16),
				_ => panic!("Attempted VDP read in write mode"),
			};
			self.queue_size += 1;
			self.queue_end = (self.queue_end + 1) % 4;
			true
		}
		else {
			false
		}
	}

	pub fn external_read_upper(&mut self) -> u8 {
		match self.data_code {
			DataCode::VramRead => {
				self.vram[self.data_address as usize]
			},
			DataCode::CramRead => {
				self.color_ram[self.data_address as usize]
			},
			DataCode::VsramRead => {
				self.vscroll_ram[self.data_address as usize]
			},
			_ => panic!("Attempted VDP write in read mode.")
		}
	}

	pub fn external_read_lower(&mut self) -> u8 {
		match self.data_code {
			DataCode::VramRead => {
				let data = self.vram[self.data_address.wrapping_add(1) as usize];
				self.data_address = self.data_address.wrapping_add(self.auto_increment);
				data
			},
			DataCode::CramRead => {
				let data = self.color_ram[(self.data_address + 1) as usize % CRAM_SIZE];
				self.data_address = (self.data_address + self.auto_increment) % (CRAM_SIZE as u16);
				data
			},
			DataCode::VsramRead => {
				let data = self.vscroll_ram[(self.data_address + 1) as usize % VSRAM_SIZE];
				self.data_address = (self.data_address + self.auto_increment) % (VSRAM_SIZE as u16);
				data
			},
			_ => panic!("Attempted VDP write in read mode.")
		}
	}

	pub fn print_vram(&self) {
		for y in 0..=0xFFF {
			print!("{:06X}   ", y * 0x10);
			for x in 0..=0xF {
				print!("{:02X} ", self.vram[y * 0x10 + x]);
			}
			println!();
		}
	}

	pub fn read_vram_u16(&self, address: u16) -> u16 {
		let address_index = address as usize;
		((self.vram[address_index] as u16) << 8) + (self.vram[address_index.wrapping_add(1)] as u16)
	}

	pub fn read_vsram_u16(&self, address: u16) -> u16 {
		let address_index = address as usize;
		((self.vscroll_ram[address_index] as u16) << 8) + (self.vscroll_ram[(address_index + 1) % VSRAM_SIZE] as u16)
	}

	pub fn read_cram_u16(&self, address: u16) -> u16 {
		let address_index = address as usize;
		((self.color_ram[address_index] as u16) << 8) + (self.color_ram[(address_index + 1) % CRAM_SIZE] as u16)
	}
}

#[derive(Debug)]
pub enum HScroll {
	Fullscreen,
	Row,
	Line,
}

#[derive(Debug)]
pub enum InterlaceMode {
	NoInterlace,
	NormalInterlace,
	DoubleInterlace,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DmaType {
	Cpu,
	Fill,
	Copy
}

#[derive(Debug, Copy, Clone)]
pub struct FifoSlot {
	pub code: DataCode,
	pub address: u16,
	pub data: u16,
	pub half_complete: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum DataCode {
	VramRead,
	VramWrite,
	VramDma,
	CramRead,
	CramWrite,
	CramDma,
	VsramRead,
	VsramWrite,
	VsramDma,
	EightBitRead,
}