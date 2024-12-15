use eureka_mmanager_core::{data_pulls::Pull, data_push::Push, DirsOptions};
use mangadex_api_schema_rust::{
    v5::{MangaAttributes, MangaObject},
    ApiObject,
};
use mangadex_api_types_rust::{
    ContentRating, Language, MangaDexDateTime, MangaState, MangaStatus, RelationshipType,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let mut options = DirsOptions::new_from_data_dir("test-output");
    // verify if the directories is present and create them if they not exits
    options.verify_and_init()?;
    let id = Uuid::new_v4();
    let created_at = MangaDexDateTime::default();
    options.push(ApiObject {
        id,
        type_: RelationshipType::Manga,
        attributes: MangaAttributes {
            title: Default::default(),
            alt_titles: Default::default(),
            description: Default::default(),
            is_locked: true,
            links: None,
            original_language: Language::English,
            last_chapter: None,
            last_volume: None,
            publication_demographic: None,
            status: MangaStatus::Cancelled,
            year: Some(2024),
            content_rating: Some(ContentRating::Safe),
            chapter_numbers_reset_on_new_volume: false,
            latest_uploaded_chapter: None,
            available_translated_languages: Default::default(),
            tags: Default::default(),
            state: MangaState::Draft,
            created_at,
            updated_at: None,
            version: 1,
        },
        relationships: Default::default(),
    })?;
    let _title = Pull::<MangaObject, _>::pull(&options, id)?;
    Ok(())
}
