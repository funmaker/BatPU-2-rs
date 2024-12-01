mod macros;

macros::isa! {
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
		CMP(a, b) => SUB(a, b, r0),
		MOV(a, c) => ADD(a, r0, c),
		LSH(a, c) => ADD(a, a, c),
		INC(a)    => ADI(a, 1),
		DEC(a)    => ADI(a, -1),
		NOT(a, c) => NOR(a, r0, c),
		NEG(a, c) => SUB(r0, a, c),
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::asm::ast;
	
	#[test]
	fn it_works() {
		let inst = Instruction::try_from(&ast::ResolvedInstruction { mnemonic: "LDI".to_string(), operands: vec![8, 64], }).unwrap();
		
		println!("{inst:?}");
	}
}
