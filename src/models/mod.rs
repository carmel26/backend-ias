pub mod user;
pub mod settings;
pub mod assessment;
pub mod question;
pub mod analysis;
pub mod report;

pub use user::*;
pub use settings::*;
pub use assessment::*;
pub use question::*;
pub use analysis::*;
pub use report::*;

use serde::Serializer; 

use surrealdb::types::{RecordId};

pub fn serialize_id<S>(
    id: &Option<RecordId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match id {
        Some(t) => match &t.key {
            surrealdb::types::RecordIdKey::String(key) => {
                serializer.serialize_str(key)
            }
            key => serializer.serialize_str(&format!("{:?}", key)),
        },
        None => serializer.serialize_none(),
    }
}
