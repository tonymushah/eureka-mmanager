use std::{
    fs::File,
    io::{BufWriter, Write},
};

use itertools::Itertools;
use mangadex_api_schema_rust::{v5::MangaObject, ApiData};
use mangadex_api_types_rust::{
    ReferenceExpansionResource, RelationshipType, ResponseType, ResultType,
};
use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions};

use super::{seed_rel, Push};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MangaRequiredRelationship {
    Author,
    Artist,
    CoverArt,
}

impl From<MangaRequiredRelationship> for RelationshipType {
    fn from(value: MangaRequiredRelationship) -> Self {
        match value {
            MangaRequiredRelationship::Author => Self::Author,
            MangaRequiredRelationship::Artist => Self::Artist,
            MangaRequiredRelationship::CoverArt => Self::CoverArt,
        }
    }
}

impl MangaRequiredRelationship {
    pub fn get_includes() -> Vec<ReferenceExpansionResource> {
        vec![
            ReferenceExpansionResource::Manga,
            ReferenceExpansionResource::CoverArt,
            ReferenceExpansionResource::Artist,
            ReferenceExpansionResource::Author,
            ReferenceExpansionResource::User,
        ]
    }
    fn validate(data: &MangaObject) -> Vec<Self> {
        let mut required = Vec::<Self>::new();
        if data.find_relationships(RelationshipType::Artist).is_empty() {
            required.push(Self::Artist);
        }
        if data.find_relationships(RelationshipType::Author).is_empty() {
            required.push(Self::Author)
        }
        if data
            .find_relationships(RelationshipType::CoverArt)
            .is_empty()
        {
            required.push(Self::CoverArt);
        }
        required
    }
    fn seed(mut input: MangaObject, seed: MangaObject) -> MangaObject {
        let required = Self::validate(&input);
        for req in required {
            seed_rel(&mut input, &seed, req.into());
        }
        input
    }
}

impl Push<MangaObject> for DirsOptions {
    fn push(&mut self, data: MangaObject) -> crate::ManagerCoreResult<()> {
        let mut file = BufWriter::new(File::create(self.mangas_add(format!("{}.json", data.id)))?);
        serde_json::to_writer(
            &mut file,
            &ApiData {
                response: ResponseType::Entity,
                data,
                result: ResultType::Ok,
            },
        )?;
        file.flush()?;
        Ok(())
    }
    fn verify_and_push(&mut self, data: MangaObject) -> crate::ManagerCoreResult<()> {
        if let Ok(inner_chapter) = <Self as Pull<MangaObject, Uuid>>::pull(self, data.id) {
            self.push(MangaRequiredRelationship::seed(data, inner_chapter))
        } else {
            let required = MangaRequiredRelationship::validate(&data);
            if required.is_empty() {
                self.push(data)
            } else {
                Err(crate::Error::MissingRelationships(
                    required
                        .into_iter()
                        .map(RelationshipType::from)
                        .collect_vec(),
                ))
            }
        }
    }
}

impl Push<Vec<MangaObject>> for DirsOptions {
    fn push(&mut self, data: Vec<MangaObject>) -> crate::ManagerCoreResult<()> {
        for manga in data {
            self.push(manga)?;
        }
        Ok(())
    }
    fn verify_and_push(&mut self, data: Vec<MangaObject>) -> crate::ManagerCoreResult<()> {
        for manga in data {
            self.verify_and_push(manga)?;
        }
        Ok(())
    }
}
