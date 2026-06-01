//! Category path representation and parsing

use crate::error::{CategoryError, Result};
use std::fmt;

/// Category path representing a hierarchical location
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryPath {
    /// Path segments (e.g., ["preferences", "communication", "style"])
    segments: Vec<String>,
}

impl CategoryPath {
    /// Create a new category path from a string
    pub fn new(path: &str) -> Result<Self> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Ok(Self { segments: vec![] });
        }

        if !trimmed.starts_with('/') {
            return Err(CategoryError::InvalidPath(
                "Path must start with /".to_string(),
            ));
        }

        let segments: Vec<String> = trimmed
            .split('/')
            .skip(1) // Skip empty string before first /
            .map(|s| s.to_string())
            .collect();

        // Validate segments
        for segment in &segments {
            if segment.is_empty() {
                return Err(CategoryError::InvalidPath(
                    "Path cannot contain empty segments (//)".to_string(),
                ));
            }
            if segment.contains('.') || segment.contains("..") {
                return Err(CategoryError::InvalidPath(
                    "Path cannot contain '.' or '..'".to_string(),
                ));
            }
        }

        Ok(Self { segments })
    }

    /// Create a root path
    pub fn root() -> Self {
        Self { segments: vec![] }
    }

    /// Check if this is a root path
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the depth of the path (number of segments)
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Get the parent path
    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            return None;
        }
        let mut segments = self.segments.clone();
        segments.pop();
        Some(Self { segments })
    }

    /// Get the last segment (name)
    pub fn name(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }

    /// Append a child segment
    pub fn child(&self, name: &str) -> Result<Self> {
        if name.is_empty() {
            return Err(CategoryError::InvalidPath(
                "Child name cannot be empty".to_string(),
            ));
        }
        if name.contains('/') {
            return Err(CategoryError::InvalidPath(
                "Child name cannot contain /".to_string(),
            ));
        }

        let mut segments = self.segments.clone();
        segments.push(name.to_string());
        Ok(Self { segments })
    }

    /// Get all segments
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        if self.is_root() {
            return "/".to_string();
        }
        format!("/{}", self.segments.join("/"))
    }

    /// Check if this path is a descendant of another path
    pub fn is_descendant_of(&self, other: &CategoryPath) -> bool {
        if self.depth() <= other.depth() {
            return false;
        }
        self.segments.starts_with(&other.segments)
    }

    /// Check if this path is an ancestor of another path
    pub fn is_ancestor_of(&self, other: &CategoryPath) -> bool {
        other.is_descendant_of(self)
    }

    /// Get the common ancestor path
    pub fn common_ancestor(&self, other: &CategoryPath) -> Self {
        let mut common_segments = Vec::new();
        for (a, b) in self.segments.iter().zip(other.segments.iter()) {
            if a == b {
                common_segments.push(a.clone());
            } else {
                break;
            }
        }
        Self {
            segments: common_segments,
        }
    }
}

impl fmt::Display for CategoryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TryFrom<String> for CategoryPath {
    type Error = CategoryError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(&value)
    }
}

impl TryFrom<&str> for CategoryPath {
    type Error = CategoryError;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let path = CategoryPath::new("/preferences/communication/style").unwrap();
        assert_eq!(path.segments(), &["preferences", "communication", "style"]);
        assert_eq!(path.depth(), 3);
        assert_eq!(path.name(), Some("style"));
        assert_eq!(path.to_string(), "/preferences/communication/style");
    }

    #[test]
    fn test_root_path() {
        let path = CategoryPath::root();
        assert!(path.is_root());
        assert_eq!(path.depth(), 0);
        assert_eq!(path.name(), None);
        assert_eq!(path.to_string(), "/");
    }

    #[test]
    fn test_empty_path() {
        let path = CategoryPath::new("").unwrap();
        assert!(path.is_root());
        assert_eq!(path.to_string(), "/");
    }

    #[test]
    fn test_invalid_paths() {
        // Missing leading slash
        assert!(CategoryPath::new("preferences/communication").is_err());

        // Empty segments
        assert!(CategoryPath::new("/preferences//communication").is_err());

        // Dot segments
        assert!(CategoryPath::new("/preferences/./communication").is_err());
        assert!(CategoryPath::new("/preferences/../communication").is_err());
    }

    #[test]
    fn test_parent() {
        let path = CategoryPath::new("/preferences/communication/style").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "/preferences/communication");
        assert_eq!(parent.depth(), 2);

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent.to_string(), "/preferences");
        assert_eq!(grandparent.depth(), 1);

        let great_grandparent = grandparent.parent().unwrap();
        assert_eq!(great_grandparent.to_string(), "/");
        assert!(great_grandparent.is_root());

        assert!(great_grandparent.parent().is_none());
    }

    #[test]
    fn test_child() {
        let path = CategoryPath::new("/preferences").unwrap();
        let child = path.child("communication").unwrap();
        assert_eq!(child.to_string(), "/preferences/communication");

        let grandchild = child.child("style").unwrap();
        assert_eq!(grandchild.to_string(), "/preferences/communication/style");
    }

    #[test]
    fn test_invalid_child() {
        let path = CategoryPath::new("/preferences").unwrap();
        assert!(path.child("").is_err());
        assert!(path.child("sub/path").is_err());
    }

    #[test]
    fn test_descendant() {
        let parent = CategoryPath::new("/preferences").unwrap();
        let child = CategoryPath::new("/preferences/communication").unwrap();
        let grandchild = CategoryPath::new("/preferences/communication/style").unwrap();
        let unrelated = CategoryPath::new("/skills").unwrap();

        assert!(child.is_descendant_of(&parent));
        assert!(grandchild.is_descendant_of(&parent));
        assert!(grandchild.is_descendant_of(&child));
        assert!(!parent.is_descendant_of(&child));
        assert!(!unrelated.is_descendant_of(&parent));
    }

    #[test]
    fn test_ancestor() {
        let parent = CategoryPath::new("/preferences").unwrap();
        let child = CategoryPath::new("/preferences/communication").unwrap();

        assert!(parent.is_ancestor_of(&child));
        assert!(!child.is_ancestor_of(&parent));
    }

    #[test]
    fn test_common_ancestor() {
        let path1 = CategoryPath::new("/preferences/communication/style").unwrap();
        let path2 = CategoryPath::new("/preferences/programming/rust").unwrap();
        let common = path1.common_ancestor(&path2);

        assert_eq!(common.to_string(), "/preferences");
    }

    #[test]
    fn test_display() {
        let path = CategoryPath::new("/preferences/communication").unwrap();
        assert_eq!(format!("{}", path), "/preferences/communication");
    }

    #[test]
    fn test_try_from() {
        let path: Result<CategoryPath> = "/preferences/communication".try_into();
        assert!(path.is_ok());
        assert_eq!(path.unwrap().to_string(), "/preferences/communication");
    }
}
