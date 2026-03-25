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
            .filter(|r| r.type_ == rel && r.attributes.is_some())
            .cloned()
            .collect_vec(),
    );
}

#[cfg(test)]
mod test {
    use mangadex_api_schema_rust::v5::ChapterObject;
    use mangadex_api_types_rust::RelationshipType;
    use non_exhaustive::non_exhaustive;
    use serde_json::json;
    #[test]
    fn test_seed() {
        let mut input = non_exhaustive! {
            ChapterObject {
                relationships: vec![]
            }
        };
        let seed: ChapterObject = {
            let raw = json!({
              "id": "c666a8b5-8012-41c9-bcb2-393d8f7a1c9c",
              "type": "chapter",
              "attributes": {
                "volume": "2",
                "chapter": "10",
                "title": "Potato Yakisoba Sandwich",
                "translatedLanguage": "en",
                "externalUrl": null,
                "isUnavailable": false,
                "publishAt": "2026-03-25T20:43:12+00:00",
                "readableAt": "2026-03-25T20:43:12+00:00",
                "createdAt": "2026-03-25T20:43:12+00:00",
                "updatedAt": "2026-03-25T20:44:11+00:00",
                "version": 3,
                "pages": 19
              },
              "relationships": [
                {
                  "id": "599e8020-b43f-499d-b546-6192cc1748a4",
                  "type": "scanlation_group",
                  "attributes": {
                    "name": "Teacup Scans",
                    "altNames": [],
                    "locked": true,
                    "website": null,
                    "ircServer": null,
                    "ircChannel": null,
                    "discord": null,
                    "contactEmail": null,
                    "description": "A one-man group. Still studying Japanese.",
                    "twitter": null,
                    "mangaUpdates": null,
                    "focusedLanguages": [],
                    "official": false,
                    "verified": false,
                    "inactive": false,
                    "publishDelay": null,
                    "exLicensed": false,
                    "createdAt": "2023-11-27T18:30:45+00:00",
                    "updatedAt": "2025-02-22T16:25:17+00:00",
                    "version": 4
                  }
                },
                {
                  "id": "3253122f-b218-4650-a5bb-8ed09854721e",
                  "type": "manga",
                  "attributes": {
                    "title": {
                      "en": "Mugi Kaoru Futari"
                    },
                    "altTitles": [
                      {
                        "ja": "麦香るふたり"
                      }
                    ],
                    "description": {
                      "en": "Hugo, the grandson of Adam, is struggling to reopen the bakery left by his late grandfather. However, he is not good at baking. Then, Adam's former apprentice, Sumino Mitsuyuki, comes to the store. Hugo decides to ask Sumino, who he's not very fond of, if he can help reopen the bakery, but it turns out that Sumino hasn't made bread for three years! Hugo is at a loss, but... how does the bread made by Sumino taste?\n\n(Source: Shogakukan, translated)",
                      "ja": "亡き祖父・アダンの残したパン屋を再開するため、ひとり奮闘していた孫のヒューゴ。しかし、彼はパン作りが苦手だった。そんな中、お店にアダンの元弟子である澄野端之がやって来る。ヒューゴはお店が再開できるなら、とちょっぴり苦手な澄野を頼ることにしたが、聞けば澄野は3年もパンを作っていないーー！？ 途方に暮れるヒューゴだったが…さて、澄野の作るパンの味は？ 『ないしょの京子姉さん』の葛西尚が送る、パン香る森のヒューマンドラマ！"
                    },
                    "isLocked": false,
                    "links": {
                      "al": "194431",
                      "ap": "mugi-kaoru-futari",
                      "bw": "series/528135/list",
                      "mu": "nqh6vgr",
                      "amz": "https://www.amazon.co.jp/dp/B0FCDCJTNP",
                      "ebj": "https://ebookjapan.yahoo.co.jp/books/903797",
                      "mal": "182756",
                      "raw": "https://www.sunday-webry.com/episode/2550912965183045852"
                    },
                    "officialLinks": null,
                    "originalLanguage": "ja",
                    "lastVolume": "3",
                    "lastChapter": "27",
                    "publicationDemographic": "shounen",
                    "status": "completed",
                    "year": 2025,
                    "contentRating": "safe",
                    "tags": [
                      {
                        "id": "b9af3a63-f058-46de-a9a0-e0c13906197a",
                        "type": "tag",
                        "attributes": {
                          "name": {
                            "en": "Drama"
                          },
                          "description": {},
                          "group": "genre",
                          "version": 1
                        },
                        "relationships": []
                      },
                      {
                        "id": "e5301a23-ebd9-49dd-a0cb-2add944c7fe9",
                        "type": "tag",
                        "attributes": {
                          "name": {
                            "en": "Slice of Life"
                          },
                          "description": {},
                          "group": "genre",
                          "version": 1
                        },
                        "relationships": []
                      },
                      {
                        "id": "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869",
                        "type": "tag",
                        "attributes": {
                          "name": {
                            "en": "Cooking"
                          },
                          "description": {},
                          "group": "theme",
                          "version": 1
                        },
                        "relationships": []
                      }
                    ],
                    "state": "published",
                    "chapterNumbersResetOnNewVolume": false,
                    "createdAt": "2025-02-01T12:12:44+00:00",
                    "updatedAt": "2026-02-23T08:46:44+00:00",
                    "version": 12,
                    "availableTranslatedLanguages": [
                      "en"
                    ],
                    "latestUploadedChapter": "c666a8b5-8012-41c9-bcb2-393d8f7a1c9c"
                  }
                },
                {
                  "id": "4ef000c7-a1b0-4f19-9bfd-6787b5ed0e07",
                  "type": "user",
                  "attributes": {
                    "username": "koushi-kun",
                    "roles": [
                      "ROLE_GROUP_LEADER",
                      "ROLE_GROUP_MEMBER",
                      "ROLE_MEMBER"
                    ],
                    "avatarFileName": null,
                    "bannerFileName": null,
                    "version": 408
                  }
                }
              ]
            });
            serde_json::from_value(raw).unwrap()
        };
        super::seed_rel(
            &mut input,
            &seed,
            mangadex_api_types_rust::RelationshipType::User,
        );
        assert_eq!(input.relationships.len(), 1);
        assert!(
            matches!(
                input.relationships[0].id.to_string().as_str(),
                "4ef000c7-a1b0-4f19-9bfd-6787b5ed0e07"
            ) || matches!(input.relationships[0].type_, RelationshipType::User)
        );
    }
}
