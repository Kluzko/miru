use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genre {
    pub id: Uuid,
    pub mal_id: Option<i32>,
    pub name: String,
}

impl Genre {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            mal_id: None,
            name,
        }
    }

    pub fn with_mal_id(mut self, mal_id: i32) -> Self {
        self.mal_id = Some(mal_id);
        self
    }
}

impl std::fmt::Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
