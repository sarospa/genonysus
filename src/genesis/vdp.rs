use bitmatch::bitmatch;

mod vdp_support;
use vdp_support::*;
use crate::genesis::cpu::CPU;

#[derive(Debug)]
pub struct VDP {
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
	regs: VDPRegisters,
    frame: u64,
	pub clock_speed: u8,
	pub countdown: u8,
}
impl VDP {
	pub fn new() -> VDP {
		let empty_fifo = FifoSlot {
			code: DataCode::VramRead,
			address: 0,
			data: 0,
			half_complete: false,
		};
		VDP {
			h_counter: 0,
			h_phase: 1,
			h_blank: false,
			access_slot: 0,
			slot_ready: false,
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
			regs: VDPRegisters::new(),
            frame: 0,
			clock_speed: 10,
			countdown: 0,
		}
	}

	pub fn advance_cycle(&mut self, cpu: &mut CPU) {
		self.dot_counter = self.dot_counter + (self.clock_speed as u16);
		let mut v_advance = false;
		if self.regs.h32_mode {
			self.h_counter = (self.h_counter + 1) & 0x1FF;
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
		else {
			self.h_counter = (self.h_counter + 1) & 0x1FF;
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
			self.clock_speed = match self.h_counter {
				0x1CD..=0x1D3 | 0x1D6..=0x1DC | 0x1DE..=0x1E4 | 0x1E7..=0x1ED => 10,
				0x1D4..=0x1D5 | 0x1E5..=0x1E6 => 9,
				_ => 8,
			};
		}
		self.countdown = self.clock_speed;

		if self.dot_counter >= 3420 {
			self.dot_counter %= 3420;
			self.h_counter = 0;
			self.h_phase = 1;
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
			if self.v_counter == 0xE0 && self.v_phase == 1 {
				self.v_blank = true;
				if self.regs.v_interrupt_enable {
					cpu.assert_interrupt(6);
					self.regs.v_interrupt_triggered = true;
				}
			}
			else if self.v_counter == 0xFF && self.v_phase == 2 {
				self.v_blank = false;
				self.frame += 1;
				println!("frame {}", self.frame);
				self.regs.v_interrupt_triggered = false;
			}
		}

		if self.slot_ready {
			self.access_vram();
			if self.regs.h32_mode {
				self.access_slot = (self.access_slot + 1) % 171;
			}
			else {
				self.access_slot = (self.access_slot + 1) % 210;
			}
			self.slot_ready = false;
		}
		else {
			self.slot_ready = true;
		}
	}

	pub fn get_v_counter(&self) -> u8 {
		(self.v_counter & 0xFF) as u8
	}
	
	pub fn get_h_counter(&self) -> u8 {
		(self.h_counter >> 1) as u8
	}

	fn access_vram(&mut self) {
		let slot_type = current_vram_slot(self.regs.h32_mode, self.access_slot, self.v_blank || (!self.regs.display_enable));
		match slot_type {
			VramSlot::ExternalAccess => {
                if self.queue_size == 0 {
                    return;
                }
                let write = &mut self.queue[self.queue_start];
                match write.code {
                    DataCode::VramDma => {
                        if self.regs.dma_enable && self.dma_active && self.regs.auto_increment > 0 {
                            match self.regs.dma_type {
                                DmaType::Cpu => {
                                    panic!("CPU DMA unimplemented.");
                                },
                                DmaType::Fill => {
                                    if self.fill_active {
                                        self.vram[self.regs.data_address as usize] = (write.data & 0xFF) as u8;
                                        self.regs.data_address = self.regs.data_address.wrapping_add(self.regs.auto_increment);
                                        self.regs.dma_length = self.regs.dma_length.wrapping_sub(1);
                                        if self.regs.dma_length == 0 {
                                            self.dma_active = false;
                                            self.fill_active = false;
                                            self.queue_size -= 1;
                                            self.queue_start = (self.queue_start + 1) % 4;
                                        }
                                    }
                                },
                                DmaType::Copy => {
                                    panic!("Copy DMA unimplemented.");
                                }
                            }
                        }
                    },
                    DataCode::VramWrite => {
                        if !write.half_complete {
                            let data = ((write.data & 0xFF00) >> 8) as u8;
                            self.vram[self.regs.data_address as usize] = data;
                            write.half_complete = true;
                        }
                        else {
                            let data = (write.data & 0x00FF) as u8;
                            self.vram[self.regs.data_address.wrapping_add(1) as usize] = data;
                            self.queue_size -= 1;
                            self.queue_start = (self.queue_start + 1) % 4;
                            self.regs.data_address = self.regs.data_address.wrapping_add(self.regs.auto_increment);
                        }
                    },
                    DataCode::CramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
                        self.color_ram[self.regs.data_address as usize] = write_upper;
                        self.color_ram[((self.regs.data_address + 1)) as usize % CRAM_SIZE] = write_lower;
                        self.queue_size -= 1;
                        self.queue_start = (self.queue_start + 1) % 4;
                        self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (CRAM_SIZE as u16);
                    },
                    DataCode::VsramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
                        self.vscroll_ram[self.regs.data_address as usize] = write_upper;
                        self.vscroll_ram[((self.regs.data_address + 1)) as usize % CRAM_SIZE] = write_lower;
                        self.queue_size -= 1;
                        self.queue_start = (self.queue_start + 1) % 4;
                        self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (VSRAM_SIZE as u16);
                    },
                    _ => panic!("Attempted VDP RAM write in read mode."),
                }
			}
			_ => ()
		}
	}

	pub fn write_ram(&mut self, data: u8, data_type: DataCode) {
		let address_index = self.regs.data_address as usize;
		match data_type {
			DataCode::VramWrite => {
				self.vram[address_index] = data;
			},
			DataCode::CramWrite => {
				self.color_ram[address_index] = data;
				self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (CRAM_SIZE as u16);
			},
			DataCode::VsramWrite => {
				self.vscroll_ram[address_index] = data;
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
				let previous_mode = self.regs.h32_mode;
				self.regs.h32_mode = h == 1;
				if self.regs.h32_mode && !previous_mode {
					self.clock_speed = 10;
				}
				if !self.regs.h32_mode && previous_mode {
					self.clock_speed = 8;
				}
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
				self.regs.data_code = match self.regs.data_type_bits {
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
			self.fill_active = true;
		}
		if self.queue_size < 4 {
			self.queue[self.queue_end] = FifoSlot {
                code: self.regs.data_code,
                address: self.regs.data_address,
                data: data,
                half_complete: false,
            };
            self.regs.data_address = match self.regs.data_code {
                DataCode::VramWrite | DataCode::VramDma => self.regs.data_address.wrapping_add(self.regs.auto_increment),
                DataCode::CramWrite | DataCode::CramDma =>
                    (self.regs.data_address + self.regs.auto_increment) % (CRAM_SIZE as u16),
                DataCode::VsramWrite | DataCode::VsramDma =>
                    (self.regs.data_address + self.regs.auto_increment) % (VSRAM_SIZE as u16),
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
		match self.regs.data_code {
			DataCode::VramRead => {
				self.vram[self.regs.data_address as usize]
			},
			DataCode::CramRead => {
				self.color_ram[self.regs.data_address as usize]
			},
			DataCode::VsramRead => {
				self.vscroll_ram[self.regs.data_address as usize]
			},
			_ => panic!("Attempted VDP write in read mode.")
		}
	}

	pub fn external_read_lower(&mut self) -> u8 {
		match self.regs.data_code {
			DataCode::VramRead => {
				let data = self.vram[self.regs.data_address.wrapping_add(1) as usize];
				self.regs.data_address = self.regs.data_address.wrapping_add(self.regs.auto_increment);
				data
			},
			DataCode::CramRead => {
				let data = self.color_ram[(self.regs.data_address + 1) as usize % CRAM_SIZE];
				self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (CRAM_SIZE as u16);
				data
			},
			DataCode::VsramRead => {
				let data = self.vscroll_ram[(self.regs.data_address + 1) as usize % VSRAM_SIZE];
				self.regs.data_address = (self.regs.data_address + self.regs.auto_increment) % (VSRAM_SIZE as u16);
				data
			},
			_ => panic!("Attempted VDP write in read mode.")
		}
	}
}