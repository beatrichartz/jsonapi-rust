//! Defines the `JsonApiTemplate` trait. This is primarily used in conjunction with
//! the [`jsonapi_template!`](../macro.jsonapi_template.html) macro to allow arbitrary
//! structs which implement `Deserialize` to be converted to/from a
//! [`JsonApiDocument`](../api/struct.JsonApiDocument.html) or
//! [`Resource`](../api/struct.Resource.html)
pub use std::collections::HashMap;
pub use crate::api::*;
use crate::errors::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value, Value, Map};

/// A trait for any struct that can be converted from/into a
/// [`Resource`](api/struct.Resource.tml). The only requirement is that your
/// struct has an `id: String` field.
/// You shouldn't be implementing JsonApiTemplate manually, look at the
/// `jsonapi_template!` macro instead.
pub trait JsonApiTemplate: Serialize
    where
            for<'de> Self: Deserialize<'de>,
{
    #[doc(hidden)]
    fn jsonapi_type(&self) -> String;
    #[doc(hidden)]
    fn relationship_fields() -> Option<&'static [&'static str]>;
    #[doc(hidden)]
    fn build_relationships(&self) -> Option<Relationships>;

    fn from_jsonapi_resource_template(resource_template: &ResourceTemplate)
                             -> Result<Self>
    {
        Self::from_serializable(Self::resource_template_to_attrs(resource_template))
    }

    /// Create a single resource object or collection of resource
    /// objects directly from 
    /// [`DocumentData`](../api/struct.DocumentData.html). This method
    /// will parse the document (the `data` and `included` resources) in an
    /// attempt to instantiate the calling struct.
    fn from_jsonapi_document(doc: &DocumentData) -> Result<Self> {
        match doc.data.as_ref() {
            Some(primary_data) => {
                match *primary_data {
                    PrimaryData::None => bail!("Document had no data"),
                    PrimaryData::Single(_) => bail!("Document had no template but a fully qualified resource"),
                    PrimaryData::Multiple(_) => bail!("Document had no templates but fully qualified resources"),
                    PrimaryData::SingleTemplate(ref resource_template) => {
                        Self::from_serializable(Self::resource_template_to_attrs(resource_template))
                    }
                    PrimaryData::MultipleTemplates(ref resource_templates) => {
                        let all: Vec<ResourceAttributes> = resource_templates
                            .iter()
                            .map(|r| Self::resource_template_to_attrs(r))
                            .collect();
                        Self::from_serializable(all)
                    }
                }
            }
            None => bail!("Document had no data"),
        }
    }

    /// Converts the instance of the struct into a
    /// [`ResourceTemplate`](../api/struct.Resource.html)
    fn to_jsonapi_resource_template(&self) -> ResourceTemplate {
        if let Value::Object(attrs) = to_value(self).unwrap() {
            let resource_template = ResourceTemplate {
                _type: self.jsonapi_type(),
                relationships: self.build_relationships(),
                attributes: Self::extract_attributes(&attrs),
                ..Default::default()
            };

            resource_template
        } else {
            panic!("{} is not a Value::Object", self.jsonapi_type())
        }
    }


    /// Converts the struct into a complete
    /// [`JsonApiDocument`](../api/struct.JsonApiDocument.html)
    fn to_jsonapi_document(&self) -> JsonApiDocument {
        let resource_template = self.to_jsonapi_resource_template();
        JsonApiDocument::Data(
            DocumentData {
                data: Some(PrimaryData::SingleTemplate(Box::new(resource_template))),
                ..Default::default()
            }
        )
    }

    #[doc(hidden)]
    fn resource_template_to_attrs(resource_template: &ResourceTemplate) -> ResourceAttributes
    {
        let mut new_attrs = HashMap::new();
        new_attrs.clone_from(&resource_template.attributes);

        if let Some(relations) = resource_template.relationships.as_ref() {
                for (name, relation) in relations {
                    let value = match relation.data {
                        Some(IdentifierData::None) => Value::Null,
                        Some(IdentifierData::Single(ref identifier)) => to_value(identifier).expect("Cast of single relation to value"),
                        Some(IdentifierData::Multiple(ref identifiers)) => to_value(identifiers).expect("Cast of many relation to value"),
                        None => Value::Null,
                    };
                    new_attrs.insert(name.to_string(), value);
                }
        }
        new_attrs
    }

    #[doc(hidden)]
    fn build_has_one(model: &ResourceIdentifier) -> Relationship {
        Relationship {
            data: Some(IdentifierData::Single(model.clone())),
            links: None,
        }
    }

    #[doc(hidden)]
    fn build_has_many(models: &[ResourceIdentifier]) -> Relationship {
        Relationship {
            data: Some(IdentifierData::Multiple(models.to_vec())),
            links: None,
        }
    }

    /* Attribute corresponding to the template is removed from the Map
     * before calling this, so there's no need to ignore it like we do
     * with the attributes that correspond with relationships.
     * */
    #[doc(hidden)]
    fn extract_attributes(attrs: &Map<String, Value>) -> ResourceAttributes {
        attrs
            .iter()
            .filter(|&(key, _)| {
                if let Some(fields) = Self::relationship_fields() {
                    if fields.contains(&key.as_str()) {
                        return false;
                    }
                }
                true
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    #[doc(hidden)]
    fn to_resource_templates(&self) -> ResourceTemplates {
        let me = self.to_jsonapi_resource_template();
        vec![me]
    }

    #[doc(hidden)]
    fn from_serializable<S: Serialize>(s: S) -> Result<Self> {
        from_value(to_value(s)?).map_err(Error::from)
    }
}

pub trait JsonApiTemplateRelationship: Serialize
    where
            for<'de> Self: Deserialize<'de>,
{
    #[doc(hidden)]
    fn jsonapi_type(&self) -> String;
    #[doc(hidden)]
    fn jsonapi_id(&self) -> String;
}

impl JsonApiTemplateRelationship for ResourceIdentifier {
    fn jsonapi_type(&self) -> String {
        return self._type.to_string();
    }

    fn jsonapi_id(&self) -> String {
        return self.id.to_string();
    }
}

/// Converts a `vec!` of structs into
/// [`ResourceTemplates`](../api/type.ResourceTemplates.html)
///
pub fn vec_to_jsonapi_resource_templates<T: JsonApiTemplate>(
    objects: Vec<T>,
) -> ResourceTemplates {
    let templates = objects
        .iter()
        .map(|obj|
            obj.to_jsonapi_resource_template()
        )
        .collect();

    templates
}

/// Converts a `vec!` of structs into a
/// [`JsonApiDocument`](../api/struct.JsonApiDocument.html)
///
/// ```rust
/// #[macro_use] extern crate serde_derive;
/// #[macro_use] extern crate jsonapi;
/// use jsonapi::api::*;
/// use jsonapi::template::*;
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Flea {
///     name: String,
/// }
///
/// jsonapi_template!(Flea; "flea");
///
/// let fleas = vec![
///     Flea {
///         name: "rick".into(),
///     },
///     Flea {
///         name: "morty".into(),
///     },
/// ];
/// let doc = template_vec_to_jsonapi_document(fleas);
/// assert!(doc.is_valid());
/// ```
pub fn template_vec_to_jsonapi_document<T: JsonApiTemplate>(objects: Vec<T>) -> JsonApiDocument {
    let resources_templates = vec_to_jsonapi_resource_templates(objects);
    JsonApiDocument::Data(
        DocumentData {
            data: Some(PrimaryData::MultipleTemplates(resources_templates)),
            ..Default::default()
        }
    )
}

impl<M: JsonApiTemplate> JsonApiTemplate for Box<M> {
    fn jsonapi_type(&self) -> String {
        self.as_ref().jsonapi_type()
    }

    fn relationship_fields() -> Option<&'static [&'static str]> {
        M::relationship_fields()
    }

    fn build_relationships(&self) -> Option<Relationships> {
        self.as_ref().build_relationships()
    }
}

/// When applied this macro implements the
/// [`JsonApiTemplate`](template/trait.JsonApiTemplate.html) trait for the provided type
///
#[macro_export]
macro_rules! jsonapi_template {
    ($template:ty; $type:expr) => (
        impl JsonApiTemplate for $template {
            fn jsonapi_type(&self) -> String { $type.to_string() }
            fn relationship_fields() -> Option<&'static [&'static str]> { None }
            fn build_relationships(&self) -> Option<Relationships> { None }
        }
    );
    ($template:ty; $type:expr;
        has one $( $has_one:ident ),*
    ) => (
        jsonapi_template!($template; $type; has one $( $has_one ),*; has many);
    );
    ($template:ty; $type:expr;
        has many $( $has_many:ident ),*
    ) => (
        jsonapi_template!($template; $type; has one; has many $( $has_many ),*);
    );
    ($template:ty; $type:expr;
        has one $( $has_one:ident ),*;
        has many $( $has_many:ident ),*
    ) => (
        impl JsonApiTemplate for $template {
            fn jsonapi_type(&self) -> String { $type.to_string() }

            fn relationship_fields() -> Option<&'static [&'static str]> {
                static FIELDS: &'static [&'static str] = &[
                     $( stringify!($has_one),)*
                     $( stringify!($has_many),)*
                ];

                Some(FIELDS)
            }

            fn build_relationships(&self) -> Option<Relationships> {
                let mut relationships = HashMap::new();
                $(
                    relationships.insert(stringify!($has_one).into(),
                        Self::build_has_one(&self.$has_one)
                    );
                )*
                $(
                    relationships.insert(
                        stringify!($has_many).into(),
                        {
                            Self::build_has_many(&self.$has_many)
                        }
                    );
                )*
                Some(relationships)
            }
        }
    );
}
