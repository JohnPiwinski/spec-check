use crate::rust_parser::RustItem;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComparisonResult {
    pub missing_in_spec: Vec<RustItem>,
    pub missing_in_code: Vec<RustItem>,
    pub signature_mismatches: Vec<SignatureMismatch>,
    pub attribute_mismatches: Vec<AttributeMismatch>,
}

#[derive(Debug)]
pub struct SignatureMismatch {
    pub code_item: RustItem,
    pub spec_item: RustItem,
    pub first_diff_pos: Option<usize>,
}

#[derive(Debug)]
pub struct AttributeMismatch {
    pub code_item: RustItem,
    pub spec_item: RustItem,
}

impl ComparisonResult {
    pub fn has_errors(&self) -> bool {
        !self.missing_in_spec.is_empty() 
            || !self.missing_in_code.is_empty() 
            || !self.signature_mismatches.is_empty()
            || !self.attribute_mismatches.is_empty()
    }
}

fn find_first_diff(s1: &str, s2: &str) -> Option<usize> {
    s1.chars()
        .zip(s2.chars())
        .position(|(c1, c2)| c1 != c2)
        .or_else(|| {
            // If one string is a prefix of the other
            if s1.len() != s2.len() {
                Some(s1.len().min(s2.len()))
            } else {
                None
            }
        })
}

fn normalize_attributes(attrs: &[String], ignored_attributes: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = attrs.iter()
        .filter(|a| {
            // Check if any ignored attribute name is a prefix of this attribute
            // e.g., if ignored is "doc", it matches "#[doc = ...]"
            !ignored_attributes.iter().any(|ignored| {
                a.contains(ignored) || a.starts_with(&format!("#[{}(", ignored)) || a.starts_with(&format!("#[{}", ignored))
            })
        })
        .map(|a| a.trim().to_string())
        .collect();
    normalized.sort();
    normalized
}

pub fn compare_items(
    code_items: Vec<RustItem>,
    spec_items: Vec<RustItem>,
    ignored_attributes: &[String],
) -> ComparisonResult {
    // Create maps for efficient lookup by (name, kind)
    let mut code_map: HashMap<(String, String), &RustItem> = HashMap::new();
    let mut spec_map: HashMap<(String, String), &RustItem> = HashMap::new();
    
    for item in &code_items {
        let key = (item.name.clone(), format!("{:?}", item.kind));
        code_map.insert(key, item);
    }
    
    for item in &spec_items {
        let key = (item.name.clone(), format!("{:?}", item.kind));
        spec_map.insert(key, item);
    }
    
    let mut missing_in_spec = Vec::new();
    let mut missing_in_code = Vec::new();
    let mut signature_mismatches = Vec::new();
    let mut attribute_mismatches = Vec::new();
    
    // Check items in code
    for code_item in &code_items {
        let key = (code_item.name.clone(), format!("{:?}", code_item.kind));
        
        if let Some(spec_item) = spec_map.get(&key) {
            // Item exists in both - compare using token streams
            let code_tokens = code_item.tokens.to_string();
            let spec_tokens = spec_item.tokens.to_string();
            
            if code_tokens != spec_tokens {
                let first_diff_pos = find_first_diff(&code_item.signature, &spec_item.signature);
                signature_mismatches.push(SignatureMismatch {
                    code_item: code_item.clone(),
                    spec_item: (*spec_item).clone(),
                    first_diff_pos,
                });
            }
            
            // Check attributes
            let code_attrs = normalize_attributes(&code_item.attributes, ignored_attributes);
            let spec_attrs = normalize_attributes(&spec_item.attributes, ignored_attributes);
            
            if code_attrs != spec_attrs {
                attribute_mismatches.push(AttributeMismatch {
                    code_item: code_item.clone(),
                    spec_item: (*spec_item).clone(),
                });
            }
        } else {
            // Item in code but not in spec
            missing_in_spec.push(code_item.clone());
        }
    }
    
    // Check for items in spec but not in code
    for spec_item in &spec_items {
        let key = (spec_item.name.clone(), format!("{:?}", spec_item.kind));
        
        if !code_map.contains_key(&key) {
            missing_in_code.push(spec_item.clone());
        }
    }

    ComparisonResult {
        missing_in_spec,
        missing_in_code,
        signature_mismatches,
        attribute_mismatches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_parser::{RustItem, ItemKind};
    use quote::quote;

    #[test]
    fn test_identical_items() {
        let tokens = quote!(struct Foo {});
        let items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo {}".to_string(), tokens.clone(), vec![], 1),
        ];
        
        let result = compare_items(items.clone(), items, &[]);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_missing_in_spec() {
        let tokens = quote!(struct Foo {});
        let code_items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo {}".to_string(), tokens, vec![], 1),
        ];
        let spec_items = vec![];
        
        let result = compare_items(code_items, spec_items, &[]);
        assert_eq!(result.missing_in_spec.len(), 1);
        assert!(result.has_errors());
    }

    #[test]
    fn test_whitespace_independence() {
        let tokens1 = quote!(struct Foo { pub x: i32 });
        let tokens2 = quote!(struct Foo { pub x: i32 });
        let code_items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo{pub x:i32}".to_string(), tokens1, vec![], 1),
        ];
        let spec_items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo { pub x: i32 }".to_string(), tokens2, vec![], 1),
        ];
        
        let result = compare_items(code_items, spec_items, &[]);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_attribute_mismatch() {
        let tokens = quote!(struct Foo {});
        let code_items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo {}".to_string(), tokens.clone(), vec![], 1),
        ];
        let spec_items = vec![
            RustItem::new("Foo".to_string(), ItemKind::Struct, "struct Foo {}".to_string(), tokens, vec!["#[derive(Debug)]".to_string()], 1),
        ];
        
        let result = compare_items(code_items, spec_items, &[]);
        assert_eq!(result.attribute_mismatches.len(), 1);
        assert!(result.has_errors());
    }
}
