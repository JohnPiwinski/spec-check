use anyhow::Result;
use syn::{visit::Visit, File, ItemStruct, ItemTrait, ItemFn, ItemEnum, TraitItem, TraitItemFn, Visibility};
use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct RustItem {
    pub name: String,
    pub kind: ItemKind,
    pub signature: String,  // Original for display
    pub tokens: TokenStream,  // For comparison
    pub attributes: Vec<String>,
    pub line_number: usize,  // Line number in source file
}

// Manual PartialEq and Eq that only compare name and kind for HashSet
impl PartialEq for RustItem {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.kind == other.kind
    }
}

impl Eq for RustItem {}

impl std::hash::Hash for RustItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        format!("{:?}", self.kind).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Struct,
    Enum,
    Trait,
    TraitMethod { trait_name: String },
    Function,
}

impl RustItem {
    pub fn new(name: String, kind: ItemKind, signature: String, tokens: TokenStream, attributes: Vec<String>, line_number: usize) -> Self {
        Self { name, kind, signature, tokens, attributes, line_number }
    }
}

struct ItemCollector {
    items: Vec<RustItem>,
    current_trait: Option<String>,
    check_private: bool,
    source_text: String,  // Store source text for line number calculation
}

impl ItemCollector {
    fn new(check_private: bool, source_text: String) -> Self {
        Self {
            items: Vec::new(),
            current_trait: None,
            check_private,
            source_text,
        }
    }

    fn should_include(&self, vis: &Visibility) -> bool {
        self.check_private || matches!(vis, Visibility::Public(_))
    }

    /// Calculate line number by finding the identifier in source text
    /// This is a heuristic approach since proc_macro2 spans don't provide location info
    fn calculate_line_number(&self, ident_name: &str, search_start: usize) -> usize {
        // Find the identifier in the source text starting from search_start
        if let Some(pos) = self.source_text[search_start..].find(ident_name) {
            let actual_pos = search_start + pos;
            // Count newlines up to this position
            self.source_text[..actual_pos]
                .chars()
                .filter(|&c| c == '\n')
                .count() + 1
        } else {
            1  // Fallback to line 1 if not found
        }
    }
}

/// Recursively strip attributes from a syn node
trait StripAttrs {
    fn strip_attrs(&mut self);
}

impl StripAttrs for ItemStruct {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
        match &mut self.fields {
            syn::Fields::Named(fields) => {
                for field in &mut fields.named {
                    field.attrs.clear();
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &mut fields.unnamed {
                    field.attrs.clear();
                }
            }
            syn::Fields::Unit => {}
        }
    }
}

impl StripAttrs for ItemTrait {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
        for item in &mut self.items {
            match item {
                syn::TraitItem::Const(c) => c.attrs.clear(),
                syn::TraitItem::Fn(f) => f.attrs.clear(),
                syn::TraitItem::Type(t) => t.attrs.clear(),
                syn::TraitItem::Macro(m) => m.attrs.clear(),
                _ => {}
            }
        }
    }
}

impl StripAttrs for ItemFn {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
    }
}

impl StripAttrs for ItemEnum {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
        for variant in self.variants.iter_mut() {
            StripAttrs::strip_attrs(variant);
        }
    }
}

impl StripAttrs for syn::Variant {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
        match &mut self.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter_mut() {
                    field.attrs.clear();
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in fields.unnamed.iter_mut() {
                    field.attrs.clear();
                }
            }
            syn::Fields::Unit => {}
        }
    }
}

impl StripAttrs for TraitItemFn {
    fn strip_attrs(&mut self) {
        self.attrs.clear();
    }
}

impl<'ast> Visit<'ast> for ItemCollector {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.should_include(&node.vis) {
            let name = node.ident.to_string();
            let line_number = self.calculate_line_number(&name, 0);
            
            // Extract attributes
            let attributes: Vec<String> = node.attrs.iter()
                .map(|attr| quote::quote!(#attr).to_string())
                .collect();
            
            // Build signature and tokens without attributes
            let mut item_without_attrs = node.clone();
            item_without_attrs.strip_attrs();
            let signature = quote::quote!(#item_without_attrs).to_string();
            let tokens: TokenStream = quote::quote!(#item_without_attrs);
            
            self.items.push(RustItem::new(
                name,
                ItemKind::Struct,
                signature,
                tokens,
                attributes,
                line_number,
            ));
        }
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if self.should_include(&node.vis) {
            let name = node.ident.to_string();
            let line_number = self.calculate_line_number(&name, 0);
            
            // Extract attributes
            let attributes: Vec<String> = node.attrs.iter()
                .map(|attr| quote::quote!(#attr).to_string())
                .collect();
            
            // Build signature and tokens without attributes
            let mut item_without_attrs = node.clone();
            item_without_attrs.strip_attrs();
            let signature = quote::quote!(#item_without_attrs).to_string();
            let tokens: TokenStream = quote::quote!(#item_without_attrs);
            
            self.items.push(RustItem::new(
                name,
                ItemKind::Enum,
                signature,
                tokens,
                attributes,
                line_number,
            ));
        }
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if self.should_include(&node.vis) {
            let trait_name = node.ident.to_string();
            let line_number = self.calculate_line_number(&trait_name, 0);
            
            // Extract attributes
            let attributes: Vec<String> = node.attrs.iter()
                .map(|attr| quote::quote!(#attr).to_string())
                .collect();
            
            // Build signature and tokens without attributes
            let mut item_without_attrs = node.clone();
            item_without_attrs.strip_attrs();
            let signature = quote::quote!(#item_without_attrs).to_string();
            let tokens: TokenStream = quote::quote!(#item_without_attrs);
            
            self.items.push(RustItem::new(
                trait_name.clone(),
                ItemKind::Trait,
                signature,
                tokens,
                attributes,
                line_number,
            ));

            // Visit trait methods
            let old_trait = self.current_trait.replace(trait_name.clone());
            for item in &node.items {
                if let TraitItem::Fn(method) = item {
                    let method_name = method.sig.ident.to_string();
                    let line_number = self.calculate_line_number(&method_name, 0);
                    
                    // Extract attributes
                    let attributes: Vec<String> = method.attrs.iter()
                        .map(|attr| quote::quote!(#attr).to_string())
                        .collect();
                    
                    // Build signature and tokens without attributes
                    let mut method_without_attrs = method.clone();
                    method_without_attrs.strip_attrs();
                    let method_sig = quote::quote!(#method_without_attrs).to_string();
                    let method_tokens: TokenStream = quote::quote!(#method_without_attrs);
                    
                    self.items.push(RustItem::new(
                        method_name,
                        ItemKind::TraitMethod { trait_name: trait_name.clone() },
                        method_sig,
                        method_tokens,
                        attributes,
                        line_number,
                    ));
                }
            }
            self.current_trait = old_trait;
        }
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Only collect top-level functions (not trait methods or impl methods)
        if self.current_trait.is_none() && self.should_include(&node.vis) {
            let name = node.sig.ident.to_string();
            let line_number = self.calculate_line_number(&name, 0);
            
            // Extract attributes
            let attributes: Vec<String> = node.attrs.iter()
                .map(|attr| quote::quote!(#attr).to_string())
                .collect();
            
            // Build signature and tokens without attributes
            let mut item_without_attrs = node.clone();
            item_without_attrs.strip_attrs();
            let signature = quote::quote!(#item_without_attrs).to_string();
            let tokens: TokenStream = quote::quote!(#item_without_attrs);
            
            self.items.push(RustItem::new(
                name,
                ItemKind::Function,
                signature,
                tokens,
                attributes,
                line_number,
            ));
        }
    }
}

pub fn parse_rust_file(content: &str, check_private: bool) -> Result<Vec<RustItem>> {
    let syntax_tree: File = syn::parse_file(content)?;
    
    let mut collector = ItemCollector::new(check_private, content.to_string());
    collector.visit_file(&syntax_tree);
    
    Ok(collector.items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_struct() {
        let code = r#"
            pub struct MyStruct {
                pub field: i32,
            }
        "#;
        
        let items = parse_rust_file(code, false).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "MyStruct");
        assert!(matches!(items[0].kind, ItemKind::Struct));
    }

    #[test]
    fn test_parse_trait() {
        let code = r#"
            pub trait MyTrait {
                fn method(&self) -> i32;
            }
        "#;
        
        let items = parse_rust_file(code, false).unwrap();
        assert_eq!(items.len(), 2); // trait + method
        
        let trait_item = items.iter().find(|i| matches!(i.kind, ItemKind::Trait)).unwrap();
        assert_eq!(trait_item.name, "MyTrait");
        
        let method_item = items.iter().find(|i| matches!(i.kind, ItemKind::TraitMethod { .. })).unwrap();
        assert_eq!(method_item.name, "method");
    }

    #[test]
    fn test_ignores_private_items() {
        let code = r#"
            struct PrivateStruct {}
            pub struct PublicStruct {}
        "#;
        
        let items = parse_rust_file(code, false).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "PublicStruct");
    }

    #[test]
    fn test_parse_enum() {
        let code = r#"
            pub enum MyEnum {
                VariantA,
                VariantB(i32),
                VariantC { x: f32 },
            }
        "#;
        
        let items = parse_rust_file(code, false).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "MyEnum");
        assert!(matches!(items[0].kind, ItemKind::Enum));
    }
}
