use actix_web::{delete, get, patch, put, web, HttpRequest, HttpResponse};
use chrono::Date;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::vec::Vec;
use uuid::Uuid;

use crate::error::{APIError, Result};
use crate::model::lesson::Lesson;
use crate::model::permission::{LessonPermission, PermissionType};
use crate::model::repeat::Repeat;
use crate::token::authorize_headers;
use crate::util::deserialize_optional_field;

#[get("/lesson/{id}")]
pub async fn get_lesson(
    db: web::Data<PgPool>,
    lesson_id: web::Path<Uuid>,
    request: HttpRequest,
) -> Result<Lesson> {
    let account_id = authorize_headers(request.headers())?;

    let lesson_id = lesson_id.into_inner();

    let lesson = Lesson::of_user(db.get_ref(), lesson_id)
        .await?
        .ok_or(APIError::LessonDosNotExist)?;

    LessonPermission::type_of_entity(db.get_ref(), &account_id, &lesson_id)
        .await?
        .ok_or(APIError::NoReadAccess)?;

    Ok(lesson.into())
}

#[derive(Deserialize)]
pub struct LessonCreateRequest {
    title: String,
    description: Option<String>,
    repeats: Vec<Repeat>,
}

#[put("/lesson")]
pub async fn put_lesson(
    db: web::Data<PgPool>,
    lesson: web::Json<LessonCreateRequest>,
    request: HttpRequest,
) -> Result<Lesson> {
    let account_id = authorize_headers(request.headers())?;

    let LessonCreateRequest {
        title,
        description,
        repeats,
    } = lesson.into_inner();
    Ok(
        Lesson::create(db.get_ref(), title, description, repeats, &account_id)
            .await?
            .into(),
    )
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct LessonUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeats: Option<Vec<Repeat>>,
}

#[patch("/lesson/{id}")]
pub async fn patch_lesson(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    patch: web::Json<LessonUpdateRequest>,
    request: HttpRequest,
) -> std::result::Result<HttpResponse, APIError> {
    let lesson_id = id.into_inner();
    let account_id = authorize_headers(request.headers())?;

    if let Some(PermissionType::ReadWrite) =
        LessonPermission::type_of_entity(db.get_ref(), &account_id, &lesson_id).await?
    {
        let LessonUpdateRequest {
            title,
            repeats,
            description,
        } = patch.into_inner();
        Lesson::update(db.get_ref(), &lesson_id, &title, &repeats, &description).await?;
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(APIError::NoWriteAccess)
    }
}

#[delete("/lesson/{id}")]
pub async fn delete_lesson(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    request: HttpRequest,
) -> std::result::Result<HttpResponse, APIError> {
    let lesson_id = id.into_inner();
    let account_id = authorize_headers(request.headers())?;

    if let Some(PermissionType::ReadWrite) =
        LessonPermission::type_of_entity(db.get_ref(), &account_id, &lesson_id).await?
    {
        Lesson::delete(db.get_ref(), &lesson_id).await?;
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(APIError::NoWriteAccess)
    }
}

pub fn configure_lesson_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_lesson)
        .service(put_lesson)
        .service(patch_lesson)
        .service(delete_lesson);
}
