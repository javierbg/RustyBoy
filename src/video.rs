use mem_map;

const N_SPRITES: usize = (mem_map::SPRITE_RAM_LENGTH as usize) / 4;

pub struct PPU {
	sprite_ram: [Sprite ; N_SPRITES],
	pub vram: [u8 ; mem_map::VRAM_LENGTH as usize],

	// LCD Control
	lcd_display_enabled: bool,
	window_tile_map_address: bool,
	window_enabled: bool,
	background_window_tile_data_address: bool,
	background_tile_map_address: bool,
	sprite_size: bool, // true means 16px height sprites, false means 8px height
	sprites_enabled: bool,
	background_enabled: bool,

	//TODO: LCDC status

	scroll_y: u8,
	scroll_x: u8,

	lcdc_y_coordinate: u8,
	ly_compare: u8,

	background_palette: [Color ; 4],
	object_palettes: [[Color ; 4] ; 2], // Color 0 in each of these two won't really be used (it's always transparency)

	window_y_position: u8,
	window_x_position: u8,
}

impl Default for PPU {
	fn default() -> PPU {
		PPU {
			//sprite_ram: [0u8 ; mem_map::SPRITE_RAM_LENGTH as usize],
			sprite_ram: [Sprite::default() ; N_SPRITES],
			vram: [0u8 ; mem_map::VRAM_LENGTH as usize],

			lcd_display_enabled: false,
			window_tile_map_address: false,
			window_enabled: false,
			background_window_tile_data_address: false,
			background_tile_map_address: false,
			sprite_size: false, // false means 8px height, true means 16px height sprites
			sprites_enabled: false,
			background_enabled: false,

			scroll_y: 0,
			scroll_x: 0,

			lcdc_y_coordinate: 0,
			ly_compare: 0,

			background_palette: [Color::White ; 4],
			object_palettes: [[Color::White ; 4] ; 2],

			window_y_position: 0,
			window_x_position: 0,
		}
	}
}

// LCDC masks
const LCDC_LCD_DISPLAY_ENABLE_MASK         : u8 = 0b1000_0000;
const LCDC_WINDOW_TILE_MAP_ADDRESS_MASK    : u8 = 0b0100_0000;
const LCDC_WINDOW_DISPLAY_ENABLE_MASK      : u8 = 0b0010_0000;
const LCDC_BG_WINDOW_TILE_DATA_ADDRESS_MASK: u8 = 0b0001_0000;
const LCDC_BG_TILE_MAP_ADDRESS_MASK        : u8 = 0b0000_1000;
const LCDC_SPRITE_SIZE_MASK                : u8 = 0b0000_0100;
const LCDC_SPRITE_DISPLAY_ENABLE_MASK      : u8 = 0b0000_0010;
const LCDC_BG_DISPLAY_ENABLE_MASK          : u8 = 0b0000_0001;

impl PPU {
	pub fn write_ppu(&mut self, addr: u8, val: u8) {
		match addr {
			0x40 => self.write_lcd_control(val),
			0x42 => self.scroll_y = val,
			0x43 => self.scroll_x = val,

			0x45 => self.ly_compare = val,

			0x47 => Self::write_palette(&mut self.background_palette, val),

			0x48 => Self::write_palette(&mut self.object_palettes[0], val),
			0x49 => Self::write_palette(&mut self.object_palettes[1], val),

			0x4A => self.window_y_position = val,
			0x4B => self.window_x_position = val,

			_ => panic!("Invalid PPU write {:02X}", addr)
		}
	}

	pub fn read_ppu(&self, addr: u8) -> u8 {
		match addr {
			0x42 => self.scroll_y,
			0x43 => self.scroll_x,
			0x44 => self.lcdc_y_coordinate,
			0x45 => self.ly_compare,

			0x47 => Self::read_palette(&self.background_palette),

			0x48 => Self::read_palette(&self.object_palettes[0]),
			0x49 => Self::read_palette(&self.object_palettes[1]),

			0x4A => self.window_y_position,
			0x4B => self.window_x_position,

			_ => panic!("Invalid PPU read {:02X}", addr)
		}
	}

	pub fn write_sprite_entry(&mut self, addr: u8, val: u8) {
		let sprite_index = (addr >> 2) as usize;
		let sprite_byte = addr & 0b11;

		let mut sprite = self.sprite_ram[sprite_index];

		match sprite_byte {
			0b00 => sprite.position_y = val,
			0b01 => sprite.position_x = val,
			0b10 => sprite.tile_number = val,
			0b11 => {
				sprite.priority = if (val & 0b1000_0000) != 0 { true } else { false };
				sprite.flip_y = if (val & 0b0100_0000) != 0 { true } else { false };
				sprite.flip_x = if (val & 0b0010_0000) != 0 { true } else { false };
				sprite.palette = if (val & 0b1000_0000) != 0 { 1 } else { 0 };
			}

			_ => {}
		}
	}

	pub fn read_sprite_entry(&self, addr: u8) -> u8 {
		let sprite_index = (addr >> 2) as usize;
		let sprite_byte = addr & 0b11;

		let sprite = self.sprite_ram[sprite_index];

		match sprite_byte {
			0b00 => sprite.position_y,
			0b01 => sprite.position_x,
			0b10 => sprite.tile_number,
			0b11 => 0 +
				if sprite.priority { SPRITE_PRIORITY_MASK } else { 0 } +
				if sprite.flip_y { SPRITE_FLIP_Y_MASK } else { 0 } +
				if sprite.flip_x { SPRITE_FLIP_X_MASK } else { 0 } +
				(sprite.palette as u8),

			_ => panic!("It's impossible that this happened, as there's no fifth byte to see"),
		}
	}

	fn read_palette(palette: &[Color ; 4]) -> u8 {
		0 +
		 Color::to_bits(palette[0]) +
		(Color::to_bits(palette[1]) << 2) +
		(Color::to_bits(palette[2]) << 4) +
		(Color::to_bits(palette[3]) << 6)
	}

	fn write_palette(palette: &mut [Color ; 4], val: u8) {
		// Color for 00
		let c0 = val & 0b0000_0011;
		//Color for 01
		let c1 = (val & 0b0000_1100) >> 2;
		//Color for 10
		let c2 = (val & 0b0011_0000) >> 4;
		//Color for 11
		let c3 = (val & 0b1100_0000) >> 6;

		palette[0] = Color::from_bits(c0);
		palette[1] = Color::from_bits(c1);
		palette[2] = Color::from_bits(c2);
		palette[3] = Color::from_bits(c3);
	}

	fn write_lcd_control(&mut self, val: u8) {
		self.lcd_display_enabled = if (val & LCDC_LCD_DISPLAY_ENABLE_MASK) != 0 {true} else {false};
		self.window_tile_map_address = if (val & LCDC_WINDOW_TILE_MAP_ADDRESS_MASK) != 0 {true} else {false};
		self.window_enabled = if (val & LCDC_WINDOW_DISPLAY_ENABLE_MASK) != 0 {true} else {false};
		self.background_window_tile_data_address = if (val & LCDC_BG_WINDOW_TILE_DATA_ADDRESS_MASK) != 0 {true} else {false};
		self.background_tile_map_address = if (val & LCDC_BG_TILE_MAP_ADDRESS_MASK) != 0 {true} else {false};
		self.sprite_size = if (val & LCDC_SPRITE_SIZE_MASK) != 0 {true} else {false};
		self.sprites_enabled = if (val & LCDC_SPRITE_DISPLAY_ENABLE_MASK) != 0 {true} else {false};
		self.background_enabled = if (val & LCDC_BG_DISPLAY_ENABLE_MASK) != 0 {true} else {false};
	}

	fn read_lcd_control(&self) -> u8 {
		0 +
		if self.lcd_display_enabled { LCDC_LCD_DISPLAY_ENABLE_MASK } else {0} +
		if self.window_tile_map_address { LCDC_WINDOW_TILE_MAP_ADDRESS_MASK } else {0} +
		if self.window_enabled { LCDC_WINDOW_DISPLAY_ENABLE_MASK } else {0} +
		if self.background_window_tile_data_address { LCDC_BG_WINDOW_TILE_DATA_ADDRESS_MASK } else {0} +
		if self.background_tile_map_address { LCDC_BG_TILE_MAP_ADDRESS_MASK } else {0} +
		if self.sprite_size { LCDC_SPRITE_SIZE_MASK } else {0} +
		if self.sprites_enabled { LCDC_SPRITE_DISPLAY_ENABLE_MASK } else {0} +
		if self.background_enabled { LCDC_BG_DISPLAY_ENABLE_MASK } else {0}
	}
}

// The 4 displayed colors on the Game Boy
#[derive(Copy, Clone)]
enum Color {
	White, LightGray, DarkGray, Black
	// Well, more like green, other green, more green and greener
	// but you get the idea
}

impl Color {
	fn from_bits(val: u8) -> Color {
		match val {
			0x00 => Color::White,
			0x01 => Color::LightGray,
			0x02 => Color::DarkGray,
			0x03 => Color::Black,
			_ => panic!("Invalid color {:02X}", val)
		}
	}

	fn to_bits(col: Color) -> u8 {
		match col {
			Color::White     => 0b00,
			Color::LightGray => 0b01,
			Color::DarkGray  => 0b10,
			Color::Black     => 0b11,
		}
	}
}

#[derive(Default, Copy, Clone)]
struct Sprite {
	position_y: u8,
	position_x: u8,

	tile_number: u8,

	priority: bool,
	flip_y: bool,
	flip_x: bool,
	palette: usize, // usize type because it will be used to index the sprite palettes
}

const SPRITE_PRIORITY_MASK : u8 = 0b1000_0000;
const SPRITE_FLIP_Y_MASK   : u8 = 0b0100_0000;
const SPRITE_FLIP_X_MASK   : u8 = 0b0010_0000;
const SPRITE_PALETTE_MASK  : u8 = 0b0001_0000;
