use crate::{
    elf64::{
        common::{Word, Xword},
        file_header::{FileHeader, FILE_HEADER_SIZE},
        program::{Phdr, PROGRAM_HEADER_SIZE, PT_LOAD},
    },
    math::align_up,
};
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

impl Reference {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceFormat {
    /// A signed 8-bit relative offset from the end of the reference.
    /// Used in short-JMP and branching instructions.
    Rel8,

    /// A signed 32-bit relative offset from the end of the reference.
    /// Used in near-JMP and branching instructions.
    Rel32,

    /// An absolute 32-bit address, sign-extended to 64-bit.
    /// Used to compress 64-bit address values that are near the top or
    /// bottom of the 64-bit space (i.e. when the top 33 bits are equal)
    Abs32,

    /// An absolute 64-bit address.
    Abs64,
}

impl ReferenceFormat {
    pub fn len(&self) -> usize {
        match self {
            Self::Rel8 => 1,
            Self::Rel32 => 4,
            Self::Abs32 => 4,
            Self::Abs64 => 8,
        }
    }

    pub fn is_relative(&self) -> bool {
        match self {
            Self::Rel8 | Self::Rel32 => true,
            Self::Abs32 | Self::Abs64 => false,
        }
    }
    pub fn resolve(&self, own_location: u64, label_location: u64) -> Option<Vec<u8>> {
        match self {
            Self::Rel8 => {
                //FIXME This assumes that the rel8 operand is at the
                // end of the instruction.
                let relative_to = own_location + 1;
                let offset = if label_location > relative_to {
                    i8::try_from(label_location - relative_to).ok()?
                } else {
                    //FIXME This limits the negative range by 1 byte.
                    -i8::try_from(relative_to - label_location).ok()?
                };

                Some(offset.to_le_bytes().to_vec())
            }
            Self::Rel32 => {
                //FIXME This assumes that the rel32 operand is at the
                // end of the instruction.
                let relative_to = own_location + 4;
                let offset = if label_location > relative_to {
                    i32::try_from(label_location - relative_to).ok()?
                } else {
                    //FIXME This limits the negative range by 1 byte.
                    -i32::try_from(relative_to - label_location).ok()?
                };

                Some(offset.to_le_bytes().to_vec())
            }
            Self::Abs32 => {
                let location = u64::try_from(label_location).unwrap() as i64;
                let truncated_location = i32::try_from(location).ok()?;
                Some(truncated_location.to_le_bytes().to_vec())
            }
            Self::Abs64 => Some(
                u64::try_from(label_location)
                    .unwrap()
                    .to_le_bytes()
                    .to_vec(),
            ),
        }
    }
}

pub struct Segment<'a> {
    alignment: usize,
    data: Vec<u8>,
    labels: HashMap<Label<'a>, usize>,
    references: HashMap<Label<'a>, Vec<Reference>>,
}

impl<'a> Segment<'a> {
    pub fn new() -> Self {
        Self {
            alignment: 1,
            data: Vec::new(),
            labels: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn position(&self) -> usize {
        self.data.len()
    }

    pub fn align(&mut self, alignment: usize) {
        assert!(alignment.is_power_of_two());
        self.alignment = self.alignment.max(alignment);
    }

    pub fn label(&mut self, label: &'a str) {
        self.offset_label(0, label);
    }

    pub fn offset_label(&mut self, offset: usize, label: &'a str) {
        let unique = self
            .labels
            .insert(Label(label), self.data.len() + offset)
            .is_none();
        assert!(unique, "duplicate label {:?}", label);
    }

    pub fn append<T: Pod>(&mut self, val: &T) {
        // TODO build our own "POD" abstraction; bytemuck isn't useful if
        // we're compiling for a machine with different endianness
        self.extend(bytemuck::bytes_of(val).iter().copied());
    }

    pub fn append_reference(&mut self, label: &'a str, format: ReferenceFormat) {
        self.reference(label, format);
        self.data.extend(std::iter::repeat(0u8).take(format.len()));
    }

    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.data.extend(bytes);
    }

    pub fn reference(&mut self, label: &'a str, format: ReferenceFormat) {
        self.offset_reference(0, label, format);
    }

    pub fn offset_reference(&mut self, offset: usize, label: &'a str, format: ReferenceFormat) {
        self.absolute_reference(self.data.len() + offset, label, format)
    }

    pub fn absolute_reference(&mut self, location: usize, label: &'a str, format: ReferenceFormat) {
        self.references
            .entry(Label(label))
            .or_insert(Vec::new())
            .push(Reference { location, format });
    }

    pub fn label_location(&self, label: &'a str) -> Option<usize> {
        self.labels.get(&Label(label)).copied()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

pub struct ElfLinker<'a> {
    segment_headers: Vec<Phdr>,
    segments: Vec<Segment<'a>>,
}

impl<'a> ElfLinker<'a> {
    pub fn new() -> Self {
        Self {
            segment_headers: Vec::new(),
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, flags: Word, align: Xword, segment: Segment<'a>) {
        let program_header = Phdr {
            p_type: PT_LOAD,
            p_flags: flags,
            p_offset: 0, // Resolved in `finish()`
            p_vaddr: 0,  // Resolved in `finish()`
            p_paddr: 0,  //TODO
            p_filesz: segment.data.len() as u64,
            p_memsz: segment.data.len() as u64,
            p_align: align,
        };

        self.segment_headers.push(program_header);
        self.segments.push(segment);
    }

    pub fn finish(mut self) -> Linked {
        let program_header_offset = FILE_HEADER_SIZE as u64;
        let program_header_end =
            program_header_offset + self.segment_headers.len() as u64 * PROGRAM_HEADER_SIZE as u64;

        let start_vaddr = 0xffffffff_80000000_u64; // TODO parameter

        let mut current_file_offset = align_up(program_header_end, self.segment_headers[0].p_align);
        let mut current_vaddr = align_up(start_vaddr, self.segment_headers[0].p_align);

        let data_padding = current_file_offset - program_header_end;

        let mut labels = HashMap::new();

        for (header, segment) in self.segment_headers.iter_mut().zip(&self.segments) {
            // 1. Resolve file offsets and virtual addresses for this segment

            // If boundary between segments doesn't lie on a page boundary,
            // ensure the next segment is on a new page.
            // (FIXME - page size not same as alignment in some cases?)
            if (current_vaddr % header.p_align) != 0 {
                current_vaddr += header.p_align;
            }

            header.p_offset = current_file_offset;
            header.p_vaddr = current_vaddr;
            header.p_paddr = current_vaddr;

            current_file_offset += segment.data.len() as u64;
            current_vaddr += segment.data.len() as u64;

            // 2. Resolve labels in this segment to their absolute virtual addresses.
            for (&label, &label_offset) in &segment.labels {
                let previous_entry = labels.insert(label, header.p_vaddr + label_offset as u64);
                assert!(
                    previous_entry.is_none(),
                    "duplicate label definition across segments: {:?}",
                    label
                );
            }
        }

        // Resolve references in all segments
        for (header, segment) in self.segment_headers.iter().zip(&mut self.segments) {
            for (label, references) in &segment.references {
                let label_location = *labels.get(label).expect("undefined label");

                for reference in references {
                    let reference_location = header.p_vaddr + reference.location as u64;
                    let reference_value = reference.format.resolve(reference_location, label_location)
                        .ok_or_else(|| format!("relative overflow label={label:?} label_location={label_location:x} reference_location={reference_location:x}")).unwrap();
                    segment.data[reference.location..][..reference_value.len()]
                        .copy_from_slice(&reference_value);
                }
            }
        }

        let mut file_header = FileHeader::new();
        file_header.e_machine = 0x3e; // x86_64
        file_header.e_entry = labels[&Label("entry")];
        file_header.e_phnum = self
            .segment_headers
            .len()
            .try_into()
            .expect("segment table overflow");
        file_header.e_phoff = program_header_offset;

        let mut linked_bytes = Vec::new();
        linked_bytes.extend(bytemuck::bytes_of(&file_header));
        for header in &self.segment_headers {
            linked_bytes.extend(bytemuck::bytes_of(header));
        }
        linked_bytes.extend(std::iter::repeat(0u8).take(data_padding as usize));
        for segment in &self.segments {
            linked_bytes.extend(&segment.data);
        }

        Linked {
            bytes: linked_bytes,
        }
    }
}

pub struct Linked {
    bytes: Vec<u8>,
}

impl Linked {
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.bytes)
    }
}
