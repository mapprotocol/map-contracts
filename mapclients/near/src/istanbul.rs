use crate::traits::FromBytes;
use crate::types::errors::Kind;
use crate::types::header::Header;
use crate::types::istanbul::{IstanbulAggregatedSeal, IstanbulExtra, IstanbulExtraVanity};

// Retrieves the block number within an epoch. The return value will be 1-based.
// There is a special case if the number == 0. It is basically the last block of the 0th epoch,
// and should have a value of epoch_size
pub fn get_number_within_epoch(number: u64, epoch_size: u64) -> u64 {
    let number = number % epoch_size;
    if number == 0 {
        epoch_size
    } else {
        number
    }
}

pub fn get_epoch_number(number: u64, epoch_size: u64) -> u64 {
    let epoch_number = number / epoch_size;

    if is_last_block_of_epoch(number, epoch_size) {
        epoch_number
    } else {
        epoch_number + 1
    }
}

pub fn get_epoch_first_block_number(epoch_number: u64, epoch_size: u64) -> Option<u64> {
    if epoch_number == 0 {
        // no first block for epoch 0
        return None;
    }

    Some(((epoch_number - 1) * epoch_size) + 1)
}

pub fn get_epoch_last_block_number(epoch_number: u64, epoch_size: u64) -> u64 {
    if epoch_number == 0 {
        return 0;
    }

    // Epoch 0 is just the genesis bock, so epoch 1 starts at block 1 and ends at block epochSize
    // And from then on, it's epochSize more for each epoch
    epoch_number * epoch_size
}

pub fn istanbul_filtered_header(header: &Header, keep_seal: bool) -> Result<Header, Kind> {
    let mut new_header = header.clone();

    let mut extra = IstanbulExtra::from_rlp(&new_header.extra)?;
    if !keep_seal {
        extra.seal = Vec::new();
    }
    extra.aggregated_seal = IstanbulAggregatedSeal::new();

    let payload = extra.to_rlp(IstanbulExtraVanity::from_bytes(&new_header.extra)?);
    new_header.extra = payload;

    Ok(new_header)
}

pub fn is_last_block_of_epoch(number: u64, epoch_size: u64) -> bool {
    get_number_within_epoch(number, epoch_size) == epoch_size
}

pub fn min_quorum_size(total_validators: usize) -> usize {
    // non-float equivalent of:
    //  ((2.0*(total_validators as f64) / 3.0) as f64).ceil() as usize
    ((2 * total_validators) - 1 + 3) / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_quorum_size_math() {
        for (validator_set_size, expected_min_quorum_size) in vec![
            (1 as usize, 1 as usize),
            (2, 2),
            (3, 2),
            (4, 3),
            (5, 4),
            (6, 4),
            (7, 5),
        ]
        .iter()
        {
            assert_eq!(
                min_quorum_size(*validator_set_size),
                *expected_min_quorum_size
            );
        }
    }

    #[test]
    fn validates_epoch_math() {
        assert_eq!(
            vec![
                get_epoch_number(0, 3),
                get_epoch_number(3, 3),
                get_epoch_number(4, 3)
            ],
            vec![0, 1, 2]
        );

        assert_eq!(
            vec![
                get_epoch_first_block_number(0, 3),
                get_epoch_first_block_number(9, 3)
            ],
            vec![None, Some(25)]
        );

        assert_eq!(
            vec![
                get_epoch_last_block_number(0, 3),
                get_epoch_last_block_number(9, 3)
            ],
            vec![0, 27]
        );
    }
}
