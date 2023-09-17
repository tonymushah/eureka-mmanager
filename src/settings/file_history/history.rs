use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};

use super::{HistoryEntry, IsIn, Insert, Remove};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct History {
    history_list: Vec<uuid::Uuid>,
    data_type: RelationshipType,
}

impl History {
    pub fn new(data_type: RelationshipType) -> History {
        History {
            history_list: Vec::new(),
            data_type,
        }
    }
    pub fn get_history_list_mut(&mut self) -> &mut Vec<uuid::Uuid> {
        &mut (self.history_list)
    }
    pub fn get_history_list(&mut self) -> &Vec<uuid::Uuid> {
        &(self.history_list)
    }
    pub fn get_data_type_mut(&mut self) -> &mut RelationshipType {
        &mut (self.data_type)
    }
    pub fn get_data_type(&mut self) -> &RelationshipType {
        &(self.data_type)
    }
    pub fn is_this_type(&self, to_use_rel: RelationshipType) -> bool {
        self.data_type == to_use_rel
    }
}

impl IsIn<uuid::Uuid> for History {
    type Output = bool;

    fn is_in(&self, to_use : uuid::Uuid) -> Self::Output {
        self.history_list
            .iter()
            .any(|id| id.cmp(&to_use).is_eq())
    }
}

impl IsIn<HistoryEntry> for History {
    type Output = Result<bool, std::io::Error>;
    fn is_in(&self, to_use : HistoryEntry) -> Self::Output {
        if to_use.data_type == self.data_type {
            if self.is_in(to_use.id) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the relationship doesn't match",
            ))
        }
    }
}

impl Insert<uuid::Uuid> for History{
    type Output = Result<(), std::io::Error>;
    fn insert(&mut self, input : uuid::Uuid) -> Self::Output {
        let result = <Self as IsIn<uuid::Uuid>>::is_in(self, input);
        if !result {
            self.history_list.push(input);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("the uuid {} is already there", input),
            ))
        }
    }
}

impl Insert<HistoryEntry> for History{
    type Output = Result<(), std::io::Error>;

    fn insert(&mut self, input : HistoryEntry) -> Self::Output {
        let result = <Self as IsIn<HistoryEntry>>::is_in(self, input)?;
        if !result {
            self.history_list.push(input.id);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("the uuid {} is already there", input.id),
            ));
        }
        Ok(())
    }
}

impl Remove<uuid::Uuid> for History{
    type Output = Result<(), std::io::Error>;

    fn remove(&mut self, input : uuid::Uuid) -> Self::Output {
        let result = <Self as IsIn<uuid::Uuid>>::is_in(self, input);
        if result {
            self.history_list.remove(
                match self
                    .history_list
                    .iter()
                    .position(|data| data.cmp(&input).is_eq())
                {
                    Some(data) => data,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("the uuid {} is not found", input),
                        ))
                    }
                },
            );
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("the uuid {} is not found", input),
            ));
        }
        Ok(())
    }
}

impl Remove<HistoryEntry> for History{
    type Output = Result<(), std::io::Error>;
    fn remove(&mut self, input : HistoryEntry) -> Self::Output {
        let result = <Self as IsIn<HistoryEntry>>::is_in(self, input)?;
        if result {
            <Self as Remove<uuid::Uuid>>::remove(self, input.id)
        }else{
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("the uuid {} is not found", input.id),
            ))
        }
    }
}

impl IntoIterator for History {
    type Item = uuid::Uuid;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.history_list.into_iter()
    }
}