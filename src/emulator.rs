use std::collections::HashSet;
use std::io;
use std::io::Write;

use cpu::{Register, Reg8, Reg16};
use gb;

enum ExecutionMode {
	Running,
	Debugging,
}

#[derive(Clone, Copy)]
enum DebugCommand {
	Continue, // Continue until next breakpoint
	SetBreakpoint(u16), // Set breakpoint at said memory address (it should be the start of an instruction)
	PrintRegister(Register), // Print the contents of a register
	Quit,
	Disassemble(u16), // Disassemble the next n instructions
	PrintCpuRegs, // Print all CPU registers
	Step, // Execute just one CPU instruction
	LastCommand, // Repeat last command
}

/*impl Clone for DebugCommand {
	fn clone(&self) -> DebugCommand {
		match self {
			DebugCommand::PrintRegister(r) => DebugCommand::PrintRegister(r),
			_ => *self
		}
	}
}
impl Copy for DebugCommand { }*/

pub struct Emulator {
	gb: gb::GB,
	mode: ExecutionMode,

	//Debugging
	breakpoints: HashSet<u16>,
	last_command: Option<DebugCommand>,
}

impl Emulator {
	pub fn new(boot_rom: Box<[u8]>, cart_rom: Box<[u8]>, debug: bool) -> Emulator {
		Emulator {
			gb: gb::GB::new(boot_rom, cart_rom),
			mode: if debug {ExecutionMode::Debugging} else {ExecutionMode::Running},

			breakpoints: HashSet::new(),
			last_command: None,
		}
	}

	pub fn run(&mut self) {
		match self.mode {
			ExecutionMode::Running => {
				self.gb.run();
			}

			ExecutionMode::Debugging => {
				self.run_debug_mode();
			}
		}
	}

	fn execute_debug_command(&mut self, command: DebugCommand) {
		match command {
			DebugCommand::SetBreakpoint(addr) => {
				if self.breakpoints.contains(&addr) {
					self.breakpoints.remove(&addr);
					println!("Breakpoint at 0x{:04X} removed", addr);
				}
				else {
					self.breakpoints.insert(addr);
					println!("Breakpoint set at address 0x{:04X}", addr);
				}
			}

			DebugCommand::Continue => {
				while !self.breakpoints.contains(&self.gb.cpu.pc) {
					let pc_of_inst = self.gb.cpu.pc; // Needs to be retreived before step
					let inst = self.gb.step();
					println!("  {:04X} : {}", pc_of_inst, inst);
				}
			}

			DebugCommand::Step => {
				let pc_of_inst = self.gb.cpu.pc; // Needs to be retreived before step
				let inst = self.gb.step();
				println!("  {:04X} : {}", pc_of_inst, inst);
			}

			DebugCommand::PrintRegister(r) => {
				match r {
					Register::Register8(r8) => {
						let val = self.gb.read_8bit_register(r8);
						println!("{}: {:02X}", r8, val);
					}
					Register::Register16(r16) => {
						let val = self.gb.read_16bit_register(r16);
						println!("{}: {:04X}", r16, val);
					}
				}
			}

			DebugCommand::PrintCpuRegs => {
				println!("{}", self.gb.cpu);
			}

			DebugCommand::Disassemble(n) => {
				for (addr,inst) in self.gb.get_next_instructions(n) {
					println!("  {:04X} : {}", addr, inst);
				}
			}

			_ => {}
		}
	}

	fn run_debug_mode(&mut self) {
		let mut stdout = io::stdout();
		let stdin = io::stdin();
		let mut stdin_buffer = String::new();

		loop{
			print!("> ");
			stdout.flush();

			// parse it
			stdin_buffer.clear();
			if let Err(err) = stdin.read_line(&mut stdin_buffer) {
				println!("Input error: {}", err);
				continue;
			}

			if let Some(comm) = Self::parse_debug_operation(&stdin_buffer) {
				match comm {
					DebugCommand::Quit => break,
					DebugCommand::LastCommand => {
						/*if let Some(last_comm) = self.last_command {
							self.execute_debug_command(last_comm);
						}*/
					},
					_ => self.execute_debug_command(comm),
				}

				self.last_command = Some(comm);
			}

			else {
				println!("Unrecognized operation");
			}
		}
	}

	fn parse_debug_operation(input: &String) -> Option<DebugCommand> {
		let chunks = input.split_whitespace().collect::<Vec<&str>>();

		if chunks.len() == 0 {
			return None;
		}

		match chunks[0] {
			"c" => Some(DebugCommand::Continue),

			"b" => {
				if chunks.len() != 2 {
					println!("`b' syntax: b <hex_address>");
					None
				}
				else {
					if let Ok(addr) = u16::from_str_radix(chunks[1], 16) {
						Some(DebugCommand::SetBreakpoint(addr))
					}
					else {
						println!("Invalid address");
						None
					}
				}
			},

			"pa" => Some(DebugCommand::PrintCpuRegs),

			"p" => {
				if chunks.len() != 2 {
					println!("`p' syntax: p <register>");
					None
				}
				else {
					if let Some(reg) = match chunks[1].to_uppercase().as_ref() {
						"A" => Some(Register::Register8(Reg8::A)),
						"F" => Some(Register::Register8(Reg8::F)),
						"B" => Some(Register::Register8(Reg8::B)),
						"C" => Some(Register::Register8(Reg8::C)),
						"D" => Some(Register::Register8(Reg8::D)),
						"E" => Some(Register::Register8(Reg8::E)),
						"H" => Some(Register::Register8(Reg8::H)),
						"L" => Some(Register::Register8(Reg8::L)),
						"SP" => Some(Register::Register16(Reg16::SP)),
						"PC" => Some(Register::Register16(Reg16::PC)),
						"BC" => Some(Register::Register16(Reg16::BC)),
						"DE" => Some(Register::Register16(Reg16::DE)),
						"HL" => Some(Register::Register16(Reg16::HL)),
						_   => {
							println!("Unrecognized register \"{}\"", chunks[1]);
							None
						}
					} {
						Some(DebugCommand::PrintRegister(reg))
					}
					else{
						None
					}

				}
			}

			"q" => Some(DebugCommand::Quit),

			"d" => {
				if chunks.len() != 2 {
					println!("`d' syntax: d <number_of_instructions>");
					None
				}
				else {
					if let Ok(n) = u16::from_str_radix(chunks[1], 10) {
						Some(DebugCommand::Disassemble(n))
					}
					else {
						println!("Invalid number of instructions \"{}\"", chunks[1]);
						None
					}
				}
			}

			"s" => Some(DebugCommand::Step),

			"" => Some(DebugCommand::LastCommand),

			_ => None
		}
	}
}
