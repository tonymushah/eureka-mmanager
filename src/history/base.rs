use std::sync::Arc;

use tokio::sync::RwLock;

use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};

use super::{HistoryBaseError, HistoryEntry, Insert, IsIn, Remove};

pub mod error;

pub(crate) type HBResult<T> = Result<T, HistoryBaseError>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryBase {
    history_list: Vec<uuid::Uuid>,
    data_type: RelationshipType,
}

impl From<HistoryBase> for Arc<RwLock<HistoryBase>> {
    fn from(val: HistoryBase) -> Self {
        Arc::new(RwLock::new(val))
    }
}

impl HistoryBase {
    pub fn new(data_type: RelationshipType) -> HistoryBase {
        HistoryBase {
            history_list: Vec::new(),
            data_type,
        }
    }
    pub fn get_history_list_mut(&mut self) -> &mut Vec<uuid::Uuid> {
        &mut (self.history_list)
    }
    pub fn get_history_list(&self) -> &Vec<uuid::Uuid> {
        &(self.history_list)
    }
    pub fn get_data_type_mut(&mut self) -> &mut RelationshipType {
        &mut (self.data_type)
    }
    pub fn get_data_type(&self) -> &RelationshipType {
        &(self.data_type)
    }
    pub fn is_this_type(&self, to_use_rel: RelationshipType) -> bool {
        self.data_type == to_use_rel
    }
}

impl IsIn<uuid::Uuid> for HistoryBase {
    type Output = Option<usize>;

    fn is_in(&self, to_use: uuid::Uuid) -> Self::Output {
        self.get_history_list()
            .iter()
            .position(|id| id.cmp(&to_use).is_eq())
    }
}

impl IsIn<HistoryEntry> for HistoryBase {
    type Output = HBResult<bool>;

    fn is_in(&self, to_use: HistoryEntry) -> Self::Output {
        if self.is_this_type(to_use.data_type) {
            Ok(self.is_in(to_use.id).is_some())
        } else {
            Err(HistoryBaseError::InvalidRelationship(to_use.data_type))
        }
    }
}

impl Insert<uuid::Uuid> for HistoryBase {
    type Output = HBResult<()>;
    fn insert(&mut self, input: uuid::Uuid) -> Self::Output {
        let result = <Self as IsIn<uuid::Uuid>>::is_in(self, input);
        if result.is_none() {
            self.get_history_list_mut().push(input);
            Ok(())
        } else {
            Err(HistoryBaseError::AlreadyExists(input))
        }
    }
}

impl Insert<HistoryEntry> for HistoryBase {
    type Output = HBResult<()>;

    fn insert(&mut self, input: HistoryEntry) -> Self::Output {
        let result = <Self as IsIn<HistoryEntry>>::is_in(self, input)?;
        if !result {
            self.get_history_list_mut().push(input.id);
        } else {
            return Err(HistoryBaseError::AlreadyExists(input.id));
        }
        Ok(())
    }
}

impl Remove<uuid::Uuid> for HistoryBase {
    type Output = HBResult<()>;

    fn remove(&mut self, input: uuid::Uuid) -> Self::Output {
        let position = <Self as IsIn<uuid::Uuid>>::is_in(self, input)
            .ok_or(HistoryBaseError::NotFound(input))?;
        self.get_history_list_mut().remove(position);
        Ok(())
    }
}

impl Remove<HistoryEntry> for HistoryBase {
    type Output = HBResult<()>;
    fn remove(&mut self, input: HistoryEntry) -> Self::Output {
        let result = self.is_this_type(input.data_type);
        if result {
            <Self as Remove<uuid::Uuid>>::remove(self, input.id)
        } else {
            Err(HistoryBaseError::InvalidRelationship(input.data_type))
        }
    }
}

impl IntoIterator for HistoryBase {
    type Item = uuid::Uuid;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.history_list.into_iter()
    }
}
