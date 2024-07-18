pub mod chapter;
pub mod cover;
pub mod manga;

use itertools::Itertools;
use mangadex_api_schema_rust::ApiObject;
use mangadex_api_types_rust::RelationshipType;

pub trait Push<T> {
    type Error;
    fn push(&mut self, data: T) -> Result<(), Self::Error>;
    fn verify_and_push(&mut self, data: T) -> Result<(), Self::Error> {
        self.push(data)
    }
}

pub(crate) fn seed_rel<A>(input: &mut ApiObject<A>, seed: &ApiObject<A>, rel: RelationshipType) {
    input.relationships.retain(|x| x.type_ != rel);
    input.relationships.append(
        &mut seed
            .relationships
            .iter()
            .filter(|r| r.type_ == rel)
            .cloned()
            .collect_vec(),
    );
}
