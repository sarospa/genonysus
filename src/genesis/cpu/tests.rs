use super::*;
use crate::genesis::{Motorola68KBus, VDPState};
use crate::genesis::CPU_ADDRESS_SPACE;

#[test]
fn ori() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	// ORI #$8260,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x0040);
	bus.write_program(&mut cpu, 0x8260);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x9274, "Expected value of D0 is 0x00009274, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b01000, "Expected CCR: 0b01000, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn andi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$F234,D0
	// ANDI #$8260,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0xF234);
	bus.write_program(&mut cpu, 0x0240);
	bus.write_program(&mut cpu, 0x8260);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x8220, "Expected value of D0 is 0x00008220, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b01000, "Expected CCR: 0b01000, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn subi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	// SUBI #$4444,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x0440);
	bus.write_program(&mut cpu, 0x4444);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0xCDF0, "Expected value of D0 is 0x0000CDF0, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b11001, "Expected CCR: 0b11001, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn addi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	// ADDI #$DDDD,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x4321);
	bus.write_program(&mut cpu, 0x0640);
	bus.write_program(&mut cpu, 0xDDDD);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x20FE, "Expected value of D0 is 0x000020FE, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b10001, "Expected CCR: 0b10001, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn eori() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	// EORI #$FFFF,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x0A40);
	bus.write_program(&mut cpu, 0xFFFF);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0xEDCB, "Expected value of D0 is 0x0000EDCB, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b01000, "Expected CCR: 0b01000, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn cmpi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	// SUBI #$4444,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x0C40);
	bus.write_program(&mut cpu, 0x4444);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!((cpu.status_register & 0b11111) == 0b01001, "Expected CCR: 0b01001, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn btst() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$AAAA,D0
	// BTST #4,D0
	// BTST #13,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0xAAAA);
	bus.write_program(&mut cpu, 0x0800);
	bus.write_program(&mut cpu, 0x0004);
	bus.write_program(&mut cpu, 0x0800);
	bus.write_program(&mut cpu, 0x000D);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!((cpu.status_register & 0b11111) == 0b01100, "Expected CCR: 0b01100, actual value is {:#07b}", cpu.status_register & 0b11111);
	cpu.run_opcode(&mut bus);
	assert!((cpu.status_register & 0b11111) == 0b01000, "Expected CCR: 0b01000, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn movea() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	bus.write_program(&mut cpu, 0x307C);
	bus.write_program(&mut cpu, 0x8000);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.a[0] == 0xFFFF8000, "Expected value of A0 is 0xFFFF8000, actual value is {:#010X}", cpu.a[0]);
}

#[test]
fn instr_move() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$1234,D0
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x1234);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x1234, "Expected value of D0 is 0x00001234, actual value is {:#010X}", cpu.d[0]);
}

#[test]
fn move_to_sr() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE SR,#$2700
	bus.write_program(&mut cpu, 0x46FC);
	bus.write_program(&mut cpu, 0x2700);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.status_register == 0x2700, "Expected value of status register is 0x2700, actual value is {:#06X}", cpu.status_register);
}

#[test]
fn clr() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$FFFF,D0
	// LEA $2000,A0
	// MOVE #$FFFF,(A0)
	// CLR D0
	// CLR (A0)
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0xFFFF);
	bus.write_program(&mut cpu, 0x41F8);
	bus.write_program(&mut cpu, 0x2000);
	bus.write_program(&mut cpu, 0x30BC);
	bus.write_program(&mut cpu, 0xFFFF);
	bus.write_program(&mut cpu, 0x4240);
	bus.write_program(&mut cpu, 0x4250);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0, "Expected value of D0 is 0x00000000, actual value is {:#010X}", cpu.d[0]);
	assert!(bus.read_u16(0x2000) == 0, "Expected value of ($2000) is 0x00000000, actual value is {:#010X}", bus.read_u16(0x2000));
}

#[test]
fn neg() {
	panic!("TODO");
}

#[test]
fn not() {
	panic!("TODO");
}

#[test]
fn ext() {
	panic!("TODO");
}

#[test]
fn swap() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE.L #$87654321,D0
	// SWAP D0
	// SWAP D0
	bus.write_program(&mut cpu, 0x203C);
	bus.write_program(&mut cpu, 0x8765);
	bus.write_program(&mut cpu, 0x4321);
	bus.write_program(&mut cpu, 0x4840);
	bus.write_program(&mut cpu, 0x4840);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x43218765, "Expected value of D0 is 0x43218765, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b00000, "Expected CCR: 0b00000, actual value is {:#07b}", cpu.status_register & 0b11111);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x87654321, "Expected value of D0 is 0x87654321, actual value is {:#010X}", cpu.d[0]);
	assert!((cpu.status_register & 0b11111) == 0b01000, "Expected CCR: 0b01000, actual value is {:#07b}", cpu.status_register & 0b11111);
}

#[test]
fn pea() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// PEA $12345678
	bus.write_program(&mut cpu, 0x4879);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x5678);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(bus.read_u32(cpu.ssp) == 0x00345678, "Expected value pushed to stack is 0x00345678, actual value is {:#010X}", bus.read_u32(cpu.ssp));
}

#[test]
fn tst_zero() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x00;
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && flags.z, "Expected N: false and Z: true, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_neg() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x80;
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(flags.n && !flags.z, "Expected N: true and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn tst_pos() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// TST.W ($300)
	bus.write_program(&mut cpu, 0x4A78);
	bus.write_program(&mut cpu, 0x0300);
	bus.ram[0x300] = 0x01;
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	let mut flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.B ($300)
	bus.write_program(&mut cpu, 0x4A38);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
	cpu.test_reset(&mut bus);
	// TST.L ($300)
	bus.write_program(&mut cpu, 0x4AB8);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	flags = cpu.get_ccr_flags();
	assert!(!flags.n && !flags.z, "Expected N: false and Z: false, actual value is N: {}, Z: {}", flags.n, flags.z);
}

#[test]
fn link_unlnk() {
	panic!("TODO");
}

#[test]
fn move_usp() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE USP,A0
	// LEA $1234,A0
	// MOVE A0,USP
	bus.write_program(&mut cpu, 0x4E68);
	bus.write_program(&mut cpu, 0x41F8);
	bus.write_program(&mut cpu, 0x1234);
	bus.write_program(&mut cpu, 0x4E60);
	cpu.test_reset(&mut bus);
	cpu.usp = 0x00FF0000;
	cpu.run_opcode(&mut bus);
	assert!(cpu.a[0] == 0x00FF0000, "Expected A0: 0x00FF0000, actual value is {:#010X}", cpu.a[0]);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.usp == 0x00001234, "Expected USP: 0x00001234, actual value is {:#010X}", cpu.usp);
}

#[test]
fn jsr_rts() {
	panic!("TODO");
}

#[test]
fn jmp() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// JMP ($0400)
	bus.write_program(&mut cpu, 0x4EF8);
	bus.write_program(&mut cpu, 0x0400);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x0400, "Expected PC: 0x00000400, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn movem_to_reg() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE.W #$0101,($2000)
	// MOVE.W #$0202,($2002)
	// MOVE.W #$0303,($2004)
	// MOVE.W #$0404,($2006)
	// MOVE.W #$0505,($2008)
	// LEA $2000,A0
	// MOVEM (A0)+,D1/D3/D5/A2/A4
	bus.write_program(&mut cpu, 0x31FC);
	bus.write_program(&mut cpu, 0x0101);
	bus.write_program(&mut cpu, 0x2000);
	bus.write_program(&mut cpu, 0x31FC);
	bus.write_program(&mut cpu, 0x0202);
	bus.write_program(&mut cpu, 0x2002);
	bus.write_program(&mut cpu, 0x31FC);
	bus.write_program(&mut cpu, 0x0303);
	bus.write_program(&mut cpu, 0x2004);
	bus.write_program(&mut cpu, 0x31FC);
	bus.write_program(&mut cpu, 0x0404);
	bus.write_program(&mut cpu, 0x2006);
	bus.write_program(&mut cpu, 0x31FC);
	bus.write_program(&mut cpu, 0x0505);
	bus.write_program(&mut cpu, 0x2008);
	bus.write_program(&mut cpu, 0x41F8);
	bus.write_program(&mut cpu, 0x2000);
	bus.write_program(&mut cpu, 0x4C98);
	bus.write_program(&mut cpu, 0x142A);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[0] == 0x0000, "Expected D0: 0x00000000, actual value is {:#010X}", cpu.d[0]);
	assert!(cpu.d[1] == 0x0101, "Expected D1: 0x00000101, actual value is {:#010X}", cpu.d[1]);
	assert!(cpu.d[2] == 0x0000, "Expected D2: 0x00000000, actual value is {:#010X}", cpu.d[2]);
	assert!(cpu.d[3] == 0x0202, "Expected D3: 0x00000202, actual value is {:#010X}", cpu.d[3]);
	assert!(cpu.d[4] == 0x0000, "Expected D4: 0x00000000, actual value is {:#010X}", cpu.d[4]);
	assert!(cpu.d[5] == 0x0303, "Expected D5: 0x00000303, actual value is {:#010X}", cpu.d[5]);
	assert!(cpu.d[6] == 0x0000, "Expected D6: 0x00000000, actual value is {:#010X}", cpu.d[6]);
	assert!(cpu.d[7] == 0x0000, "Expected D7: 0x00000000, actual value is {:#010X}", cpu.d[7]);
	assert!(cpu.a[0] == 0x200A, "Expected A0: 0x00000000, actual value is {:#010X}", cpu.a[0]);
	assert!(cpu.a[1] == 0x0000, "Expected A1: 0x00000000, actual value is {:#010X}", cpu.a[1]);
	assert!(cpu.a[2] == 0x0404, "Expected A2: 0x00000404, actual value is {:#010X}", cpu.a[2]);
	assert!(cpu.a[3] == 0x0000, "Expected A3: 0x00000000, actual value is {:#010X}", cpu.a[3]);
	assert!(cpu.a[4] == 0x0505, "Expected A4: 0x00000505, actual value is {:#010X}", cpu.a[4]);
	assert!(cpu.a[5] == 0x0000, "Expected A5: 0x00000000, actual value is {:#010X}", cpu.a[5]);
	assert!(cpu.a[6] == 0x0000, "Expected A6: 0x00000000, actual value is {:#010X}", cpu.a[6]);
}

#[test]
fn movem_to_mem() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE.W #$0101,D1
	// MOVE.W #$0202,D3
	// MOVE.W #$0303,D5
	// LEA $0404,A2
	// LEA $0505,A4
	// LEA $200A,A0
	// MOVEM D1/D3/D5/A2/A4,-(A0)
	bus.write_program(&mut cpu, 0x323C);
	bus.write_program(&mut cpu, 0x0101);
	bus.write_program(&mut cpu, 0x363C);
	bus.write_program(&mut cpu, 0x0202);
	bus.write_program(&mut cpu, 0x3A3C);
	bus.write_program(&mut cpu, 0x0303);
	bus.write_program(&mut cpu, 0x45F8);
	bus.write_program(&mut cpu, 0x0404);
	bus.write_program(&mut cpu, 0x49F8);
	bus.write_program(&mut cpu, 0x0505);
	bus.write_program(&mut cpu, 0x41F8);
	bus.write_program(&mut cpu, 0x200A);
	bus.write_program(&mut cpu, 0x48A0);
	bus.write_program(&mut cpu, 0x5428);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(bus.read_u16(0x2000) == 0x0101, "Expected ($2000): 0x00000101, actual value is {:#010X}", bus.read_u16(0x2000));
	assert!(bus.read_u16(0x2002) == 0x0202, "Expected ($2002): 0x00000202, actual value is {:#010X}", bus.read_u16(0x2002));
	assert!(bus.read_u16(0x2004) == 0x0303, "Expected ($2004): 0x00000303, actual value is {:#010X}", bus.read_u16(0x2004));
	assert!(bus.read_u16(0x2006) == 0x0404, "Expected ($2006): 0x00000404, actual value is {:#010X}", bus.read_u16(0x2006));
	assert!(bus.read_u16(0x2008) == 0x0505, "Expected ($2008): 0x00000505, actual value is {:#010X}", bus.read_u16(0x2008));
	assert!(bus.read_u16(0x200A) == 0x0000, "Expected ($200A): 0x00000000, actual value is {:#010X}", bus.read_u16(0x200A));
	assert!(bus.read_u16(0x1FFE) == 0x0000, "Expected ($1FFE): 0x00000000, actual value is {:#010X}", bus.read_u16(0x1FFE));
}

#[test]
fn lea() {
	panic!("TODO");
}

#[test]
fn addq() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
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
	cpu.test_reset(&mut bus);
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
fn scc() {
	panic!("TODO");
}

#[test]
fn dbcc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVE #$0F,D0
	// LOOP:
    // ADDQ #1,D1
    // DBF D0,LOOP
	bus.write_program(&mut cpu, 0x303C);
	bus.write_program(&mut cpu, 0x000F);
	bus.write_program(&mut cpu, 0x5241);
	bus.write_program(&mut cpu, 0x51C8);
	bus.write_program(&mut cpu, 0xFFFC);
	cpu.test_reset(&mut bus);
	for _ in 0..33 {
		cpu.run_opcode(&mut bus);
	}
	assert!(cpu.d[0] == 0xFFFF, "Expected D0: 0x0000FFFF, actual value is {:#010X}", cpu.d[0]);
	assert!(cpu.d[1] == 0x10, "Expected D1: 0x00000010, actual value is {:#010X}", cpu.d[1]);
}

#[test]
fn bsr() {
	panic!("TODO");
}

#[test]
fn bra() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BRA *+8
	bus.write_program(&mut cpu, 0x6006);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bcc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BCC *+8
	// BCC *+8
	bus.write_program(&mut cpu, 0x6406);
	bus.write_program(&mut cpu, 0x6406);
	cpu.test_reset(&mut bus);
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
	let mut cpu = CPU::new(&mut bus);
	// BCS *+8
	// BCS *+8
	bus.write_program(&mut cpu, 0x6506);
	bus.write_program(&mut cpu, 0x6506);
	cpu.test_reset(&mut bus);
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
	let mut cpu = CPU::new(&mut bus);
	// BEQ *+8
	// BEQ *+8
	bus.write_program(&mut cpu, 0x6706);
	bus.write_program(&mut cpu, 0x6706);
	cpu.test_reset(&mut bus);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bge() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BGE *+8
	bus.write_program(&mut cpu, 0x6C06);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bgt() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BGT *+8
	bus.write_program(&mut cpu, 0x6E06);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(false);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(false);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(true);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(true);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(false);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(false);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bhi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BHI *+8
	bus.write_program(&mut cpu, 0x6206);
	cpu.test_reset(&mut bus);
	cpu.set_c(false);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(false);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(true);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(true);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn ble() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BLE *+8
	bus.write_program(&mut cpu, 0x6F06);
	cpu.test_reset(&mut bus);
	cpu.set_z(false);
	cpu.set_n(false);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(false);
	cpu.set_n(false);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(false);
	cpu.set_n(true);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(false);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(true);
	cpu.set_n(false);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(true);
	cpu.set_n(false);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(true);
	cpu.set_n(true);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_z(true);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bls() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BLS *+8
	bus.write_program(&mut cpu, 0x6306);
	cpu.test_reset(&mut bus);
	cpu.set_c(false);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(false);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(true);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_c(true);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn blt() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BLT *+8
	bus.write_program(&mut cpu, 0x6D06);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bmi() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BMI *+8
	bus.write_program(&mut cpu, 0x6B06);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bne() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BNE *+8
	// BNE *+8
	bus.write_program(&mut cpu, 0x6606);
	bus.write_program(&mut cpu, 0x6606);
	cpu.test_reset(&mut bus);
	cpu.set_z(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_z(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bpl() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BPL *+8
	bus.write_program(&mut cpu, 0x6A06);
	cpu.test_reset(&mut bus);
	cpu.set_n(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x208, "Expected PC: 0x00000208, actual value is {:#010X}", cpu.program_counter);
	cpu.test_reset(&mut bus);
	cpu.set_n(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn bvc() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// BVC *+8
	// BVC *+8
	bus.write_program(&mut cpu, 0x6806);
	bus.write_program(&mut cpu, 0x6806);
	cpu.test_reset(&mut bus);
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
	let mut cpu = CPU::new(&mut bus);
	// BVS *+8
	// BVS *+8
	bus.write_program(&mut cpu, 0x6906);
	bus.write_program(&mut cpu, 0x6906);
	cpu.test_reset(&mut bus);
	cpu.set_v(false);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x202, "Expected PC: 0x00000202, actual value is {:#010X}", cpu.program_counter);
	cpu.set_v(true);
	cpu.run_opcode(&mut bus);
	assert!(cpu.program_counter == 0x20A, "Expected PC: 0x0000020A, actual value is {:#010X}", cpu.program_counter);
}

#[test]
fn moveq() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
	// MOVEQ #$2D,D4
	bus.write_program(&mut cpu, 0x789D);
	cpu.test_reset(&mut bus);
	cpu.run_opcode(&mut bus);
	assert!(cpu.d[4] == 0xFFFFFF9D, "Expected D4: 0xFFFFFF9D, actual value is {:#010X}", cpu.d[4]);
}

#[test]
fn divu() {
	panic!("TODO");
}

#[test]
fn or() { panic!("TODO"); }

#[test]
fn sub() { panic!("TODO"); }

#[test]
fn suba() { panic!("TODO"); }

fn eor() {
	panic!("TODO");
}

#[test]
fn cmp() { panic!("TODO") }

#[test]
fn cmpa() { panic!("TODO") }

#[test]
fn mulu() { panic!("TODO"); }

#[test]
fn and() {
	panic!("TODO");
}

#[test]
fn add() { panic!("TODO"); }

fn addx() { panic!("TODO") }

#[test]
fn adda() { panic!("TODO"); }

#[test]
fn lsd_to_d() {
	let mut bus = TestBus::new();
	let mut cpu = CPU::new(&mut bus);
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
	cpu.test_reset(&mut bus);
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

fn roxd_to_d() {
	panic!("TODO");
}

#[test]
fn rod_to_d() {
	panic!("Todo");
}

// Representation of a simple memory map pointing exclusively to RAM
struct TestBus {
	ram: Vec<u8>
}
impl TestBus {
	fn new() -> TestBus {
		let mut ram = vec![0u8; CPU_ADDRESS_SPACE + 1];
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
	fn read_u8(&mut self, address: u32) -> u8 {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		self.ram[address_index]
	}
	fn write_u8(&mut self, address: u32, data: u8) {
		let address_index = (address as usize) & CPU_ADDRESS_SPACE;
		println!("{:#010X}", address_index);
		self.ram[address_index] = data;
	}

	fn assert_interrupt(&mut self, level: u16) {
		// No means of asserting external interrupts on the test bus, so this does nothing.
	}

	fn acknowledge_interrupt(&mut self) -> Option<u16> {
		None
	}

	fn expose_vdp_state(&mut self) -> &mut VDPState {
		panic!("Don't call this.");
	}
}