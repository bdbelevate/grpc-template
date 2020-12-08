use mongodb::bson::{oid::ObjectId, Bson};
use serde::{
    de, de::MapAccess, de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serializer,
};
use std::fmt;

use crate::api::error::ServiceError;

// TODO: Ideally we figure out how to have prost just convert anything with the name id into an ID
/// An ID as defined by the GraphQL specification
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ID {
    ObjectId(ObjectId),
    String(String),
    Int64(i64),
}

impl ID {
    pub fn from_string<S: Into<String>>(value: S) -> Result<Self, ServiceError> {
        let s: String = value.into();
        if s.starts_with("$oid:") {
            match ObjectId::with_string(&s[5..]) {
                Ok(oid) => Ok(ID::ObjectId(oid)),
                Err(_) => Err(ServiceError::ParseError(
                    "Unable to parse object id".to_string(),
                )),
            }
        } else if s.starts_with("$i:") {
            match s[3..].parse::<i64>() {
                Ok(i) => Ok(ID::Int64(i)),
                Err(_) => Err(ServiceError::ParseError(
                    "Unable to parse integer id".to_string(),
                )),
            }
        } else {
            Ok(ID::String(s))
        }
    }

    pub fn to_bson(&self) -> Bson {
        match self {
            ID::ObjectId(o) => Bson::ObjectId(o.clone()),
            ID::String(s) => Bson::String(s.to_string()),
            ID::Int64(i) => Bson::Int64(i.clone()),
        }
    }
}

pub fn serialize<S>(s: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if s.starts_with("$oid:") {
        match ObjectId::with_string(&s[5..]) {
            Ok(oid) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("$oid", &oid.to_string())?;
                map.end()
            }
            Err(_) => serializer.serialize_str(s),
        }
    } else if s.starts_with("$i:") {
        match s[3..].parse::<i64>() {
            Ok(i) => serializer.serialize_i64(i),
            Err(_) => serializer.serialize_str(s),
        }
    } else {
        serializer.serialize_str(s)
    }
}

struct IDVisitor;
impl<'de> Visitor<'de> for IDVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unable to parse ID - was not Bson or Json string")
    }

    fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        // send this back into the Bson deserializer
        Ok(with_bson(&Bson::deserialize(
            de::value::MapAccessDeserializer::new(access),
        )?))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("$i:{}", v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("$i:{}", v))
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(IDVisitor)
}

fn with_bson(value: &Bson) -> String {
    match value.into() {
        Bson::String(s) => s,
        Bson::ObjectId(o) => format!("$oid:{}", o.to_hex()),
        Bson::Int64(i) => format!("$i:{}", i),
        _ => panic!("Invalid id type used {:?}", value),
    }
}
