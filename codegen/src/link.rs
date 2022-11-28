use crate::elf64::common::{Word, Xword};
use bytemuck::Pod;
use std::{collections::HashMap, io::Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label<'a>(pub &'a str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ptr<'a>(pub &'a str);

pub struct Reference {
    pub location: usize,
    pub format: ReferenceFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceFormat {
    /// A signed 32-bit relative offset from the end of the reference.
    /// Used in near-JMP and branching instructions.
    Rel32,

    /// An absolute 64-bit address.
    Abs64,
}

impl ReferenceFormat {
    pub fn len(&self) -> usize {
        match self {
            Self::Rel32 => 4,
            Self::Abs64 => 8,
        }
    }
}

pub struct Segment<'a> {
    data: Vec<u8>,
    labels: HashMap<Label<'a>, usize>,
    references: HashMap<Label<'a>, Vec<Reference>>,
}

impl<'a> Segment<'a> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            labels: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn align(&mut self, alignment: usize) {
        todo!()
    }

    pub fn label(&mut self, label: &'a str) {
        self.offset_label(0, label);
    }

    pub fn offset_label(&mut self, offset: usize, label: &'a str) {
        assert!(
            self.labels.insert(Label(label), self.data.len()).is_none(),
            "duplicate label {:?}",
            label
        );
    }

    pub fn append<T: Pod>(&mut self, val: &T) {
        // TODO build our own "POD" abstraction; bytemuck isn't useful if
        // we're compiling for a machine with different endianness
        self.extend(bytemuck::bytes_of(val).iter().copied());
    }

    pub fn append_reference(&mut self, label: &'a str, format: ReferenceFormat) {
        todo!()
    }

    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.data.extend(bytes);
    }

    pub fn reference(&mut self, label: &'a str, format: ReferenceFormat) {
        self.offset_reference(0, label, format);
    }

    pub fn offset_reference(&mut self, offset: usize, label: &'a str, format: ReferenceFormat) {
        self.references
            .entry(Label(label))
            .or_insert(Vec::new())
            .push(Reference {
                location: self.data.len() + offset,
                format,
            });
    }
}

pub struct ElfLinker<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> ElfLinker<'a> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, flags: Word, align: Xword, segment: Segment<'a>) {
        self.segments.push(segment);
        todo!()
    }

    pub fn finish(mut self) -> Linked {
        // TODO adapt to cross-segment linking
        /*
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

                    ReferenceFormat::Abs64 => {
                        self.code[reference.location..][..8]
                            .copy_from_slice(&u64::try_from(label_location).unwrap().to_le_bytes());
                    }
                }
            }
        }
        */
        todo!()
    }
}

pub struct Linked {}

impl Linked {
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        todo!()
    }
}
