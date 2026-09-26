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

use surrealdb::RecordId;

pub fn serialize_id<S>(
    id: &Option<RecordId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match id {
        Some(t) => {
            let s = String::try_from(t.key().clone())
                .unwrap_or_else(|_| format!("{:?}", t.key()));
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}
