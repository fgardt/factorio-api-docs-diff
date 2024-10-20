use std::{collections::HashMap, ops::Deref};

use serde::{de::Visitor, Deserialize, Serialize};
use structdiff::{Difference, StructDiff};

use super::{
    diff_helper::{self, vec_diff, DiffableVec, DiffableVecDiff, SingleDiff},
    Image,
};

impl<T> diff_helper::Named for T
where
    T: Deref<Target = NamedCommon>,
{
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PrototypeDoc {
    #[serde(flatten)]
    common: super::Common,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub prototypes: DiffableVec<Prototype>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub types: DiffableVec<TypeConcept>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub defines: DiffableVec<crate::format::runtime::Define>,
}

impl Deref for PrototypeDoc {
    type Target = super::Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl super::Doc for PrototypeDoc {
    type Diff = PrototypeDocDiff;

    fn diff(&self, other: &Self) -> Self::Diff {
        Self::Diff {
            prototypes: self.prototypes.diff(&other.prototypes),
            types: self.types.diff(&other.types),
            defines: self.defines.diff(&other.defines),
        }
    }
}

impl super::Info for PrototypeDoc {
    fn print_info(&self) {
        self.common.print_info();

        eprintln!(" - Prototypes: {}", self.prototypes.len());
        eprintln!(" - Types:      {}", self.types.len());
        eprintln!(" - Defines:    {}", self.defines.len());
    }
}

#[derive(Serialize)]
pub struct PrototypeDocDiff {
    pub prototypes: DiffableVecDiff<Prototype>,
    pub types: DiffableVecDiff<TypeConcept>,
    pub defines: DiffableVecDiff<crate::format::runtime::Define>,
}

impl super::Info for PrototypeDocDiff {
    fn print_info(&self) {
        eprintln!("=> {} prototypes changed", self.prototypes.len());
        eprintln!("=> {} types changed", self.types.len());
        eprintln!("=> {} defines changed", self.defines.len());
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
pub struct Common {
    pub description: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lists: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CommonDiff {
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
}

impl StructDiff for Common {
    type Diff = CommonDiff;

    type DiffRef<'target> = CommonDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let cli = crate::CLI.with_borrow(Clone::clone);
        let mut res = Vec::new();

        if (cli.descriptions || cli.full) && self.description != updated.description {
            res.push(CommonDiff::Description(updated.description.clone()));
        }

        if cli.full && self.lists != updated.lists {
            res.push(CommonDiff::Lists(updated.lists.clone()));
        }

        if (cli.examples || cli.full) && self.examples != updated.examples {
            res.push(CommonDiff::Examples(updated.examples.clone()));
        }

        if cli.full && self.images != updated.images {
            res.push(CommonDiff::Images(updated.images.clone()));
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
pub struct NamedCommon {
    #[serde(flatten)]
    common: Common,

    pub name: String,
    pub order: i16,
}

impl diff_helper::Named for NamedCommon {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NamedCommonDiff {
    Name(String),
    Order(i16),
    // common fields
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
}

impl StructDiff for NamedCommon {
    type Diff = NamedCommonDiff;

    type DiffRef<'target> = NamedCommonDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.name != updated.name {
            res.push(NamedCommonDiff::Name(updated.name.clone()));
        }

        if crate::CLI.with_borrow(|c| c.full) && self.order != updated.order {
            res.push(NamedCommonDiff::Order(updated.order));
        }

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                    CommonDiff::Lists(lists) => Self::Diff::Lists(lists),
                    CommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    CommonDiff::Images(images) => Self::Diff::Images(images),
                };
                res.push(d);
            }
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

impl Deref for NamedCommon {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Prototype {
    #[serde(flatten)]
    common: NamedCommon,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visibility: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parent: String,

    #[serde(rename = "abstract")]
    pub abstract_: bool,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub typename: String,

    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub instance_limit: Option<u128>,
    #[serde(
        default,
        deserialize_with = "deserialize_instance_limit",
        skip_serializing_if = "String::is_empty"
    )]
    pub instance_limit: String,

    pub deprecated: bool,

    pub properties: DiffableVec<Property>,

    pub custom_properties: Option<CustomProperties>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PrototypeDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
    // other fields
    Visibility(Vec<String>),
    Parent(String),
    Abstract(bool),
    Typename(String),
    InstanceLimit(String),
    Deprecated(bool),
    Properties(DiffableVecDiff<Property>),
    CustomProperties(SingleDiff<CustomProperties>),
}

impl StructDiff for Prototype {
    type Diff = PrototypeDiff;

    type DiffRef<'target> = PrototypeDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    NamedCommonDiff::Name(name) => Self::Diff::Name(name),
                    NamedCommonDiff::Order(order) => Self::Diff::Order(order),
                    NamedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    NamedCommonDiff::Lists(lists) => Self::Diff::Lists(lists),
                    NamedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    NamedCommonDiff::Images(images) => Self::Diff::Images(images),
                };
                res.push(d);
            }
        }

        if self.visibility != updated.visibility {
            res.push(PrototypeDiff::Visibility(updated.visibility.clone()));
        }

        if self.parent != updated.parent {
            res.push(PrototypeDiff::Parent(updated.parent.clone()));
        }

        if self.abstract_ != updated.abstract_ {
            res.push(PrototypeDiff::Abstract(updated.abstract_));
        }

        if self.typename != updated.typename {
            res.push(PrototypeDiff::Typename(updated.typename.clone()));
        }

        if self.instance_limit != updated.instance_limit {
            res.push(PrototypeDiff::InstanceLimit(updated.instance_limit.clone()));
        }

        if self.deprecated != updated.deprecated {
            res.push(PrototypeDiff::Deprecated(updated.deprecated));
        }

        let properties_diff = self.properties.diff(&updated.properties);
        if !properties_diff.is_empty() {
            res.push(PrototypeDiff::Properties(properties_diff));
        }

        if self.custom_properties != updated.custom_properties {
            res.push(PrototypeDiff::CustomProperties(
                updated
                    .custom_properties
                    .as_ref()
                    .map(|cp| {
                        cp.diff(
                            self.custom_properties
                                .as_ref()
                                .unwrap_or(&CustomProperties::default()),
                        )
                    })
                    .unwrap_or_default(),
            ));
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

struct InstanceLimitVisitor;

impl<'de> Visitor<'de> for InstanceLimitVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value.to_owned())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(String::new())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

fn deserialize_instance_limit<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(InstanceLimitVisitor)
}

impl Deref for Prototype {
    type Target = NamedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct TypeConcept {
    #[serde(flatten)]
    common: NamedCommon,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parent: String,

    #[serde(rename = "abstract")]
    pub abstract_: bool,

    pub inline: bool,

    #[serde(rename = "type")]
    pub type_: Type,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: DiffableVec<Property>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TypeConceptDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
    // other fields
    Parent(String),
    Abstract(bool),
    Inline(bool),
    Type(<Type as StructDiff>::Diff),
    Properties(DiffableVecDiff<Property>),
}

impl StructDiff for TypeConcept {
    type Diff = TypeConceptDiff;

    type DiffRef<'target> = TypeConceptDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            if !common_diff.is_empty() {
                for d in common_diff {
                    let d = match d {
                        NamedCommonDiff::Name(name) => Self::Diff::Name(name),
                        NamedCommonDiff::Order(order) => Self::Diff::Order(order),
                        NamedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                        NamedCommonDiff::Lists(lists) => Self::Diff::Lists(lists),
                        NamedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                        NamedCommonDiff::Images(images) => Self::Diff::Images(images),
                    };
                    res.push(d);
                }
            }
        }

        if self.parent != updated.parent {
            res.push(Self::Diff::Parent(updated.parent.clone()));
        }

        if self.abstract_ != updated.abstract_ {
            res.push(Self::Diff::Abstract(updated.abstract_));
        }

        if self.inline != updated.inline {
            res.push(Self::Diff::Inline(updated.inline));
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);
            if !diff.is_empty() {
                assert!(diff.len() == 1, "type diff should have only one element");
                if !diff[0].skip() {
                    res.push(Self::Diff::Type(diff[0].clone()));
                }
            }
        }

        let properties_diff = self.properties.diff(&updated.properties);
        if !properties_diff.is_empty() {
            res.push(Self::Diff::Properties(properties_diff));
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

impl Deref for TypeConcept {
    type Target = NamedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Default)]
pub struct Property {
    #[serde(flatten)]
    common: NamedCommon,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub alt_name: String,

    #[serde(rename = "override")]
    pub override_: bool,

    #[serde(rename = "type")]
    pub type_: Type,

    pub optional: bool,
    pub default: Option<PropertyDefault>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PropertyDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
    // other fields
    AltName(String),
    Override(bool),
    Type(<Type as StructDiff>::Diff),
    Optional(bool),
    Default(Option<PropertyDefault>),
}

impl StructDiff for Property {
    type Diff = PropertyDiff;

    type DiffRef<'target> = Self::Diff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            if !common_diff.is_empty() {
                for d in common_diff {
                    let d = match d {
                        NamedCommonDiff::Name(name) => Self::Diff::Name(name),
                        NamedCommonDiff::Order(order) => Self::Diff::Order(order),
                        NamedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                        NamedCommonDiff::Lists(lists) => Self::Diff::Lists(lists),
                        NamedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                        NamedCommonDiff::Images(images) => Self::Diff::Images(images),
                    };
                    res.push(d);
                }
            }
        }

        if self.alt_name != updated.alt_name {
            res.push(Self::Diff::AltName(updated.alt_name.clone()));
        }

        if self.override_ != updated.override_ {
            res.push(Self::Diff::Override(updated.override_));
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);
            if !diff.is_empty() {
                assert!(diff.len() == 1, "type diff should have only one element");
                if !diff[0].skip() {
                    res.push(Self::Diff::Type(diff[0].clone()));
                }
            }
        }

        if self.optional != updated.optional {
            res.push(Self::Diff::Optional(updated.optional));
        }

        if self.default != updated.default {
            res.push(Self::Diff::Default(updated.default.clone()));
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

impl Deref for Property {
    type Target = NamedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Difference, Clone)]
#[serde(untagged)]
pub enum PropertyDefault {
    String(String),
    Literal(Literal),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Default, Clone, Hash)]
pub struct CustomProperties {
    #[serde(flatten)]
    common: Common,

    pub key_type: Type,
    pub value_type: Type,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CustomPropertiesDiff {
    // common fields
    Description(String),
    Lists(Vec<String>),
    Examples(Vec<String>),
    Images(Vec<Image>),
    // other fields
    KeyType(<Type as StructDiff>::Diff),
    ValueType(<Type as StructDiff>::Diff),
}

impl StructDiff for CustomProperties {
    type Diff = CustomPropertiesDiff;

    type DiffRef<'target> = Self::Diff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            if !common_diff.is_empty() {
                for d in common_diff {
                    let d = match d {
                        CommonDiff::Description(desc) => Self::Diff::Description(desc),
                        CommonDiff::Lists(lists) => Self::Diff::Lists(lists),
                        CommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                        CommonDiff::Images(images) => Self::Diff::Images(images),
                    };
                    res.push(d);
                }
            }
        }

        if self.key_type != updated.key_type {
            let diff = self.key_type.diff(&updated.key_type);
            if !diff.is_empty() {
                assert!(diff.len() == 1, "type diff should have only one element");
                if !diff[0].skip() {
                    res.push(Self::Diff::KeyType(diff[0].clone()));
                }
            }
        }

        if self.value_type != updated.value_type {
            let diff = self.value_type.diff(&updated.value_type);
            if !diff.is_empty() {
                assert!(diff.len() == 1, "type diff should have only one element");
                if !diff[0].skip() {
                    res.push(Self::Diff::ValueType(diff[0].clone()));
                }
            }
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

impl Deref for CustomProperties {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone)]
#[serde(untagged)]
pub enum Type {
    Simple(String),
    Complex(Box<ComplexType>),
}

impl Type {
    #[must_use]
    pub fn as_simple(&self) -> Option<String> {
        match self {
            Self::Simple(s) => Some(s.clone()),
            Self::Complex(_) => None,
        }
    }

    #[must_use]
    pub fn as_complex(&self) -> Option<Box<ComplexType>> {
        match self {
            Self::Complex(c) => Some(c.clone()),
            Self::Simple(_) => None,
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Simple(String::new())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum TypeDiff {
    Simple(String),
    Complex(SingleDiff<ComplexType>),
}

impl TypeDiff {
    #[must_use]
    pub fn skip(&self) -> bool {
        match self {
            Self::Simple(_) => false,
            Self::Complex(c) => c.is_empty(),
        }
    }
}

impl StructDiff for Type {
    type Diff = TypeDiff;

    type DiffRef<'target> = Self::Diff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();
        match (self, updated) {
            (Self::Simple(s), Self::Simple(updated_s)) => {
                if s != updated_s {
                    res.push(Self::Diff::Simple(updated_s.clone()));
                }
            }
            (Self::Complex(c), Self::Complex(updated_c)) => {
                let diff = c.diff(updated_c);
                if !diff.is_empty() {
                    res.push(Self::Diff::Complex(diff));
                }
            }
            (_, Self::Simple(updated_s)) => {
                res.push(Self::Diff::Simple(updated_s.clone()));
            }
            (_, Self::Complex(updated_c)) => {
                res.push(Self::Diff::Complex(ComplexType::default().diff(updated_c)));
            }
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone)]
#[serde(tag = "complex_type", rename_all = "snake_case")]
pub enum ComplexType {
    Array {
        value: Type,
    },
    Dictionary {
        key: Type,
        value: Type,
    },
    Tuple {
        values: Vec<Type>,
    },
    Union {
        options: Vec<Type>,
        full_format: bool,
    },
    Type {
        value: Type,
        description: String,
    },
    Literal(Literal),
    Struct,
}

impl ComplexType {
    #[must_use]
    pub fn as_array(&self) -> Option<Type> {
        match self {
            Self::Array { value } => Some(value.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_dictionary(&self) -> Option<Self> {
        match self {
            Self::Dictionary { .. } => Some(self.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_tuple(&self) -> Option<Vec<Type>> {
        match self {
            Self::Tuple { values } => Some(values.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_union(&self) -> Option<Self> {
        match self {
            Self::Union { .. } => Some(self.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_type(&self) -> Option<Self> {
        match self {
            Self::Type { .. } => Some(self.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_literal(&self) -> Option<Literal> {
        match self {
            Self::Literal(l) => Some(l.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_struct(&self) -> Option<()> {
        match self {
            Self::Struct => Some(()),
            _ => None,
        }
    }
}

impl Default for ComplexType {
    fn default() -> Self {
        Self::Struct {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ComplexTypeDiff {
    ComplexType(String),
    Value(<Type as StructDiff>::Diff),
    Key(<Type as StructDiff>::Diff),
    Values(Vec<<Type as StructDiff>::Diff>),
    Options(Vec<<Type as StructDiff>::Diff>),
    FullFormat(bool),
    Description(String),
    #[serde(rename = "value")]
    Literal(LiteralValue),
}

impl StructDiff for ComplexType {
    type Diff = ComplexTypeDiff;

    type DiffRef<'target> = Self::Diff;

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        match (self, updated) {
            (Self::Array { value }, Self::Array { value: u_value }) => {
                if value != u_value {
                    let diff = value.diff(u_value);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::Value(diff[0].clone()));
                    }
                }
            }
            (
                Self::Dictionary { key, value },
                Self::Dictionary {
                    key: u_key,
                    value: u_value,
                },
            ) => {
                if key != u_key {
                    let diff = key.diff(u_key);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::Key(diff[0].clone()));
                    }
                }

                if value != u_value {
                    let diff = value.diff(u_value);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::Value(diff[0].clone()));
                    }
                }
            }
            (Self::Tuple { values }, Self::Tuple { values: u_values }) => {
                if values != u_values {
                    let diff = vec_diff(values, u_values)
                        .iter()
                        .flatten()
                        .filter(|v| !v.skip())
                        .cloned()
                        .collect::<Vec<_>>();

                    if !diff.is_empty() {
                        res.push(Self::Diff::Values(diff));
                    }
                }
            }
            (
                Self::Union {
                    options,
                    full_format,
                },
                Self::Union {
                    options: u_options,
                    full_format: u_full_format,
                },
            ) => {
                if options != u_options {
                    let diff = vec_diff(options, u_options)
                        .iter()
                        .flatten()
                        .filter(|o| !o.skip())
                        .cloned()
                        .collect::<Vec<_>>();

                    if !diff.is_empty() {
                        res.push(Self::Diff::Options(diff));
                    }
                }

                if full_format != u_full_format {
                    res.push(Self::Diff::FullFormat(*u_full_format));
                }
            }
            (
                Self::Type { value, description },
                Self::Type {
                    value: u_value,
                    description: updated_description,
                },
            ) => {
                if value != u_value {
                    let diff = value.diff(u_value);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::Value(diff[0].clone()));
                    }
                }

                if crate::CLI.with_borrow(|c| c.descriptions || c.full)
                    && description != updated_description
                {
                    res.push(Self::Diff::Description(updated_description.clone()));
                }
            }
            (Self::Literal(l), Self::Literal(updated_l)) => {
                if l != updated_l {
                    let diff = l.diff(updated_l);
                    if !diff.is_empty() {
                        for d in diff {
                            match d {
                                LiteralDiff::Value(v) => res.push(Self::Diff::Literal(v)),
                                LiteralDiff::Description(d) => {
                                    if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                                        res.push(Self::Diff::Description(d));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            (Self::Struct, Self::Struct) => {}
            _ => match updated {
                Self::Array { value } => {
                    res.push(Self::Diff::ComplexType("array".to_owned()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::Dictionary { key, value } => {
                    res.push(Self::Diff::ComplexType("dictionary".to_owned()));
                    res.push(Self::Diff::Key(Type::default().diff(key)[0].clone()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::Tuple { values } => {
                    res.push(Self::Diff::ComplexType("tuple".to_owned()));
                    res.push(Self::Diff::Values(
                        values
                            .iter()
                            .map(|v| Type::default().diff(v)[0].clone())
                            .collect(),
                    ));
                }
                Self::Union {
                    options,
                    full_format,
                } => {
                    res.push(Self::Diff::ComplexType("union".to_owned()));
                    res.push(Self::Diff::Options(
                        options
                            .iter()
                            .map(|o| Type::default().diff(o)[0].clone())
                            .collect(),
                    ));
                    res.push(Self::Diff::FullFormat(*full_format));
                }
                Self::Type { value, description } => {
                    res.push(Self::Diff::ComplexType("type".to_owned()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));

                    if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                        res.push(Self::Diff::Description(description.clone()));
                    }
                }
                Self::Literal(l) => {
                    res.push(Self::Diff::ComplexType("literal".to_owned()));

                    let diff = Literal::default().diff(l);
                    for d in diff {
                        match d {
                            LiteralDiff::Value(v) => res.push(Self::Diff::Literal(v)),
                            LiteralDiff::Description(d) => {
                                if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                                    res.push(Self::Diff::Description(d));
                                }
                            }
                        }
                    }
                }
                Self::Struct => res.push(Self::Diff::ComplexType("struct".to_owned())),
            },
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Default, Clone)]
pub struct Literal {
    pub value: LiteralValue,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LiteralDiff {
    Value(LiteralValue),
    Description(String),
}

impl StructDiff for Literal {
    type Diff = LiteralDiff;

    type DiffRef<'target> = Self::Diff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.value != updated.value {
            res.push(Self::Diff::Value(updated.value.clone()));
        }

        if crate::CLI.with_borrow(|c| c.descriptions || c.full)
            && self.description != updated.description
        {
            res.push(Self::Diff::Description(updated.description.clone()));
        }

        res
    }

    fn diff_ref<'target>(&'target self, _updated: &'target Self) -> Vec<Self::DiffRef<'target>> {
        unimplemented!()
    }

    fn apply_single(&mut self, _diff: Self::Diff) {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Difference, Clone)]
#[serde(untagged)]
pub enum LiteralValue {
    String(String),
    UInt(u64),
    Int(i64),
    Float(f64),
    Boolean(bool),
}

impl std::hash::Hash for LiteralValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        match self {
            Self::String(s) => s.hash(state),
            Self::UInt(u) => u.hash(state),
            Self::Int(i) => i.hash(state),
            Self::Float(f) => f.to_bits().hash(state),
            Self::Boolean(b) => b.hash(state),
        }
    }
}

impl Eq for LiteralValue {}

impl LiteralValue {
    #[must_use]
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_uint(&self) -> Option<u64> {
        match self {
            Self::UInt(u) => Some(*u),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl Default for LiteralValue {
    fn default() -> Self {
        Self::String(String::new())
    }
}
