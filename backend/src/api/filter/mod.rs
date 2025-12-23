use anyhow::{anyhow, Result};
use std::fmt;

pub mod parser;
pub mod fields;
pub mod sql_builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,
    Ne,
    In,
    NotIn,
    Contains,
    NotContains,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl FilterOperator {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "eq" => Ok(Self::Eq),
            "ne" => Ok(Self::Ne),
            "in" => Ok(Self::In),
            "not_in" => Ok(Self::NotIn),
            "contains" => Ok(Self::Contains),
            "not_contains" => Ok(Self::NotContains),
            "gt" => Ok(Self::Gt),
            "gte" => Ok(Self::Gte),
            "lt" => Ok(Self::Lt),
            "lte" => Ok(Self::Lte),
            _ => Err(anyhow!("Unknown operator: {}", s)),
        }
    }
}

impl fmt::Display for FilterOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::In => "in",
            Self::NotIn => "not_in",
            Self::Contains => "contains",
            Self::NotContains => "not_contains",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Lt => "lt",
            Self::Lte => "lte",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    Single(String),
    List(Vec<String>),
}

impl FilterValue {
    pub fn as_single(&self) -> Result<&str> {
        match self {
            Self::Single(s) => Ok(s),
            Self::List(_) => Err(anyhow!("Expected single value, got list")),
        }
    }

    pub fn as_list(&self) -> Result<&[String]> {
        match self {
            Self::List(list) => Ok(list),
            Self::Single(_) => Err(anyhow!("Expected list value, got single")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterExpression {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

pub struct SqlFilter {
    pub where_clause: String,
    pub params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl fmt::Debug for SqlFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlFilter")
            .field("where_clause", &self.where_clause)
            .field("params_count", &self.params.len())
            .finish()
    }
}

impl SqlFilter {
    pub fn empty() -> Self {
        Self {
            where_clause: String::new(),
            params: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.where_clause.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Integer,
    Real,
    Text,
    Enum(&'static [&'static str]),
}

pub struct FieldDefinition {
    pub name: &'static str,
    pub sql_path: &'static str,
    pub field_type: FieldType,
    pub allowed_operators: &'static [FilterOperator],
}
