use crate::genesis::Motorola68KBus;
use crate::genesis::screen::Screen;
use crate::genesis::{VDPState, DmaType, DataCode, HScroll, VSRAM_SIZE, CRAM_SIZE};

#[derive(Debug)]
pub struct VDP {
    frame: u64,
}
impl VDP {
	pub fn new() -> VDP {
		VDP {
            frame: 0,
		}
	}

	pub fn advance_cycle(&mut self, bus: &mut dyn Motorola68KBus, screen: &mut dyn Screen) {
		let mut v_interrupt = false;
		{
			let state = bus.expose_vdp_state();
			state.dot_counter = state.dot_counter + (state.clock_speed as u16);
			if state.dot_counter >= 3420 {
				state.dot_counter %= 3420;
				state.h_counter = 0;
				state.h_phase = 1;
			}
			let mut v_advance = false;
			if state.h32_mode {
				state.h_counter = (state.h_counter + 1) & 0x1FF;
				if state.h_counter == 0x128 && state.h_phase == 1 {
					state.h_phase = 2;
					state.h_counter = 0x1D2;
				} else if state.h_counter == 0x00 && state.h_phase == 2 {
					state.h_phase = 1;
				}
				if state.h_counter == 0x126 { state.h_blank = true; } else if state.h_counter == 0x0A { state.h_blank = false; }

				if state.h_counter == 0x10A { v_advance = true; }
			} else {
				state.h_counter = (state.h_counter + 1) & 0x1FF;
				if state.h_counter == 0x16D && state.h_phase == 1 {
					state.h_phase = 2;
					state.h_counter = 0x1C9 + 1;
				} else if state.h_counter == 0x00 && state.h_phase == 2 {
					state.h_phase = 1;
				}
				if state.h_counter == 0x166 { state.h_blank = true; } else if state.h_counter == 0x0C { state.h_blank = false; }

				if state.h_counter == 0x14A { v_advance = true; }
				state.clock_speed = match state.h_counter {
					0x1CD..=0x1D3 | 0x1D6..=0x1DC | 0x1DE..=0x1E4 | 0x1E7..=0x1ED => 10,
					0x1D4..=0x1D5 | 0x1E5..=0x1E6 => 9,
					_ => 8,
				};
			}
			state.countdown = state.clock_speed;

			if v_advance {
				state.v_counter = (state.v_counter + 1) & 0x1FF;
				if state.v_counter == 0x0EB && state.v_phase == 1 {
					state.v_phase = 2;
					state.v_counter = 0x0E5;
				} else if state.v_counter == 0x100 && state.v_phase == 2 {
					state.v_phase = 1;
					state.v_counter = 0x000;
				}
				if state.v_counter == 0xE0 && state.v_phase == 1 {
					state.v_blank = true;
                    screen.frame_complete();
					if state.v_interrupt_enable {
						v_interrupt = true;
						state.v_interrupt_triggered = true;
					}
				} else if state.v_counter == 0xFF && state.v_phase == 2 {
					state.v_blank = false;
					self.frame += 1;
					if self.frame % 60 == 0 {
						println!("frame {}", self.frame);
						if self.frame == 600 {
							panic!("10 seconds");
						}
					}
					state.v_interrupt_triggered = false;
				}
			}

			if state.slot_ready {
				self.access_vram(state);
				if state.h32_mode {
					state.access_slot = (state.access_slot + 1) % 171;
				} else {
					state.access_slot = (state.access_slot + 1) % 210;
				}
				state.slot_ready = false;
			}
			else {
				state.slot_ready = true;
			}
			self.compose_pixel(state, screen);
		}
		if v_interrupt {
			bus.assert_interrupt(6);
		}
	}

	fn access_vram(&mut self, state: &mut VDPState) {
		let slot_type = current_vram_slot(state.h32_mode, state.access_slot, state.v_blank || (!state.display_enable));
		match slot_type {
			VramSlot::ExternalAccess => {
                if state.queue_size == 0 {
                    return;
                }
                let write = &mut state.queue[state.queue_start];
                match write.code {
                    DataCode::VramDma => {
                        if state.dma_enable && state.dma_active && state.auto_increment > 0 {
                            match state.dma_type {
                                DmaType::Cpu => {
                                    panic!("CPU DMA unimplemented.");
                                },
                                DmaType::Fill => {
                                    if state.fill_active {
										state.vram[write.address as usize] = (write.data & 0xFF) as u8;
										write.address = write.address.wrapping_add(state.auto_increment);
										state.dma_length = state.dma_length.wrapping_sub(1);
                                        if state.dma_length == 0 {
											state.dma_active = false;
											state.fill_active = false;
											state.queue_size -= 1;
											state.queue_start = (state.queue_start + 1) % 4;
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
							state.vram[write.address as usize] = data;
                            write.half_complete = true;
                        }
                        else {
                            let data = (write.data & 0x00FF) as u8;
							state.vram[write.address.wrapping_add(1) as usize] = data;
							state.queue_size -= 1;
							state.queue_start = (state.queue_start + 1) % 4;
                        }
                    },
                    DataCode::CramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
						state.color_ram[write.address as usize] = write_upper;
						state.color_ram[((write.address + 1)) as usize % CRAM_SIZE] = write_lower;
						state.queue_size -= 1;
						state.queue_start = (state.queue_start + 1) % 4;
                    },
                    DataCode::VsramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
						state.vscroll_ram[write.address as usize] = write_upper;
						state.vscroll_ram[((write.address + 1)) as usize % VSRAM_SIZE] = write_lower;
						state.queue_size -= 1;
						state.queue_start = (state.queue_start + 1) % 4;
                    },
                    _ => panic!("Attempted VDP RAM write in read mode."),
                }
			}
			_ => ()
		}
	}

	fn compose_pixel(&self, state: &VDPState, screen: &mut dyn Screen) {
		if (state.h32_mode && (state.h_counter >= 256 || state.v_counter >= 224))
			|| (!state.h32_mode) && (state.h_counter >= 320 || state.v_counter >= 224) {
			return;
		}
		// Apparently scroll bytes are interleaved, first plane B, then plane A.
		let x_offset_b = match state.h_scroll {
			HScroll::Fullscreen => state.read_vram_u16(state.h_scroll_address as u16),
			HScroll::Row => state.read_vram_u16((state.h_scroll_address as u16) + (state.h_counter / 2)),
			HScroll::Line => state.read_vram_u16((state.h_scroll_address as u16) + (state.h_counter * 4)),
		};
		let y_offset_b = match state.column_scroll {
			false => state.read_vsram_u16(0),
			true => state.read_vram_u16(state.v_counter / 8),
		};
		let x = (state.h_counter + x_offset_b) % state.plane_width;
		let y = (state.v_counter + y_offset_b) % state.plane_height;
		let nametable_index_b = (state.plane_b_address as u16) + ((x / 8) * 2) + ((y / 8) * (state.plane_width / 4));
		let tile_data = state.read_vram_u16(nametable_index_b);
		let _high_priority = ((tile_data >> 15) & 0x1) == 0x1;
		let palette = (tile_data >> 13) & 0b11;
		let v_flip = (tile_data >> 12) & 0x1;
		let h_flip = (tile_data >> 11) & 0x1;
		let tile_index = (tile_data & 0x7F) << 5;
		let tile_x = (x & 0b111) ^ (h_flip * 0b111);
		let tile_y = (y & 0b111) ^ (v_flip * 0b111);
		let mut palette_color: u16 = state.vram[(tile_index + (tile_x / 2) + (tile_y * 4)) as usize] as u16;
		palette_color = if x % 2 == 0 { (palette_color & 0xF0) >> 4 } else { palette_color & 0x0F };
		let color = state.read_cram_u16((palette << 5) + (palette_color * 2));
		screen.output_pixel(color, state.h_counter, state.v_counter);
	}
}

#[derive(Debug, Clone, Copy)]
pub enum VramSlot {
	HscrollData,
	LayerAMapping,
	LayerAPattern,
	LayerBMapping,
	LayerBPattern,
	SpriteMapping,
	SpritePattern,
	ExternalAccess,
	Refresh,
}

const PHASE_1_LIST: [VramSlot; 13] = [VramSlot::HscrollData, VramSlot::SpritePattern, VramSlot::SpritePattern,
	VramSlot::SpritePattern, VramSlot::SpritePattern, VramSlot::LayerAMapping, VramSlot::SpritePattern,
	VramSlot::LayerAPattern, VramSlot::LayerAPattern, VramSlot::LayerBMapping, VramSlot::SpritePattern,
	VramSlot::LayerBPattern, VramSlot::LayerBPattern];
const PHASE_2_LIST: [VramSlot; 32] = [VramSlot::LayerAMapping, VramSlot::ExternalAccess, VramSlot::LayerAPattern,
	VramSlot::LayerAPattern, VramSlot::LayerBMapping, VramSlot::SpriteMapping, VramSlot::LayerBPattern,
	VramSlot::LayerBPattern, VramSlot::LayerAMapping, VramSlot::ExternalAccess, VramSlot::LayerAPattern,
	VramSlot::LayerAPattern, VramSlot::LayerBMapping, VramSlot::SpriteMapping, VramSlot::LayerBPattern,
	VramSlot::LayerBPattern, VramSlot::LayerAMapping, VramSlot::ExternalAccess, VramSlot::LayerAPattern,
	VramSlot::LayerAPattern, VramSlot::LayerBMapping, VramSlot::SpriteMapping, VramSlot::LayerBPattern,
	VramSlot::LayerBPattern, VramSlot::LayerAMapping, VramSlot::Refresh, VramSlot::LayerAPattern,
	VramSlot::LayerAPattern, VramSlot::LayerBMapping, VramSlot::SpriteMapping, VramSlot::LayerBPattern,
	VramSlot::LayerBPattern];

pub fn current_vram_slot(h32_mode: bool, slot: u16, display_disable: bool) -> VramSlot {
	let slot_index = slot as usize;
	if display_disable {
		return VramSlot::ExternalAccess
	}
	if h32_mode {
		if slot_index < 13 {
			PHASE_1_LIST[slot_index]
		}
		else if slot_index < 141 {
			PHASE_2_LIST[(slot_index - 13) % 32]
		}
		else {
			if slot == 141 || slot == 142 || slot == 156 || slot == 170 {
				VramSlot::ExternalAccess
			}
			else {
				VramSlot::SpritePattern
			}
		}
	}
	else {
		if slot_index < 13 {
			PHASE_1_LIST[slot_index]
		}
		else if slot_index < 173 {
			PHASE_2_LIST[(slot_index - 13) % 8]
		}
		else {
			if slot == 173 || slot == 174 || slot == 198 {
				VramSlot::ExternalAccess
			}
			else {
				VramSlot::SpritePattern
			}
		}
	}
}