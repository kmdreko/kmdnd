use crate::typedid::{TypedId, TypedIdMarker};

pub type UserId = TypedId<User>;

#[derive(Clone, Debug)]
pub struct User;

impl TypedIdMarker for User {
    fn tag() -> &'static str {
        "USR"
    }
}
