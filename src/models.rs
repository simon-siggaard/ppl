use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: i64,
    pub name: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub team: Option<String>,
    pub department: Option<String>,
    pub job_title: Option<String>,
    pub birthday: Option<String>,
    pub employment_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub id: i64,
    pub person_id: i64,
    pub rel_type: String,
    pub name: String,
    pub birthday: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub person_id: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    pub id: i64,
    pub person_id: i64,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct DateEvent {
    pub person_id: i64,
    pub person_name: String,
    pub event_type: String,
    pub event_date: String,
    pub subject_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonDetail {
    pub person: Person,
    pub relationships: Vec<Relationship>,
    pub tags: Vec<String>,
    pub notes: Vec<Note>,
    pub custom_fields: Vec<CustomField>,
}
