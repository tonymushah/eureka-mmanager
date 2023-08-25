pub mod get;
pub mod patch;
pub mod delete;
pub mod put;

pub trait DefaultOffsetLimit<'a>: serde::Deserialize<'a> {
    type OffsetOutput;
    type LimitOutput;
    fn default_offset() -> Self::OffsetOutput;
    fn default_limit() -> Self::LimitOutput;
}