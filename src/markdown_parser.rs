use anyhow::Result;
use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind};

pub fn extract_rust_blocks(markdown: &str) -> Result<Vec<String>> {
    let parser = Parser::new(markdown);
    let mut rust_blocks = Vec::new();
    let mut in_rust_block = false;
    let mut current_block = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if lang.as_ref() == "rust" {
                    in_rust_block = true;
                    current_block.clear();
                }
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if lang.as_ref() == "rust" && in_rust_block {
                    rust_blocks.push(current_block.clone());
                    in_rust_block = false;
                }
            }
            Event::Text(text) => {
                if in_rust_block {
                    current_block.push_str(&text);
                }
            }
            _ => {}
        }
    }

    Ok(rust_blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_rust_block() {
        let markdown = r#"
# Title

Some text

```rust
pub struct MyStruct {}
```

More text
        "#;

        let blocks = extract_rust_blocks(markdown).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("pub struct MyStruct"));
    }

    #[test]
    fn test_extract_multiple_rust_blocks() {
        let markdown = r#"
```rust
pub struct First {}
```

```rust
pub struct Second {}
```
        "#;

        let blocks = extract_rust_blocks(markdown).unwrap();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_ignores_non_rust_blocks() {
        let markdown = r#"
```python
def foo():
    pass
```

```rust
pub struct MyStruct {}
```
        "#;

        let blocks = extract_rust_blocks(markdown).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("MyStruct"));
    }
}
