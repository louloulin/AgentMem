# AgentMem Category Hierarchy System

## Overview

The Category Hierarchy System provides a file-system-like organization for memory items, enabling hierarchical navigation and browsing of memories by topic rather than by type.

## Features

- **Hierarchical Organization**: Categories are organized in a tree structure with paths like `/preferences/communication/style`
- **Auto-Parent Creation**: Creating a category automatically creates all parent categories
- **Path Navigation**: Navigate to any category using familiar file-system paths
- **Multi-Tenancy**: Support for user_id and optional agent_id scoping
- **Semantic Search**: Category search by name and summary
- **Tree Operations**: Build and traverse category trees
- **Item Counting**: Track the number of memory items in each category

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-mem-category = "0.1.0"
```

## Quick Start

```rust
use agent_mem_category::{InMemoryCategoryManager, CategoryManager, CategoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = InMemoryCategoryManager::new();
    let scope = CategoryScope::new("user-123".to_string());

    // Create a category (automatically creates parents)
    let category = manager.create_category(
        "/preferences/communication/style",
        scope.clone()
    ).await?;

    println!("Created category: {}", category.name);

    // Navigate to a category
    let category = manager.navigate_path(
        "/preferences/communication",
        &scope
    ).await?;

    println!("Navigated to: {}", category.name);

    // Browse children
    let children = manager.browse_path(
        "/preferences/communication",
        &scope
    ).await?;

    println!("Children: {} items", children.len());

    Ok(())
}
```

## Core Concepts

### Category

A `Category` represents a folder-like entity in the hierarchy:

- `id`: Unique identifier
- `path`: Hierarchical path (e.g., "/preferences/communication/style")
- `name`: Display name (e.g., "style")
- `parent_id`: Parent category ID
- `children_ids`: Child category IDs
- `summary`: Optional LLM-generated summary
- `embedding`: Optional embedding for semantic search
- `item_count`: Number of memory items in this category
- `status`: Active, Archived, or Deleted

### CategoryPath

A `CategoryPath` represents a hierarchical location:

```rust
use agent_mem_category::CategoryPath;

let path = CategoryPath::new("/preferences/communication/style")?;
assert_eq!(path.depth(), 3);
assert_eq!(path.name(), Some("style"));
```

### CategoryScope

A `CategoryScope` provides multi-tenancy support:

```rust
use agent_mem_category::CategoryScope;

// User scope
let scope = CategoryScope::new("user-123".to_string());

// Agent scope
let scope = CategoryScope::with_agent(
    "user-123".to_string(),
    "agent-456".to_string()
);
```

## API Reference

### CategoryManager Trait

The `CategoryManager` trait defines all category operations:

#### Create Category

```rust
async fn create_category(
    &mut self,
    path: &str,
    scope: CategoryScope
) -> Result<Category>
```

Creates a new category at the given path. Automatically creates parent categories if they don't exist.

#### Get Category

```rust
async fn get_category(&self, id: &CategoryId) -> Result<Category>
async fn get_category_by_path(&self, path: &str, scope: &CategoryScope) -> Result<Category>
```

Retrieves a category by ID or path.

#### Navigate

```rust
async fn navigate_path(&self, path: &str, scope: &CategoryScope) -> Result<Category>
async fn browse_path(&self, path: &str, scope: &CategoryScope) -> Result<Vec<Category>>
```

Navigate to a category or browse its children.

#### Search

```rust
async fn search_categories(
    &self,
    query: &str,
    scope: &CategoryScope,
    limit: usize
) -> Result<Vec<Category>>
```

Search categories by name or summary.

#### Tree Operations

```rust
async fn get_tree(
    &self,
    path: &str,
    scope: &CategoryScope,
    depth: usize
) -> Result<CategoryTreeNode>
```

Build a category tree rooted at a path.

#### Update Operations

```rust
async fn update_category(&mut self, category: Category) -> Result<()>
async fn update_summary(&mut self, id: &CategoryId, summary: String) -> Result<()>
async fn increment_item_count(&mut self, id: &CategoryId) -> Result<()>
async fn decrement_item_count(&mut self, id: &CategoryId) -> Result<()>
```

Update category properties.

#### Move

```rust
async fn move_category(
    &mut self,
    id: &CategoryId,
    new_parent_path: &str,
    scope: &CategoryScope
) -> Result<()>
```

Move a category to a new parent.

#### Delete

```rust
async fn delete_category(&mut self, id: &CategoryId) -> Result<()>
```

Soft delete a category (sets status to Deleted).

## Advanced Usage

### Building Category Trees

```rust
use agent_mem_category::{CategoryManager, CategoryTreeNode};

let tree = manager.get_tree("/preferences", &scope, 3).await?;
println!("{}", tree.pretty_print(0));
```

### Category Path Manipulation

```rust
use agent_mem_category::CategoryPath;

let path = CategoryPath::new("/preferences/communication/style")?;

// Get parent
let parent = path.parent().unwrap();
assert_eq!(parent.to_string(), "/preferences/communication");

// Add child
let child = path.child("formal")?;
assert_eq!(child.to_string(), "/preferences/communication/style/formal");

// Check relationships
let other = CategoryPath::new("/preferences/programming")?;
let common = path.common_ancestor(&other);
assert_eq!(common.to_string(), "/preferences");
```

### Item Count Management

```rust
let category = manager.create_category("/test", scope.clone()).await?;

// Increment item count
manager.increment_item_count(&category.id).await?;

// Decrement item count
manager.decrement_item_count(&category.id).await?;
```

## Testing

Run the test suite:

```bash
cargo test -p agent-mem-category
```

## Implementation Details

- **Storage**: In-memory HashMap-based storage (easily extensible to persistent backends)
- **Thread Safety**: Uses `Arc<RwLock<>>` for concurrent access
- **Async/Await**: Fully async API using Tokio
- **Error Handling**: Comprehensive error types using `thiserror`

## Future Enhancements

- Persistent storage backends (SQLite, PostgreSQL, LibSQL)
- LLM-driven summary generation
- Category embedding generation and semantic search
- Category import/export
- Category permissions and access control
- Category event notifications

## License

Apache-2.0

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.
