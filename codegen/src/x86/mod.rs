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

    pub fn finish(mut self) -> Segment<'a> {
        // Handle instructions with undefined and absolute references;
        // fall back to largest encoding.
        // FIXME this assumes that last listed encoding is largest
        // FIXME this assumes that all encoding options have same references.
        self.pending_instructions.retain(|pending| {
            let encoding = pending.options.last().unwrap();
            if encoding.references().into_iter().any(|(label, reference)| {
                self.segment.label_location(label.0).is_none() || !reference.format.is_relative()
            }) {
                let dest = &mut self.segment.data_mut()[pending.location..][..pending.size];
                dest.fill(0x90); // Fill with NOP just in case encoding is short

                let encoded: Vec<u8> = encoding.serialize().into_iter().collect();
                dest[..encoded.len()].copy_from_slice(&encoded);

                for (label, reference) in encoding.references() {
                    self.segment.absolute_reference(
                        pending.location + reference.location,
                        label.0,
                        reference.format,
                    );
                }

                false // Remove from pending_instructions
            } else {
                true // Retain in pending_instructions
            }
        });

        'pending: for pending in &self.pending_instructions {
            'options: for encoding in &pending.options {
                let mut encoded: Vec<u8> = encoding.serialize().into_iter().collect();

                for (label, reference) in encoding.references() {
                    let location = pending.location + reference.location;
                    let label_location = self.segment.label_location(label.0).unwrap();

                    if let Some(bytes) = reference
                        .format
                        .resolve(location as u64, label_location as u64)
                    {
                        encoded[reference.location..][..bytes.len()].copy_from_slice(&bytes);
                    } else {
                        continue 'options;
                    }
                }

                let dest = &mut self.segment.data_mut()[pending.location..][..pending.size];
                //TODO actually reduce size allocated for instruction
                dest.fill(0x90); // NOP
                dest[..encoded.len()].copy_from_slice(&encoded);
                continue 'pending;
            }
            panic!("no viable encoding options");
        }

        self.segment
    }
}
