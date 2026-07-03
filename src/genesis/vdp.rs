use bitmatch::bitmatch;

mod vdp_support;
use vdp_support::*;

#[derive(Debug)]
pub struct VDP {
	h_cycles: u8,
	h_counter: u16,
	h_phase: u8,
	h_blank: bool,
	serial_clock: u16,
	v_counter: u16,
	v_phase: u8,
	v_blank: bool,
	dot_counter: u16,
	vram: Vec<u8>,
	color_ram: Vec<u8>,
	vscroll_ram: Vec<u8>,
	dma_active: bool,
	fill_active: bool,
	queue: [u16; 4],
	queue_index: usize,
	queue_size: u8,
	data_command: bool,
	regs: VDPRegisters,
}
impl VDP {
	pub fn new() -> VDP {
		VDP {
			h_cycles: 0,
			h_counter: 0,
			h_phase: 1,
			h_blank: false,
			serial_clock: 0,
			v_counter: 0,
			v_phase: 1,
			v_blank: false,
			dot_counter: 0,
			vram: vec!(0u8; VRAM_SIZE),
			color_ram: vec!(0u8; CRAM_SIZE),
			vscroll_ram: vec!(0u8; VSRAM_SIZE),
			dma_active: false,
			fill_active: false,
			queue: [0, 0, 0, 0],
			queue_index: 0,
			queue_size: 0,
			data_command: false,
			regs: VDPRegisters::new()
		}
	}
	
	pub fn advance_master_cycle(&mut self) {
		self.dot_counter = (self.dot_counter + 1) % 3420;
		let mut v_advance = false;
		let mut access_slot = false;
		self.h_cycles += 1;
		if self.regs.h32_mode {
			if self.h_cycles % 5 == 0 {
				self.serial_clock = (self.serial_clock + 1) % 684;
				access_slot = (self.serial_clock % 4) == 0
			}
			if self.h_cycles >= 10 {
				self.h_counter = (self.h_counter + 1) & 0x1FF;
				self.h_cycles = 0;
				if self.h_counter == 0x128 && self.h_phase == 1 {
					self.h_phase = 2;
					self.h_counter = 0x1D2;
				}
				else if self.h_counter == 0x00 && self.h_phase == 2 {
					self.h_phase = 1;
				}
				if self.h_counter == 0x126 { self.h_blank = true; }
				else if self.h_counter == 0x0A { self.h_blank = false; }
				
				if self.h_counter == 0x10A { v_advance = true; }
			}
		}
		else {
			let cycle_count = match self.h_counter {
				0x1CD..=0x1D3 | 0x1D6..=0x1DC | 0x1DE..=0x1E4 | 0x1E7..=0x1ED => 10,
				0x1D4..=0x1D5 | 0x1E5..=0x1E6 => 9,
				_ => 8,
			};
			if self.h_cycles == cycle_count / 2 || self.h_cycles == cycle_count {
				self.serial_clock = (self.serial_clock + 1) % 840;
				access_slot = (self.serial_clock % 4) == 0
			}
			if self.h_cycles >= cycle_count {
				self.h_counter = (self.h_counter + 1) & 0x1FF;
				self.h_cycles = 0;
				if self.h_counter == 0x16D && self.h_phase == 1 {
					self.h_phase = 2;
					self.h_counter = 0x1C9 + 1;
				}
				else if self.h_counter == 0x00 && self.h_phase == 2 {
					self.h_phase = 1;
				}
				if self.h_counter == 0x166 { self.h_blank = true; }
				else if self.h_counter == 0x0C { self.h_blank = false; }
				
				if self.h_counter == 0x14A { v_advance = true; }
			}
		}
		if self.dot_counter == 0 {
			self.h_cycles = 0;
			self.h_counter = 0;
			self.h_phase = 1;
			self.serial_clock = 0;
		}
		if v_advance {
			self.v_counter = (self.v_counter + 1) & 0x1FF;
			if self.v_counter == 0x0EB && self.v_phase == 1 {
				self.v_phase = 2;
				self.v_counter = 0x0E5;
			}
			else if self.v_counter == 0x100 && self.v_phase == 2 {
				self.v_phase = 1;
				self.v_counter = 0x000;
			}
			if self.v_counter == 0xE0 && self.v_phase == 1 { self.v_blank = true; }
			else if self.v_counter == 0xFF && self.v_phase == 1 { self.v_blank = false; }
		}
		if access_slot {
			self.access_vram();
		}
	}
	
	pub fn get_v_counter(&self) -> u8 {
		(self.v_counter & 0xFF) as u8
	}
	
	pub fn get_h_counter(&self) -> u8 {
		(self.h_counter >> 1) as u8
	}

	fn access_vram(&mut self) {
		let slot_number = self.serial_clock / 4;
		let slot_type = current_vram_slot(self.regs.h32_mode, slot_number, self.v_blank);
		match slot_type {
			VramSlot::ExternalAccess => {
				if self.regs.dma_enable && self.dma_active && self.regs.auto_increment > 0 {
					match self.regs.dma_type {
						DmaType::Cpu => {
							panic!("CPU DMA unimplemented.");
						},
						DmaType::Fill => {
							if self.fill_active {
								self.vram[self.regs.data_address as usize] = self.regs.dma_fill;
								self.regs.data_address = self.regs.data_address.wrapping_add(self.regs.auto_increment);
								self.regs.dma_length = self.regs.dma_length.wrapping_sub(1);
								if self.regs.dma_length == 0 {
									self.dma_active = false;
									self.fill_active = false;
								}
							}
						},
						DmaType::Copy => {
							panic!("Copy DMA unimplemented.");
						}
					}
				}
			}
			_ => ()
		}
	}

	pub fn write_ram_u16(&mut self, data: u16, data_type: DataType) {
		let address_index = self.regs.data_address as usize;
		match data_type {
			DataType::VramWrite => {
				self.vram[address_index] = ((data & 0xFF00) >> 8) as u8;
				self.vram[address_index.wrapping_add(1)] = (data & 0x00FF) as u8;
				self.regs.data_address = self.regs.data_address.wrapping_add(self.regs.auto_increment);
			},
			DataType::CramWrite => {
				self.color_ram[address_index] = ((data & 0xFF00) >> 8) as u8;
				self.color_ram[address_index.wrapping_add(1)] = (data & 0x00FF) as u8;
				self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (CRAM_SIZE as u16);
			},
			DataType::VsramWrite => {
				self.vscroll_ram[address_index] = ((data & 0xFF00) >> 8) as u8;
				self.vscroll_ram[address_index.wrapping_add(1)] = (data & 0x00FF) as u8;
				self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (VSRAM_SIZE as u16);
			},
			_ => panic!("Attempted VDP RAM write in read mode."),
		}
	}
	
	#[bitmatch]
	pub fn write_register(&mut self, data: u16) {
		if data & 0xF000 != 0 {
			self.data_command = false;
		}
		#[bitmatch]
		match data {
			"1000_0000_??cd_?fgh" => {
				self.regs.left_blank = c == 1;
				self.regs.h_interrupt_enable = d == 1;
				self.regs.palette_select = f == 1;
				self.regs.hv_counter_latch = g == 1;
				self.regs.display_disable = h == 1;
			},
			"1000_0001_abcd_ef??" => {
				println!("VDP reg $01 to {:#06X}", data);
				self.regs.vram_128k = a == 1;
				self.regs.display_enable = b == 1;
				self.regs.v_interrupt_enable = c == 1;
				self.regs.dma_enable = d == 1;
				self.regs.v_resolution = e == 1;
				self.regs.video_mode = f == 1;
			},
			"1000_0010_?aaa_a???" => {
				self.regs.plane_a_address = (a as u32) << 13;
			},
			"1000_0011_?www_www?" => {
				self.regs.window_address = (w as u32) << 11;
			},
			"1000_0100_????_bbbb" => {
				self.regs.plane_b_address = (b as u32) << 13;
			},
			"1000_0101_ssss_ssss" => {
				self.regs.sprite_table_address = (s as u32) << 9;
			},
			// Something about the sprite data in 128K RAM mode, finding contradictory info about it
			"1000_0110_????_????" => {
			},
			"1000_0111_??pp_cccc" => {
				self.regs.background_palette = p;
				self.regs.background_color = c;
			}
			"1000_1000_hhhh_hhhh" => {
				self.regs.sms_h_scroll = h;
			},
			"1000_1001_vvvv_vvvv" => {
				self.regs.sms_v_scroll = v;
			},
			"1000_1010_hhhh_hhhh" => {
				self.regs.h_interrupt_counter = h;
			},
			"1000_1011_????ivhh" => {
				self.regs.ext_interrupt_enable = i == 1;
				self.regs.column_scroll = v == 1;
				self.regs.h_scroll = match h {
					0b00 => HScroll::Fullscreen,
					0b10 => HScroll::EightPixel,
					0b11 => HScroll::OnePixel,
					_ => panic!("Invalid hscroll mode {:#04b}.", h),
				}
			},
			"1000_1100_h???_sii?" => {
				self.regs.h32_mode = h == 1;
				self.regs.shadow_highlight_mode = s == 1;
				self.regs.interlace_mode = match i {
					0x00 => InterlaceMode::NoInterlace,
					0x01 => InterlaceMode::NormalInterlace,
					0x11 => InterlaceMode::DoubleInterlace,
					_ => panic!("Invalid interlace mode {:#04b}.", i),
				}
			},
			"1000_1101_?hhh_hhhh" => {
				self.regs.h_scroll_address = (h as u32) << 10;
			}
			// Something about plane tables in 128K RAM mode, finding contradictory info about it
			"1000_1110_????_????" => {
			},
			"1000_1111_iiii_iiii" => {
				self.regs.auto_increment = i;
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
				self.regs.plane_height = height;
				self.regs.plane_height = width;
			},
			"1001_0001_r??h_hhhh" => {
				self.regs.window_right = r == 1;
				self.regs.window_h = h;
			},
			"1001_0010_d??v_vvvv" => {
				self.regs.window_down = d == 1;
				self.regs.window_v = v;
			},
			"1001_0011_dddd_dddd" => {
				self.regs.dma_length = (self.regs.dma_length & 0xFF00) | d;
			}
			"1001_0100_dddd_dddd" => {
				self.regs.dma_length = (self.regs.dma_length & 0x00FF) | (d << 8);
			},
			"1001_0101_aaaa_aaaa" => {
				self.regs.dma_source = (self.regs.dma_source & 0xFFFF00) | (a as u32);
			},
			"1001_0110_aaaa_aaaa" => {
				self.regs.dma_source = (self.regs.dma_source & 0xFF00FF) | ((a as u32) << 8);
			},
			"1001_0111_ttaa_aaaa" => {
				self.regs.dma_source = (self.regs.dma_source & 0x00FFFF) | ((a as u32) << 16);
				self.regs.dma_type = match t {
					0b10 => DmaType::Fill,
					0b11 => DmaType::Copy,
					_ => {
						self.regs.dma_source = (self.regs.dma_source & 0x3FFFFF) | (((t as u32) & 0x1) << 22);
						DmaType::Cpu
					}
				};
			},
			"0000_0000_cccc_00aa" if self.data_command => {
				self.regs.data_type_bits = self.regs.data_type_bits | (c << 2);
				self.regs.data_address = self.regs.data_address | (a << 14);
				self.regs.data_type = match self.regs.data_type_bits & 0xF {
					0b0000 => DataType::VramRead,
					0b0001 => DataType::VramWrite,
					0b1000 => DataType::CramRead,
					0b0011 => DataType::CramWrite,
					0b0100 => DataType::VsramRead,
					0b0101 => DataType::VsramWrite,
					0b1100 => DataType::EightBit,
					_ => panic!("Invalid VDP data type."),
				};
				self.dma_active = (self.regs.data_type_bits & 0b100000) == 0b100000;
				self.data_command = false;
			},
			"ccaa_aaaa_aaaa_aaaa" => {
				self.regs.data_type_bits = c;
				self.regs.data_address = a;
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
		let bit_7 = if self.regs.v_interrupt_triggered { 0b10000000 }
		else { 0 };
		let bit_6 = if self.regs.scanline_sprite_overflow {0b1000000 }
		else { 0 };
		let bit_5 = if self.regs.sprite_overlap { 0b100000 }
		else { 0 };
		let bit_4 = if self.regs.odd_frame { 0b10000 }
		else { 0 };
		let bit_3 = if self.v_blank { 0b1000 }
		else { 0 };
		let bit_2 = if self.h_blank { 0b100 }
		else { 0 };
		let bit_1 = if self.dma_active { 0b10 }
		else { 0 };
		let bit_0 = if self.regs.v_resolution { 0b1 }
		else { 0 };
		bit_7 | bit_6 | bit_5 | bit_4 | bit_3 | bit_2 | bit_1 | bit_0
	}

	pub fn external_write(&mut self, data: u16) -> bool {
		if self.dma_active && self.regs.dma_type == DmaType::Fill {
			self.regs.dma_fill = (data & 0xFF) as u8;
			self.fill_active = true;
		}
		if self.queue_size < 4 {
			self.queue[self.queue_index] = data;
			self.queue_size += 1;
			self.queue_index = (self.queue_index + 1) % 4;
			true
		}
		else {
			false
		}
	}
}