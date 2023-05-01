use core::fmt;

use image::{GrayImage, RgbImage, RgbaImage};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GrayImg {
    #[serde(
        serialize_with = "serialize_gray_image",
        deserialize_with = "deserialize_gray_image"
    )]
    pub inner: GrayImage,
}

impl GrayImg {
    pub fn new(inner: GrayImage) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RgbImg {
    #[serde(
        serialize_with = "serialize_rgb_image",
        deserialize_with = "deserialize_rgb_image"
    )]
    pub inner: RgbImage,
}
impl RgbImg {
    pub fn new(inner: RgbImage) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RgbaImg {
    #[serde(
        serialize_with = "serialize_rgba_image",
        deserialize_with = "deserialize_rgba_image"
    )]
    pub inner: RgbaImage,
}
impl RgbaImg {
    pub fn new(inner: RgbaImage) -> Self {
        Self { inner }
    }
}

pub fn serialize_rgb_image<S>(x: &RgbImage, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser = s.serialize_struct("RgbImage", 3)?;
    ser.serialize_field("width", &x.width())?;
    ser.serialize_field("height", &x.height())?;
    ser.serialize_field("inner", &x.as_raw())?;
    ser.end()
}

pub fn serialize_gray_image<S>(x: &GrayImage, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser = s.serialize_struct("GrayImage", 3)?;
    ser.serialize_field("width", &x.width())?;
    ser.serialize_field("height", &x.height())?;
    ser.serialize_field("inner", &x.as_raw())?;
    ser.end()
}

pub fn serialize_rgba_image<S>(x: &RgbaImage, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser = s.serialize_struct("RgbImage", 3)?;
    ser.serialize_field("width", &x.width())?;
    ser.serialize_field("height", &x.height())?;
    ser.serialize_field("inner", &x.as_raw())?;
    ser.end()
}

pub fn deserialize_rgb_image<'de, D>(deserializer: D) -> Result<RgbImage, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Width,
        Height,
        Inner,
    }

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("`width` or `height`, or `inner`")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: de::Error,
                {
                    match value {
                        "width" => Ok(Field::Width),
                        "height" => Ok(Field::Height),
                        "inner" => Ok(Field::Inner),
                        _ => Err(de::Error::unknown_field(value, FIELDS)),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct RgbImageVisitor;

    impl<'de> Visitor<'de> for RgbImageVisitor {
        type Value = RgbImage;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct RgbImage")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<RgbImage, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let width = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let height = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let inner = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;
            Ok(RgbImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }

        fn visit_map<V>(self, mut map: V) -> Result<RgbImage, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut width = None;
            let mut height = None;
            let mut inner = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Width => {
                        if width.is_some() {
                            return Err(de::Error::duplicate_field("width"));
                        }
                        width = Some(map.next_value()?);
                    }
                    Field::Height => {
                        if height.is_some() {
                            return Err(de::Error::duplicate_field("height"));
                        }
                        height = Some(map.next_value()?);
                    }
                    Field::Inner => {
                        if inner.is_some() {
                            return Err(de::Error::duplicate_field("inner"));
                        }
                        inner = Some(map.next_value()?);
                    }
                }
            }
            let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
            let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
            let inner = inner.ok_or_else(|| de::Error::missing_field("inner"))?;
            Ok(RgbImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }
    }

    const FIELDS: &'static [&'static str] = &["secs", "nanos"];
    deserializer.deserialize_struct("Duration", FIELDS, RgbImageVisitor)
}

pub fn deserialize_gray_image<'de, D>(deserializer: D) -> Result<GrayImage, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Width,
        Height,
        Inner,
    }

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("`width` or `height`, or `inner`")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: de::Error,
                {
                    match value {
                        "width" => Ok(Field::Width),
                        "height" => Ok(Field::Height),
                        "inner" => Ok(Field::Inner),
                        _ => Err(de::Error::unknown_field(value, FIELDS)),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct GrayImageVisitor;

    impl<'de> Visitor<'de> for GrayImageVisitor {
        type Value = GrayImage;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct RgbImage")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<GrayImage, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let width = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let height = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let inner = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;
            Ok(GrayImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }

        fn visit_map<V>(self, mut map: V) -> Result<GrayImage, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut width = None;
            let mut height = None;
            let mut inner = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Width => {
                        if width.is_some() {
                            return Err(de::Error::duplicate_field("width"));
                        }
                        width = Some(map.next_value()?);
                    }
                    Field::Height => {
                        if height.is_some() {
                            return Err(de::Error::duplicate_field("height"));
                        }
                        height = Some(map.next_value()?);
                    }
                    Field::Inner => {
                        if inner.is_some() {
                            return Err(de::Error::duplicate_field("inner"));
                        }
                        inner = Some(map.next_value()?);
                    }
                }
            }
            let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
            let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
            let inner = inner.ok_or_else(|| de::Error::missing_field("inner"))?;
            Ok(GrayImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }
    }

    const FIELDS: &'static [&'static str] = &["secs", "nanos"];
    deserializer.deserialize_struct("Duration", FIELDS, GrayImageVisitor)
}

pub fn deserialize_rgba_image<'de, D>(deserializer: D) -> Result<RgbaImage, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Width,
        Height,
        Inner,
    }

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("`width` or `height`, or `inner`")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: de::Error,
                {
                    match value {
                        "width" => Ok(Field::Width),
                        "height" => Ok(Field::Height),
                        "inner" => Ok(Field::Inner),
                        _ => Err(de::Error::unknown_field(value, FIELDS)),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct RgbaImageVisitor;

    impl<'de> Visitor<'de> for RgbaImageVisitor {
        type Value = RgbaImage;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct RgbImage")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<RgbaImage, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let width = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let height = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let inner = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;
            Ok(RgbaImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }

        fn visit_map<V>(self, mut map: V) -> Result<RgbaImage, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut width = None;
            let mut height = None;
            let mut inner = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Width => {
                        if width.is_some() {
                            return Err(de::Error::duplicate_field("width"));
                        }
                        width = Some(map.next_value()?);
                    }
                    Field::Height => {
                        if height.is_some() {
                            return Err(de::Error::duplicate_field("height"));
                        }
                        height = Some(map.next_value()?);
                    }
                    Field::Inner => {
                        if inner.is_some() {
                            return Err(de::Error::duplicate_field("inner"));
                        }
                        inner = Some(map.next_value()?);
                    }
                }
            }
            let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
            let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
            let inner = inner.ok_or_else(|| de::Error::missing_field("inner"))?;
            Ok(RgbaImage::from_raw(width, height, inner)
                .ok_or(de::Error::custom("something went really wrong"))?)
        }
    }

    const FIELDS: &'static [&'static str] = &["secs", "nanos"];
    deserializer.deserialize_struct("Duration", FIELDS, RgbaImageVisitor)
}
