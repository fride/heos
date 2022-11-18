use std::collections::BTreeMap;
use std::fmt::Display;

use serde::de::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct HalList<A> {
    contents: Vec<A>,
}

impl<A> HalList<A>
where
    A: Sized + Clone,
{
    pub fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }
    pub fn with(mut self, value: A) -> Self {
        self.contents.push(value);
        self
    }
    pub fn push(&mut self, value: A) {
        self.contents.push(value);
    }
}

impl<A> Into<HalList<A>> for Vec<A> {
    fn into(self) -> HalList<A> {
        HalList { contents: self }
    }
}

impl<T> serde::Serialize for HalList<T>
where
    T: serde::Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.contents.is_empty() {
            ().serialize(serializer)
        } else if self.contents.len() == 1 {
            self.contents.first().serialize(serializer)
        } else {
            self.contents.serialize(serializer)
        }
    }
}

impl<'de, T> serde::Deserialize<'de> for HalList<T>
where
    for<'d> T: serde::Deserialize<'d> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Value = serde::Deserialize::deserialize(deserializer)?;
        if value.is_array() {
            let values: Vec<T> = serde_json::from_value(value)
                .map_err(|err| D::Error::custom(format!("JSON Error: {:?}", err)))?;
            Ok(values.into())
        } else {
            let value: T = serde_json::from_value(value)
                .map_err(|err| D::Error::custom(format!("JSON Error: {:?}", err)))?;
            Ok(HalList::new().with(value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HalResource {
    #[serde(rename = "_links", default, skip_serializing_if = "BTreeMap::is_empty")]
    links: BTreeMap<String, HalList<Link>>,
    #[serde(flatten)]
    values: BTreeMap<String, Value>,
    #[serde(rename = "_nested")]
    nested: BTreeMap<String, HalList<HalResource>>,
}

impl HalResource {
    pub fn with_self<A: Display>(self_link: A) -> Self {
        let link = Link::href(self_link);
        Self {
            values: BTreeMap::default(),
            links: BTreeMap::default(),
            nested: BTreeMap::default(),
        }
        .add_link("self", link)
    }

    // adds the objects properties.
    pub fn add_object<V>(mut self, value: V) -> Self
    where
        V: serde::Serialize,
    {
        let value = serde_json::to_value(value).unwrap();
        if let Value::Object(values) = value {
            for (k, v) in values {
                self.values.insert(k, v);
            }
        }
        self
    }

    pub fn add_state<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Display,
        V: serde::Serialize,
    {
        let value = serde_json::to_value(value).unwrap();
        self.values.insert(name.to_string(), value);
        self
    }

    pub fn add_link<C, D>(mut self, name: C, link: D) -> Self
    where
        C: Display,
        D: Into<Link>,
    {
        let link_list = self
            .links
            .entry(format!("{}", name))
            .or_insert(HalList::new());
        link_list.push(link.into());
        self
    }

    pub fn with_embedded<A: Into<HalResource>, D: Display>(mut self, name: D, value: A) -> Self {
        let resources = self
            .nested
            .entry(name.to_string())
            .or_insert(HalList::new());
        resources.push(value.into());
        self
    }
    pub fn with_resources<D: Display, A: IntoIterator<Item = HalResource>>(
        self,
        name: D,
        resources: A,
    ) -> Self {
        let name = name.to_string();
        resources
            .into_iter()
            .fold(self, |s, resource| s.with_embedded(name.clone(), resource))
    }
}

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Link {
    href: String,
    #[serde(default)]
    templated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "type")]
    context_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl Link {
    pub fn href<A: Display>(href: A) -> Self {
        Link {
            href: format!("{}", href),
            ..Default::default()
        }
    }
}

impl<A: Display> From<A> for Link {
    fn from(href: A) -> Self {
        Link {
            href: format!("{}", href),
            ..Default::default()
        }
    }
}

pub trait HalContext {
    fn create_link<I, A>(&self, name: &str, parameters: A) -> (&str, Link)
    where
        A: IntoIterator<Item = I>,
        I: Display;
}

pub trait ToResource {
    fn get_links<C: HalContext>(&self, context: C) -> Vec<(&str, Link)>;
    fn to_resource<C: HalContext>(self, context: C) -> HalResource;
}
