use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::{FieldDefinition, FieldType, FilterOperator};

static RATING_TYPES: &[&str] = &["all", "1y", "2y", "3y", "4y", "5y"];
static CONFIDENCE_LEVELS: &[&str] = &["ESTABLISHED", "EMERGING", "PROVISIONAL", "UNRANKED"];

static FIELD_REGISTRY: Lazy<HashMap<&'static str, FieldDefinition>> = Lazy::new(|| {
    use FilterOperator::*;

    let mut registry = HashMap::new();

    registry.insert(
        "id",
        FieldDefinition {
            name: "id",
            sql_path: "p.id",
            field_type: FieldType::Integer,
            allowed_operators: &[Eq, Ne, In, NotIn, Gt, Gte, Lt, Lte],
        },
    );

    registry.insert(
        "name",
        FieldDefinition {
            name: "name",
            sql_path: "p.name",
            field_type: FieldType::Text,
            allowed_operators: &[Eq, Ne, Contains, NotContains],
        },
    );

    registry.insert(
        "rating",
        FieldDefinition {
            name: "rating",
            sql_path: "r.rating",
            field_type: FieldType::Real,
            allowed_operators: &[Eq, Ne, Gt, Gte, Lt, Lte],
        },
    );

    registry.insert(
        "rating_type",
        FieldDefinition {
            name: "rating_type",
            sql_path: "r.rating_type",
            field_type: FieldType::Enum(RATING_TYPES),
            allowed_operators: &[Eq, Ne, In, NotIn],
        },
    );

    registry.insert(
        "games_played",
        FieldDefinition {
            name: "games_played",
            sql_path: "r.games_played",
            field_type: FieldType::Integer,
            allowed_operators: &[Eq, Ne, Gt, Gte, Lt, Lte],
        },
    );

    registry.insert(
        "confidence_level",
        FieldDefinition {
            name: "confidence_level",
            sql_path: "r.confidence_level",
            field_type: FieldType::Enum(CONFIDENCE_LEVELS),
            allowed_operators: &[Eq, Ne, In, NotIn],
        },
    );

    registry.insert(
        "cuescore_id",
        FieldDefinition {
            name: "cuescore_id",
            sql_path: "p.cuescore_id",
            field_type: FieldType::Integer,
            allowed_operators: &[Eq, Ne, In, NotIn],
        },
    );

    registry
});

pub fn get_field_definition(field_name: &str) -> Result<&'static FieldDefinition> {
    FIELD_REGISTRY
        .get(field_name)
        .ok_or_else(|| {
            let valid_fields: Vec<&str> = FIELD_REGISTRY.keys().copied().collect();
            anyhow!(
                "Unknown field '{}'. Valid fields are: {}",
                field_name,
                valid_fields.join(", ")
            )
        })
}

pub fn validate_operator(field_def: &FieldDefinition, operator: FilterOperator) -> Result<()> {
    if field_def.allowed_operators.contains(&operator) {
        Ok(())
    } else {
        let allowed: Vec<String> = field_def
            .allowed_operators
            .iter()
            .map(|op| op.to_string())
            .collect();
        Err(anyhow!(
            "Operator '{}' is not allowed for field '{}'. Allowed operators: {}",
            operator,
            field_def.name,
            allowed.join(", ")
        ))
    }
}

pub fn validate_value(field_def: &FieldDefinition, value: &str) -> Result<()> {
    match field_def.field_type {
        FieldType::Integer => {
            value.parse::<i64>().map_err(|_| {
                anyhow!(
                    "Invalid integer value '{}' for field '{}'",
                    value,
                    field_def.name
                )
            })?;
        }
        FieldType::Real => {
            value.parse::<f64>().map_err(|_| {
                anyhow!(
                    "Invalid real number value '{}' for field '{}'",
                    value,
                    field_def.name
                )
            })?;
        }
        FieldType::Text => {
            // All strings are valid for text fields
        }
        FieldType::Enum(valid_values) => {
            if !valid_values.contains(&value) {
                return Err(anyhow!(
                    "Invalid value '{}' for field '{}'. Valid values are: {}",
                    value,
                    field_def.name,
                    valid_values.join(", ")
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid_field() {
        let result = get_field_definition("id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "id");
    }

    #[test]
    fn test_get_invalid_field() {
        let result = get_field_definition("invalid_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_allowed_operator() {
        let field_def = get_field_definition("id").unwrap();
        let result = validate_operator(field_def, FilterOperator::Eq);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_disallowed_operator() {
        let field_def = get_field_definition("name").unwrap();
        let result = validate_operator(field_def, FilterOperator::Gt);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_integer_value() {
        let field_def = get_field_definition("id").unwrap();
        assert!(validate_value(field_def, "123").is_ok());
        assert!(validate_value(field_def, "abc").is_err());
    }

    #[test]
    fn test_validate_real_value() {
        let field_def = get_field_definition("rating").unwrap();
        assert!(validate_value(field_def, "1500.5").is_ok());
        assert!(validate_value(field_def, "abc").is_err());
    }

    #[test]
    fn test_validate_enum_value() {
        let field_def = get_field_definition("rating_type").unwrap();
        assert!(validate_value(field_def, "all").is_ok());
        assert!(validate_value(field_def, "1y").is_ok());
        assert!(validate_value(field_def, "invalid").is_err());
    }

    #[test]
    fn test_validate_text_value() {
        let field_def = get_field_definition("name").unwrap();
        assert!(validate_value(field_def, "any text").is_ok());
    }
}
