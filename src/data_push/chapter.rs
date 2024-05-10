use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use itertools::Itertools;
use mangadex_api_schema_rust::{v5::ChapterObject, ApiData};
use mangadex_api_types_rust::{RelationshipType, ResponseType, ResultType};
use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions, ManagerCoreResult};

use super::Push;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChapterRequiredRelationship {
    Manga,
    ScanlationGroup,
    Uploader,
}

impl ChapterRequiredRelationship {
    fn validate(data: &ChapterObject) -> Vec<Self> {
        let mut required = Vec::<Self>::new();
        if data.find_relationships(RelationshipType::Manga).is_empty() {
            required.push(Self::Manga);
        }
        if data
            .find_relationships(RelationshipType::ScanlationGroup)
            .is_empty()
        {
            required.push(Self::ScanlationGroup)
        }
        if data.find_relationships(RelationshipType::User).is_empty() {
            required.push(Self::Uploader);
        }
        required
    }
    fn seed(mut input: ChapterObject, seed: ChapterObject) -> ChapterObject {
        let required = Self::validate(&input);
        for req in required {
            match req {
                ChapterRequiredRelationship::Manga => {
                    input
                        .relationships
                        .retain(|x| x.type_ != RelationshipType::Manga);
                    input.relationships.append(
                        &mut seed
                            .relationships
                            .iter()
                            .filter(|r| r.type_ == RelationshipType::Manga)
                            .cloned()
                            .collect_vec(),
                    );
                }
                ChapterRequiredRelationship::ScanlationGroup => {
                    input
                        .relationships
                        .retain(|x| x.type_ != RelationshipType::ScanlationGroup);
                    input.relationships.append(
                        &mut seed
                            .relationships
                            .iter()
                            .filter(|r| r.type_ == RelationshipType::ScanlationGroup)
                            .cloned()
                            .collect_vec(),
                    );
                }
                ChapterRequiredRelationship::Uploader => {
                    input
                        .relationships
                        .retain(|x| x.type_ != RelationshipType::User);
                    input.relationships.append(
                        &mut seed
                            .relationships
                            .iter()
                            .filter(|r| r.type_ == RelationshipType::User)
                            .cloned()
                            .collect_vec(),
                    );
                }
            }
        }
        input
    }
}

impl From<ChapterRequiredRelationship> for RelationshipType {
    fn from(value: ChapterRequiredRelationship) -> Self {
        match value {
            ChapterRequiredRelationship::Manga => Self::Manga,
            ChapterRequiredRelationship::ScanlationGroup => Self::ScanlationGroup,
            ChapterRequiredRelationship::Uploader => Self::User,
        }
    }
}

impl Push<ChapterObject> for DirsOptions {
    fn push(&mut self, data: ChapterObject) -> crate::ManagerCoreResult<()> {
        let chapter_path = self.chapters_id_add(data.id);
        create_dir_all(&chapter_path)?;
        let mut file = BufWriter::new(File::create(chapter_path.join("data.json"))?);
        serde_json::to_writer(
            &mut file,
            &ApiData {
                response: ResponseType::Entity,
                result: ResultType::Ok,
                data,
            },
        )?;
        file.flush()?;
        Ok(())
    }
    fn verify_and_push(&mut self, data: ChapterObject) -> ManagerCoreResult<()> {
        if let Ok(inner_chapter) = <Self as Pull<ChapterObject, Uuid>>::pull(self, data.id) {
            self.push(ChapterRequiredRelationship::seed(data, inner_chapter))
        } else {
            let required = ChapterRequiredRelationship::validate(&data);
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

impl Push<Vec<ChapterObject>> for DirsOptions {
    fn push(&mut self, data: Vec<ChapterObject>) -> ManagerCoreResult<()> {
        for item in data {
            self.push(item)?;
        }
        Ok(())
    }
}
