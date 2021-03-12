#[cfg(test)]
mod server_test {
    use core::panic;

    use actix_web::{App, dev::{Body, ResponseBody}, http::{Cookie, StatusCode}, test::{self}, web::{self, Bytes}};
    use dotenv::dotenv;
    use chrono::Local;
    use crate::Pool;
    use crate::operations;
    use crate::models::*;
    use diesel::{RunQueryDsl, SqliteConnection, prelude::*, r2d2::{ConnectionManager}};
    use crate::config::ConnectionOptions;
    fn init_test() -> Pool {
        use crate::schema::user::dsl::*;
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .expect("Unable to locate the database.\nTry setting the 'DATABASE_URL' variable.");
        let database = Pool::builder()
            .max_size(16)
            .connection_customizer(Box::new(ConnectionOptions {
                enable_wal: true,
                enable_foreign_keys: false,
                busy_timeout: Some(std::time::Duration::from_secs(30)),
            }))
            .build(ConnectionManager::<SqliteConnection>::new(database_url))
            .expect("Unable to open the database.");
        let db_connection = database.get().unwrap();
        let alice = PostUser {
            id: 1,
            name: String::from("Alice"),
            register_date: Local::now().naive_local(),
        };
        let bob = PostUser {
            id: 2,
            name: String::from("Bob"),
            register_date: Local::now().naive_local(),
        };
        let _ = diesel::insert_into(user)
            .values(&alice)
            .execute(&db_connection);
        let _ = diesel::insert_into(user)
            .values(&bob)
            .execute(&db_connection);
        let hi = PostMessage {
            id: 1,
            user: user.filter(name.eq("Alice")).first::<PostUser>(&db_connection).unwrap().id,
            title: String::from("Hi"),
            content: String::from("Hello, world!"),
            pub_date: Local::now().naive_local(),
        };
        let this_is_a_title = PostMessage {
            id: 2,
            user: user.filter(name.eq("Bob")).first::<PostUser>(&db_connection).unwrap().id,
            title: String::from("This is a title"),
            content: String::from("This is my content"),
            pub_date: Local::now().naive_local(),
        };
        let _ = diesel::insert_into(crate::schema::message::dsl::message)
            .values(&hi)
            .execute(&db_connection);
        let _ = diesel::insert_into(crate::schema::message::dsl::message)
            .values(&this_is_a_title)
            .execute(&db_connection);
        database
    }
    fn end_test(database: Pool) {
        let db_connection = database.get().unwrap();
        let _ = diesel::delete(crate::schema::user::dsl::user)
            .execute(&db_connection);
        let _ = diesel::delete(crate::schema::message::dsl::message)
            .execute(&db_connection);
    }

    #[actix_rt::test]
    async fn test_can_reach() {
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .service(operations::get_message)
        ).await;
        let req = test::TestRequest::get().uri("/api/message").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
    #[actix_rt::test]
    async fn test_message_can_be_fetched() {
        use serde_json::json;
        let database = init_test();
        let db_connection = database.get().unwrap();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .service(operations::get_message)
        ).await;
        let req = test::TestRequest::get().uri("/api/message").to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status() == StatusCode::OK);
        let offset = 0;
        let limit = 100;
        let expected_result: Vec<String> = crate::schema::message::dsl::message
            .order(crate::schema::message::dsl::id)
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<PostMessage>(&db_connection)
            .unwrap()
            .into_iter()
            .map(|x| MessageJson::from(x))
            .map(|x| json!(x).to_string())
            .collect();
        end_test(database);
        assert_eq!(resp.status(), StatusCode::OK);
        let expected_result = json!(expected_result).to_string();
        let actual_result = match resp.take_body() {
            ResponseBody::Body(b) => {
                if let Body::Bytes(bytes) = b {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            },
            ResponseBody::Other(body) => {
                if let Body::Bytes(bytes) = body {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            }
        };
        assert_eq!(actual_result, expected_result);
    }
    #[actix_rt::test]
    async fn test_add_new_message() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let db_connection = database.get().unwrap();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from("Test title");
        let content = String::from("My test message");
        let user = String::from("Student");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
            content: content,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let actual_result = match resp.take_body() {
            ResponseBody::Body(b) => {
                if let Body::Bytes(bytes) = b {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            },
            ResponseBody::Other(body) => {
                if let Body::Bytes(bytes) = body {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            }
        };
        if let Err(_) = crate::schema::user::dsl::user
            .filter(crate::schema::user::dsl::name.eq("Student"))
            .first::<PostUser>(&db_connection) {
                end_test(database);
                panic!("No user named 'Student' found, panicking.");
            }
        if let Err(_) = crate::schema::message::dsl::message
            .filter(crate::schema::message::dsl::title.eq("Test title"))
            .filter(crate::schema::message::dsl::content.eq("My test message"))
            .first::<PostMessage>(&db_connection) {
                end_test(database);
                panic!("No message found, panicking.");
            } 
        end_test(database);
        assert_eq!(actual_result, "message was sent successfully");
    }
    #[actix_rt::test]
    async fn test_title_lost() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let content = String::from("My test message");
        let user = String::from("Student");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            content: content,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_content_lost() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from("Test title");
        let user = String::from("Student");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_user_name_too_long() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from("Test title");
        let content = String::from("My test message");
        let user = String::from("ThisUserHasTooLongANameSoThatTheServerWillCrash");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
            content: content,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_content_too_long() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from("The Rustonomicon");
        let content = String::from(
            "Instead of the programs I had hoped for, there came only a shuddering blackness and ineffable loneliness; 
            and I saw at last a fearful truth which no one had ever dared to breathe before — the unwhisperable secret 
            of secrets — The fact that this language of stone and stridor is not a sentient perpetuation of Rust as London 
            is of Old London and Paris of Old Paris, but that it is in fact quite unsafe, its sprawling body imperfectly 
            embalmed and infested with queer animate things which have nothing to do with it as it was in compilation."
        );
        let user = String::from("Student");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
            content: content,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    #[actix_rt::test]
    async fn test_title_too_long() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from(
            "Rustonomicon\n
            The Dark Arts of Advanced and Unsafe Rust Programming
            Nicknamed \"theNomicon.\"\n
            This book digs into all the awful details that are necessary to understand in order to write 
            correct Unsafe Rust programs. Due to the nature of this problem, it may lead to unleashing 
            untold horrors that shatter your psyche into a billion infinitesimal fragments of despair."
        );
        let content = String::from("My test message");
        let user = String::from("Student");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
            content: content,
        })
        .cookie(Cookie::new("user", user))
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_unknown_user() {
        use serde::{Deserialize, Serialize};
        let database = init_test();
        let db_connection = database.get().unwrap();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .route("/api/message", web::post().to(operations::get_post_message))
        ).await;
        let title = String::from("Test title");
        let content = String::from("My test message");
        #[derive(Deserialize, Serialize)]
        struct TempJson {
            title: String,
            content: String,
        };
        let req = test::TestRequest::post().uri("/api/message").set_json(&TempJson {
            title: title,
            content: content,
        })
        .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let actual_result = match resp.take_body() {
            ResponseBody::Body(b) => {
                if let Body::Bytes(bytes) = b {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            },
            ResponseBody::Other(body) => {
                if let Body::Bytes(bytes) = body {
                    bytes
                } else {
                    Bytes::from("NO")
                }
            }
        };
        if let Err(_) = crate::schema::user::dsl::user
            .filter(crate::schema::user::dsl::name.eq("Unknown"))
            .first::<PostUser>(&db_connection) {
                end_test(database);
                panic!("No user named 'Unknown' found, panicking.");
            }
        if let Err(_) = crate::schema::message::dsl::message
            .filter(crate::schema::message::dsl::title.eq("Test title"))
            .filter(crate::schema::message::dsl::content.eq("My test message"))
            .first::<PostMessage>(&db_connection) {
                end_test(database);
                panic!("No message found, panicking.");
            } 
        end_test(database);
        assert_eq!(actual_result, "message was sent successfully");
    }

    #[actix_rt::test]
    async fn test_clear_message() {
        use crate::schema::message::dsl::*;
        let database = init_test();
        let db_connection = database.get().unwrap();
        let mut app = test::init_service(
            App::new()
            .data(database.clone())
            .service(operations::clear_message)
        ).await;
        let req = test::TestRequest::get().uri("/api/clearmessage").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        if let Ok(_) = message.first::<PostMessage>(&db_connection) {
            panic!("Database is not cleared!");
        }
    }
}