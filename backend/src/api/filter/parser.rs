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

    let operator = FilterOperator::parse(operator_str)
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

    // Additional tests

    #[test]
    fn test_parse_all_comparison_operators() {
        let test_cases = vec![
            ("rating:gt:600", FilterOperator::Gt),
            ("rating:gte:600", FilterOperator::Gte),
            ("rating:lt:400", FilterOperator::Lt),
            ("rating:lte:400", FilterOperator::Lte),
            ("name:ne:John", FilterOperator::Ne),
        ];

        for (input, expected_op) in test_cases {
            let result = parse_filter_dsl(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let exprs = result.unwrap();
            assert_eq!(exprs[0].operator, expected_op, "Wrong operator for: {}", input);
        }
    }

    #[test]
    fn test_parse_contains_operator() {
        let result = parse_filter_dsl("name:contains:ski");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs[0].operator, FilterOperator::Contains);
        match &exprs[0].value {
            FilterValue::Single(s) => assert_eq!(s, "ski"),
            _ => panic!("Expected single value"),
        }
    }

    #[test]
    fn test_parse_not_contains_operator() {
        let result = parse_filter_dsl("name:not_contains:test");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs[0].operator, FilterOperator::NotContains);
    }

    #[test]
    fn test_parse_not_in_operator() {
        let result = parse_filter_dsl("id:not_in:1,2,3");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs[0].operator, FilterOperator::NotIn);
        match &exprs[0].value {
            FilterValue::List(list) => {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0], "1");
                assert_eq!(list[1], "2");
                assert_eq!(list[2], "3");
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_parse_in_with_whitespace() {
        let result = parse_filter_dsl("id:in: 1 , 2 , 3 ");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        match &exprs[0].value {
            FilterValue::List(list) => {
                assert_eq!(list.len(), 3);
                // Values should be trimmed
                assert_eq!(list[0], "1");
                assert_eq!(list[1], "2");
                assert_eq!(list[2], "3");
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_parse_three_expressions() {
        let result = parse_filter_dsl("rating:gt:500|name:contains:a|games:gte:10");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 3);
        assert_eq!(exprs[0].field, "rating");
        assert_eq!(exprs[1].field, "name");
        assert_eq!(exprs[2].field, "games");
    }

    #[test]
    fn test_parse_too_many_values_in_list() {
        // Build a list with 101 values (exceeds max of 100)
        let values: Vec<String> = (1..=101).map(|i| i.to_string()).collect();
        let input = format!("id:in:{}", values.join(","));
        let result = parse_filter_dsl(&input);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Too many values"));
    }

    #[test]
    fn test_parse_preserves_field_case() {
        let result = parse_filter_dsl("RatingType:eq:all");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs[0].field, "RatingType");
    }

    #[test]
    fn test_parse_value_with_special_chars() {
        let result = parse_filter_dsl("name:contains:O'Brien");
        assert!(result.is_ok());
        let exprs = result.unwrap();
        match &exprs[0].value {
            FilterValue::Single(s) => assert_eq!(s, "O'Brien"),
            _ => panic!("Expected single value"),
        }
    }

    #[test]
    fn test_filter_value_as_single() {
        let single = FilterValue::Single("test".to_string());
        assert_eq!(single.as_single().unwrap(), "test");

        let list = FilterValue::List(vec!["a".to_string()]);
        assert!(list.as_single().is_err());
    }

    #[test]
    fn test_filter_value_as_list() {
        let list = FilterValue::List(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(list.as_list().unwrap(), &["a".to_string(), "b".to_string()]);

        let single = FilterValue::Single("test".to_string());
        assert!(single.as_list().is_err());
    }
}
