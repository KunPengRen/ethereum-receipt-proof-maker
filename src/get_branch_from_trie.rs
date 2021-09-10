use crate::errors::AppError;
use crate::nibble_utils::get_nibbles_from_bytes;
use crate::nibble_utils::Nibbles;
use crate::rlp_codec::rlp_encode_transaction_index;
use crate::state::State;
use crate::trie::Trie;
use crate::types::{NodeStack, Result};
use crate::utils::convert_hex_to_u256;
use hex;

fn convert_usize_index_to_trie_key(index: usize) -> Result<Nibbles> {
    convert_hex_to_u256(hex::encode(index.to_be_bytes()))
        .and_then(|u256| rlp_encode_transaction_index(&u256))
        .map(get_nibbles_from_bytes)
}

pub fn get_branch_from_trie(receipts_trie: Trie, index: usize) -> Result<NodeStack> {
    receipts_trie
        .find(convert_usize_index_to_trie_key(index)?)
        .and_then(
            |(_, _, found_stack, remaining_key)| match remaining_key.len() {
                0 => Ok(found_stack),
                _ => Err(AppError::Custom(format!(
                    "✘ Error! No receipt in trie at given index: {}",
                    index
                ))),
            },
        )
}

pub fn get_branch_from_trie_and_put_in_state(state: State) -> Result<State> {
    info!("✔ Pulling branch from trie...");
    get_branch_from_trie(
        state.get_receipts_trie_from_state()?.clone(),
        state.get_index_from_state()?.clone(),
    )
    .and_then(|branch| state.set_branch_in_state(branch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        get_sample_trie_with_sample_receipts, get_sample_tx_hashes_1,
        get_valid_state_with_receipts_trie_and_index, SAMPLE_RECEIPT_JSONS_1_PATH,
    };

    #[test]
    fn should_convert_usize_to_trie_key() {
        let index = 10;
        let expected_result = Nibbles {
            data: vec![0x0a],
            offset: 0,
        };
        let result = convert_usize_index_to_trie_key(index).unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_get_branch_from_trie() {
        let index = 14;
        let trie = get_sample_trie_with_sample_receipts(
            SAMPLE_RECEIPT_JSONS_1_PATH.to_string(),
            get_sample_tx_hashes_1(),
        );
        get_branch_from_trie(trie, index).unwrap();
    }

    #[test]
    fn should_fail_to_get_non_existent_branch_from_trie_correctly() {
        let non_existent_index = get_sample_tx_hashes_1().len() + 1;
        let expected_error = format!(
            "✘ Error! No receipt in trie at given index: {}",
            non_existent_index
        );
        let trie = get_sample_trie_with_sample_receipts(
            SAMPLE_RECEIPT_JSONS_1_PATH.to_string(),
            get_sample_tx_hashes_1(),
        );
        match get_branch_from_trie(trie, non_existent_index) {
            Err(AppError::Custom(e)) => assert!(e == expected_error),
            _ => panic!("Getting branch should not have succeeded!"),
        }
    }

    #[test]
    fn should_get_branch_and_put_in_state() {
        let trie = get_sample_trie_with_sample_receipts(
            SAMPLE_RECEIPT_JSONS_1_PATH.to_string(),
            get_sample_tx_hashes_1(),
        );
        let state_before = get_valid_state_with_receipts_trie_and_index(
            SAMPLE_RECEIPT_JSONS_1_PATH.to_string(),
            get_sample_tx_hashes_1(),
        )
        .unwrap();
        let index = state_before.get_index_from_state().unwrap();
        let expected_branch = get_branch_from_trie(trie, *index).unwrap();
        if let Ok(_) = state_before.get_branch_from_state() {
            panic!("Should not have branch in state yet!")
        };
        let state_after = get_branch_from_trie_and_put_in_state(state_before).unwrap();
        match state_after.get_branch_from_state() {
            Err(_) => panic!("Should have branch in state now!"),
            Ok(branch) => {
                for i in 0..branch.len() {
                    assert!(branch[i] == expected_branch[i]);
                }
            }
        };
    }
}
