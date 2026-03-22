use std::ops::Deref;

use crate::types::BaseId;

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct Id {
    pub value: String,
}

impl Id {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn uuid() -> Self {
        Self {
            value: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl BaseId for Id {
    fn get_id(&self) -> &str {
        &self.value
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.value
    }
}
