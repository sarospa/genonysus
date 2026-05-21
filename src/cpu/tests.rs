use super::*;

fn test_cpu() -> CPU {
	let mut rom: Vec<u8> = vec![0u8; 0x1000];
	rom[6] = 0x02;
	CPU::new(&rom)
}

#[test]
fn movea() {
	let mut cpu = test_cpu();
	cpu.write_rom(0x30);
	cpu.write_rom(0x7C);
	cpu.write_rom(0x80);
	cpu.write_rom(0x00);
	cpu.test_reset();
	cpu.run_opcode();
	assert!(cpu.a[0] == 0xFFFF8000, "Expected value of A0 is 0xFFFF8000, actual value is {:#010X}", cpu.a[0]);
}

#[test]
fn instr_move() {
	let mut cpu = test_cpu();
	// MOVE #$1234,D0
	cpu.write_rom(0x30);
	cpu.write_rom(0x3C);
	cpu.write_rom(0x12);
	cpu.write_rom(0x34);
	cpu.test_reset();
	cpu.run_opcode();
	assert!(cpu.d[0] == 0x1234, "Expected value of D0 is 0x00001234, actual value is {:#010X}", cpu.d[0]);
}

#[test]
fn move_to_sr() {
	let mut cpu = test_cpu();
	// MOVE SR,#$2700
	cpu.write_rom(0x46);
	cpu.write_rom(0xFC);
	cpu.write_rom(0x27);
	cpu.write_rom(0x00);
	cpu.test_reset();
	cpu.run_opcode();
	assert!(cpu.status_register == 0x2700, "Expected value of status register is 0x2700, actual value is {:#06X}", cpu.status_register);
}

#[test]
fn tst_zero() {
	let mut cpu = test_cpu();
	// TST.W ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x78);
	cpu.write_rom(0x03);
	cpu.write_rom(0x00);
	cpu.cart_memory[0x300] = 0x00;
	cpu.test_reset();
	cpu.run_opcode();
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.B ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x38);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.L ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0xB8);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_neg() {
	let mut cpu = test_cpu();
	// TST.W ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x78);
	cpu.write_rom(0x03);
	cpu.write_rom(0x00);
	cpu.cart_memory[0x300] = 0x80;
	cpu.test_reset();
	cpu.run_opcode();
	let mut flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.B ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x38);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.L ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0xB8);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_pos() {
	let mut cpu = test_cpu();
	// TST.W ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x78);
	cpu.write_rom(0x03);
	cpu.write_rom(0x00);
	cpu.cart_memory[0x300] = 0x01;
	cpu.test_reset();
	cpu.run_opcode();
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.B ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0x38);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset();
	// TST.L ($300)
	cpu.write_rom(0x4A);
	cpu.write_rom(0xB8);
	cpu.test_reset();
	cpu.run_opcode();
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn moveq() {
	let mut cpu = test_cpu();
	// MOVEQ #$2D,D4
	cpu.write_rom(0x78);
	cpu.write_rom(0x9D);
	cpu.test_reset();
	cpu.run_opcode();
	assert!(cpu.d[4] == 0xFFFFFF9D, "Expected D4: 0xFFFFFF9D, actual value is {:#010X}", cpu.d[4]);
}