use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
};

use mangadex_api_schema_rust::{
    v5::{CoverObject, RelatedAttributes},
    ApiData,
};
use mangadex_api_types_rust::{
    ReferenceExpansionResource, RelationshipType, ResponseType, ResultType,
};

use crate::DirsOptions;

use super::Push;

pub fn required_cover_references() -> Vec<ReferenceExpansionResource> {
    vec![
        ReferenceExpansionResource::Manga,
        ReferenceExpansionResource::User,
    ]
}

impl Push<CoverObject> for DirsOptions {
    type Error = crate::Error;
    fn push(&mut self, data: CoverObject) -> crate::ManagerCoreResult<()> {
        let cover_path = self.covers_add(format!("{}.json", data.id));
        let mut file = BufWriter::new(File::create(cover_path)?);
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
    fn verify_and_push(&mut self, data: CoverObject) -> crate::ManagerCoreResult<()> {
        if data
            .find_first_relationships(RelationshipType::Manga)
            .and_then(|c| {
                c.attributes
                    .as_ref()
                    .map(|a| matches!(a, RelatedAttributes::Manga(_)))
            })
            .unwrap_or_default()
        {
            self.push(data)
        } else {
            Err(crate::Error::MissingRelationships(vec![
                RelationshipType::Manga,
            ]))
        }
    }
}

impl<R: Read> Push<(CoverObject, R)> for DirsOptions {
    type Error = crate::Error;
    fn push(&mut self, (data, mut image): (CoverObject, R)) -> crate::ManagerCoreResult<()> {
        let cover_image_path = self.cover_images_add(&data.attributes.file_name);
        self.push(data)?;
        let mut file = BufWriter::new(File::create(cover_image_path)?);
        io::copy(&mut image, &mut file)?;
        file.flush()?;
        Ok(())
    }
    fn verify_and_push(
        &mut self,
        (data, mut image): (CoverObject, R),
    ) -> crate::ManagerCoreResult<()> {
        let cover_image_path = self.cover_images_add(&data.attributes.file_name);
        self.verify_and_push(data)?;
        let mut file = BufWriter::new(File::create(cover_image_path)?);
        io::copy(&mut image, &mut file)?;
        file.flush()?;
        Ok(())
    }
}

impl Push<Vec<CoverObject>> for DirsOptions {
    type Error = crate::Error;
    fn push(&mut self, data: Vec<CoverObject>) -> crate::ManagerCoreResult<()> {
        for cover in data {
            self.push(cover)?;
        }
        Ok(())
    }
    fn verify_and_push(&mut self, data: Vec<CoverObject>) -> crate::ManagerCoreResult<()> {
        for cover in data {
            self.verify_and_push(cover)?;
        }
        Ok(())
    }
}

impl<R: Read> Push<Vec<(CoverObject, R)>> for DirsOptions {
    type Error = crate::Error;
    fn push(&mut self, data: Vec<(CoverObject, R)>) -> crate::ManagerCoreResult<()> {
        for cover_n_image in data {
            self.push(cover_n_image)?;
        }
        Ok(())
    }
    fn verify_and_push(&mut self, data: Vec<(CoverObject, R)>) -> crate::ManagerCoreResult<()> {
        for cover_n_image in data {
            self.verify_and_push(cover_n_image)?;
        }
        Ok(())
    }
}
