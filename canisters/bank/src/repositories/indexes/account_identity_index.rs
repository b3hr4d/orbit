use crate::{
    core::{with_memory_manager, Memory, ACCOUNT_IDENTITY_INDEX_MEMORY_ID},
    models::{
        indexes::account_identity_index::{AccountIdentityIndex, AccountIdentityIndexCriteria},
        AccountId,
    },
};
use ic_canister_core::repository::IndexRepository;
use ic_stable_structures::{memory_manager::VirtualMemory, StableBTreeMap};
use std::{cell::RefCell, collections::HashSet};

thread_local! {
  /// The memory reference to the Transfer repository.
  static DB: RefCell<StableBTreeMap<AccountIdentityIndex, (), VirtualMemory<Memory>>> = with_memory_manager(|memory_manager| {
    RefCell::new(
      StableBTreeMap::init(memory_manager.get(ACCOUNT_IDENTITY_INDEX_MEMORY_ID))
    )
  })
}

/// A repository that enables managing transfer in stable memory.
#[derive(Default, Debug)]
pub struct AccountIdentityIndexRepository {}

impl IndexRepository<AccountIdentityIndex, AccountId> for AccountIdentityIndexRepository {
    type FindByCriteria = AccountIdentityIndexCriteria;

    fn exists(&self, index: &AccountIdentityIndex) -> bool {
        DB.with(|m| m.borrow().get(index).is_some())
    }

    fn insert(&self, index: AccountIdentityIndex) {
        DB.with(|m| m.borrow_mut().insert(index, ()));
    }

    fn remove(&self, index: &AccountIdentityIndex) -> bool {
        DB.with(|m| m.borrow_mut().remove(index).is_some())
    }

    fn find_by_criteria(&self, criteria: Self::FindByCriteria) -> HashSet<AccountId> {
        DB.with(|db| {
            let start_key = AccountIdentityIndex {
                identity_id: criteria.identity_id,
                account_id: [u8::MIN; 16],
            };
            let end_key = AccountIdentityIndex {
                identity_id: criteria.identity_id,
                account_id: [u8::MAX; 16],
            };

            db.borrow()
                .range(start_key..=end_key)
                .map(|(index, _)| index.account_id)
                .collect::<HashSet<AccountId>>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_account_identity_index_repository() {
        let repository = AccountIdentityIndexRepository::default();
        let index = AccountIdentityIndex {
            identity_id: Principal::anonymous(),
            account_id: [1; 16],
        };

        assert!(!repository.exists(&index));

        repository.insert(index.clone());

        assert!(repository.exists(&index));
        assert!(repository.remove(&index));
        assert!(!repository.exists(&index));
    }

    #[test]
    fn test_find_by_identity() {
        let repository = AccountIdentityIndexRepository::default();
        let index = AccountIdentityIndex {
            identity_id: Principal::anonymous(),
            account_id: [1; 16],
        };

        repository.insert(index.clone());

        let result = repository.find_by_criteria(AccountIdentityIndexCriteria {
            identity_id: Principal::anonymous(),
        });

        assert!(!result.is_empty());
        assert!(result.contains(&[1; 16]));
    }
}