use super::*;
use crate::genesis::Motorola68KBus;
use crate::genesis::CPU_ADDRESS_SPACE;

#[test]
fn andi() {
	panic!("TODO");
}

#[test]
fn movea() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	bus.write_program(&mut cpu, 0x307C);
	bus.write_program(&mut cpu, 0x8000);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.a[0] == 0xFFFF8000, "Expected value of A0 is 0xFFFF8000, actual value is {:#010X}", cpu.a[0]);
}

#[test]
fn instr_move() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE #$1234,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x1234, "Expected value of D0 is 0x00001234, actual value is {:#010X}", cpu.d[0]);
}

#[test]
fn move_to_sr() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE SR,#$2700
	bus.write_program(&mut cpu, 0x46FC);
	bus.write_program(&mut cpu, 0x2700);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.status_register == 0x2700, "Expected value of status register is 0x2700, actual value is {:#06X}", cpu.status_register);
}

#[test]
fn tst_zero() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x00;
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_neg() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x80;
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_pos() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x01;
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn move_usp() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE USP,A0
	// LEA $1234,A0
	// MOVE A0,USP
	bus.write_program(&mut cpu, 0x4E68);
	bus.write_program(&mut cpu, 0x41F8);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x4E60);
	cpu.test_reset(&bus);
	cpu.usp = 0x00FF0000;
	cpu.run_opcode(&mut bus);
	assert!(cpu.a[0] == 0x00FF0000, "Expected A0: 0x00FF0000, actual value is {:#010X}", cpu.a[0]);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.usp == 0x00001234, "Expected USP: 0x00001234, actual value is {:#010X}", cpu.usp);
}

#[test]
fn movem() {
	panic!("TODO");
}

#[test]
fn lea() {
	panic!("TODO");
}

#[test]
fn addq() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE #$24,D0
    // MOVE #$FF,D1
    // MOVE #$7F,D2
    // MOVE #$FA,D3
    // ADDQ.B #3,D0
    // ADDQ.B #4,D1
    // ADDQ.B #2,D2
    // ADDQ.B #6,D3
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x0024);
	bus.write_program(&mut cpu, 0x323C);
	bus.write_program(&mut cpu, 0x00FF);
	bus.write_program(&mut cpu, 0x343C);
	bus.write_program(&mut cpu, 0x007F);
	bus.write_program(&mut cpu, 0x363C);
	bus.write_program(&mut cpu, 0x00FA);
	bus.write_program(&mut cpu, 0x5600);
	bus.write_program(&mut cpu, 0x5801);
	bus.write_program(&mut cpu, 0x5402);
	bus.write_program(&mut cpu, 0x5C03);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus); // MOVE #$24,D0
	cpu.run_opcode(&mut bus); // MOVE #$FF,D1
	cpu.run_opcode(&mut bus); // MOVE #$7F,D2
	cpu.run_opcode(&mut bus); // MOVE #$FA,D3
	cpu.run_opcode(&mut bus); // ADDQ.B #3,D0
	assert!(cpu.d[0] == 0x27, "Expected D0: 0x00000027, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b00000, "Expected CCR: 0b00000, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // ADDQ.B #4,D1
	assert!(cpu.d[1] == 0x03, "Expected D1: 0x00000003, actual value is {:#010X}", cpu.d[1]);
	assert!((cpu.status_register & 0b11111) == 0b10001, "Expected CCR: 0b10001, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // ADDQ.B #2,D2
	assert!(cpu.d[2] == 0x81, "Expected D0: 0x00000081, actual value is {:#010X}", cpu.d[2]);
	assert!((cpu.status_register & 0b11111) == 0b01010, "Expected CCR: 0b01010, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // ADDQ.B #6,D3
	assert!(cpu.d[3] == 0x00, "Expected D0: 0x00000000, actual value is {:#010X}", cpu.d[3]);
	assert!((cpu.status_register & 0b11111) == 0b10101, "Expected CCR: 0b10101, actual value is {:#07b}", cpu.status_register);
}

#[test]
fn subq() {
	panic!("TODO");
}

#[test]
fn bra() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BRA *+8
	bus.write_program(&mut cpu, 0x6006);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bcc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BCC *+8
	// BCC *+8
	bus.write_program(&mut cpu, 0x6406);
	bus.write_program(&mut cpu, 0x6406);
	cpu.test_reset(&bus);
	cpu.set_c(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_c(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bcs() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BCS *+8
	// BCS *+8
	bus.write_program(&mut cpu, 0x6506);
	bus.write_program(&mut cpu, 0x6506);
	cpu.test_reset(&bus);
	cpu.set_c(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_c(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn beq() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BEQ *+8
	// BEQ *+8
	bus.write_program(&mut cpu, 0x6706);
	bus.write_program(&mut cpu, 0x6706);
	cpu.test_reset(&bus);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bge() {
	panic!("TODO");
}

#[test]
fn bgt() {
	panic!("TODO");
}

#[test]
fn bhi() {
	panic!("TODO");
}

#[test]
fn ble() {
	panic!("TODO");
}

#[test]
fn bls() {
	panic!("TODO");
}

#[test]
fn blt() {
	panic!("TODO");
}

#[test]
fn bmi() {
	panic!("TODO");
}

#[test]
fn bne() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BNE *+8
	// BNE *+8
	bus.write_program(&mut cpu, 0x6606);
	bus.write_program(&mut cpu, 0x6606);
	cpu.test_reset(&bus);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bpl() {
	panic!("TODO");
}

#[test]
fn bvc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BVC *+8
	// BVC *+8
	bus.write_program(&mut cpu, 0x6806);
	bus.write_program(&mut cpu, 0x6806);
	cpu.test_reset(&bus);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bvs() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// BVS *+8
	// BVS *+8
	bus.write_program(&mut cpu, 0x6906);
	bus.write_program(&mut cpu, 0x6906);
	cpu.test_reset(&bus);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn jmp() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// JMP ($0400)
	bus.write_program(&mut cpu, 0x4EF8);
	bus.write_program(&mut cpu, 0x0400);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x0400, "Expected PC: 0x00000400, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn moveq() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVEQ #$2D,D4
	bus.write_program(&mut cpu, 0x789D);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[4] == 0xFFFFFF9D, "Expected D4: 0xFFFFFF9D, actual value is {:#010X}", cpu.d[4]);
}

#[test]
fn dbcc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE #$0F,D0
	// LOOP:
    // ADDQ #1,D1
    // DBF D0,LOOP
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x000F);
	bus.write_program(&mut cpu, 0x5241);
	bus.write_program(&mut cpu, 0x51C8);
	bus.write_program(&mut cpu, 0xFFFC);
	cpu.test_reset(&bus);
	for _ in 0..33 {
		cpu.run_opcode(&mut bus);
	}
	assert!(cpu.d[0] == 0xFFFF, "Expected D0: 0x0000FFFF, actual value is {:#010X}", cpu.d[0]);
	assert!(cpu.d[1] == 0x10, "Expected D1: 0x00000010, actual value is {:#010X}", cpu.d[1]);
}

#[test]
fn lsd_to_d() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&bus);
	// MOVE.L #$1234567F,D0
	// MOVE.L #$1234567F,D1
	// MOVE.L #$1234567F,D2
	// MOVE #12,D3
	// MOVE #0,D4
	// LSL.L #4,D0
	// LSR.B #8,D2
	// LSL.W D3,D1
	// LSR.W D4,D0
	bus.write_program(&mut cpu, 0x203C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x567F);
	bus.write_program(&mut cpu, 0x223C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x567F);
	bus.write_program(&mut cpu, 0x243C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x567F);
	bus.write_program(&mut cpu, 0x363C);
	bus.write_program(&mut cpu, 0x000C);
	bus.write_program(&mut cpu, 0x383C);
	bus.write_program(&mut cpu, 0x0000);
	bus.write_program(&mut cpu, 0xE988);
	bus.write_program(&mut cpu, 0xE00A);
	bus.write_program(&mut cpu, 0xE769);
	bus.write_program(&mut cpu, 0xE868);
	cpu.test_reset(&bus);
	cpu.run_opcode(&mut bus); // MOVE.L #$1234567F,D0
	cpu.run_opcode(&mut bus); // MOVE.L #$1234567F,D1
	cpu.run_opcode(&mut bus); // MOVE.L #$1234567F,D2
	cpu.run_opcode(&mut bus); // MOVE #12,D3
	cpu.run_opcode(&mut bus); // MOVE #0,D4
	cpu.run_opcode(&mut bus); // LSL.L #4,D0
	assert!(cpu.d[0] == 0x234567F0, "Expected D0: 0x234567F0, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b10001, "Expected CCR: 0b10001, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // LSR.B #8,D2
	assert!(cpu.d[2] == 0x12345600, "Expected D2: 0x12345600, actual value is {:#010X}", cpu.d[2]);
	assert!((cpu.status_register & 0b11111) == 0b00100, "Expected CCR: 0b00100, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // LSL.W D3,D1
	assert!(cpu.d[1] == 0x1234F000, "Expected D1: 0x1234F000, actual value is {:#010X}", cpu.d[1]);
	assert!((cpu.status_register & 0b11111) == 0b11001, "Expected CCR: 0b11001, actual value is {:#07b}", cpu.status_register);
	cpu.run_opcode(&mut bus); // LSR.W D4,D0
	assert!(cpu.d[0] == 0x234567F0, "Expected D0: 0x234567F0, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b10000, "Expected CCR: 0b10000, actual value is {:#07b}", cpu.status_register);
}

// Representation of a simple memory map pointing exclusively to RAM
struct TestBus {
	ram: Vec<u8>
}
impl TestBus {
	fn new() -> TestBus {
		let mut ram = vec![0u8; CPU_ADDRESS_SPACE];
		ram[6] = 0x02;
		TestBus {
			ram: ram
		}
	}
	
	// Helper function for manually writing 68K test code.
	fn write_program(&mut self, cpu: &mut CPU, data: u16) {
		self.ram[cpu.program_counter as usize] = (data >> 8) as u8;
		cpu.program_counter += 1;
		self.ram[cpu.program_counter as usize] = (data & 0xFF) as u8;
		cpu.program_counter += 1;
	}
}
impl Motorola68KBus for TestBus {
	fn read_u8(&self, address: u32) -> u8 {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		self.ram[address_index]
	}
	fn write_u8(&mut self, address: u32, data: u8) {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		self.ram[address_index] = data;
	}
}