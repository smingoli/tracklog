use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    pub internal_code: String,
    pub title: String,
    pub status: String,
    pub description: Option<String>,
    pub lyrics: Option<String>,
    pub notes: Option<String>,
    pub bpm: Option<i64>,
    pub key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInput {
    pub internal_code: String,
    pub title: String,
    pub status: String,
    pub description: Option<String>,
    pub lyrics: Option<String>,
    pub notes: Option<String>,
    pub bpm: Option<i64>,
    pub key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackListRow {
    pub id: i64,
    pub internal_code: String,
    pub title: String,
    pub status: String,
    pub description: Option<String>,
    pub lyrics: Option<String>,
    pub notes: Option<String>,
    pub bpm: Option<i64>,
    pub key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub assigned_release_id: Option<i64>,
    pub assigned_release_title: Option<String>,
    pub availability: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: i64,
    pub internal_code: String,
    pub title: String,
    pub r#type: String,
    pub status: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub track_count: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInput {
    pub internal_code: String,
    pub title: String,
    pub r#type: String,
    pub status: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseTrackRow {
    pub track_order: i64,
    pub track_id: i64,
    pub internal_code: String,
    pub title: String,
    pub status: String,
    pub description: Option<String>,
    pub lyrics: Option<String>,
    pub bpm: Option<i64>,
    pub key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_tracks: i64,
    pub available_tracks: i64,
    pub total_releases: i64,
    pub recent_tracks: Vec<Track>,
    pub recent_releases: Vec<Release>,
}
