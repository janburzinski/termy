use std::{env, net::SocketAddr};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};
use thiserror::Error;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

const SESSION_COOKIE_NAME: &str = "theme_store_session";

#[derive(Clone)]
struct AppState {
    db: PgPool,
    auth: AuthConfig,
    http_client: Client,
}

#[derive(Clone)]
struct AuthConfig {
    github_client_id: String,
    github_client_secret: String,
    github_redirect_uri: String,
    session_cookie_secure: bool,
    session_ttl_hours: i64,
    post_auth_redirect: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let _ = dotenvy::dotenv();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set for termy_theme_store"))?;
    let bind_addr = env::var("THEME_STORE_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let auth = AuthConfig {
        github_client_id: env::var("GITHUB_CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_ID must be set"))?,
        github_client_secret: env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_SECRET must be set"))?,
        github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
            .map_err(|_| anyhow::anyhow!("GITHUB_REDIRECT_URI must be set"))?,
        session_cookie_secure: env::var("SESSION_COOKIE_SECURE")
            .ok()
            .map(|value| parse_bool_env(&value, "SESSION_COOKIE_SECURE"))
            .transpose()?
            .unwrap_or(false),
        session_ttl_hours: env::var("SESSION_TTL_HOURS")
            .ok()
            .map(|value| parse_i64_env(&value, "SESSION_TTL_HOURS"))
            .transpose()?
            .unwrap_or(24 * 7),
        post_auth_redirect: env::var("POST_AUTH_REDIRECT").ok(),
    };

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let state = AppState {
        db: pool,
        auth,
        http_client: Client::builder().build()?,
    };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let socket_addr: SocketAddr = listener.local_addr()?;
    info!(%socket_addr, "theme store API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/github/login", get(auth_github_login))
        .route("/auth/github/callback", get(auth_github_callback))
        .route("/auth/me", get(auth_me))
        .route("/auth/logout", axum::routing::post(auth_logout))
        .route("/themes", get(list_themes).post(create_theme))
        .route("/themes/{slug}", get(get_theme).patch(update_theme))
        .route(
            "/themes/{slug}/versions",
            get(list_theme_versions).post(publish_theme_version),
        )
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(?err, "failed to install Ctrl+C signal handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                error!(?err, "failed to install terminate signal handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termy_theme_store=info".into()),
        )
        .compact()
        .init();
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Theme {
    id: Uuid,
    name: String,
    slug: String,
    description: String,
    latest_version: Option<String>,
    file_key: Option<String>,
    github_username_claim: String,
    github_user_id_claim: Option<i64>,
    is_public: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct ThemeVersion {
    id: Uuid,
    theme_id: Uuid,
    version: String,
    file_key: String,
    changelog: String,
    checksum_sha256: Option<String>,
    created_by: Option<String>,
    published_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct AuthUser {
    id: Uuid,
    github_user_id: i64,
    github_login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateThemeRequest {
    name: String,
    slug: String,
    description: Option<String>,
    latest_version: Option<String>,
    file_key: Option<String>,
    github_username_claim: Option<String>,
    is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateThemeRequest {
    name: Option<String>,
    description: Option<String>,
    is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublishThemeVersionRequest {
    version: String,
    file_key: String,
    changelog: Option<String>,
    checksum_sha256: Option<String>,
    created_by: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthGithubLoginQuery {
    redirect_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthGithubCallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GithubUserResponse {
    id: i64,
    login: String,
    avatar_url: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ThemeWithVersionsResponse {
    theme: Theme,
    versions: Vec<ThemeVersion>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishThemeVersionResponse {
    theme: Theme,
    version: ThemeVersion,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn auth_github_login(
    State(state): State<AppState>,
    Query(query): Query<AuthGithubLoginQuery>,
) -> Result<Redirect, ApiError> {
    sqlx::query("DELETE FROM oauth_state WHERE expires_at < NOW()")
        .execute(&state.db)
        .await?;

    let oauth_state = Uuid::new_v4().simple().to_string();
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query("INSERT INTO oauth_state (state, redirect_to, expires_at) VALUES ($1, $2, $3)")
        .bind(&oauth_state)
        .bind(query.redirect_to)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

    let mut authorize_url =
        Url::parse("https://github.com/login/oauth/authorize").map_err(|_| {
            ApiError::ExternalAuth("failed to build GitHub authorization URL".to_string())
        })?;

    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", &state.auth.github_client_id)
        .append_pair("redirect_uri", &state.auth.github_redirect_uri)
        .append_pair("scope", "read:user")
        .append_pair("state", &oauth_state);

    Ok(Redirect::to(authorize_url.as_str()))
}

async fn auth_github_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthGithubCallbackQuery>,
) -> Result<Response, ApiError> {
    let redirect_to = sqlx::query_scalar::<_, Option<String>>(
        "DELETE FROM oauth_state WHERE state = $1 AND expires_at > NOW() RETURNING redirect_to",
    )
    .bind(&query.state)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::BadRequest("invalid or expired OAuth state".to_string()))?;

    let token_response = state
        .http_client
        .post("https://github.com/login/oauth/access_token")
        .header(header::ACCEPT, "application/json")
        .form(&[
            ("client_id", state.auth.github_client_id.as_str()),
            ("client_secret", state.auth.github_client_secret.as_str()),
            ("code", query.code.as_str()),
            ("redirect_uri", state.auth.github_redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|err| {
            ApiError::ExternalAuth(format!("failed to call GitHub token endpoint: {err}"))
        })?;

    if !token_response.status().is_success() {
        return Err(ApiError::ExternalAuth(format!(
            "GitHub token exchange failed with status {}",
            token_response.status()
        )));
    }

    let token_payload = token_response
        .json::<GithubTokenResponse>()
        .await
        .map_err(|err| {
            ApiError::ExternalAuth(format!("invalid token response from GitHub: {err}"))
        })?;

    let github_user_response = state
        .http_client
        .get("https://api.github.com/user")
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, "termy-theme-store")
        .bearer_auth(&token_payload.access_token)
        .send()
        .await
        .map_err(|err| ApiError::ExternalAuth(format!("failed to fetch GitHub user: {err}")))?;

    if !github_user_response.status().is_success() {
        return Err(ApiError::ExternalAuth(format!(
            "GitHub user fetch failed with status {}",
            github_user_response.status()
        )));
    }

    let github_user = github_user_response
        .json::<GithubUserResponse>()
        .await
        .map_err(|err| {
            ApiError::ExternalAuth(format!("invalid user response from GitHub: {err}"))
        })?;

    let auth_user = sqlx::query_as::<_, AuthUser>(
        r#"
        INSERT INTO user_account (github_user_id, github_login, avatar_url, name)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (github_user_id)
        DO UPDATE SET
            github_login = EXCLUDED.github_login,
            avatar_url = EXCLUDED.avatar_url,
            name = EXCLUDED.name,
            updated_at = NOW()
        RETURNING id, github_user_id, github_login
        "#,
    )
    .bind(github_user.id)
    .bind(github_user.login)
    .bind(github_user.avatar_url)
    .bind(github_user.name)
    .fetch_one(&state.db)
    .await?;

    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::hours(state.auth.session_ttl_hours);

    sqlx::query("INSERT INTO user_session (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(auth_user.id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

    let cookie = build_session_cookie(
        &token,
        state.auth.session_ttl_hours,
        state.auth.session_cookie_secure,
    );

    let target = redirect_to
        .or_else(|| state.auth.post_auth_redirect.clone())
        .unwrap_or_else(|| "/themes".to_string());
    let mut response = Redirect::to(&target).into_response();

    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|_| ApiError::ExternalAuth("failed to set session cookie".to_string()))?,
    );

    Ok(response)
}

async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthUser>, ApiError> {
    let user = require_auth_user(&state, &headers).await?;
    Ok(Json(user))
}

async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(token) = extract_cookie(&headers, SESSION_COOKIE_NAME) {
        let token_hash = hash_token(&token);
        sqlx::query("DELETE FROM user_session WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&state.db)
            .await?;
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_session_cookie(state.auth.session_cookie_secure))
            .map_err(|_| ApiError::ExternalAuth("failed to clear session cookie".to_string()))?,
    );

    Ok(response)
}

async fn list_themes(State(state): State<AppState>) -> Result<Json<Vec<Theme>>, ApiError> {
    let themes = sqlx::query_as::<_, Theme>(
        r#"
        SELECT
            id,
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public,
            created_at,
            updated_at
        FROM theme
        WHERE is_public = TRUE
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(themes))
}

async fn get_theme(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Theme>, ApiError> {
    let theme = fetch_theme_by_slug(&state.db, &slug).await?;
    ensure_can_read_theme(&theme, &state, &headers).await?;
    Ok(Json(theme))
}

async fn create_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateThemeRequest>,
) -> Result<(StatusCode, Json<Theme>), ApiError> {
    let auth_user = require_auth_user(&state, &headers).await?;

    validate_slug(&request.slug)?;
    let name = parse_required_field(request.name, "name")?;
    let slug = parse_required_field(request.slug, "slug")?;

    if let Some(claimed_login) = request.github_username_claim
        && !claimed_login.eq_ignore_ascii_case(&auth_user.github_login)
    {
        return Err(ApiError::BadRequest(
            "githubUsernameClaim must match the authenticated GitHub user".to_string(),
        ));
    }

    if request.latest_version.is_some() ^ request.file_key.is_some() {
        return Err(ApiError::BadRequest(
            "latestVersion and fileKey must either both be set or both be omitted".to_string(),
        ));
    }

    let description = request.description.unwrap_or_default();
    let is_public = request.is_public.unwrap_or(true);

    let theme_result = sqlx::query_as::<_, Theme>(
        r#"
        INSERT INTO theme (
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public,
            created_at,
            updated_at
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(request.latest_version)
    .bind(request.file_key)
    .bind(auth_user.github_login)
    .bind(auth_user.github_user_id)
    .bind(is_public)
    .fetch_one(&state.db)
    .await;

    match theme_result {
        Ok(theme) => Ok((StatusCode::CREATED, Json(theme))),
        Err(err) if is_unique_violation(&err) => {
            Err(ApiError::Conflict("theme slug already exists".to_string()))
        }
        Err(err) => Err(ApiError::from(err)),
    }
}

async fn update_theme(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateThemeRequest>,
) -> Result<Json<Theme>, ApiError> {
    let auth_user = require_auth_user(&state, &headers).await?;
    let current_theme = fetch_theme_by_slug(&state.db, &slug).await?;
    ensure_theme_owner(&current_theme, &auth_user)?;

    let has_updates =
        request.name.is_some() || request.description.is_some() || request.is_public.is_some();

    if !has_updates {
        return Err(ApiError::BadRequest(
            "provide at least one field to update".to_string(),
        ));
    }

    let name = request
        .name
        .map(|value| parse_required_field(value, "name"))
        .transpose()?;

    let theme = sqlx::query_as::<_, Theme>(
        r#"
        UPDATE theme
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_public = COALESCE($4, is_public),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public,
            created_at,
            updated_at
        "#,
    )
    .bind(current_theme.id)
    .bind(name)
    .bind(request.description)
    .bind(request.is_public)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(theme))
}

async fn publish_theme_version(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PublishThemeVersionRequest>,
) -> Result<(StatusCode, Json<PublishThemeVersionResponse>), ApiError> {
    let auth_user = require_auth_user(&state, &headers).await?;
    let current_theme = fetch_theme_by_slug(&state.db, &slug).await?;
    ensure_theme_owner(&current_theme, &auth_user)?;

    let version = parse_required_field(request.version, "version")?;
    let file_key = parse_required_field(request.file_key, "fileKey")?;

    let mut transaction = state.db.begin().await?;

    let version_result = sqlx::query_as::<_, ThemeVersion>(
        r#"
        INSERT INTO theme_version (
            theme_id,
            version,
            file_key,
            changelog,
            checksum_sha256,
            created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            theme_id,
            version,
            file_key,
            changelog,
            checksum_sha256,
            created_by,
            published_at,
            created_at
        "#,
    )
    .bind(current_theme.id)
    .bind(version.clone())
    .bind(file_key.clone())
    .bind(request.changelog.unwrap_or_default())
    .bind(request.checksum_sha256)
    .bind(request.created_by)
    .fetch_one(&mut *transaction)
    .await;

    let inserted_version = match version_result {
        Ok(inserted) => inserted,
        Err(err) if is_unique_violation(&err) => {
            return Err(ApiError::Conflict(
                "this version already exists for the theme".to_string(),
            ));
        }
        Err(err) => return Err(ApiError::from(err)),
    };

    let updated_theme = sqlx::query_as::<_, Theme>(
        r#"
        UPDATE theme
        SET
            latest_version = $1,
            file_key = $2,
            updated_at = NOW()
        WHERE id = $3
        RETURNING
            id,
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public,
            created_at,
            updated_at
        "#,
    )
    .bind(version)
    .bind(file_key)
    .bind(current_theme.id)
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(PublishThemeVersionResponse {
            theme: updated_theme,
            version: inserted_version,
        }),
    ))
}

async fn list_theme_versions(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ThemeWithVersionsResponse>, ApiError> {
    let theme = fetch_theme_by_slug(&state.db, &slug).await?;
    ensure_can_read_theme(&theme, &state, &headers).await?;

    let versions = sqlx::query_as::<_, ThemeVersion>(
        r#"
        SELECT
            id,
            theme_id,
            version,
            file_key,
            changelog,
            checksum_sha256,
            created_by,
            published_at,
            created_at
        FROM theme_version
        WHERE theme_id = $1
        ORDER BY published_at DESC, created_at DESC
        "#,
    )
    .bind(theme.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ThemeWithVersionsResponse { theme, versions }))
}

async fn fetch_theme_by_slug(pool: &PgPool, slug: &str) -> Result<Theme, ApiError> {
    let theme = sqlx::query_as::<_, Theme>(
        r#"
        SELECT
            id,
            name,
            slug,
            description,
            latest_version,
            file_key,
            github_username_claim,
            github_user_id_claim,
            is_public,
            created_at,
            updated_at
        FROM theme
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("theme not found".to_string()))?;

    Ok(theme)
}

async fn require_auth_user(state: &AppState, headers: &HeaderMap) -> Result<AuthUser, ApiError> {
    let token = extract_cookie(headers, SESSION_COOKIE_NAME)
        .ok_or_else(|| ApiError::Unauthorized("authentication required".to_string()))?;
    let token_hash = hash_token(&token);

    let user = sqlx::query_as::<_, AuthUser>(
        r#"
        SELECT ua.id, ua.github_user_id, ua.github_login
        FROM user_session us
        JOIN user_account ua ON ua.id = us.user_id
        WHERE us.token_hash = $1 AND us.expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::Unauthorized("invalid or expired session".to_string()))?;

    Ok(user)
}

async fn ensure_can_read_theme(
    theme: &Theme,
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), ApiError> {
    if theme.is_public {
        return Ok(());
    }

    let user = require_auth_user(state, headers).await?;
    ensure_theme_owner(theme, &user)
}

fn ensure_theme_owner(theme: &Theme, user: &AuthUser) -> Result<(), ApiError> {
    let owner_matches = if let Some(github_user_id_claim) = theme.github_user_id_claim {
        github_user_id_claim == user.github_user_id
    } else {
        theme
            .github_username_claim
            .eq_ignore_ascii_case(&user.github_login)
    };

    if owner_matches {
        Ok(())
    } else {
        Err(ApiError::Forbidden("you do not own this theme".to_string()))
    }
}

fn parse_required_field(value: String, field_name: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "{field_name} is required and cannot be empty"
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_slug(slug: &str) -> Result<(), ApiError> {
    let slug = slug.trim();
    let is_valid = !slug.is_empty()
        && slug.len() <= 64
        && slug
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-');

    if !is_valid {
        return Err(ApiError::BadRequest(
            "slug must be lowercase alphanumeric plus hyphens and cannot start/end with '-'"
                .to_string(),
        ));
    }

    Ok(())
}

fn parse_bool_env(value: &str, key: &str) -> anyhow::Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(anyhow::anyhow!(
            "{key} must be one of: true/false/1/0/yes/no/on/off"
        )),
    }
}

fn parse_i64_env(value: &str, key: &str) -> anyhow::Result<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("{key} must be a valid integer"))?;
    if parsed <= 0 {
        return Err(anyhow::anyhow!("{key} must be > 0"));
    }
    Ok(parsed)
}

fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    for cookie_header in headers.get_all(header::COOKIE) {
        let cookie_value = match cookie_header.to_str() {
            Ok(value) => value,
            Err(_) => continue,
        };
        for part in cookie_value.split(';') {
            let mut parts = part.trim().splitn(2, '=');
            let Some(name) = parts.next() else {
                continue;
            };
            let Some(value) = parts.next() else {
                continue;
            };
            if name == cookie_name {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn build_session_cookie(token: &str, ttl_hours: i64, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{name}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}",
        name = SESSION_COOKIE_NAME,
        max_age = ttl_hours * 3600,
        secure = secure_flag,
    )
}

fn clear_session_cookie(secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}",
        name = SESSION_COOKIE_NAME,
        secure = secure_flag,
    )
}

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("23505")
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    ExternalAuth(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::ExternalAuth(_) => StatusCode::BAD_GATEWAY,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match self {
            Self::Database(error) => {
                error!(?error, "database request failed");
                ErrorBody {
                    error: "internal server error".to_string(),
                }
            }
            other => ErrorBody {
                error: other.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::validate_slug;

    #[test]
    fn accepts_valid_slug() {
        assert!(validate_slug("tokyo-night").is_ok());
    }

    #[test]
    fn rejects_invalid_slug() {
        assert!(validate_slug("Tokyo-Night").is_err());
        assert!(validate_slug("-tokyo-night").is_err());
        assert!(validate_slug("tokyo-night-").is_err());
        assert!(validate_slug("tokyo_night").is_err());
    }
}
