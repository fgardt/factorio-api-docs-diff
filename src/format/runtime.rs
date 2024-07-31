use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use structdiff::StructDiff;

use super::{
    diff_helper::{self, vec_diff, DiffableVec, DiffableVecDiff, SingleDiff},
    prototype::LiteralValue,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RuntimeDoc {
    #[serde(flatten)]
    common: super::Common,

    pub classes: DiffableVec<Class>,
    pub events: DiffableVec<Event>,
    pub defines: DiffableVec<Define>,
    pub builtin_types: DiffableVec<BuiltinType>,
    pub concepts: DiffableVec<Concept>,
    pub global_objects: DiffableVec<GlobalObject>,
    pub global_functions: DiffableVec<Method>,
}

impl Deref for RuntimeDoc {
    type Target = super::Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl super::Doc for RuntimeDoc {
    type Diff = RuntimeDocDiff;

    fn diff(&self, other: &Self) -> Self::Diff {
        Self::Diff {
            classes: self.classes.diff(&other.classes),
            events: self.events.diff(&other.events),
            defines: self.defines.diff(&other.defines),
            builtin_types: self.builtin_types.diff(&other.builtin_types),
            concepts: self.concepts.diff(&other.concepts),
            global_objects: self.global_objects.diff(&other.global_objects),
            global_functions: self.global_functions.diff(&other.global_functions),
        }
    }
}

impl super::Info for RuntimeDoc {
    fn print_info(&self) {
        self.common.print_info();

        eprintln!(" - Classes:          {}", self.classes.len());
        eprintln!(" - Events:           {}", self.events.len());
        eprintln!(" - Defines:          {}", self.defines.len());
        eprintln!(" - Builtin Types:    {}", self.builtin_types.len());
        eprintln!(" - Concepts:         {}", self.concepts.len());
        eprintln!(" - Global Objects:   {}", self.global_objects.len());
        eprintln!(" - Global Functions: {}", self.global_functions.len());
    }
}

#[derive(Serialize)]
pub struct RuntimeDocDiff {
    pub classes: DiffableVecDiff<Class>,
    pub events: DiffableVecDiff<Event>,
    pub defines: DiffableVecDiff<Define>,
    pub builtin_types: DiffableVecDiff<BuiltinType>,
    pub concepts: DiffableVecDiff<Concept>,
    pub global_objects: DiffableVecDiff<GlobalObject>,
    pub global_functions: DiffableVecDiff<Method>,
}

impl super::Info for RuntimeDocDiff {
    fn print_info(&self) {
        eprintln!("=> {} classes changed", self.classes.len());
        eprintln!("=> {} events changed", self.events.len());
        eprintln!("=> {} defines changed", self.defines.len());
        eprintln!("=> {} builtin types changed", self.builtin_types.len());
        eprintln!("=> {} concepts changed", self.concepts.len());
        eprintln!("=> {} global objects changed", self.global_objects.len());
        eprintln!(
            "=> {} global functions changed",
            self.global_functions.len()
        );
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
pub struct Common {
    pub name: String,
    pub order: i16, // could be a float

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

impl diff_helper::Named for Common {
    fn name(&self) -> &str {
        &self.name
    }
}

pub type BuiltinType = Common;
pub type BasicMember = Common;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CommonDiff {
    Name(String),
    Order(i16),
    Description(String),
}

pub type BuiltinTypeDiff = CommonDiff;
pub type BasicMemberDiff = CommonDiff;

impl StructDiff for Common {
    type Diff = CommonDiff;
    type DiffRef<'target> = CommonDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let cli = crate::CLI.with_borrow(Clone::clone);
        let mut res = Vec::new();

        if self.name != updated.name {
            res.push(Self::Diff::Name(updated.name.clone()));
        }

        if self.description != updated.description && (cli.descriptions || cli.full) {
            res.push(Self::Diff::Description(updated.description.clone()));
        }

        if self.order != updated.order && cli.full {
            res.push(Self::Diff::Order(updated.order));
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
pub struct ExtendedCommon {
    #[serde(flatten)]
    common: Common,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

impl Deref for ExtendedCommon {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExtendedCommonDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
}

impl StructDiff for ExtendedCommon {
    type Diff = ExtendedCommonDiff;
    type DiffRef<'target> = ExtendedCommonDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let cli = crate::CLI.with_borrow(Clone::clone);
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.notes != updated.notes && (cli.descriptions || cli.full) {
            res.push(Self::Diff::Notes(updated.notes.clone()));
        }

        if self.examples != updated.examples && (cli.examples || cli.full) {
            res.push(Self::Diff::Examples(updated.examples.clone()));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Class {
    #[serde(flatten)]
    common: ExtendedCommon,

    pub methods: DiffableVec<Method>,
    pub attributes: DiffableVec<Attribute>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub operators: DiffableVec<Operator>,

    #[serde(rename = "abstract")]
    pub abstract_: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_classes: Vec<String>,
}

impl diff_helper::Named for Class {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Deref for Class {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ClassDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
    // class fields
    Abstract(bool),
    BaseClasses(Vec<String>),
    Methods(DiffableVecDiff<Method>),
    Attributes(DiffableVecDiff<Attribute>),
    Operators(DiffableVecDiff<Operator>),
}

impl StructDiff for Class {
    type Diff = ClassDiff;
    type DiffRef<'target> = ClassDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    ExtendedCommonDiff::Name(name) => Self::Diff::Name(name),
                    ExtendedCommonDiff::Order(order) => Self::Diff::Order(order),
                    ExtendedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    ExtendedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    ExtendedCommonDiff::Notes(notes) => Self::Diff::Notes(notes),
                };
                res.push(d);
            }
        }

        if self.abstract_ != updated.abstract_ {
            res.push(Self::Diff::Abstract(updated.abstract_));
        }

        if self.base_classes != updated.base_classes {
            res.push(Self::Diff::BaseClasses(updated.base_classes.clone()));
        }

        if self.methods != updated.methods {
            let diff = self.methods.diff(&updated.methods);

            if !diff.is_empty() {
                res.push(Self::Diff::Methods(diff));
            }
        }

        if self.attributes != updated.attributes {
            let diff = self.attributes.diff(&updated.attributes);

            if !diff.is_empty() {
                res.push(Self::Diff::Attributes(diff));
            }
        }

        if self.operators != updated.operators {
            let diff = self.operators.diff(&updated.operators);

            if !diff.is_empty() {
                res.push(Self::Diff::Operators(diff));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(untagged)]
pub enum Operator {
    Method(Method),
    Attribute(Attribute),

    #[default]
    #[serde(skip)]
    Unknown,
}

impl Deref for Operator {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Method(m) => m,
            Self::Attribute(a) => a,
            Self::Unknown => panic!("unknown operator"),
        }
    }
}

impl diff_helper::Named for Operator {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum OperatorDiff {
    Method(SingleDiff<Method>),
    Attribute(SingleDiff<Attribute>),
}

impl StructDiff for Operator {
    type Diff = OperatorDiff;
    type DiffRef<'target> = OperatorDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        match (self, updated) {
            (Self::Method(m), Self::Method(u)) => {
                let diff = m.diff(u);

                if !diff.is_empty() {
                    res.push(Self::Diff::Method(diff));
                }
            }
            (Self::Attribute(a), Self::Attribute(u)) => {
                let diff = a.diff(u);

                if !diff.is_empty() {
                    res.push(Self::Diff::Attribute(diff));
                }
            }
            (_, Self::Method(u)) => {
                res.push(Self::Diff::Method(Method::default().diff(u)));
            }
            (_, Self::Attribute(u)) => {
                res.push(Self::Diff::Attribute(Attribute::default().diff(u)));
            }
            (_, Self::Unknown) => {
                eprintln!("unknown operator");
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Event {
    #[serde(flatten)]
    common: ExtendedCommon,

    pub data: DiffableVec<Parameter>,
}

impl Deref for Event {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Event {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
    // event fields
    Data(DiffableVecDiff<Parameter>),
}

impl StructDiff for Event {
    type Diff = EventDiff;
    type DiffRef<'target> = EventDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    ExtendedCommonDiff::Name(name) => Self::Diff::Name(name),
                    ExtendedCommonDiff::Order(order) => Self::Diff::Order(order),
                    ExtendedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    ExtendedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    ExtendedCommonDiff::Notes(notes) => Self::Diff::Notes(notes),
                };
                res.push(d);
            }
        }

        if self.data != updated.data {
            let diff = self.data.diff(&updated.data);

            if !diff.is_empty() {
                res.push(Self::Diff::Data(diff));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Define {
    #[serde(flatten)]
    common: Common,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub values: DiffableVec<BasicMember>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub subkeys: DiffableVec<Define>,
}

impl Deref for Define {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Define {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DefineDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // define fields
    Values(DiffableVecDiff<BasicMember>),
    Subkeys(DiffableVecDiff<Define>),
}

impl StructDiff for Define {
    type Diff = DefineDiff;
    type DiffRef<'target> = DefineDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.values != updated.values {
            let diff = self.values.diff(&updated.values);

            if !diff.is_empty() {
                res.push(Self::Diff::Values(diff));
            }
        }

        if self.subkeys != updated.subkeys {
            let diff = self.subkeys.diff(&updated.subkeys);

            if !diff.is_empty() {
                res.push(Self::Diff::Subkeys(diff));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Concept {
    #[serde(flatten)]
    common: ExtendedCommon,

    #[serde(rename = "type")]
    pub type_: Type,
}

impl Deref for Concept {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Concept {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ConceptDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
    // concept fields
    Type(TypeDiff),
}

impl StructDiff for Concept {
    type Diff = ConceptDiff;
    type DiffRef<'target> = ConceptDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    ExtendedCommonDiff::Name(name) => Self::Diff::Name(name),
                    ExtendedCommonDiff::Order(order) => Self::Diff::Order(order),
                    ExtendedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    ExtendedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    ExtendedCommonDiff::Notes(notes) => Self::Diff::Notes(notes),
                };
                res.push(d);
            }
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);

            if !diff.is_empty() && !diff[0].skip() {
                res.push(Self::Diff::Type(diff[0].clone()));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct GlobalObject {
    #[serde(flatten)]
    common: Common,

    #[serde(rename = "type")]
    pub type_: Type,
}

impl Deref for GlobalObject {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for GlobalObject {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GlobalObjectDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // global object fields
    Type(TypeDiff),
}

impl StructDiff for GlobalObject {
    type Diff = GlobalObjectDiff;
    type DiffRef<'target> = GlobalObjectDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);

            if !diff.is_empty() && !diff[0].skip() {
                res.push(Self::Diff::Type(diff[0].clone()));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
pub struct EventRaised {
    #[serde(flatten)]
    common: Common,

    pub timeframe: TimeFrame,
    pub optional: bool,
}

impl Deref for EventRaised {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for EventRaised {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TimeFrame {
    #[default]
    Instantly,
    CurrentTick,
    FutureTick,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventRaisedDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // event raised fields
    Timeframe(TimeFrame),
    Optional(bool),
}

impl StructDiff for EventRaised {
    type Diff = EventRaisedDiff;
    type DiffRef<'target> = EventRaisedDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.timeframe != updated.timeframe {
            res.push(Self::Diff::Timeframe(updated.timeframe.clone()));
        }

        if self.optional != updated.optional {
            res.push(Self::Diff::Optional(updated.optional));
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
        Self::Simple(String::default())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum TypeDiff {
    Simple(String),
    Complex(Vec<<ComplexType as StructDiff>::Diff>),
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
    type DiffRef<'target> = TypeDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        match (self, updated) {
            (Self::Simple(s), Self::Simple(u_s)) => {
                if s != u_s {
                    res.push(Self::Diff::Simple(u_s.clone()));
                }
            }
            (Self::Complex(c), Self::Complex(u_c)) => {
                if c != u_c {
                    let diff = c.diff(u_c);

                    if !diff.is_empty() {
                        res.push(Self::Diff::Complex(diff));
                    }
                }
            }
            (_, Self::Simple(u_s)) => {
                res.push(Self::Diff::Simple(u_s.clone()));
            }
            (_, Self::Complex(u_c)) => {
                res.push(Self::Diff::Complex(ComplexType::Unknown.diff(u_c)));
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
    Type {
        value: Type,
        description: String,
    },
    Union {
        options: Vec<Type>,
        full_format: bool,
    },
    Array {
        value: Type,
    },
    Dictionary {
        key: Type,
        value: Type,
    },
    #[serde(rename = "LuaCustomTable")]
    LuaCustomTable {
        key: Type,
        value: Type,
    },
    Function {
        parameters: Vec<Type>,
    },
    Literal(super::prototype::Literal),
    #[serde(rename = "LuaLazyLoadedValue")]
    LuaLazyLoadedValue {
        value: Type,
    },
    #[serde(rename = "LuaStruct")]
    LuaStruct {
        attributes: Vec<Attribute>,
    },
    Table {
        parameters: Vec<Parameter>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        variant_parameter_groups: Vec<ParameterGroup>,

        #[serde(default, skip_serializing_if = "String::is_empty")]
        variant_parameter_description: String,
    },
    Tuple {
        parameters: Vec<Parameter>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        variant_parameter_groups: Vec<ParameterGroup>,

        #[serde(default, skip_serializing_if = "String::is_empty")]
        variant_parameter_description: String,
    },

    #[serde(skip)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ComplexTypeDiff {
    ComplexType(String),
    Value(TypeDiff),
    Key(TypeDiff),
    Options(Vec<TypeDiff>),
    FullFormat(bool),
    Description(String),
    Attributes(DiffableVecDiff<Attribute>),
    FunctionParameters(Vec<TypeDiff>),
    TableTupleParameters(DiffableVecDiff<Parameter>),
    VariantParameterGroups(DiffableVecDiff<ParameterGroup>),
    VariantParameterDescription(String),
    #[serde(rename = "value")]
    Literal(LiteralValue),
}

impl StructDiff for ComplexType {
    type Diff = ComplexTypeDiff;
    type DiffRef<'target> = ComplexTypeDiff;

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        match (self, updated) {
            (
                Self::Type { value, description },
                Self::Type {
                    value: u_value,
                    description: u_desc,
                },
            ) => {
                if value != u_value {
                    let diff = value.diff(u_value);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::Value(diff[0].clone()));
                    }
                }

                if crate::CLI.with_borrow(|c| c.descriptions || c.full) && description != u_desc {
                    res.push(Self::Diff::Description(u_desc.clone()));
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
                    res.push(Self::Diff::Options(
                        vec_diff(options, u_options)
                            .iter()
                            .flatten()
                            .filter(|o| !o.skip())
                            .cloned()
                            .collect(),
                    ));
                }

                if full_format != u_full_format {
                    res.push(Self::Diff::FullFormat(*u_full_format));
                }
            }
            (Self::Array { value }, Self::Array { value: u_value })
            | (Self::LuaLazyLoadedValue { value }, Self::LuaLazyLoadedValue { value: u_value }) => {
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
            )
            | (
                Self::LuaCustomTable { key, value },
                Self::LuaCustomTable {
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
            (
                Self::Function { parameters },
                Self::Function {
                    parameters: u_params,
                },
            ) => {
                if parameters != u_params {
                    res.push(Self::Diff::FunctionParameters(
                        vec_diff(parameters, u_params)
                            .iter()
                            .flatten()
                            .filter(|p| !p.skip())
                            .cloned()
                            .collect(),
                    ));
                }
            }
            (Self::Literal(l), Self::Literal(u_l)) => {
                if l != u_l {
                    let diff = l.diff(u_l);

                    for d in diff {
                        match d {
                            super::prototype::LiteralDiff::Value(v) => {
                                res.push(Self::Diff::Literal(v));
                            }
                            super::prototype::LiteralDiff::Description(d) => {
                                if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                                    res.push(Self::Diff::Description(d));
                                }
                            }
                        }
                    }
                }
            }
            (
                Self::LuaStruct { attributes },
                Self::LuaStruct {
                    attributes: u_attrs,
                },
            ) => {
                if attributes != u_attrs {
                    let orig: DiffableVec<Attribute> = attributes.clone().into();
                    let updated: DiffableVec<Attribute> = u_attrs.clone().into();
                    res.push(Self::Diff::Attributes(orig.diff(&updated)));
                }
            }
            (
                Self::Table {
                    parameters: param,
                    variant_parameter_groups: vparam_g,
                    variant_parameter_description: vparam_desc,
                },
                Self::Table {
                    parameters: u_param,
                    variant_parameter_groups: u_vparam_g,
                    variant_parameter_description: u_vparam_desc,
                },
            )
            | (
                Self::Tuple {
                    parameters: param,
                    variant_parameter_groups: vparam_g,
                    variant_parameter_description: vparam_desc,
                },
                Self::Tuple {
                    parameters: u_param,
                    variant_parameter_groups: u_vparam_g,
                    variant_parameter_description: u_vparam_desc,
                },
            ) => {
                if param != u_param {
                    let orig: DiffableVec<Parameter> = param.clone().into();
                    let updated: DiffableVec<Parameter> = u_param.clone().into();
                    res.push(Self::Diff::TableTupleParameters(orig.diff(&updated)));
                }

                if vparam_g != u_vparam_g {
                    let orig: DiffableVec<ParameterGroup> = vparam_g.clone().into();
                    let updated: DiffableVec<ParameterGroup> = u_vparam_g.clone().into();
                    res.push(Self::Diff::VariantParameterGroups(orig.diff(&updated)));
                }

                if crate::CLI.with_borrow(|c| c.descriptions || c.full)
                    && vparam_desc != u_vparam_desc
                {
                    res.push(Self::Diff::VariantParameterDescription(
                        u_vparam_desc.clone(),
                    ));
                }
            }
            _ => match updated {
                Self::Type { value, description } => {
                    res.push(Self::Diff::ComplexType("type".to_owned()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));

                    if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                        res.push(Self::Diff::Description(description.clone()));
                    }
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
                Self::Array { value } => {
                    res.push(Self::Diff::ComplexType("array".to_owned()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::Dictionary { key, value } => {
                    res.push(Self::Diff::ComplexType("dictionary".to_owned()));
                    res.push(Self::Diff::Key(Type::default().diff(key)[0].clone()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::LuaCustomTable { key, value } => {
                    res.push(Self::Diff::ComplexType("LuaCustomTable".to_owned()));
                    res.push(Self::Diff::Key(Type::default().diff(key)[0].clone()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::Function { parameters } => {
                    res.push(Self::Diff::ComplexType("function".to_owned()));
                    res.push(Self::Diff::FunctionParameters(
                        parameters
                            .iter()
                            .flat_map(|p| Type::default().diff(p))
                            .collect(),
                    ));
                }
                Self::Literal(l) => {
                    res.push(Self::Diff::ComplexType("literal".to_owned()));

                    let diff = super::prototype::Literal::default().diff(l);
                    for d in diff {
                        match d {
                            super::prototype::LiteralDiff::Value(v) => {
                                res.push(Self::Diff::Literal(v));
                            }
                            super::prototype::LiteralDiff::Description(d) => {
                                if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                                    res.push(Self::Diff::Description(d));
                                }
                            }
                        }
                    }
                }
                Self::LuaLazyLoadedValue { value } => {
                    res.push(Self::Diff::ComplexType("LuaLazyLoadedValue".to_owned()));
                    res.push(Self::Diff::Value(Type::default().diff(value)[0].clone()));
                }
                Self::LuaStruct { attributes } => {
                    res.push(Self::Diff::ComplexType("LuaStruct".to_owned()));

                    let attributes: DiffableVec<Attribute> = attributes.clone().into();
                    res.push(Self::Diff::Attributes(attributes.full()));
                }
                Self::Table {
                    parameters,
                    variant_parameter_groups,
                    variant_parameter_description,
                } => {
                    res.push(Self::Diff::ComplexType("table".to_owned()));

                    let params: DiffableVec<Parameter> = parameters.clone().into();
                    res.push(Self::Diff::TableTupleParameters(params.full()));

                    let groups: DiffableVec<ParameterGroup> =
                        variant_parameter_groups.clone().into();
                    res.push(Self::Diff::VariantParameterGroups(groups.full()));

                    if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                        res.push(Self::Diff::VariantParameterDescription(
                            variant_parameter_description.clone(),
                        ));
                    }
                }
                Self::Tuple {
                    parameters,
                    variant_parameter_groups,
                    variant_parameter_description,
                } => {
                    res.push(Self::Diff::ComplexType("tuple".to_owned()));

                    let params: DiffableVec<Parameter> = parameters.clone().into();
                    res.push(Self::Diff::TableTupleParameters(params.full()));

                    let groups: DiffableVec<ParameterGroup> =
                        variant_parameter_groups.clone().into();
                    res.push(Self::Diff::VariantParameterGroups(groups.full()));

                    if crate::CLI.with_borrow(|c| c.descriptions || c.full) {
                        res.push(Self::Diff::VariantParameterDescription(
                            variant_parameter_description.clone(),
                        ));
                    }
                }
                Self::Unknown => {
                    eprintln!("unknown complex type");
                }
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
pub struct Parameter {
    #[serde(flatten)]
    common: Common,

    #[serde(rename = "type")]
    pub type_: Type,
    pub optional: bool,
}

impl Deref for Parameter {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Parameter {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // parameter fields
    Type(TypeDiff),
    Optional(bool),
}

impl StructDiff for Parameter {
    type Diff = ParameterDiff;
    type DiffRef<'target> = ParameterDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);

            if !diff.is_empty() && !diff[0].skip() {
                res.push(Self::Diff::Type(diff[0].clone()));
            }
        }

        if self.optional != updated.optional {
            res.push(Self::Diff::Optional(updated.optional));
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
pub struct ReturnParameter {
    pub order: i16, // could be a float
    pub description: String,

    #[serde(rename = "type")]
    pub type_: Type,
    pub optional: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReturnParameterDiff {
    Order(i16),
    Description(String),
    Type(TypeDiff),
    Optional(bool),
}

impl StructDiff for ReturnParameter {
    type Diff = ReturnParameterDiff;
    type DiffRef<'target> = ReturnParameterDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.order != updated.order && crate::CLI.with_borrow(|c| c.full) {
            res.push(Self::Diff::Order(updated.order));
        }

        if self.description != updated.description
            && crate::CLI.with_borrow(|c| c.descriptions || c.full)
        {
            res.push(Self::Diff::Description(updated.description.clone()));
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);

            if !diff.is_empty() && !diff[0].skip() {
                res.push(Self::Diff::Type(diff[0].clone()));
            }
        }

        if self.optional != updated.optional {
            res.push(Self::Diff::Optional(updated.optional));
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
pub struct ParameterGroup {
    #[serde(flatten)]
    common: Common,

    pub parameters: Vec<Parameter>,
}

impl Deref for ParameterGroup {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for ParameterGroup {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParameterGroupDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // parameter group fields
    Parameters(DiffableVecDiff<Parameter>),
}

impl StructDiff for ParameterGroup {
    type Diff = ParameterGroupDiff;
    type DiffRef<'target> = ParameterGroupDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    CommonDiff::Name(name) => Self::Diff::Name(name),
                    CommonDiff::Order(order) => Self::Diff::Order(order),
                    CommonDiff::Description(desc) => Self::Diff::Description(desc),
                };
                res.push(d);
            }
        }

        if self.parameters != updated.parameters {
            let orig: DiffableVec<Parameter> = self.parameters.clone().into();
            let updated: DiffableVec<Parameter> = updated.parameters.clone().into();
            let diff = orig.diff(&updated);

            if !diff.is_empty() {
                res.push(Self::Diff::Parameters(diff));
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Method {
    #[serde(flatten)]
    common: ExtendedCommon,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub raises: DiffableVec<EventRaised>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subclasses: Vec<String>,

    pub parameters: DiffableVec<Parameter>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variant_parameter_groups: DiffableVec<ParameterGroup>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub variant_parameter_description: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variadic_type: Option<Type>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub variadic_description: String,

    pub takes_table: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_is_optional: Option<bool>,

    pub return_values: Vec<ReturnParameter>,
}

impl Deref for Method {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Method {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MethodDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
    // method fields
    Raises(DiffableVecDiff<EventRaised>),
    Subclasses(Vec<String>),
    Parameters(DiffableVecDiff<Parameter>),
    VariantParameterGroups(DiffableVecDiff<ParameterGroup>),
    VariantParameterDescription(String),
    VariadicType(Option<TypeDiff>),
    VariadicDescription(String),
    TakesTable(bool),
    TableIsOptional(Option<bool>),
    ReturnValues(Vec<SingleDiff<ReturnParameter>>),
}

impl StructDiff for Method {
    type Diff = MethodDiff;
    type DiffRef<'target> = MethodDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    ExtendedCommonDiff::Name(name) => Self::Diff::Name(name),
                    ExtendedCommonDiff::Order(order) => Self::Diff::Order(order),
                    ExtendedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    ExtendedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    ExtendedCommonDiff::Notes(notes) => Self::Diff::Notes(notes),
                };
                res.push(d);
            }
        }

        if self.raises != updated.raises {
            let diff = self.raises.diff(&updated.raises);

            if !diff.is_empty() {
                res.push(Self::Diff::Raises(diff));
            }
        }

        if self.subclasses != updated.subclasses {
            res.push(Self::Diff::Subclasses(updated.subclasses.clone()));
        }

        if self.parameters != updated.parameters {
            let diff = self.parameters.diff(&updated.parameters);

            if !diff.is_empty() {
                res.push(Self::Diff::Parameters(diff));
            }
        }

        if self.variant_parameter_groups != updated.variant_parameter_groups {
            let diff = self
                .variant_parameter_groups
                .diff(&updated.variant_parameter_groups);

            if !diff.is_empty() {
                res.push(Self::Diff::VariantParameterGroups(diff));
            }
        }

        if self.variant_parameter_description != updated.variant_parameter_description
            && crate::CLI.with_borrow(|c| c.descriptions || c.full)
        {
            res.push(Self::Diff::VariantParameterDescription(
                updated.variant_parameter_description.clone(),
            ));
        }

        if self.variadic_type != updated.variadic_type {
            match (&self.variadic_type, &updated.variadic_type) {
                (Some(t), Some(u_t)) => {
                    let diff = t.diff(u_t);

                    if !diff.is_empty() && !diff[0].skip() {
                        res.push(Self::Diff::VariadicType(Some(diff[0].clone())));
                    }
                }
                (None, Some(u_t)) => {
                    res.push(Self::Diff::VariadicType(Some(
                        Type::default().diff(u_t)[0].clone(),
                    )));
                }
                (_, None) => {
                    res.push(Self::Diff::VariadicType(None));
                }
            }
        }

        if self.variadic_description != updated.variadic_description
            && crate::CLI.with_borrow(|c| c.descriptions || c.full)
        {
            res.push(Self::Diff::VariadicDescription(
                updated.variadic_description.clone(),
            ));
        }

        if self.takes_table != updated.takes_table {
            res.push(Self::Diff::TakesTable(updated.takes_table));
        }

        if self.table_is_optional != updated.table_is_optional {
            res.push(Self::Diff::TableIsOptional(updated.table_is_optional));
        }

        if self.return_values != updated.return_values {
            res.push(Self::Diff::ReturnValues(vec_diff(
                &self.return_values,
                &updated.return_values,
            )));
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
pub struct Attribute {
    #[serde(flatten)]
    common: ExtendedCommon,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raises: Vec<EventRaised>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subclasses: Vec<String>,

    #[serde(rename = "type")]
    pub type_: Type,

    pub optional: bool,
    pub read: bool,
    pub write: bool,
}

impl Deref for Attribute {
    type Target = ExtendedCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl diff_helper::Named for Attribute {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AttributeDiff {
    // common fields
    Name(String),
    Order(i16),
    Description(String),
    // extended fields
    Notes(Vec<String>),
    Examples(Vec<String>),
    // attribute fields
    Raises(DiffableVecDiff<EventRaised>),
    Subclasses(Vec<String>),
    Type(TypeDiff),
    Optional(bool),
    Read(bool),
    Write(bool),
}

impl StructDiff for Attribute {
    type Diff = AttributeDiff;
    type DiffRef<'target> = AttributeDiff;

    fn diff(&self, updated: &Self) -> Vec<Self::Diff> {
        let mut res = Vec::new();

        if self.common != updated.common {
            let common_diff = self.common.diff(&updated.common);

            for d in common_diff {
                let d = match d {
                    ExtendedCommonDiff::Name(name) => Self::Diff::Name(name),
                    ExtendedCommonDiff::Order(order) => Self::Diff::Order(order),
                    ExtendedCommonDiff::Description(desc) => Self::Diff::Description(desc),
                    ExtendedCommonDiff::Examples(examples) => Self::Diff::Examples(examples),
                    ExtendedCommonDiff::Notes(notes) => Self::Diff::Notes(notes),
                };
                res.push(d);
            }
        }

        if self.raises != updated.raises {
            let orig: DiffableVec<EventRaised> = self.raises.clone().into();
            let updated: DiffableVec<EventRaised> = updated.raises.clone().into();
            let diff = orig.diff(&updated);

            if !diff.is_empty() {
                res.push(Self::Diff::Raises(diff));
            }
        }

        if self.subclasses != updated.subclasses {
            res.push(Self::Diff::Subclasses(updated.subclasses.clone()));
        }

        if self.type_ != updated.type_ {
            let diff = self.type_.diff(&updated.type_);

            if !diff.is_empty() && !diff[0].skip() {
                res.push(Self::Diff::Type(diff[0].clone()));
            }
        }

        if self.optional != updated.optional {
            res.push(Self::Diff::Optional(updated.optional));
        }

        if self.read != updated.read {
            res.push(Self::Diff::Read(updated.read));
        }

        if self.write != updated.write {
            res.push(Self::Diff::Write(updated.write));
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
