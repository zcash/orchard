//! Gadget and chips for the Sinsemilla hash function.
use crate::circuit::gadget::utilities::{CellValue, Var};
use ff::PrimeFieldBits;
use halo2::{arithmetic::FieldExt, circuit::Cell};
use std::{convert::TryInto, fmt::Debug, ops::Range};

/// A [`Message`] composed of several [`MessagePiece`]s.
#[derive(Clone, Debug)]
pub struct Message<F: FieldExt, const K: usize, const MAX_WORDS: usize>(Vec<MessagePiece<F, K>>);

impl<F: FieldExt + PrimeFieldBits, const K: usize, const MAX_WORDS: usize>
    From<Vec<MessagePiece<F, K>>> for Message<F, K, MAX_WORDS>
{
    fn from(pieces: Vec<MessagePiece<F, K>>) -> Self {
        // A message cannot contain more than `MAX_WORDS` words.
        assert!(pieces.iter().map(|piece| piece.num_words()).sum::<usize>() < MAX_WORDS);
        Message(pieces)
    }
}

impl<F: FieldExt + PrimeFieldBits, const K: usize, const MAX_WORDS: usize> std::ops::Deref
    for Message<F, K, MAX_WORDS>
{
    type Target = [MessagePiece<F, K>];

    fn deref(&self) -> &[MessagePiece<F, K>] {
        &self.0
    }
}

/// A [`MessagePiece`] of some bitlength.
///
/// The piece must fit within a base field element, which means its length
/// cannot exceed the base field's `NUM_BITS`.
#[derive(Clone, Debug)]
pub struct MessagePiece<F: FieldExt, const K: usize> {
    cell_value: CellValue<F>,
    // The number of K-bit words in this message piece.
    num_words: usize,
    // The subpieces in this piece, if any.
    subpieces: Option<Vec<MessageSubPiece<F, K>>>,
}

impl<F: FieldExt + PrimeFieldBits, const K: usize> MessagePiece<F, K> {
    pub fn new(
        cell_value: CellValue<F>,
        num_words: usize,
        subpieces: Option<Vec<MessageSubPiece<F, K>>>,
    ) -> Self {
        assert!(num_words * K < F::NUM_BITS as usize);
        Self {
            cell_value,
            num_words,
            subpieces,
        }
    }

    pub fn num_words(&self) -> usize {
        self.num_words
    }

    // Returns the cell_value of the witnessed MessagePiece.
    pub fn cell_value(&self) -> CellValue<F> {
        self.cell_value
    }

    pub fn cell(&self) -> Cell {
        self.cell_value.cell()
    }

    pub fn value(&self) -> Option<F> {
        self.cell_value.value()
    }

    pub fn subpieces(&self) -> &[MessageSubPiece<F, K>] {
        if let Some(subpieces) = &self.subpieces {
            &subpieces
        } else {
            &[]
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageSubPiece<F: FieldExt, const K: usize> {
    Unwitnessed(Option<F>, Range<usize>),
    Witnessed(CellValue<F>, Range<usize>),
}

impl<F: FieldExt, const K: usize> From<(Option<F>, Range<usize>)> for MessageSubPiece<F, K> {
    fn from(field_elem_bit_range: (Option<F>, Range<usize>)) -> Self {
        Self::Unwitnessed(field_elem_bit_range.0, field_elem_bit_range.1)
    }
}

impl<F: FieldExt, const K: usize> From<(CellValue<F>, Range<usize>)> for MessageSubPiece<F, K> {
    fn from(cell_value_bit_range: (CellValue<F>, Range<usize>)) -> Self {
        Self::Witnessed(cell_value_bit_range.0, cell_value_bit_range.1)
    }
}

impl<F: FieldExt + PrimeFieldBits, const K: usize> MessageSubPiece<F, K> {
    // Returns the original field element before taking a subset of its bitrange.
    pub fn field_elem(&self) -> Option<F> {
        match self {
            Self::Unwitnessed(value, _) => *value,
            _ => unreachable!("This subpiece has already been witnessed."),
        }
    }

    // Returns the bitrange with which to subset the unwitnessed field element.
    pub fn bit_range(&self) -> Range<usize> {
        match self {
            Self::Unwitnessed(_, bitrange) | Self::Witnessed(_, bitrange) => bitrange.clone(),
        }
    }

    // Returns the cell_value of the witnessed MessageSubPiece.
    pub fn cell_value(&self) -> CellValue<F> {
        match self {
            Self::Witnessed(cell_value, _) => *cell_value,
            _ => unreachable!("This subpiece has not yet been witnessed."),
        }
    }

    // Returns the value of the witnessed message subpiece.
    pub fn value(&self) -> Option<F> {
        match self {
            Self::Witnessed(cell_value, _) => cell_value.value(),
            _ => unreachable!("This subpiece has not yet been witnessed."),
        }
    }

    // Returns the cell of the witnessed subpiece.
    pub fn cell(&self) -> Cell {
        match self {
            Self::Witnessed(cell_value, _) => cell_value.cell(),
            _ => unreachable!("This subpiece has not yet been witnessed."),
        }
    }

    // Returns the bitrange subset of the original field element.
    pub fn bitrange_subset(&self) -> Option<Vec<bool>> {
        let (value, bit_range) = match self {
            Self::Unwitnessed(_, _) => (self.field_elem(), self.bit_range()),
            Self::Witnessed(_, _) => (self.value(), 0..self.bit_range().len()),
        };

        if let Some(value) = value {
            let bits = &value
                .to_le_bits()
                .iter()
                .by_val()
                .take(F::NUM_BITS as usize)
                .collect::<Vec<_>>()[bit_range];
            Some(bits.to_vec())
        } else {
            None
        }
    }

    // Returns the field element representation of bitrange_subset.
    pub fn field_elem_subset(&self) -> Option<F> {
        self.bitrange_subset().map(bits_to_field_elem)
    }

    pub fn assemble_field_elem(subpieces: &[Self]) -> Option<F> {
        let mut bits = Vec::new();
        for subpiece in subpieces.iter() {
            if let Some(subpiece_bits) = subpiece.bitrange_subset() {
                bits.extend_from_slice(&subpiece_bits);
            } else {
                return None;
            }
        }

        Some(bits_to_field_elem(bits))
    }
}

fn bits_to_field_elem<F: FieldExt>(bits: Vec<bool>) -> F {
    assert!(bits.len() <= 256);
    // Pad to 256 bits
    let pad_len = 256 - bits.len();
    let bits: Vec<bool> = bits
        .iter()
        .cloned()
        .chain(std::iter::repeat(false).take(pad_len))
        .collect();

    let bytearray: Vec<u8> = bits
        .chunks_exact(8)
        .map(|byte| byte.iter().rev().fold(0u8, |acc, bit| acc * 2 + *bit as u8))
        .collect();

    F::from_bytes(&bytearray.try_into().unwrap()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::MessageSubPiece;
    use crate::primitives::sinsemilla::K;
    use ff::{PrimeField, PrimeFieldBits};
    use pasta_curves::{arithmetic::FieldExt, pallas};

    #[test]
    fn bits_to_field_elem() {
        let field_elems = [
            pallas::Base::zero(),
            -pallas::Base::one(),
            pallas::Base::rand(),
        ];
        for field_elem in field_elems.iter() {
            let bits = field_elem
                .to_le_bits()
                .iter()
                .by_val()
                .take(pallas::Base::NUM_BITS as usize)
                .collect();

            let computed_field_elem = super::bits_to_field_elem(bits);
            assert_eq!(*field_elem, computed_field_elem)
        }
    }

    #[test]
    fn message_subpiece_api() {
        let real_subpiece_1 = pallas::Base::from_u64(1 << 14); // 15-bit value
        let real_subpiece_2 = pallas::Base::from_u128(1 << 117).square(); // 235-bit value
        let real_field_elem = real_subpiece_1 + real_subpiece_2 * pallas::Base::from_u64(1 << 15);

        let subpiece_1: MessageSubPiece<pallas::Base, K> = (Some(real_field_elem), 0..15).into();
        let subpiece_2: MessageSubPiece<pallas::Base, K> = (Some(real_field_elem), 15..250).into();

        {
            let piece_value =
                MessageSubPiece::assemble_field_elem(&[subpiece_1.clone(), subpiece_2.clone()])
                    .unwrap();
            assert_eq!(piece_value, real_field_elem);
        }

        {
            let subpiece_1 = subpiece_1.field_elem_subset().unwrap();
            let subpiece_2 = subpiece_2.field_elem_subset().unwrap();

            assert_eq!(subpiece_1, real_subpiece_1);
            assert_eq!(subpiece_2, real_subpiece_2);
        }

        {
            let bitrange = {
                let subpiece_1 = subpiece_1.bitrange_subset().unwrap();
                let subpiece_2 = subpiece_2.bitrange_subset().unwrap();
                let mut bitrange = subpiece_1;
                bitrange.extend_from_slice(&subpiece_2);
                bitrange
            };
            let expected_bitrange = real_field_elem
                .to_le_bits()
                .iter()
                .by_val()
                .take(250)
                .collect::<Vec<_>>();
            assert_eq!(bitrange, expected_bitrange);
        }
    }
}
