use actix_web::{HttpMessage, HttpRequest, HttpResponse, http::Cookie, Responder, get, web::Bytes};
use qstring::QString;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct PostData {
    title: String,
    content: String,
}

#[get("/api/message")]
pub async fn get_message(request: HttpRequest) -> impl Responder {
    let query_string = request.query_string();
    let query_string  = QString::from(query_string);

    let limit = query_string.get("limit");
    let _limit = match limit {
        Some(limit_str) => {
            let limit = limit_str.parse::<i32>();
            match limit {
                Ok(val) => val,
                Err(_e) => {
                    return HttpResponse::BadRequest().body("Field 'limit' is not a valid number");
                }
            }
        },
        None => 100
    };

    let offset = query_string.get("offset");
    let _offset = match offset {
        Some(offset_str) => {
            let offset = offset_str.parse::<i32>();
            match offset {
                Ok(val) => val,
                Err(_e) => {
                    return HttpResponse::BadRequest().body("Field 'offset' is not a valid number");
                }
            }
        },
        None => 400
    };
    let return_json_object = json!({
        "title": "Gettysburg Address",
        "message": "Four score and seven years ago our fathers brought forth on this continent a new nation conceived...",
        "user": "Abraham Lincoln",
        "timestamp": "1863.11.19",
    }).to_string(); //假装这里有查询
    HttpResponse::Ok().body(return_json_object)
}

pub async fn get_post_message(request_raw: Bytes, request: HttpRequest) -> impl Responder {
    let name = request.cookie("user");
    let name = match name {
        Some(cookie) => String::from(cookie.value()),
        None => String::from("Unknown")
    };
    //假装这里有数据库操作验证这个用户的身份
    if let Ok(text) = String::from_utf8(request_raw.to_vec()) {
        match serde_json::from_str::<PostData>(&text) {
            Ok(post_data) => {
                if post_data.title.len() > 100 {
                    return HttpResponse::BadRequest().body("Field 'title' Too Long");
                } else if post_data.content.len() > 400 {
                    return HttpResponse::BadRequest().body("Field 'content' Too Long");
                } else {
                    //假装这里有添加数据库内容的代码
                    return HttpResponse::Ok().body("Successfully created.");
                }
            },
            Err(_) => return HttpResponse::BadRequest().body("Json Parse Error")
        }
    }
    HttpResponse::BadRequest().body("Unknown Error")
}

#[get("/api/clearmessage")]
pub async fn clear_message() -> impl Responder {
    //假装这里操作了数据库清除了所有信息
    HttpResponse::Ok().body("Successfully cleared messages.")
}