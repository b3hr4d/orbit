use super::indexes::user_group_name_index::UserGroupNameIndexRepository;
use crate::{
    core::{with_memory_manager, Memory, USER_GROUP_MEMORY_ID},
    models::{indexes::user_group_name_index::UserGroupNameIndexCriteria, UserGroup},
};
use ic_canister_core::repository::{IndexRepository, RefreshIndexMode};
use ic_canister_core::{repository::Repository, types::UUID};
use ic_stable_structures::{memory_manager::VirtualMemory, StableBTreeMap};
use lazy_static::lazy_static;
use std::cell::RefCell;

thread_local! {
  static DB: RefCell<StableBTreeMap<UUID, UserGroup, VirtualMemory<Memory>>> = with_memory_manager(|memory_manager| {
    RefCell::new(
      StableBTreeMap::init(memory_manager.get(USER_GROUP_MEMORY_ID))
    )
  })
}

lazy_static! {
    pub static ref USER_GROUP_REPOSITORY: UserGroupRepository = UserGroupRepository::default();
}

/// A repository that enables managing users in stable memory.
#[derive(Default, Debug)]
pub struct UserGroupRepository {
    name_index: UserGroupNameIndexRepository,
}

impl Repository<UUID, UserGroup> for UserGroupRepository {
    fn list(&self) -> Vec<UserGroup> {
        DB.with(|m| m.borrow().iter().map(|(_, v)| v).collect())
    }

    fn get(&self, key: &UUID) -> Option<UserGroup> {
        DB.with(|m| m.borrow().get(key))
    }

    fn insert(&self, key: UUID, value: UserGroup) -> Option<UserGroup> {
        DB.with(|m| {
            let prev = m.borrow_mut().insert(key, value.clone());
            self.name_index
                .refresh_index_on_modification(RefreshIndexMode::Value {
                    previous: prev.clone().clone().map(|prev| prev.to_index_by_name()),
                    current: Some(value.to_index_by_name()),
                });

            prev
        })
    }

    fn remove(&self, key: &UUID) -> Option<UserGroup> {
        DB.with(|m| {
            let prev = m.borrow_mut().remove(key);
            self.name_index
                .refresh_index_on_modification(RefreshIndexMode::CleanupValue {
                    current: prev.clone().map(|prev| prev.to_index_by_name()),
                });

            prev
        })
    }

    fn len(&self) -> usize {
        DB.with(|m| m.borrow().len()) as usize
    }
}

impl UserGroupRepository {
    pub fn find_by_name(&self, name: &str) -> Option<UserGroup> {
        let user_group_ids = self
            .name_index
            .find_by_criteria(UserGroupNameIndexCriteria {
                name: name.to_string(),
            });

        if user_group_ids.is_empty() {
            return None;
        }

        let user_group_id = user_group_ids.iter().next().unwrap();

        self.get(user_group_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_group_test_utils;

    #[test]
    fn test_crud() {
        let repository = UserGroupRepository::default();
        let user_group = user_group_test_utils::mock_user_group();

        assert!(repository.get(&user_group.id).is_none());

        repository.insert(user_group.id.to_owned(), user_group.clone());

        assert!(repository.get(&user_group.id).is_some());
        assert!(repository.remove(&user_group.id).is_some());
        assert!(repository.get(&user_group.id).is_none());
    }

    #[test]
    fn test_find_by_name() {
        let repository = UserGroupRepository::default();
        let user_group = user_group_test_utils::mock_user_group();

        assert!(repository.find_by_name(&user_group.name).is_none());

        repository.insert(user_group.id, user_group.clone());

        assert!(repository.find_by_name(&user_group.name).is_some());
        assert!(repository.remove(&user_group.id).is_some());
        assert!(repository.find_by_name(&user_group.name).is_none());
    }
}