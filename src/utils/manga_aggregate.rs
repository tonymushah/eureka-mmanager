use mangadex_api_schema_rust::v5::manga_aggregate::{ChapterAggregate, VolumeAggregate};

pub mod stream;

pub trait ShouldBe<'a, T: ?Sized> {
    type Output;
    fn should_be(&'a mut self, input: T) -> Self::Output;
}

pub enum FromWhere<'a> {
    Volume(&'a mut VolumeAggregate),
    Chapter(&'a mut ChapterAggregate),
}

pub enum IsHere {
    AlreadyHere,
    Inserted,
}
