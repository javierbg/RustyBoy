pub struct GBIO {
	interrupt: Interrupt,
	sound: Sound,
	joypad: Joypad,
	serial: SerialData,
	ppu: PPU,

	boot: bool
}

impl GBIO {
	pub fn new() -> GBIO {
		GBIO {
			interrupt: Interrupt::default(),
			sound: Sound{},
			joypad: Joypad::default(),
			serial: SerialData{},
			ppu: PPU{},

			boot: true
		}
	}

	// Determines if the system is in its boot sequence
	pub fn boot_sequence(&self) -> bool {
		self.boot
	}

	pub fn write_byte(&mut self, addr: u8, val: u8) {
		match addr {
			0x00 => self.joypad.write_joypad(val),

			//0x01 ... 0x03 => // Serial data transfer

			//0x04 ... 0x07 => // Timer

			0x10 ... 0x26 => println!("Write to SOUND"),

			//0x40 ... 0x4B => // PPU

			0x50 => self.boot = val == 0,

			0x0F => self.interrupt.write_flags(val),
			0xFF => self.interrupt.write_enable(val),

			_ => {
				panic!("Unimplemented IO Write {:02X}", addr);
			}
		}
	}

	pub fn read_byte(&self, addr: u8) -> u8 {
		match addr {
			0x00 => self.joypad.read_joypad(),
			0x0F => self.interrupt.read_flags(),
			0xFF => self.interrupt.read_enable(),
			_ => panic!("Unimplemented IO Read {:02X}", addr)
		}
	}

}

#[derive(Default)]
struct Interrupt {
	enabled_vblank: bool,
	enabled_lcdstat: bool,
	enabled_timer: bool,
	enabled_serial: bool,
	enabled_joypad: bool,

	flagged_vblank: bool,
	flagged_lcdstat: bool,
	flagged_timer: bool,
	flagged_serial: bool,
	flagged_joypad: bool,
}

const INTERRUPT_VBLANK_MASK : u8 = 0b0000_0001;
const INTERRUPT_LCDSTAT_MASK: u8 = 0b0000_0010;
const INTERRUPT_TIMER_MASK  : u8 = 0b0000_0100;
const INTERRUPT_SERIAL_MASK : u8 = 0b0000_1000;
const INTERRUPT_JOYPAD_MASK : u8 = 0b0001_0000;

impl Interrupt {
	pub fn write_flags(&mut self, val: u8) {
		if (val & INTERRUPT_VBLANK_MASK) != 0 {
			self.flagged_vblank = true;
		} else {
			self.flagged_vblank = false;
		}

		if (val & INTERRUPT_LCDSTAT_MASK) != 0 {
			self.flagged_lcdstat = true;
		} else {
			self.flagged_lcdstat = false;
		}

		if (val & INTERRUPT_TIMER_MASK) != 0 {
			self.flagged_timer = true;
		} else {
			self.flagged_timer = false;
		}

		if (val & INTERRUPT_SERIAL_MASK) != 0 {
			self.flagged_serial = true;
		} else {
			self.flagged_serial = false;
		}

		if (val & INTERRUPT_JOYPAD_MASK) != 0 {
			self.flagged_joypad = true;
		} else {
			self.flagged_joypad = false;
		}
	}

	pub fn write_enable(&mut self, val: u8) {
		if (val & INTERRUPT_VBLANK_MASK) != 0 {
			self.enabled_vblank = true;
		} else {
			self.enabled_vblank = false;
		}

		if (val & INTERRUPT_LCDSTAT_MASK) != 0 {
			self.enabled_lcdstat = true;
		} else {
			self.enabled_lcdstat = false;
		}

		if (val & INTERRUPT_TIMER_MASK) != 0 {
			self.enabled_timer = true;
		} else {
			self.enabled_timer = false;
		}

		if (val & INTERRUPT_SERIAL_MASK) != 0 {
			self.enabled_serial = true;
		} else {
			self.enabled_serial = false;
		}

		if (val & INTERRUPT_JOYPAD_MASK) != 0 {
			self.enabled_joypad = true;
		} else {
			self.enabled_joypad = false;
		}
	}

	pub fn read_flags(&self) -> u8 {
		0 +
		if self.flagged_vblank  { INTERRUPT_VBLANK_MASK } else { 0 } +
		if self.flagged_lcdstat { INTERRUPT_LCDSTAT_MASK } else { 0 } +
		if self.flagged_timer   { INTERRUPT_TIMER_MASK } else { 0 } +
		if self.flagged_serial  { INTERRUPT_SERIAL_MASK } else { 0 } +
		if self.flagged_joypad  { INTERRUPT_JOYPAD_MASK } else { 0 }
	}

	pub fn read_enable(&self) -> u8 {
		0 +
		if self.enabled_vblank  { INTERRUPT_VBLANK_MASK } else { 0 } +
		if self.enabled_lcdstat { INTERRUPT_LCDSTAT_MASK } else { 0 } +
		if self.enabled_timer   { INTERRUPT_TIMER_MASK } else { 0 } +
		if self.enabled_serial  { INTERRUPT_SERIAL_MASK } else { 0 } +
		if self.enabled_joypad  { INTERRUPT_JOYPAD_MASK } else { 0 }
	}
}

struct Sound {
	// Lots of stuff
}

// Used for input mapping
// When a write in the joypad gpio occurs, the selected input becomes one of the following
enum SelectedInput {
	Direction, Buttons
}

struct Joypad {
	selected_input: SelectedInput,

	input_up: bool,
	input_down: bool,
	input_left: bool,
	input_right: bool,

	input_a: bool,
	input_b: bool,
	input_start: bool,
	input_select: bool,
}

impl Default for Joypad {
	fn default() -> Joypad {
		Joypad {
			selected_input: SelectedInput::Direction,
			input_up: false,
			input_down: false,
			input_left: false,
			input_right: false,

			input_a: false,
			input_b: false,
			input_start: false,
			input_select: false,
		}
	}
}

// TODO: What should be the behaviour if both are set? It shouldn't happen...
const SELECT_BUTTONS_MASK: u8 = 0b0010_0000;
const SELECT_DIRECTION_MASK: u8 = 0b0001_0000;

const INPUT_RIGHT_VALUE: u8 = 0b0000_0001;
const INPUT_LEFT_VALUE : u8 = 0b0000_0010;
const INPUT_UP_VALUE   : u8 = 0b0000_0100;
const INPUT_DOWN_VALUE : u8 = 0b0000_1000;

const INPUT_A_VALUE     : u8 = INPUT_RIGHT_VALUE;
const INPUT_B_VALUE     : u8 = INPUT_LEFT_VALUE;
const INPUT_SELECT_VALUE: u8 = INPUT_UP_VALUE;
const INPUT_START_VALUE : u8 = INPUT_DOWN_VALUE;

impl Joypad {
	pub fn write_joypad(&mut self, val: u8) {
		if (val & SELECT_BUTTONS_MASK) != 0 {
			self.selected_input = SelectedInput::Buttons;
		}
		else if (val & SELECT_DIRECTION_MASK) != 0 {
			self.selected_input = SelectedInput::Direction;
		}
	}

	pub fn read_joypad(&self) -> u8 {
		match self.selected_input {
			SelectedInput::Direction => {
				0 +
				if self.input_right { INPUT_RIGHT_VALUE } else { 0 } +
				if self.input_left  { INPUT_LEFT_VALUE } else { 0 } +
				if self.input_up    { INPUT_UP_VALUE } else { 0 } +
				if self.input_down  { INPUT_DOWN_VALUE } else { 0 }
			},

			SelectedInput::Buttons => {
				0 +
				if self.input_a      { INPUT_A_VALUE } else { 0 } +
				if self.input_b      { INPUT_B_VALUE } else { 0 } +
				if self.input_select { INPUT_SELECT_VALUE } else { 0 } +
				if self.input_start  { INPUT_START_VALUE } else { 0 }
			}
		}
	}
}

// Link cable!
struct SerialData {

}

struct PPU {

}