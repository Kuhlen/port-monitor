use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateCheck {
    UpToDate,
    Available { version: String },
}

// No Send bound, deliberate. Never dyn'd.
#[allow(async_fn_in_trait)]
pub trait UpdateApi {
    async fn check_update(&self) -> Result<UpdateCheck, AppError>;
    async fn install_update(&self) -> Result<(), AppError>;
}
