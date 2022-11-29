pub mod address;
pub mod instruction;
pub mod register;

use self::instruction::Instruction;
use crate::link::Segment;

pub struct Assembler<'a> {
    segment: Segment<'a>,
}

impl<'a> Assembler<'a> {
    pub fn new() -> Self {
        Self {
            segment: Segment::new(),
        }
    }

    pub fn label(&mut self, label: &'a str) {
        self.segment.label(label);
    }

    pub fn push<I>(&mut self, instruction: I)
    where
        I: Instruction<'a>,
    {
        let encoded = instruction.encode();
        for (label, reference) in encoded.references() {
            self.segment
                .offset_reference(reference.location, label.0, reference.format);
        }
        self.segment.extend(encoded.serialize());
    }

    pub fn finish(self) -> Segment<'a> {
        self.segment
    }
}
