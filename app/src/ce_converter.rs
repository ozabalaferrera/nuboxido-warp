use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use cloudevents::event::{AttributeValue, UriReference};
use cloudevents::event::{EventBuilderError, ExtensionValue};
use cloudevents::{AttributesWriter, Data, Event};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::error::Error;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub source: UriReference,
    pub datacontenttype: Option<String>,
    pub dataschema: Option<Url>,
    pub subject: Option<String>,
    pub time: Option<DateTime<Utc>>,
}

pub fn try_from_event<T>(event: Event) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    Ok(serde_json::from_value(Value::try_from(InternalEvent(
        event,
    ))?)?)
}

pub fn try_to_event<T>(obj: T) -> Result<Event, Box<dyn Error>>
where
    T: Serialize,
{
    Ok(InternalEvent::try_from(serde_json::to_value(obj)?)?.0)
}

#[derive(Debug)]
pub struct InternalEvent(pub Event);

impl TryFrom<InternalEvent> for Value {
    type Error = Box<dyn Error>;
    fn try_from(event: InternalEvent) -> Result<Value, Self::Error> {
        let mut event = event.0;
        let mut root_map: Map<String, Value> = Map::new();

        root_map.extend(event.iter().map(|(k, a)| {
            (
                k.to_owned(),
                match a {
                    AttributeValue::SpecVersion(s) => Value::String(s.to_string()),
                    AttributeValue::Boolean(b) => Value::Bool(*b),
                    AttributeValue::Integer(i) => Value::Number((*i).into()),
                    AttributeValue::String(s) => Value::String(s.to_owned()),
                    AttributeValue::URIRef(r) => Value::String(r.to_owned()),
                    AttributeValue::URI(u) => Value::String(u.to_string()),
                    AttributeValue::Time(t) => Value::String(t.to_rfc3339()),
                },
            )
        }));

        let data: Value = match event.take_data() {
            (_, _, Some(Data::Json(value))) => value,
            (_, _, Some(Data::String(string))) => Value::from(string.to_owned()),
            (_, _, Some(Data::Binary(binary))) => {
                if root_map
                    .get("datacontenttype")
                    .is_some_and(|v| v.as_str().is_some_and(|dct| dct.contains("json")))
                {
                    Value::from_str(String::from_utf8(binary.to_owned())?.as_str())?
                } else {
                    Value::from(general_purpose::STANDARD.encode(binary))
                }
            }
            (_, _, None) => Value::Null,
        };

        root_map.insert("data".to_owned(), data);
        Ok(Value::Object(root_map))
    }
}

impl TryFrom<Value> for InternalEvent {
    type Error = Box<dyn Error>;
    fn try_from(mut value: Value) -> Result<InternalEvent, Self::Error> {
        let value = value
            .as_object_mut()
            .ok_or("Expected Value::Object to convert to Event.")?;

        let mut event = Event::default();
        let id = remove_required_string(value, "id")?;
        let ty = remove_required_string(value, "type")?;
        let source = remove_required_string(value, "source")?;
        let subject = remove_optional_string(value, "subject");
        let datacontenttype = remove_optional_string(value, "datacontenttype");
        let dataschema = remove_optional_string(value, "dataschema")
            .and_then(|s| Url::from_str(s.as_str()).ok());
        let time: Option<DateTime<Utc>> = value
            .remove("time")
            .and_then(|v| v.as_str().and_then(|s| DateTime::from_str(s).ok()));

        event.set_id(id);
        event.set_type(ty);
        event.set_source(source);
        event.set_subject(subject);
        event.set_datacontenttype(datacontenttype.clone());
        event.set_dataschema(dataschema);
        event.set_time(time);

        if let Some(data) = value.remove("data") {
            if let Some(datacontenttype) = datacontenttype {
                event.set_data(datacontenttype, Data::Json(data));
            }
        }

        for (k, v) in value {
            let ext = match v {
                Value::Bool(b) => Some(ExtensionValue::Boolean(*b)),
                Value::Number(n) => n.as_i64().map(ExtensionValue::Integer),
                Value::String(s) => Some(ExtensionValue::String(std::mem::take(s))),
                _ => None,
            };
            if let Some(ext) = ext {
                event.set_extension(k, ext)
            }
        }

        Ok(InternalEvent(event))
    }
}

fn remove_required_string(
    value: &mut Map<String, Value>,
    attribute: &'static str,
) -> Result<String, EventBuilderError> {
    value
        .remove(attribute)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or(EventBuilderError::MissingRequiredAttribute {
            attribute_name: attribute,
        })
}

fn remove_optional_string(
    value: &mut Map<String, Value>,
    attribute: &'static str,
) -> Option<String> {
    value
        .remove(attribute)
        .and_then(|v| v.as_str().map(str::to_owned))
}
