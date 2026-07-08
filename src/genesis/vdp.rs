use crate::genesis::DataCode;
use crate::genesis::Motorola68KBus;
use crate::genesis::VDPState;
use crate::genesis::DmaType;
use crate::genesis::VSRAM_SIZE;
use crate::genesis::CRAM_SIZE;

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

	pub fn advance_cycle(&mut self, bus: &mut dyn Motorola68KBus) {
		let mut v_interrupt = false;
		{
			let state = bus.expose_vdp_state();
			state.dot_counter = state.dot_counter + (state.clock_speed as u16);
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

			if state.dot_counter >= 3420 {
				state.dot_counter %= 3420;
				state.h_counter = 0;
				state.h_phase = 1;
			}

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
					if state.v_interrupt_enable {
						v_interrupt = true;
						state.v_interrupt_triggered = true;
					}
				} else if state.v_counter == 0xFF && state.v_phase == 2 {
					state.v_blank = false;
					self.frame += 1;
					println!("frame {}", self.frame);
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
										state.vram[state.data_address as usize] = (write.data & 0xFF) as u8;
										state.data_address = state.data_address.wrapping_add(state.auto_increment);
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
							state.vram[state.data_address as usize] = data;
                            write.half_complete = true;
                        }
                        else {
                            let data = (write.data & 0x00FF) as u8;
							state.vram[state.data_address.wrapping_add(1) as usize] = data;
							state.queue_size -= 1;
							state.queue_start = (state.queue_start + 1) % 4;
							state.data_address = state.data_address.wrapping_add(state.auto_increment);
                        }
                    },
                    DataCode::CramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
						state.color_ram[state.data_address as usize] = write_upper;
						state.color_ram[((state.data_address + 1)) as usize % CRAM_SIZE] = write_lower;
						state.queue_size -= 1;
						state.queue_start = (state.queue_start + 1) % 4;
						state.data_address = (state.data_address + state.auto_increment) % (CRAM_SIZE as u16);
                    },
                    DataCode::VsramWrite => {
                        let write_lower = (write.data & 0x00FF) as u8;
                        let write_upper = ((write.data & 0xFF00) >> 8) as u8;
						state.vscroll_ram[state.data_address as usize] = write_upper;
						state.vscroll_ram[((state.data_address + 1)) as usize % VSRAM_SIZE] = write_lower;
						state.queue_size -= 1;
						state.queue_start = (state.queue_start + 1) % 4;
						state.data_address = (state.data_address + state.auto_increment) % (VSRAM_SIZE as u16);
                    },
                    _ => panic!("Attempted VDP RAM write in read mode."),
                }
			}
			_ => ()
		}
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