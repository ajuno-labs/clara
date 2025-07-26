use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub folders: Vec<Folder>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub id: String,   // e.g. "F1"
    pub name: String, // display name
    pub lists: Vec<List>,
}

// List is empty for now; tasks stay as before
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct List {
    pub id: String,   // e.g. "F1-L1"
    pub name: String,
    pub tasks: Vec<crate::task::Task>,
}