#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TargetLayout {
    pub(crate) pointer_size: usize,
    pub(crate) pointer_alignment: usize,
    pub(crate) index_size: usize,
    pub(crate) integer_alignments: [usize; 4],
    pub(crate) float_alignments: [usize; 2],
    pub(crate) supports_pointer_alignment: bool,
}

impl TargetLayout {
    pub(crate) fn natural(pointer_size: usize, index_size: usize) -> Option<Self> {
        (pointer_size.is_power_of_two() && index_size.is_power_of_two()).then_some(Self {
            pointer_size,
            pointer_alignment: pointer_size,
            index_size,
            integer_alignments: [1, 2, 4, 8],
            float_alignments: [4, 8],
            supports_pointer_alignment: true,
        })
    }

    pub(crate) fn scalar_alignment(self, bits: u8, floating: bool) -> Option<usize> {
        if floating {
            return match bits {
                32 => Some(self.float_alignments[0]),
                64 => Some(self.float_alignments[1]),
                _ => None,
            };
        }
        match bits {
            8 => Some(self.integer_alignments[0]),
            16 => Some(self.integer_alignments[1]),
            32 => Some(self.integer_alignments[2]),
            64 => Some(self.integer_alignments[3]),
            _ => None,
        }
    }
}

pub(crate) fn parse(data_layout: &str) -> Option<TargetLayout> {
    let pointer = data_layout.split('-').find_map(|component| {
        component
            .strip_prefix("p:")
            .or_else(|| component.strip_prefix("p0:"))
    });
    let (pointer_bits, pointer_alignment_bits, index_bits) = if let Some(pointer) = pointer {
        let fields = pointer.split(':').collect::<Vec<_>>();
        let pointer_bits = fields.first()?.parse::<usize>().ok()?;
        let pointer_alignment_bits = fields.get(1)?.parse::<usize>().ok()?;
        let index_bits = fields
            .get(3)
            .map_or(Some(pointer_bits), |bits| bits.parse().ok())?;
        (pointer_bits, pointer_alignment_bits, index_bits)
    } else {
        (64, 64, 64)
    };
    let byte_alignment = |bits: usize| {
        bits.is_multiple_of(8)
            .then_some(bits / 8)
            .filter(|bytes| bytes.is_power_of_two())
    };
    let supported_size =
        |bits: usize| byte_alignment(bits).filter(|bytes| matches!(bytes, 1 | 2 | 4 | 8));
    let mut layout = TargetLayout {
        pointer_size: supported_size(pointer_bits)?,
        pointer_alignment: byte_alignment(pointer_alignment_bits)?,
        index_size: supported_size(index_bits)?,
        integer_alignments: [1, 2, 4, 8],
        float_alignments: [4, 8],
        supports_pointer_alignment: !data_layout.split('-').any(|component| {
            component
                .strip_prefix("ni:")
                .is_some_and(|spaces| spaces.split(':').any(|space| space == "0"))
        }),
    };
    for component in data_layout.split('-') {
        let (floating, fields) = if let Some(fields) = component.strip_prefix('i') {
            (false, fields)
        } else if let Some(fields) = component.strip_prefix('f') {
            (true, fields)
        } else {
            continue;
        };
        let mut fields = fields.split(':');
        let bits = fields.next()?.parse::<u8>().ok()?;
        let alignment = byte_alignment(fields.next()?.parse::<usize>().ok()?)?;
        match (floating, bits) {
            (false, 8) => layout.integer_alignments[0] = alignment,
            (false, 16) => layout.integer_alignments[1] = alignment,
            (false, 32) => layout.integer_alignments[2] = alignment,
            (false, 64) => layout.integer_alignments[3] = alignment,
            (true, 32) => layout.float_alignments[0] = alignment,
            (true, 64) => layout.float_alignments[1] = alignment,
            _ => {}
        }
    }
    Some(layout)
}
