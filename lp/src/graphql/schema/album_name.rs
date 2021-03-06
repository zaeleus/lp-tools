use juniper::ID;

use crate::models::AlbumName;

#[juniper::object]
impl AlbumName {
    fn id(&self) -> ID {
        ID::from(format!("{}", self.id))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn locale(&self) -> &str {
        &self.locale
    }

    fn is_default(&self) -> bool {
        self.is_default
    }

    fn is_original(&self) -> bool {
        self.is_original
    }
}
