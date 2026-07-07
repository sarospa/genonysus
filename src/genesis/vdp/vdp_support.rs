pub const VRAM_SIZE: usize = 0x10000;
pub const CRAM_SIZE: usize = 128;
pub const VSRAM_SIZE: usize = 80;

#[derive(Debug)]
pub enum HScroll {
    Fullscreen,
    EightPixel,
    OnePixel,
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

#[derive(Debug, Copy, Clone)]
pub struct FifoSlot {
    pub code: DataCode,
    pub address: u16,
    pub data: u16,
    pub half_complete: bool,
}

#[derive(Debug)]
pub struct VDPRegisters {
    pub left_blank: bool,
    pub palette_select: bool,
    pub h_interrupt_enable: bool,
    pub hv_counter_latch: bool,
    pub display_disable: bool,
    pub vram_128k: bool,
    pub display_enable: bool,
    pub v_interrupt_enable: bool,
    pub dma_enable: bool,
    pub v_resolution: bool,
    pub video_mode: bool,
    pub plane_a_address: u32,
    pub window_address: u32,
    pub plane_b_address: u32,
    pub sprite_table_address: u32,
    pub background_palette: u16,
    pub background_color: u16,
    pub sms_h_scroll: u16,
    pub sms_v_scroll: u16,
    pub h_interrupt_counter: u16,
    pub h_interrupt_countdown: u16,
    pub ext_interrupt_enable: bool,
    pub column_scroll: bool,
    pub h_scroll: HScroll,
    pub h32_mode: bool,
    pub shadow_highlight_mode: bool,
    pub interlace_mode: InterlaceMode,
    pub h_scroll_address: u32,
    pub auto_increment: u16,
    pub plane_height: u16,
    pub plane_width: u16,
    pub window_right: bool,
    pub window_h: u16,
    pub window_down: bool,
    pub window_v: u16,
    pub dma_length: u16,
    pub dma_source: u32,
    pub dma_type: DmaType,
    pub data_address: u16,
    pub data_code: DataCode,
    pub data_type_bits: u16,
    pub v_interrupt_triggered: bool,
    pub scanline_sprite_overflow: bool,
    pub sprite_overlap: bool,
    pub odd_frame: bool,
}
impl VDPRegisters {
    pub fn new() -> VDPRegisters {
        VDPRegisters {
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