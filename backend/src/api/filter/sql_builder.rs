use anyhow::{anyhow, Context, Result};

use super::fields::{get_field_definition, validate_operator, validate_value};
use super::{FilterExpression, FilterOperator, SqlFilter};

pub fn build_sql_filter(expressions: Vec<FilterExpression>) -> Result<SqlFilter> {
    if expressions.is_empty() {
        return Ok(SqlFilter::empty());
    }

    if expressions.len() > 10 {
        return Err(anyhow!(
            "Too many filter expressions ({}). Maximum is 10.",
            expressions.len()
        ));
    }

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    for expr in expressions {
        let (clause, mut expr_params) = build_expression_sql(&expr)
            .with_context(|| format!("Failed to build SQL for expression on field '{}'", expr.field))?;

        where_clauses.push(clause);
        params.append(&mut expr_params);
    }

    Ok(SqlFilter {
        where_clause: where_clauses.join(" AND "),
        params,
    })
}

fn build_expression_sql(
    expr: &FilterExpression,
) -> Result<(String, Vec<Box<dyn rusqlite::ToSql>>)> {
    // Get and validate field definition
    let field_def = get_field_definition(&expr.field)?;

    // Validate operator is allowed for this field
    validate_operator(field_def, expr.operator)?;

    let sql_path = field_def.sql_path;
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    let clause = match expr.operator {
        FilterOperator::Eq => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} = ?", sql_path)
        }
        FilterOperator::Ne => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} != ?", sql_path)
        }
        FilterOperator::In => {
            let values = expr.value.as_list()?;
            for value in values {
                validate_value(field_def, value)?;
                params.push(Box::new(value.clone()));
            }
            let placeholders = vec!["?"; values.len()].join(", ");
            format!("{} IN ({})", sql_path, placeholders)
        }
        FilterOperator::NotIn => {
            let values = expr.value.as_list()?;
            for value in values {
                validate_value(field_def, value)?;
                params.push(Box::new(value.clone()));
            }
            let placeholders = vec!["?"; values.len()].join(", ");
            format!("{} NOT IN ({})", sql_path, placeholders)
        }
        FilterOperator::Contains => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(format!("%{}%", value)));
            format!("{} LIKE ?", sql_path)
        }
        FilterOperator::NotContains => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(format!("%{}%", value)));
            format!("{} NOT LIKE ?", sql_path)
        }
        FilterOperator::Gt => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} > ?", sql_path)
        }
        FilterOperator::Gte => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} >= ?", sql_path)
        }
        FilterOperator::Lt => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} < ?", sql_path)
        }
        FilterOperator::Lte => {
            let value = expr.value.as_single()?;
            validate_value(field_def, value)?;
            params.push(Box::new(value.to_string()));
            format!("{} <= ?", sql_path)
        }
    };

    Ok((clause, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::filter::{FilterValue, FilterOperator};

    #[test]
    fn test_build_empty_filter() {
        let result = build_sql_filter(vec![]);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_build_eq_filter() {
        let expr = FilterExpression {
            field: "rating_type".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Single("all".to_string()),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.where_clause, "r.rating_type = ?");
        assert_eq!(filter.params.len(), 1);
    }

    #[test]
    fn test_build_in_filter() {
        let expr = FilterExpression {
            field: "id".to_string(),
            operator: FilterOperator::In,
            value: FilterValue::List(vec!["1".to_string(), "5".to_string(), "12".to_string()]),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.where_clause, "p.id IN (?, ?, ?)");
        assert_eq!(filter.params.len(), 3);
    }

    #[test]
    fn test_build_contains_filter() {
        let expr = FilterExpression {
            field: "name".to_string(),
            operator: FilterOperator::Contains,
            value: FilterValue::Single("john".to_string()),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.where_clause, "p.name LIKE ?");
        assert_eq!(filter.params.len(), 1);
    }

    #[test]
    fn test_build_multiple_filters() {
        let exprs = vec![
            FilterExpression {
                field: "rating_type".to_string(),
                operator: FilterOperator::Eq,
                value: FilterValue::Single("all".to_string()),
            },
            FilterExpression {
                field: "rating".to_string(),
                operator: FilterOperator::Gte,
                value: FilterValue::Single("1500".to_string()),
            },
        ];

        let result = build_sql_filter(exprs);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.where_clause, "r.rating_type = ? AND r.rating >= ?");
        assert_eq!(filter.params.len(), 2);
    }

    #[test]
    fn test_build_invalid_field() {
        let expr = FilterExpression {
            field: "invalid_field".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Single("value".to_string()),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_invalid_operator_for_field() {
        let expr = FilterExpression {
            field: "name".to_string(),
            operator: FilterOperator::Gt,
            value: FilterValue::Single("value".to_string()),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_invalid_value_type() {
        let expr = FilterExpression {
            field: "id".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::Single("not_a_number".to_string()),
        };

        let result = build_sql_filter(vec![expr]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_too_many_expressions() {
        let exprs: Vec<FilterExpression> = (0..11)
            .map(|_| FilterExpression {
                field: "rating_type".to_string(),
                operator: FilterOperator::Eq,
                value: FilterValue::Single("all".to_string()),
            })
            .collect();

        let result = build_sql_filter(exprs);
        assert!(result.is_err());
    }
}
