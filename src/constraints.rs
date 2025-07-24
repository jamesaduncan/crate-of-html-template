//! Constraint evaluation system
//! 
//! This module implements the constraint evaluation system that handles
//! data-scope and data-constraint attributes for conditional rendering.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::types::{Constraint, ConstraintType};
use crate::value::RenderValue;

/// Context for constraint evaluation
pub struct ConstraintContext<'a> {
    /// Current data being rendered
    data: &'a dyn RenderValue,
    /// Map of element IDs to their data values
    id_map: HashMap<String, &'a dyn RenderValue>,
    /// Current scope name if any
    current_scope: Option<&'a str>,
}

impl<'a> ConstraintContext<'a> {
    /// Create a new constraint context
    pub fn new(data: &'a dyn RenderValue) -> Self {
        Self {
            data,
            id_map: HashMap::new(),
            current_scope: None,
        }
    }
    
    /// Set the current scope
    pub fn with_scope(mut self, scope: &'a str) -> Self {
        self.current_scope = Some(scope);
        self
    }
    
    /// Register an element with an @id
    pub fn register_id(&mut self, id: &str, value: &'a dyn RenderValue) {
        self.id_map.insert(id.to_string(), value);
    }
    
    /// Evaluate a constraint
    pub fn evaluate(&self, constraint: &Constraint) -> Result<bool> {
        match &constraint.constraint_type {
            ConstraintType::Scope(scope_name) => {
                // Check if we're in the specified scope
                Ok(self.current_scope == Some(scope_name))
            }
            ConstraintType::Expression(expr) => {
                self.evaluate_expression(expr)
            }
        }
    }
    
    /// Evaluate a constraint expression
    fn evaluate_expression(&self, expr: &str) -> Result<bool> {
        // Simple expression parser for constraints
        // Supports: property comparisons, existence checks, etc.
        
        let expr = expr.trim();
        
        // Check for existence operator (just a property name)
        if !expr.contains(['=', '!', '<', '>', ' ']) {
            return self.check_property_exists(expr);
        }
        
        // Check for equality
        if let Some((left, right)) = expr.split_once("==") {
            return self.evaluate_equality(left.trim(), right.trim());
        }
        
        // Check for inequality
        if let Some((left, right)) = expr.split_once("!=") {
            return self.evaluate_inequality(left.trim(), right.trim());
        }
        
        // Check for greater than or equal (must come before >)
        if let Some((left, right)) = expr.split_once(">=") {
            return self.evaluate_greater_equal(left.trim(), right.trim());
        }
        
        // Check for less than or equal (must come before <)
        if let Some((left, right)) = expr.split_once("<=") {
            return self.evaluate_less_equal(left.trim(), right.trim());
        }
        
        // Check for greater than
        if let Some((left, right)) = expr.split_once('>') {
            return self.evaluate_greater_than(left.trim(), right.trim());
        }
        
        // Check for less than
        if let Some((left, right)) = expr.split_once('<') {
            return self.evaluate_less_than(left.trim(), right.trim());
        }
        
        Err(Error::parse_owned(format!("Invalid constraint expression: {}", expr)))
    }
    
    /// Check if a property exists and is truthy
    fn check_property_exists(&self, prop: &str) -> Result<bool> {
        let value = self.resolve_value(prop)?;
        match value {
            Some(v) => {
                // Check for falsy values
                match v.as_str() {
                    "false" | "0" | "" => Ok(false),
                    _ => Ok(true),
                }
            }
            None => Ok(false),
        }
    }
    
    /// Evaluate equality comparison
    fn evaluate_equality(&self, left: &str, right: &str) -> Result<bool> {
        let left_val = self.resolve_value(left)?;
        let right_val = self.resolve_value(right)?;
        
        match (left_val, right_val) {
            (Some(l), Some(r)) => Ok(l == r),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// Evaluate inequality comparison
    fn evaluate_inequality(&self, left: &str, right: &str) -> Result<bool> {
        self.evaluate_equality(left, right).map(|result| !result)
    }
    
    /// Evaluate greater than comparison
    fn evaluate_greater_than(&self, left: &str, right: &str) -> Result<bool> {
        let left_val = self.resolve_value(left)?;
        let right_val = self.resolve_value(right)?;
        
        match (left_val, right_val) {
            (Some(l), Some(r)) => {
                // Try to parse as numbers
                if let (Ok(l_num), Ok(r_num)) = (l.parse::<f64>(), r.parse::<f64>()) {
                    Ok(l_num > r_num)
                } else {
                    // String comparison
                    Ok(l > r)
                }
            }
            _ => Ok(false),
        }
    }
    
    /// Evaluate less than comparison
    fn evaluate_less_than(&self, left: &str, right: &str) -> Result<bool> {
        let left_val = self.resolve_value(left)?;
        let right_val = self.resolve_value(right)?;
        
        match (left_val, right_val) {
            (Some(l), Some(r)) => {
                // Try to parse as numbers
                if let (Ok(l_num), Ok(r_num)) = (l.parse::<f64>(), r.parse::<f64>()) {
                    Ok(l_num < r_num)
                } else {
                    // String comparison
                    Ok(l < r)
                }
            }
            _ => Ok(false),
        }
    }
    
    /// Evaluate greater than or equal comparison
    fn evaluate_greater_equal(&self, left: &str, right: &str) -> Result<bool> {
        let less = self.evaluate_less_than(left, right)?;
        Ok(!less)
    }
    
    /// Evaluate less than or equal comparison
    fn evaluate_less_equal(&self, left: &str, right: &str) -> Result<bool> {
        let greater = self.evaluate_greater_than(left, right)?;
        Ok(!greater)
    }
    
    /// Resolve a value from a reference
    fn resolve_value(&self, reference: &str) -> Result<Option<String>> {
        // Check if it's a literal string
        if reference.starts_with('"') && reference.ends_with('"') {
            return Ok(Some(reference[1..reference.len()-1].to_string()));
        }
        
        // Check if it's a literal number
        if reference.parse::<f64>().is_ok() {
            return Ok(Some(reference.to_string()));
        }
        
        // Check if it's a boolean literal
        if reference == "true" || reference == "false" {
            return Ok(Some(reference.to_string()));
        }
        
        // Check if it's an @id reference
        if reference.starts_with('@') {
            let id = &reference[1..];
            if let Some(value) = self.id_map.get(id) {
                // For now, convert to string representation
                return Ok(value.get_property(&[]).map(|cow| cow.to_string()));
            }
            return Ok(None);
        }
        
        // Otherwise, treat as property path
        let path: Vec<String> = reference.split('.').map(String::from).collect();
        Ok(self.data.get_property(&path).map(|cow| cow.to_string()))
    }
}

/// Constraint evaluator that can be used during rendering
pub struct ConstraintEvaluator;

impl ConstraintEvaluator {
    /// Create a new constraint evaluator
    pub fn new() -> Self {
        Self
    }
    
    /// Check if an element should be rendered based on constraints
    pub fn should_render(
        &self,
        constraint: &Constraint,
        context: &ConstraintContext,
    ) -> Result<bool> {
        context.evaluate(constraint)
    }
}

impl Default for ConstraintEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_property_exists() {
        let data = json!({
            "name": "John",
            "age": 30
        });
        
        let context = ConstraintContext::new(&data);
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("name".to_string()),
            scope: None,
        };
        
        assert!(context.evaluate(&constraint).unwrap());
        
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("missing".to_string()),
            scope: None,
        };
        
        assert!(!context.evaluate(&constraint).unwrap());
    }
    
    #[test]
    fn test_equality_comparison() {
        let data = json!({
            "status": "active",
            "count": 5
        });
        
        let context = ConstraintContext::new(&data);
        
        // String equality
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("status == \"active\"".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
        
        // Number equality
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("count == 5".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
        
        // Inequality
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("status != \"inactive\"".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
    }
    
    #[test]
    fn test_numeric_comparisons() {
        let data = json!({
            "age": 25,
            "limit": 30
        });
        
        let context = ConstraintContext::new(&data);
        
        // Greater than
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("age > 20".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
        
        // Less than
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("age < limit".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
        
        // Greater than or equal
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("age >= 25".to_string()),
            scope: None,
        };
        assert!(context.evaluate(&constraint).unwrap());
    }
    
    #[test]
    fn test_scope_constraint() {
        let data = json!({});
        
        let context = ConstraintContext::new(&data).with_scope("admin");
        
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Scope("admin".to_string()),
            scope: Some("admin".to_string()),
        };
        
        assert!(context.evaluate(&constraint).unwrap());
        
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Scope("user".to_string()),
            scope: Some("user".to_string()),
        };
        
        assert!(!context.evaluate(&constraint).unwrap());
    }
    
    #[test]
    fn test_id_references() {
        let data = json!({
            "current": 25
        });
        
        let reference_data = json!(30);
        
        let mut context = ConstraintContext::new(&data);
        context.register_id("limit", &reference_data);
        
        let constraint = Constraint {
            element_selector: "div".to_string(),
            constraint_type: ConstraintType::Expression("current < @limit".to_string()),
            scope: None,
        };
        
        assert!(context.evaluate(&constraint).unwrap());
    }
}