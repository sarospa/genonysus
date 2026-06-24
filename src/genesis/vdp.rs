use bitmatch::bitmatch;

#[derive(Debug)]
pub struct VDP {
	h_cycles: u8,
	h_counter: u16,
	h_phase: u8,
	h_blank: bool,
	v_counter: u16,
	v_phase: u8,
	v_blank: bool,
	dot_counter: u16,
	vram: Vec<u8>,
	color_ram: Vec<u8>,
	vscroll_ram: Vec<u8>,
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
}
impl VDP {
	pub fn new() -> VDP {
		VDP {
			h_cycles: 0,
			h_counter: 0,
			h_phase: 1,
			h_blank: false,
			v_counter: 0,
			v_phase: 1,
			v_blank: false,
			dot_counter: 0,
			vram: vec!(0u8; 0x10000),
			color_ram: vec!(0u8; 128),
			vscroll_ram: vec!(0u8; 80),
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
			plane_height: 0,
			plane_width: 0,
		}
	}
	
	pub fn advance_master_cycle(&mut self) {
		self.dot_counter = (self.dot_counter + 1) % 3420;
		let mut v_advance = false;
		self.h_cycles += 1;
		if self.h32_mode {
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
	}
	
	pub fn get_v_counter(&self) -> u8 {
		(self.v_counter & 0xFF) as u8
	}
	
	pub fn get_h_counter(&self) -> u8 {
		(self.h_counter >> 1) as u8
	}
	
	#[bitmatch]
	pub fn write_register(&mut self, data: u16) {
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
					0b10 => HScroll::EightPixel,
					0b11 => HScroll::OnePixel,
					_ => panic!("Invalid hscroll mode {:#04b}.", h),
				}
			},
			"1000_1100_h???_sii?" => {
				self.h32_mode = h == 1;
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
				self.plane_height = width;
			},
			_ => panic!("Unimplemented VDP register {:#04X}.", (data >> 8)),
		}
	}
}

#[derive(Debug)]
enum HScroll {
	Fullscreen,
	EightPixel,
	OnePixel,
}

#[derive(Debug)]
enum InterlaceMode {
	NoInterlace,
	NormalInterlace,
	DoubleInterlace,
}