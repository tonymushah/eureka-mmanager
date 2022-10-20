use std::fs::File;
use std::io::Write;
use mangadex_api_schema::v5::{
    CoverAttributes, 
    MangaAttributes
};
use mangadex_api_schema::{
    ApiData, 
    ApiObject
};
use try_catch::catch;
use mangadex_api_types::RelationshipType;
use tokio::task::yield_now;
use actix_web::{
    get, 
    error, 
    post, 
    put, 
    patch, 
    delete,
    web, 
    App, 
    HttpResponse, 
    HttpServer, 
    Responder, 
    Result, 
    dev
};
use actix_web::http::{header::ContentType, header, StatusCode};
use crate::chapter_download::{
    download_chapter, 
    download_chapter_saver
};
use crate::cover_download::{
    all_covers_download_quality_by_manga_id, 
    cover_download_by_manga_id, 
    cover_download_by_cover,
    cover_download_quality_by_cover,
    cover_download_quality_by_manga_id
};
use crate::utils::{find_all_downloades_by_manga_id, patch_manga_by_chapter};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use std::path::Path;
use derive_more::{Display, Error};
use actix_files as fs;
mod chapter_download;
mod cover_download;
mod utils;
// NOTE all get methods

// try if the app is ok
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::json!({
            "result" : "ok",
            "message" : "The mangadex desktop api works !!"
        }).to_string())
}

// find a downloaded manga
#[get("/manga/{id}")]
async fn find_manga_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("mangas/{}.json", id);

    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga in the api"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// find a cover by his id
#[get("/cover/{id}")]
async fn find_cover_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("covers/{}.json", id);

    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
            let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(jsons.to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "cover",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga in the api"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                .body(jsons.to_string())
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// find a cover by his id
#[get("/cover/{id}/image")]
async fn find_cover_image_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("covers/{}.json", id);
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                let cover_data: ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(jsons.as_str()).expect("cannot covert the cover json to the rust api");
                let filename = cover_data.data.attributes.file_name;
                let filename_path = format!("covers/images/{}", filename);
                HttpResponse::Ok()
                    .content_type(ContentType::jpeg())
                    .body(std::fs::read(filename_path).expect("Error on opening the file"))
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "cover",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga in the api"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

// find a downloaded manga cover
#[get("/manga/{id}/cover")]
async fn find_manga_cover_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("mangas/{}.json", id);

    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                let manga_data: ApiData<ApiObject<MangaAttributes>> = serde_json::from_str(jsons.as_str()).expect("cannot covert the cover json to the rust api");
                let cover_id = manga_data
                    .data
                    .relationships
                    .iter()
                    .find(|related| related.type_ == RelationshipType::CoverArt)
                    .expect("no cover art found for manga")
                    .id;
                let filename_path = format!("covers/{}.json", cover_id.hyphenated());
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(std::fs::read_to_string(filename_path).expect(format!("can't find the covers/{}.json", id).as_str()))
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga in the api"
                });
                HttpResponse::NotFound()
                .content_type(ContentType::json())
                .body(jsons.to_string())
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// find a downloaded covers manga
#[get("/manga/{id}/covers")]
async fn find_manga_covers_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("covers/lists/{}.json", id);

    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga cover list in the api"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
async fn find_chapters_data_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("chapters/{}/data", id);

    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<String> = Vec::new();
                for files in list_dir {
                    vecs.push(files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string());
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "type" : "collection",
                        "data" : vecs
                    }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : format!("Cannot find the chapter {} data in the api", id)
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// find a chapters data-saver (json data) by his id
#[get("/chapter/{id}/data-saver")]
async fn find_chapters_data_saver_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("chapters/{}/data-saver", id);
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<String> = Vec::new();
                for files in list_dir {
                    vecs.push(files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string());
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "type" : "collection",
                        "data" : vecs
                    }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : format!("Cannot find the chapter {} data-saver in the api", id)
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

// find a chapters data image by his id
#[get("/chapter/data/{id}/{filename}")]
async fn find_chapters_data_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    let path = format!("chapters/{}/data/{}", id, filename);
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::jpeg())
                    .body(std::fs::read(path).expect("can't read the given file"))
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : format!("Cannot find the chapter {} data-saver in the api", id)
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

// find a chapters data-saver image by his id
#[get("/chapter/data-saver/{id}/{filename}")]
async fn find_chapters_data_saver_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    let path = format!("chapters/{}/data-saver/{}", id, filename);
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::jpeg())
                    .body(std::fs::read(path).expect("can't read the given file"))
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : format!("Cannot find the chapter {} data-saver in the api", id)
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

// find a chapter (json data) by his id
#[get("/chapter/{id}")]
async fn find_chapter_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("chapters/{}/data.json", id);
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "type" : "manga",
                    "id" : id.as_str(),
                    "message" : "Cannot find the manga in the api"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

}

// find a chapters data-saver (json data) by his id
#[get("/chapter/all")]
async fn find_all_downloaded_chapter() -> impl Responder {
    let path = format!("chapters");
    
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<String> = Vec::new();
                for files in list_dir {
                    vecs.push(files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string());
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "type" : "collection",
                        "data" : vecs
                    }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : "can't find the chapters directory"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

#[get("/mangas/all")]
async fn find_all_downloaded_manga() -> impl Responder {
    let path = format!("mangas");
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<String> = Vec::new();
                for files in list_dir {
                    vecs.push(files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string().replace(".json", ""));
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "type" : "collection",
                        "data" : vecs
                }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : "can't find the chapters directory"
                });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

// find a downloaded covers manga
#[get("/manga/{id}/chapters")]
async fn find_manga_chapters_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let resp = find_all_downloades_by_manga_id(id.to_string()).await;
            let jsons = resp.expect("error on extracting data");

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(serde_json::json!({
                    "result" : "ok",
                    "type" : "collection",
                    "data" : jsons
                }).to_string())
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}





// NOTE update api data

// update a cover json data by his id
#[patch("/cover/{id}")]
async fn update_cover_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("covers/{}.json", id);
    catch!{
        try{
            let http_client = reqwest::Client::new();
            let get_cover = http_client
                .get(
                    format!("{}/cover/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", 
                        mangadex_api::constants::API_URL, 
                        id
                    )
                )
                .send()
                .await
                .expect("Can't rend request");
        
                let bytes_ = get_cover.bytes()
                .await
                .expect("error on exporting to bytes");
        
                let mut cover_data = File::create(format!("covers/{}.json", id))
                .expect("Error on creating file");

            cover_data.write_all(&bytes_).unwrap();
                
            let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
        
}

#[patch("/chapter/{id}")]
async fn update_chapter_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("chapters/{}/data.json", id);
    catch! {
        try{
            let http_client = reqwest::Client::new();
            let get_cover = http_client
                .get(
                    format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", 
                        mangadex_api::constants::API_URL, 
                        id
                    )
                )
                .send()
                .await
                .expect("Can't rend request");
                
                let bytes_ = get_cover.bytes()
                .await
                .expect("error on exporting to bytes");
                
                let mut cover_data = File::create(format!("chapters/{}/data.json", id))
                .expect("Error on creating file");

            cover_data.write_all(&bytes_).unwrap();
                
            let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
        
}

#[patch("/chapters/all")]
async fn patch_all_chapter() -> impl Responder{
    let path = format!("chapters");
    catch! {
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<serde_json::Value> = Vec::new();
                for files in list_dir {
                    let id = files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string();
                    vecs.push(utils::update_chap_by_id(id.clone()).await.expect("can't update chapter"));
                    println!("downloaded chapter data {}", id);
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "tasks" : "patched",
                        "type" : "collection",
                        "data" : vecs
                }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : "can't find the chapters directory"
                });
                HttpResponse::NotFound()
                .content_type(ContentType::json())
                .body(jsons.to_string())
            }    
        }
        catch err {
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

#[patch("/chapters/all/patch-manga")]
async fn patch_all_chapter_manga() -> impl Responder{
    let path = format!("chapters");
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<serde_json::Value> = Vec::new();
                for files in list_dir {
                    let id = files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string();
                    vecs.push(utils::patch_manga_by_chapter(id.clone()).await.expect("can't update chapter"));
                    println!("downloaded manga data {}", id);
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "tasks" : "patched",
                        "type" : "collection",
                        "data" : vecs
                    }).to_string())
            }else{
                let jsons = serde_json::json!({
                        "result" : "error",
                        "message" : "can't find the chapters directory"
                    });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err {
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}

#[patch("/chapter/{id}/patch-manga")]
async fn update_chapter_manga_by_id(id: web::Path<String>) -> impl Responder {
    let path = format!("chapters/{}/data.json", id);
    catch!{
        try {
            if Path::new(path.as_str()).exists() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(patch_manga_by_chapter(id.to_string()).await.expect("can't finish request").to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : "can't find the chapters data"
                });
            HttpResponse::NotFound()
                .content_type(ContentType::json())
                .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }    
}

#[patch("/mangas/all/cover")]
async fn patch_all_manga_cover() -> impl Responder {
    let path = format!("mangas");
    catch!{
        try{
            if Path::new(path.as_str()).exists() == true {
                let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
                let mut vecs: Vec<serde_json::Value> = Vec::new();
                for files in list_dir {
                    let manga_id = files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string().replace(".json", "");
                    //let mg_id = manga_id.clone();
                    vecs.push(cover_download_by_manga_id(manga_id.as_str()).await.expect("can't download cover"));
                }
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::json!({
                        "result" : "ok",
                        "type" : "collection",
                        "data" : vecs
                    }).to_string())
            }else{
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : "can't find the chapters directory"
                    });
                HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }    
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
    
}


// NOTE all delete methods

#[delete("/chapter/{id}")]
async fn delete_chapter_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let jsons = serde_json::json!({
                "result" : "ok",
                "type" : "manga",
                "id" : id.as_str()
            });
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
}




// NOTE All download methods

// download a manga (req only)
#[put("/manga/{id}")]
async fn download_manga_by_id(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            let http_client = reqwest::Client::new();
            let resp = http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)).send().await.unwrap();
            let mut file = File::create(format!("mangas/{}.json", id)).unwrap();

            file.write_all(&(resp.bytes().await.unwrap())).unwrap();
            let jsons = serde_json::json!({
                "result" : "ok",
                "type" : "manga",
                "id" : id.as_str()
            });
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// download all manga covers
#[put("/manga/{id}/covers")]
async fn download_manga_covers(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            let response = all_covers_download_quality_by_manga_id(format!("{id}").as_str(), 100).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }
}

// download the top manga cover
#[put("/manga/{id}/cover")]
async fn download_manga_cover(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            let response = cover_download_by_manga_id(format!("{id}").as_str()).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// download the top manga cover with defined quality
#[put("/manga/{id}/cover/{quality}")]
async fn download_manga_cover_quality(id: web::Path<String>, quality: web::Path<u32>) -> impl Responder {

    catch!{
        try{
            let response = cover_download_quality_by_manga_id(format!("{id}").as_str(), *quality).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// download cover by id
#[put("/cover/{id}")]
async fn download_cover(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            let response = cover_download_by_cover(format!("{id}").as_str()).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// download cover by id with defined quality
#[put("/cover/{id}/{quality}")]
async fn download_cover_quality(id: web::Path<String>, quality: web::Path<u32>) -> impl Responder {

    catch!{
        try{
            let response = cover_download_quality_by_cover(format!("{id}").as_str(), *quality).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
}

// download chapter by id
#[put("/chapter/{id}")]
async fn download_chapter_byid(id: web::Path<String>) -> impl Responder {
    format!("Start downloading manga {id}");

    catch!{
        try{
            let response = download_chapter(format!("{id}").as_str()).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(response.unwrap().to_string())
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
    //download_chapter(id.as_str())
}

// download chapter data by id
#[put("/chapter/{id}/data")]
async fn download_chapter_data_byid(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            format!("Start downloading manga {id}");
            let response = download_chapter(format!("{id}").as_str()).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(format!("{}", response.unwrap().to_string()))
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
    //download_chapter(id.as_str())
}

// download chapter data-saver by id
#[put("/chapter/{id}/data-saver")]
async fn download_chapter_data_saver_byid(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            format!("Start downloading manga {id}");
            let response = download_chapter_saver(format!("{id}").as_str()).await;
            if response.is_ok() == true {
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(format!("{}", response.unwrap().to_string()))
            }else{
                HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(format!("Error while downloading {id} \n {:?}", response.expect_err("")))
            }
        }
        catch err{
            let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", err)
                });
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(jsons.to_string())
        }
    }

    
    //download_chapter(id.as_str())
}






fn add_error_header<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

#[actix_web::main]
async fn launch_server(address: &str, port : u16) -> std::io::Result<()>{
    println!("launching mangadex-desktop-api on {}:{}", address, port);
    let habdle = HttpServer::new(|| {
        App::new()
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header)
            )
            .service(hello)
            .service(find_chapters_data_img_by_id)
            .service(find_chapters_data_saver_img_by_id)
            .service(download_chapter_byid)
            .service(download_chapter_data_saver_byid)
            .service(download_chapter_data_byid)
            .service(download_manga_covers)
            .service(download_manga_cover)
            .service(download_manga_by_id)
            .service(find_manga_by_id)
            .service(find_cover_image_by_id)
            .service(find_manga_cover_by_id)
            .service(find_manga_covers_by_id)
            .service(find_manga_covers_by_id)
            .service(update_cover_by_id)
            .service(find_chapters_data_by_id)
            .service(find_chapters_data_saver_by_id)
            .service(find_all_downloaded_chapter)
            .service(update_chapter_by_id)
            .service(patch_all_chapter)
            .service(find_chapter_by_id)
            .service(find_manga_chapters_by_id)
            .service(find_all_downloaded_manga)
            .service(patch_all_chapter_manga)
            .service(update_chapter_manga_by_id)
            .service(patch_all_manga_cover)
        })
    .bind((address, port))?
    .run()
    .await;
    println!("closing mangadex-desktop-api on {}:{}", address, port);
    habdle
}



fn main() -> std::io::Result<()> {
    launch_server("127.0.0.1", 8082)
}
