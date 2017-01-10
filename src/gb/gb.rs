use super::Interconnect;
use super::cpu;

#[derive(Debug)]
pub struct GB {
	cpu: cpu::Cpu,
	interconnect: Interconnect
}

#[allow(dead_code)]
impl GB {
	pub fn new(boot_rom: Box<[u8]>, cart_rom: Box<[u8]>) -> GB {
		GB {
			cpu: cpu::Cpu::default(),
			interconnect: Interconnect::new(boot_rom, cart_rom)
		}
	}

	pub fn step(&mut self) {
		self.cpu.step(&mut self.interconnect);
	}

	pub fn run(&mut self) {
		self.cpu.run(&mut self.interconnect);
	}
}
