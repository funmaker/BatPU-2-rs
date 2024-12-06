use std::error::Error as StdError;

use crate::isa::Instruction;

pub trait Code {
	type Error: StdError + 'static;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error>;
	fn len(&self) -> usize;
}

impl<T: Into<Instruction> + Copy> Code for [T] {
	type Error = !;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		Ok(self.get(pc as usize)
		       .copied()
		       .map(Into::into))
	}
	
	fn len(&self) -> usize {
		self.len()
	}
}

impl<T: Into<Instruction> + Copy, const N: usize> Code for [T; N] {
	type Error = !;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(*self.as_slice()).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(*self.as_slice()).len()
	}
}

impl<T: Into<Instruction> + Copy> Code for Vec<T> {
	type Error = !;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(*self.as_slice()).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(*self.as_slice()).len()
	}
}

impl<T: Code + ?Sized> Code for &T {
	type Error = T::Error;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(*self).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(*self).len()
	}
}

impl<T: Code + ?Sized> Code for Box<T> {
	type Error = T::Error;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(**self).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(**self).len()
	}
}

impl<T: Code + ?Sized> Code for std::rc::Rc<T> {
	type Error = T::Error;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(**self).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(**self).len()
	}
}

impl<T: Code + ?Sized> Code for std::sync::Arc<T> {
	type Error = T::Error;
	
	fn instruction(&self, pc: u16) -> Result<Option<Instruction>, Self::Error> {
		(**self).instruction(pc)
	}
	
	fn len(&self) -> usize {
		(**self).len()
	}
}