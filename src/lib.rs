use std::fs::File;
use std::io::Write;
use actix_web::dev::Server;
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
use actix_web::{
    get,  
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
use crate::settings::{verify_settings_dir, initialise_settings_dir, initialise_data_dir, verify_data_dir};
use crate::utils::{find_all_downloades_by_manga_id, patch_manga_by_chapter, find_and_delete_all_downloades_by_manga_id};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use std::path::Path;
use settings::server_options;
use settings::files_dirs::DirsOptions;
pub mod settings;

pub mod chapter_download;
pub mod cover_download;
pub mod utils;
// NOTE all get methods

/// try if the app is ok
/// # How to use
/// {app deployed url}/
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::json!({
            "result" : "ok",
            "message" : "The mangadex desktop api works !!"
        }).to_string())
}

/// Find a downloaded manga
/// # How to use
/// {app deployed url}/manga/{manga_id}
#[get("/manga/{id}")]
async fn find_manga_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
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

/// find a cover by his id
/// # How to use
/// {app deployed url}/cover/{cover_id}
#[get("/cover/{id}")]
async fn find_cover_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let path = file_dirs.covers_add(format!("{}.json", id).as_str());
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

/// find a cover by his id
#[get("/cover/{id}/image")]
async fn find_cover_image_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let file_dir_clone = file_dirs.clone();
            let path = file_dirs.covers_add(format!("{}.json", id).as_str());
            if Path::new(path.as_str()).exists() == true {
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                let cover_data: ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(jsons.as_str()).expect("cannot covert the cover json to the rust api");
                let filename = cover_data.data.attributes.file_name;
                let filename_path = file_dir_clone.covers_add(format!("images/{}", filename).as_str());
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

/// find a downloaded manga cover
#[get("/manga/{id}/cover")]
async fn find_manga_cover_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let file_dir_clone = file_dirs.clone();
            let path = file_dirs.mangas_add(format!("{}.json", id).as_str());            
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
                let filename_path = file_dir_clone.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());
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

/// find a downloaded covers manga
#[get("/manga/{id}/covers")]
async fn find_manga_covers_by_id(id: web::Path<String>) -> impl Responder {
    
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            //let file_dir_clone = file_dirs.clone();
            let path = file_dirs.mangas_add(format!("lists/{}.json", id).as_str());
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

/// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
async fn find_chapters_data_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            //let file_dir_clone = file_dirs.clone();
            let path = file_dirs.chapters_add(format!("{}/data", id).as_str());
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

/// find a chapters data-saver (json data) by his id
#[get("/chapter/{id}/data-saver")]
async fn find_chapters_data_saver_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            //let file_dir_clone = file_dirs.clone();
            let path = file_dirs.chapters_add(format!("{}/data-saver", id).as_str());
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

/// find a chapters data image by his id
#[get("/chapter/{id}/data/{filename}")]
async fn find_chapters_data_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    //let path = ;
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
                //let file_dir_clone = file_dirs.clone();
                let path = file_dirs.chapters_add(format!("{}/data/{}", id, filename).as_str());
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

/// find a chapters data-saver image by his id
#[get("/chapter/{id}/data-saver/{filename}")]
async fn find_chapters_data_saver_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    //let path = format!("chapters/{}/data-saver/{}", id, filename);
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
                //let file_dir_clone = file_dirs.clone();
                let path = file_dirs.chapters_add(format!("{}/data-saver/{}", id, filename).as_str());
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

/// find a chapter (json data) by his id
#[get("/chapter/{id}")]
async fn find_chapter_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
                //let file_dir_clone = file_dirs.clone();
                let path = file_dirs.chapters_add(format!("{}/data.json", id).as_str());
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

/// find a chapters data-saver (json data) by his id
#[get("/chapter/all")]
async fn find_all_downloaded_chapter() -> impl Responder {
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
                //let file_dir_clone = file_dirs.clone();
                let path = file_dirs.chapters_add("");
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

/// find all downloaded manga 
#[get("/mangas/all")]
async fn find_all_downloaded_manga() -> impl Responder {
    //let path = format!("mangas");
    catch!{
        try{
            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
                //let file_dir_clone = file_dirs.clone();
                let path = file_dirs.mangas_add("");
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

/// find all downloaded chapter manga
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

/// update a cover json data by his id
#[patch("/cover/{id}")]
async fn update_cover_by_id(id: web::Path<String>) -> impl Responder {
    
    catch!{
        try{
            let path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .covers_add(format!("{}.json", id).as_str());
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
        
                let mut cover_data = File::create(path.clone())
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

/// update a chapter by his id
#[patch("/chapter/{id}")]
async fn update_chapter_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    
    catch! {
        try{
            let path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .chapters_add(format!("{}.json", id).as_str());
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
                
                let mut cover_data = File::create(path.clone())
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

/// update all chapters data
#[patch("/chapters/all")]
async fn patch_all_chapter() -> impl Responder{
    //let path = format!("chapters");
    
    catch! {
        try{
            let path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .chapters_add("");
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

/// patch all chapters manga data
#[patch("/chapters/all/patch-manga")]
async fn patch_all_chapter_manga() -> impl Responder{
    
    catch!{
        try{
            let path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .chapters_add("");
        //println!("{}", path);
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

/// patch a chapter manga data
#[patch("/chapter/{id}/patch-manga")]
async fn update_chapter_manga_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    let path = DirsOptions::new()
        .expect("Can't load the dirOption api")
        .chapters_add(format!("chapters/{}/data.json", id).as_str());
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

/// patch all manga cover
#[patch("/mangas/all/cover")]
async fn patch_all_manga_cover() -> impl Responder {
    
    catch!{
        try{
            let path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .mangas_add("");
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

/// delete a chapter from the api
#[delete("/chapter/{id}")]
async fn delete_chapter_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{
            let mut jsons = serde_json::json!({});
            let chapter_path = DirsOptions::new()
                .expect("Can't load the dirOption api")
                .mangas_add(format!("{}", id).as_str());
            if Path::new(chapter_path.as_str()).exists() == true {
                std::fs::remove_dir_all(chapter_path).expect("can't delete chapter dir");
                jsons = serde_json::json!({
                    "result" : "ok"
                });
            }else{
                jsons = serde_json::json!({
                    "result" : "error"
                });
                panic!("can't find chapter {}", id);
            }
            
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

/// delete a  manga
#[delete("/manga/{id}")]
async fn delete_manga_chapters_by_id(id: web::Path<String>) -> impl Responder {
    catch!{
        try{


            let file_dirs = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let file_dir_clone = file_dirs.clone();
            let path = file_dirs.mangas_add(format!("{}.json", id).as_str());            
                let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
                let manga_data: ApiData<ApiObject<MangaAttributes>> = serde_json::from_str(jsons.as_str()).expect("cannot covert the cover json to the rust api");
                let cover_id = manga_data
                    .data
                    .relationships
                    .iter()
                    .find(|related| related.type_ == RelationshipType::CoverArt)
                    .expect("no cover art found for manga")
                    .id;
                let filename_path1 = file_dir_clone.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());            

            let file_dirs2 = DirsOptions::new().expect("can't initialise dirsOPtions api");
            let file_dir_clone2 = file_dirs2.clone();
            let path2 = file_dirs2.covers_add(format!("{}.json", cover_id).as_str());
                let jsons = std::fs::read_to_string(path2.as_str()).expect("Cannot open file");
                let cover_data: ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(jsons.as_str()).expect("cannot covert the cover json to the rust api");
                let filename = cover_data.data.attributes.file_name;
                let filename_path2 = file_dir_clone2.covers_add(format!("images/{}", filename).as_str());


            let resp = find_and_delete_all_downloades_by_manga_id(id.to_string()).await;
            let jsons = resp.expect("error on extracting data");


            std::fs::remove_file(
                DirsOptions::new().expect("Can't load the DirOptionsApi")
                    .mangas_add(format!("{}.json", id).as_str())
            )?;
            std::fs::remove_file(
                filename_path1
            )?;
            std::fs::remove_file(
                filename_path2
            )?;
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(serde_json::json!({
                    "result" : "ok",
                    "type" : "collection",
                    "data" : jsons,
                    "message" : "deleted "
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




// NOTE All download methods

/// download a manga (req only)
#[put("/manga/{id}")]
async fn download_manga_by_id(id: web::Path<String>) -> impl Responder {

    catch!{
        try{
            let http_client = reqwest::Client::new();
            let resp = http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)).send().await.unwrap();
            let mut file = File::create(DirsOptions::new()
                .expect("Can't load the dirOption api")
                .mangas_add(format!("{}.json", id).as_str())).unwrap();

            file.write_all(&(resp.bytes().await.unwrap())).unwrap();
            cover_download_by_manga_id(format!("{}", id).as_str()).await?;
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

/// download all manga covers
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

/// download the top manga cover
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

/// download the top manga cover with defined quality
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

/// download cover by id
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

/// download cover by id with defined quality
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

/// download chapter by id
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

/// download chapter data by id
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

/// download chapter data-saver by id
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
/// it's launch the server in the given adrress and the given port
/// a call like this 
/// 
/// # Example
/// ```
/// fn main() -> std::io:Result<()> {
///     let address = "127.0.0.1"; 
///     let port : u16 = 8090;
///     launch_server(address, port)
///     // it launch the server at 127.0.0.1:8090
/// }
/// ```
pub async fn launch_server(address: &str, port : u16) -> std::io::Result<()>{
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
            .service(delete_manga_chapters_by_id)
        })
    .bind((address, port))?
    .run()
    .await;
    println!("closing mangadex-desktop-api on {}:{}", address, port);
    habdle
}

pub fn launch_async_server(address: &str, port : u16) -> std::io::Result<Server>{
    Ok(HttpServer::new(|| {
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
            .service(delete_manga_chapters_by_id)
        })
    .bind((address, port))?
    .run())
}

pub fn launch_async_server_default() -> std::io::Result<Server>{
    println!("launching server");
    let serve : server_options::ServerOptions = server_options::ServerOptions::new().expect("Can't load the server option api");
    launch_async_server(serve.hostname.as_str(), serve.port)
}


/// Verify if the data dir and the settings are all there
/// if on of them are not defined or not found , it automatically create the dir corresponding to the error
pub fn verify_all_fs() -> std::io::Result<()> {

    catch!{
        try{
            println!("{}", verify_settings_dir()?);
        }catch error{
            println!("{}", error);
            println!("Settings dir not found \n\tInitializing...");
            initialise_settings_dir().unwrap();
            println!("Initilized settings dir !");
        }
    }
    catch!{
        try{
            println!("{}", verify_data_dir()?);
        }catch error{
            println!("{}", error);
            println!("Data dir not found \n\tInitializing...");
            initialise_data_dir().unwrap();
            println!("Initilized package manager dir !");
        }
    }

    Ok(())
}

/// It's launch the server with the given data in the settings/server_option.json
/// 
/// # Example 

/// if we have a settings/server_option.json like this : 
/// ```
/// {
///   "hostname" : "127.0.0.1",
///    "port" : 8090
/// }
/// ```
/// 
/// and we launch the function :
/// ```
/// fn main() -> std:io::Result<()> {
///     launch_server_default()
///     // it will launch the server at 127.0.0.1:8090
/// }
/// ```
pub fn launch_server_default() -> std::io::Result<()>{
    println!("launching server");
    let serve : server_options::ServerOptions = server_options::ServerOptions::new().expect("Can't load the server option api");
    launch_server(serve.hostname.as_str(), serve.port)
}
