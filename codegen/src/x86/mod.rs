pub mod address;
pub mod instruction;
pub mod register;

use self::instruction::Instruction;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label<'a>(pub &'a str);

pub struct Reference {
    location: usize,
    format: ReferenceFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceFormat {
    /// A signed 32-bit relative offset from the end of the reference.
    /// Used in near-JMP and branching instructions.
    Rel32,
}

impl ReferenceFormat {
    fn len(&self) -> usize {
        match self {
            Self::Rel32 => 4,
        }
    }
}

pub struct Assembler<'a> {
    code: Vec<u8>,
    labels: HashMap<Label<'a>, usize>,
    references: HashMap<Label<'a>, Vec<Reference>>,
}

impl<'a> Assembler<'a> {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            labels: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn label(&mut self, label: &'a str) {
        assert!(
            self.labels.insert(Label(label), self.code.len()).is_none(),
            "duplicate label {:?}",
            label
        );
    }

    pub fn push<I>(&mut self, instruction: I)
    where
        I: Instruction<'a>,
    {
        let encoded = instruction.encode();
        let location = self.code.len();
        self.code.extend(encoded.serialize());
        for (label, mut reference) in encoded.references() {
            reference.location += location;
            self.references
                .entry(label)
                .or_insert(Vec::new())
                .push(reference);
        }
    }

    pub fn finish(mut self) -> Code<'a> {
        for (label, references) in &self.references {
            let label_location = *self.labels.get(label).expect("undefined label");

            for reference in references {
                match reference.format {
                    ReferenceFormat::Rel32 => {
                        //FIXME This assumes that the rel32 operand is at the
                        // end of the instruction.
                        let relative_to = reference.location + 4;
                        let offset = if label_location > relative_to {
                            i32::try_from(label_location - relative_to).expect("relative overflow")
                        } else {
                            //FIXME This limits the negative range by 1 byte.
                            -i32::try_from(relative_to - label_location).expect("relative overflow")
                        };

                        self.code[reference.location..][..4].copy_from_slice(&offset.to_le_bytes())
                    }
                }
            }
        }
        Code {
            bytes: self.code,
            labels: self.labels,
        }
    }
}

pub struct Code<'a> {
    bytes: Vec<u8>,
    labels: HashMap<Label<'a>, usize>,
}

impl<'a> Code<'a> {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn label(&self, name: &'a str) -> usize {
        self.labels[&Label(name)]
    }
}
