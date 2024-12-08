mod macros;
mod common;

pub use common::*;

macros::isa! {
	pub word = u16;
	pub operand = i16;
	
	pub operands {
		opcode: u8 = unsigned 0b_1111_0000_0000_0000,
		a: u8      = unsigned 0b_0000_1111_0000_0000,
		b: u8      = unsigned 0b_0000_0000_1111_0000,
		c: u8      = unsigned 0b_0000_0000_0000_1111,
		imm: u8    =      any 0b_0000_0000_1111_1111,
		addr: u16  = unsigned 0b_0000_0011_1111_1111,
		cond: Cond = unsigned 0b_0000_1100_0000_0000,
		offset: i8 =   signed 0b_0000_0000_0000_1111,
	}
	
	pub instructions {
		NOP               = 0x0,
		HLT               = 0x1,
		ADD(a, b, c)      = 0x2,
		SUB(a, b, c)      = 0x3,
		NOR(a, b, c)      = 0x4,
		AND(a, b, c)      = 0x5,
		XOR(a, b, c)      = 0x6,
		RSH(a,    c)      = 0x7,
		LDI(a,  imm)      = 0x8,
		ADI(a,  imm)      = 0x9,
		JMP(      addr)   = 0xA,
		BRH(cond, addr)   = 0xB,
		CAL(      addr)   = 0xC,
		RET               = 0xD,
		LOD(a, b, offset) = 0xE,
		STR(a, b, offset) = 0xF,
	}
	
	pub aliases {
		CMP(a, b) => SUB(a, b, 0),
		MOV(a, c) => ADD(a, 0, c),
		LSH(a, c) => ADD(a, a, c),
		INC(a)    => ADI(a, 1),
		DEC(a)    => ADI(a, 0xFF),
		NOT(a, c) => NOR(a, 0, c),
		NEG(a, c) => SUB(0, a, c),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn to_machine_code() {
		assert_eq!(Instruction::NOP                                     .as_word(), 0x0000, "NOP");
		assert_eq!(Instruction::HLT                                     .as_word(), 0x1000, "HLT");
		assert_eq!(Instruction::ADD{ a: 0x3,     b: 0x4,      c: 0x5   }.as_word(), 0x2345, "ADD");
		assert_eq!(Instruction::SUB{ a: 0x9,     b: 0x3,      c: 0x9   }.as_word(), 0x3939, "SUB");
		assert_eq!(Instruction::NOR{ a: 0xE,     b: 0xE,      c: 0xF   }.as_word(), 0x4EEF, "NOR");
		assert_eq!(Instruction::AND{ a: 0x4,     b: 0x0,      c: 0x5   }.as_word(), 0x5405, "AND");
		assert_eq!(Instruction::XOR{ a: 0x1,     b: 0x2,      c: 0x3   }.as_word(), 0x6123, "XOR");
		assert_eq!(Instruction::RSH{ a: 0xA,                  c: 0xB   }.as_word(), 0x7A0B, "RSH");
		assert_eq!(Instruction::LDI{ a: 0x9,                imm: 0xFF  }.as_word(), 0x89FF, "LDI");
		assert_eq!(Instruction::ADI{ a: 0x8,                imm: 0x42  }.as_word(), 0x9842, "ADI");
		assert_eq!(Instruction::JMP{                       addr: 0x3FF }.as_word(), 0xA3FF, "JMP");
		assert_eq!(Instruction::BRH{ cond: Cond::Zero,     addr: 0x222 }.as_word(), 0xB222, "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::NotZero,  addr: 0x123 }.as_word(), 0xB523, "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::Carry,    addr: 0x009 }.as_word(), 0xB809, "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::NotCarry, addr: 0x3FF }.as_word(), 0xBFFF, "BRH");
		assert_eq!(Instruction::CAL{                       addr: 0x137 }.as_word(), 0xC137, "CAL");
		assert_eq!(Instruction::RET                                     .as_word(), 0xD000, "RET");
		assert_eq!(Instruction::LOD{ a: 0x1,     b: 0x2, offset: 0x3   }.as_word(), 0xE123, "LOD");
		assert_eq!(Instruction::STR{ a: 0xF,     b: 0xF, offset:-0x1   }.as_word(), 0xFFFF, "STR");
	}
	
	#[test]
	fn from_machine_code() {
		assert_eq!(Instruction::NOP,                                      Instruction::from(0x0000), "NOP");
		assert_eq!(Instruction::HLT,                                      Instruction::from(0x1000), "HLT");
		assert_eq!(Instruction::ADD{ a: 0x3,     b: 0x4,      c: 0x5   }, Instruction::from(0x2345), "ADD");
		assert_eq!(Instruction::SUB{ a: 0x9,     b: 0x3,      c: 0x9   }, Instruction::from(0x3939), "SUB");
		assert_eq!(Instruction::NOR{ a: 0xE,     b: 0xE,      c: 0xF   }, Instruction::from(0x4EEF), "NOR");
		assert_eq!(Instruction::AND{ a: 0x4,     b: 0x0,      c: 0x5   }, Instruction::from(0x5405), "AND");
		assert_eq!(Instruction::XOR{ a: 0x1,     b: 0x2,      c: 0x3   }, Instruction::from(0x6123), "XOR");
		assert_eq!(Instruction::RSH{ a: 0xA,                  c: 0xB   }, Instruction::from(0x7A0B), "RSH");
		assert_eq!(Instruction::LDI{ a: 0x9,                imm: 0xFF  }, Instruction::from(0x89FF), "LDI");
		assert_eq!(Instruction::ADI{ a: 0x8,                imm: 0x42  }, Instruction::from(0x9842), "ADI");
		assert_eq!(Instruction::JMP{                       addr: 0x3FF }, Instruction::from(0xA3FF), "JMP");
		assert_eq!(Instruction::BRH{ cond: Cond::Zero,     addr: 0x222 }, Instruction::from(0xB222), "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::NotZero,  addr: 0x123 }, Instruction::from(0xB523), "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::Carry,    addr: 0x009 }, Instruction::from(0xB809), "BRH");
		assert_eq!(Instruction::BRH{ cond: Cond::NotCarry, addr: 0x3FF }, Instruction::from(0xBFFF), "BRH");
		assert_eq!(Instruction::CAL{                       addr: 0x137 }, Instruction::from(0xC137), "CAL");
		assert_eq!(Instruction::RET,                                      Instruction::from(0xD000), "RET");
		assert_eq!(Instruction::LOD{ a: 0x1,     b: 0x2, offset: 0x3   }, Instruction::from(0xE123), "LOD");
		assert_eq!(Instruction::STR{ a: 0xF,     b: 0xF, offset:-0x1   }, Instruction::from(0xFFFF), "STR");
	}
}
