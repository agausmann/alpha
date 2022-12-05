pub mod address;
pub mod instruction;
pub mod register;

use self::instruction::{InstructionBuilder, InstructionOptions};
use crate::link::Segment;

struct PendingInstruction<'a> {
    location: usize,
    size: usize,
    options: Vec<InstructionBuilder<'a>>,
}

pub struct Assembler<'a> {
    segment: Segment<'a>,
    pending_instructions: Vec<PendingInstruction<'a>>,
}

impl<'a> Assembler<'a> {
    pub fn new() -> Self {
        Self {
            segment: Segment::new(),
            pending_instructions: Vec::new(),
        }
    }

    pub fn label(&mut self, label: &'a str) {
        self.segment.label(label);
    }

    pub fn push<I>(&mut self, instruction: I)
    where
        I: InstructionOptions<'a>,
    {
        let options = instruction.options();
        if options.len() == 1 {
            // Trivial case
            let encoded = &options[0];
            for (label, reference) in encoded.references() {
                self.segment
                    .offset_reference(reference.location, label.0, reference.format);
            }
            self.segment.extend(encoded.serialize());
        } else {
            let worst_case_size = options
                .iter()
                .map(|encoding| encoding.serialize().into_iter().count())
                .max()
                .expect("no encoding options provided");
            let location = self.segment.position();
            self.segment
                .extend(std::iter::repeat(0u8).take(worst_case_size));
            self.pending_instructions.push(PendingInstruction {
                location,
                size: worst_case_size,
                options,
            });
        }
    }

    pub fn finish(self) -> Segment<'a> {
        self.segment
    }
}
