use anyhow::{anyhow, Context, Result};

use super::{FilterExpression, FilterOperator, FilterValue};

pub fn parse_filter_dsl(filter_str: &str) -> Result<Vec<FilterExpression>> {
    if filter_str.is_empty() {
        return Ok(Vec::new());
    }

    filter_str
        .split('|')
        .map(|expr| parse_single_expression(expr.trim()))
        .collect()
}

fn parse_single_expression(expr: &str) -> Result<FilterExpression> {
    let parts: Vec<&str> = expr.split(':').collect();

    if parts.len() != 3 {
        return Err(anyhow!(
            "Invalid filter expression '{}'. Expected format: field:operator:value",
            expr
        ));
    }

    let field = parts[0].trim().to_string();
    let operator_str = parts[1].trim();
    let value_str = parts[2].trim();

    let operator = FilterOperator::from_str(operator_str)
        .with_context(|| format!("Invalid operator '{}' in expression '{}'", operator_str, expr))?;

    let value = match operator {
        FilterOperator::In | FilterOperator::NotIn => {
            // Parse comma-separated list
            let list: Vec<String> = value_str
                .split(',')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect();

            if list.is_empty() {
                return Err(anyhow!(
                    "Operator '{}' requires at least one value in expression '{}'",
                    operator,
                    expr
                ));
            }

            if list.len() > 100 {
                return Err(anyhow!(
                    "Too many values ({}) for operator '{}' in expression '{}'. Maximum is 100.",
                    list.len(),
                    operator,
                    expr
                ));
            }

            FilterValue::List(list)
        }
        _ => {
            if value_str.is_empty() {
                return Err(anyhow!(
                    "Empty value for operator '{}' in expression '{}'",
                    operator,
                    expr
                ));
            }
            FilterValue::Single(value_str.to_string())
        }
    };

    Ok(FilterExpression {
        field,
        operator,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_filter() {
        let result = parse_filter_dsl("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_single_eq_expression() {
        let result = parse_filter_dsl("rating_type:eq:all");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].field, "rating_type");
        assert_eq!(exprs[0].operator, FilterOperator::Eq);
    }

    #[test]
    fn test_parse_in_expression() {
        let result = parse_filter_dsl("id:in:1,5,12");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].field, "id");
        assert_eq!(exprs[0].operator, FilterOperator::In);
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let result = parse_filter_dsl("rating_type:eq:all|name:contains:john");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 2);
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse_filter_dsl("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_operator() {
        let result = parse_filter_dsl("field:invalid:value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_value() {
        let result = parse_filter_dsl("field:eq:");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_in_list() {
        let result = parse_filter_dsl("id:in:");
        assert!(result.is_err());
    }
}
