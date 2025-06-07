use anycase::to_snake;
use either::{Either, Left, Right};
use serde::{Deserializer, Deserialize, serde_if_integer128};
use serde::de::{self, DeserializeSeed, IntoDeserializer};
use serde::de::value::{StringDeserializer, StrDeserializer};
use std::fmt::{self, Display, Formatter};
use std::vec::{self};
use tvxaml_screen_base::trim_text;

const XAML: &'static str = "https://a1-triard.github.io/tvxaml/2025/xaml";
const XML: &'static str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug)]
pub enum Error {
    Custom(String),
    ReaderError(no_std_xml::reader::Error),
    Unexpected { expected: String },
    UnknownOrMissingXmlns,
    InvalidLiteral(String),
    InvalidBase64,
}

impl From<no_std_xml::reader::Error> for Error {
    fn from(e: no_std_xml::reader::Error) -> Self {
        Error::ReaderError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Custom(s) => Display::fmt(s, f),
            Error::ReaderError(e) => Display::fmt(e, f),
            Error::Unexpected { expected } => write!(f, "expected {expected}"),
            Error::UnknownOrMissingXmlns => write!(f, "unknown or missing xmlns"),
            Error::InvalidLiteral(b) => write!(f, "invalid literal ({b})"),
            Error::InvalidBase64 => write!(f, "invalid base64"),
        }
    }
}

impl core::error::Error for Error { }

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self { Error::Custom(format!("{msg}")) }
}

fn skip_whitespace<S: Iterator<Item=u8>>(
    reader: &mut no_std_xml::EventReader<S>
) -> Result<no_std_xml::reader::XmlEvent, Error> {
    loop {
        let e = reader.next()?;
        if let no_std_xml::reader::XmlEvent::Whitespace(_) = &e {
        } else {
            break Ok(e);
        }
    }
}

pub fn from_iter<'a, T>(s: impl Iterator<Item=u8> + 'a) -> Result<T, Error> where T: Deserialize<'a>
{
    let mut reader = no_std_xml::EventReader::new(s);
    let no_std_xml::reader::XmlEvent::StartDocument { .. } = reader.next()? else {
        return Err(Error::Unexpected { expected: "document start".to_string() });
    };
    let no_std_xml::reader::XmlEvent::StartElement { name, attributes, .. } = skip_whitespace(&mut reader)? else {
        return Err(Error::Unexpected { expected: "element start".to_string() });
    };
    if name.namespace_ref() != Some(XAML) { return Err(Error::UnknownOrMissingXmlns); }
    let deserializer = XamlDeserializer {
        reader: &mut reader,
        name: name.local_name,
        attributes: attributes.into_iter(),
    };
    let res = T::deserialize(deserializer)?;
    let e = skip_whitespace(&mut reader)?;
    let no_std_xml::reader::XmlEvent::EndDocument = &e else {
        return Err(Error::Unexpected { expected: format!("document end {e:?}") });
    };
    Ok(res)
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error> where T: Deserialize<'a> {
    from_iter(s.bytes())
}

struct XamlObjectAccess<'a, S: Iterator<Item=u8>> {
    reader: &'a mut no_std_xml::EventReader<S>,
    name: String,
    attributes: Option<vec::IntoIter<no_std_xml::attribute::OwnedAttribute>>,
    done: bool,
}

impl<'a, 'de, S: Iterator<Item=u8> + 'de> de::MapAccess<'de> for XamlObjectAccess<'a, S> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> { Some(1) }
    
    fn next_key_seed<K>(
        &mut self, seed: K
    ) -> Result<Option<K::Value>, Self::Error> where K: DeserializeSeed<'de> {
        if self.done { return Ok(None); }
        self.done = true;
        let name = seed.deserialize::<StrDeserializer<Self::Error>>(self.name.as_str().into_deserializer())?;
        Ok(Some(name))
    }
    
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error> where V: DeserializeSeed<'de> {
        let attributes = self.attributes.take().unwrap();
        let object_name_prefix = self.name.clone() + ".";
        seed.deserialize(XamlPropertiesDeserializer {
            reader: self.reader,
            object_name_prefix,
            attributes,
        })
    }
}

struct XamlPropertiesAccess<'a, S: Iterator<Item=u8>> {
    reader: &'a mut no_std_xml::EventReader<S>,
    object_name_prefix: String,
    attributes: vec::IntoIter<no_std_xml::attribute::OwnedAttribute>,
    value: Option<Either<String, (String, Vec<no_std_xml::attribute::OwnedAttribute>)>>,
    default_property_name: Option<&'static str>,
    preserve_spaces: bool,
}

impl<'a, 'de, S: Iterator<Item=u8> + 'de> de::MapAccess<'de> for XamlPropertiesAccess<'a, S> {
    type Error = Error;

    fn next_key_seed<K>(
        &mut self, seed: K
    ) -> Result<Option<K::Value>, Self::Error> where K: DeserializeSeed<'de> {
        let attribute = loop {
            if let Some(attribute) = self.attributes.next() {
                if let Some(ns) = attribute.name.namespace_ref() {
                    if ns == XML && attribute.name.local_name == "space" {
                        match attribute.value.as_str() {
                            "default" => self.preserve_spaces = false,
                            "preserve" => self.preserve_spaces = true,
                            _ => return Err(Error::Unexpected { expected: "default or preserve".to_string() }),
                        }
                        continue;
                    }
                    return Err(Error::Unexpected { expected: "attribute without namespace or xml:space".to_string() });
                }
                break Some(attribute);
            } else {
                break None;
            }
        };
        if let Some(attribute) = attribute {
            let property_name = seed.deserialize::<StringDeserializer<Self::Error>>(
                to_snake(attribute.name.local_name).into_deserializer()
            )?;
            self.value = Some(Left(attribute.value));
            Ok(Some(property_name))
        } else {
            match skip_whitespace(self.reader) ? {
                no_std_xml::reader::XmlEvent::StartElement { name, attributes, .. } => {
                    if name.namespace_ref() != Some(XAML) { return Err(Error::UnknownOrMissingXmlns); }
                    let property_name = if !name.local_name.starts_with(&self.object_name_prefix) {
                        self.value = Some(Right((name.local_name, attributes)));
                        let Some(default_property_name) = self.default_property_name else {
                            return Err(Error::Unexpected { expected: "property tag".to_string() });
                        };
                        default_property_name
                    } else {
                        if !attributes.is_empty() {
                            return Err(Error::Unexpected { expected: "property tag without attributes".to_string() });
                        }
                        &name.local_name[self.object_name_prefix.len() ..]
                    };
                    let property_name = to_snake(property_name);
                    let property_name = seed.deserialize::<StringDeserializer<Self::Error>>(
                        property_name.into_deserializer()
                    )?;
                    Ok(Some(property_name))
                },
                no_std_xml::reader::XmlEvent::Characters(mut text) => {
                    if !self.preserve_spaces {
                        text = trim_text(&text).to_string();
                    }
                    self.value = Some(Left(text));
                    let Some(default_property_name) = self.default_property_name else {
                        return Err(Error::Unexpected { expected: "property tag".to_string() });
                    };
                    let property_name = to_snake(default_property_name);
                    let property_name = seed.deserialize::<StringDeserializer<Self::Error>>(
                        property_name.into_deserializer()
                    )?;
                    Ok(Some(property_name))
                },
                no_std_xml::reader::XmlEvent::EndElement { .. } => Ok(None),
                x => {
                    Err(Error::Unexpected { expected: format!("element start or element end or characters ({x:?})") })
                },
            }
        }
    }
    
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error> where V: DeserializeSeed<'de> {
        if let Some(value) = self.value.take() {
            match value {
                Left(attribute_value) => seed.deserialize(TextDeserializer { text: attribute_value }),
                Right((name, attributes)) => {
                    seed.deserialize(XamlDeserializer {
                        reader: self.reader,
                        name,
                        attributes: attributes.into_iter(),
                    })
                },
            }
        } else {
            match skip_whitespace(self.reader)? {
                no_std_xml::reader::XmlEvent::StartElement { name, attributes, .. } => {
                    if name.namespace_ref() != Some(XAML) { return Err(Error::UnknownOrMissingXmlns); }
                    let res = seed.deserialize(XamlDeserializer {
                        reader: self.reader,
                        name: name.local_name,
                        attributes: attributes.into_iter(),
                    })?;
                    let no_std_xml::reader::XmlEvent::EndElement { .. } = skip_whitespace(self.reader)? else {
                        return Err(Error::Unexpected { expected: "element end".to_string() });
                    };
                    Ok(res)
                },
                no_std_xml::reader::XmlEvent::Characters(text) => {
                    let no_std_xml::reader::XmlEvent::EndElement { .. } = skip_whitespace(self.reader)? else {
                        return Err(Error::Unexpected { expected: "element end".to_string() });
                    };
                    seed.deserialize(TextDeserializer { text })
                },
                _ => {
                    Err(Error::Unexpected { expected: "element start or characters".to_string() })
                },
            }
        }
    }
}

struct XamlPropertiesDeserializer<'a, S: Iterator<Item=u8>> {
    reader: &'a mut no_std_xml::EventReader<S>,
    object_name_prefix: String,
    attributes: vec::IntoIter<no_std_xml::attribute::OwnedAttribute>,
}

impl<'a, 'de, S: Iterator<Item=u8> + 'de> Deserializer<'de> for XamlPropertiesDeserializer<'a, S> {
    type Error = Error;

    fn is_human_readable(&self) -> bool { true }

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_bool<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }

        fn deserialize_u128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_str<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_string<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_unit_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_newtype_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_tuple<V>(
        self, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_tuple_struct<V>(
        self, _: &'static str, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_struct<V>(
        self, name: &'static str, _: &'static [&'static str], visitor: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        let default_property_name = name.split('@').skip(1).last();
        visitor.visit_map(XamlPropertiesAccess {
            reader: self.reader,
            attributes: self.attributes,
            value: None,
            object_name_prefix: self.object_name_prefix,
            default_property_name,
            preserve_spaces: false,
        })
    }

    fn deserialize_enum<V>(
        self, _: &'static str, _: &'static [&'static str], _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }
}

struct XamlDeserializer<'a, S: Iterator<Item=u8>> {
    reader: &'a mut no_std_xml::EventReader<S>,
    name: String,
    attributes: vec::IntoIter<no_std_xml::attribute::OwnedAttribute>,
}

impl<'a, 'de, S: Iterator<Item=u8> + 'de> Deserializer<'de> for XamlDeserializer<'a, S> {
    type Error = Error;

    fn is_human_readable(&self) -> bool { true }

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_bool<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }

        fn deserialize_u128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_str<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_string<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_unit_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_newtype_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_map(XamlObjectAccess {
            reader: self.reader,
            name: self.name,
            attributes: Some(self.attributes),
            done: false
        })
    }

    fn deserialize_tuple<V>(
        self, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_tuple_struct<V>(
        self, _: &'static str, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_struct<V>(
        self, _: &'static str, _: &'static [&'static str], _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_enum<V>(
        self, _: &'static str, _: &'static [&'static str], _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }
}

struct TextDeserializer {
    text: String,
}

impl<'de> Deserializer<'de> for TextDeserializer {
    type Error = Error;

    fn is_human_readable(&self) -> bool { true }

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported ({})", self.text)
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_bool<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_i64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_f64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u8<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u16<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u32<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_u64<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }

        fn deserialize_u128<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
            panic!("not supported")
        }
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_string(self.text)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_string(self.text)
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_unit_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_newtype_struct<V>(
        self, _: &'static str, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_tuple<V>(
        self, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_tuple_struct<V>(
        self, _: &'static str, _: usize, _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_struct<V>(
        self, _: &'static str, _: &'static [&'static str], _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }

    fn deserialize_enum<V>(
        self, _: &'static str, _: &'static [&'static str], _: V
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        panic!("not supported")
    }
}
