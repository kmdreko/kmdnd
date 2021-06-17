use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use mongodb::bson::Bson;
use serde::{de::Error, Deserialize, Serialize};
use uuid::Uuid;

pub trait TypedIdMarker {
    fn tag() -> &'static str;
}

pub struct TypedId<T: TypedIdMarker>(Uuid, PhantomData<T>);

impl<T: TypedIdMarker> TypedId<T> {
    pub fn new() -> TypedId<T> {
        TypedId(Uuid::new_v4(), PhantomData)
    }
}

impl<T: TypedIdMarker> Copy for TypedId<T> {}

impl<T: TypedIdMarker> Clone for TypedId<T> {
    fn clone(&self) -> TypedId<T> {
        *self
    }
}

impl<T: TypedIdMarker> Display for TypedId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}-{:X}", T::tag(), self.0)
    }
}

impl<T: TypedIdMarker> Debug for TypedId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Display::fmt(self, f)
    }
}

impl<T: TypedIdMarker> FromStr for TypedId<T> {
    type Err = TypedIdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = s.find('-').ok_or(TypedIdParseError::InvalidFormat)?;
        let (tag, id) = s.split_at(index);

        if tag != T::tag() {
            return Err(TypedIdParseError::InvalidTag);
        }

        let uuid = Uuid::from_str(&id[1..]).map_err(|_| TypedIdParseError::InvalidUuid)?;

        Ok(TypedId(uuid, PhantomData))
    }
}

impl<T: TypedIdMarker> Serialize for TypedId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de, T: TypedIdMarker> Deserialize<'de> for TypedId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TypedId::from_str(&s).map_err(|e| D::Error::custom(e))
    }
}

impl<T: TypedIdMarker> From<TypedId<T>> for Bson {
    fn from(id: TypedId<T>) -> Bson {
        id.to_string().into()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypedIdParseError {
    InvalidFormat,
    InvalidTag,
    InvalidUuid,
}

impl Display for TypedIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, f)
    }
}
