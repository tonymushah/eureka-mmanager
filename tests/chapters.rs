use mangadex_api::MangaDexClient;
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::{IncludeExternalUrl, ReferenceExpansionResource};
use mangadex_desktop_api2::{Error as ApiError, ManagerCoreResult};

pub async fn get(client: &MangaDexClient) -> ManagerCoreResult<Vec<ChapterObject>> {
    client
        .chapter()
        .get()
        .limit(100u32)
        .includes(vec![
            ReferenceExpansionResource::Manga,
            ReferenceExpansionResource::User,
            ReferenceExpansionResource::ScanlationGroup,
        ])
        .include_external_url(IncludeExternalUrl::Exclude)
        .send()
        .await
        .map(|i| i.data)
        .map_err(ApiError::MangadexAPIError)
}
