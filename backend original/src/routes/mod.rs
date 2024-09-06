use actix_web::web;
use sync::sync_handler;

pub mod sync;

pub fn sync_route() -> actix_web::Scope {
    web::scope("/api").service(sync_handler)
}
