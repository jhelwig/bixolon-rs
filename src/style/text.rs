//! Styled text AST and rendering.
//!
//! Provides [`StyledNode`] for representing formatted text as a tree,
//! and [`Styleable`] trait for ergonomic text styling.
//!
//! # Example
//!
//! ```
//! use bixolon::style::text::Styleable;
//!
//! // Simple styling
//! let node = "Hello".bold();
//!
//! // Nested styling
//! let node = "Bold and ".bold().append("underlined".underlined());
//!
//! // Complex nesting preserves all styles
//! let node = "outer".bold().append(
//!     "inner".underlined().append("deep".reversed())
//! );
//! ```

use super::{StyleSet, style_transition_commands};
use crate::command::Command;
use crate::command::basic::LineFeed;

/// A node in the styled text AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyledNode {
    /// Plain text without styling.
    Text(String),
    /// Styled content with children.
    Styled {
        /// Style applied to children.
        style: StyleSet,
        /// Child nodes.
        children: Vec<StyledNode>,
    },
}

impl StyledNode {
    /// Create a text node.
    pub fn text(s: impl Into<String>) -> Self {
        StyledNode::Text(s.into())
    }

    /// Create a styled node with a single text child.
    pub fn styled(style: StyleSet, content: impl Into<String>) -> Self {
        StyledNode::Styled {
            style,
            children: vec![StyledNode::Text(content.into())],
        }
    }

    /// Wrap this node with additional style.
    pub fn with_style(self, style: StyleSet) -> Self {
        StyledNode::Styled {
            style,
            children: vec![self],
        }
    }

    /// Append another node as a sibling.
    ///
    /// Creates a neutral container so styles don't leak between siblings.
    pub fn append(self, other: StyledNode) -> Self {
        // Always create a neutral wrapper to prevent style inheritance between siblings
        StyledNode::Styled {
            style: StyleSet::default(),
            children: vec![self, other],
        }
    }

    /// Render to bytes, including style commands.
    ///
    /// Returns the byte sequence ready to send to the printer.
    pub fn render(&self) -> Vec<u8> {
        let mut output = Vec::new();
        let mut style_stack: Vec<StyleSet> = vec![StyleSet::default()];
        let mut current_effective = StyleSet::default();

        self.render_recursive(&mut output, &mut style_stack, &mut current_effective);

        // Reset to default style at end
        let default_style = StyleSet::default();
        let reset_commands = style_transition_commands(&current_effective, &default_style);
        for cmd in reset_commands {
            output.extend(cmd);
        }

        output
    }

    /// Render to bytes and append a line feed.
    pub fn render_line(&self) -> Vec<u8> {
        let mut output = self.render();
        output.extend(LineFeed.encode());
        output
    }

    fn render_recursive(
        &self,
        output: &mut Vec<u8>,
        style_stack: &mut Vec<StyleSet>,
        current_effective: &mut StyleSet,
    ) {
        match self {
            StyledNode::Text(text) => {
                output.extend(text.as_bytes());
            }
            StyledNode::Styled {
                style,
                children,
            } => {
                // Push style onto stack
                style_stack.push(style.clone());

                // Compute new effective style
                let new_effective = StyleSet::from_stack(style_stack);

                // Generate transition commands
                let transition = style_transition_commands(current_effective, &new_effective);
                for cmd in transition {
                    output.extend(cmd);
                }
                *current_effective = new_effective;

                // Render children
                for child in children {
                    child.render_recursive(output, style_stack, current_effective);
                }

                // Pop style from stack
                style_stack.pop();

                // Compute style after popping
                let popped_effective = StyleSet::from_stack(style_stack);

                // Generate transition back
                let transition_back =
                    style_transition_commands(current_effective, &popped_effective);
                for cmd in transition_back {
                    output.extend(cmd);
                }
                *current_effective = popped_effective;
            }
        }
    }
}

impl From<&str> for StyledNode {
    fn from(s: &str) -> Self {
        StyledNode::Text(s.to_string())
    }
}

impl From<String> for StyledNode {
    fn from(s: String) -> Self {
        StyledNode::Text(s)
    }
}

/// Trait for ergonomic text styling.
///
/// Implemented for `&str` and `String` to allow fluent style application.
pub trait Styleable: Sized {
    /// Convert to a StyledNode.
    fn into_node(self) -> StyledNode;

    /// Apply bold/emphasized style.
    fn bold(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_bold(true))
    }

    /// Apply underline style.
    fn underlined(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_underline(true))
    }

    /// Apply double underline style.
    fn double_underlined(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_double_underline(true))
    }

    /// Apply reverse (white on black) style.
    fn reversed(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_reverse(true))
    }

    /// Apply double-strike style.
    fn double_strike(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_double_strike(true))
    }

    /// Apply upside-down style.
    fn upside_down(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_upside_down(true))
    }

    /// Apply 90-degree rotation.
    fn rotated(self) -> StyledNode {
        self.into_node().with_style(StyleSet::default().with_rotated(true))
    }

    /// Append another styled node.
    fn append(self, other: impl Styleable) -> StyledNode {
        self.into_node().append(other.into_node())
    }
}

impl Styleable for &str {
    fn into_node(self) -> StyledNode {
        StyledNode::Text(self.to_string())
    }
}

impl Styleable for String {
    fn into_node(self) -> StyledNode {
        StyledNode::Text(self)
    }
}

impl Styleable for StyledNode {
    fn into_node(self) -> StyledNode {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::ESC;

    #[test]
    fn text_node_renders_plain() {
        let node = StyledNode::text("Hello");
        assert_eq!(node.render(), b"Hello");
    }

    #[test]
    fn bold_renders_with_commands() {
        let node = "Hello".bold();
        let output = node.render();

        // Should have: ESC E 1, "Hello", ESC E 0
        assert!(output.starts_with(&[ESC, b'E', 1]));
        assert!(output.windows(5).any(|w| w == b"Hello"));
        assert!(output.ends_with(&[ESC, b'E', 0]));
    }

    #[test]
    fn underlined_renders_with_commands() {
        let node = "Hello".underlined();
        let output = node.render();

        // Should have: ESC - 1, "Hello", ESC - 0
        assert!(output.starts_with(&[ESC, b'-', 1]));
        assert!(output.ends_with(&[ESC, b'-', 0]));
    }

    #[test]
    fn nested_styles_both_apply() {
        // "Hello" with bold containing underline
        let node = "Hello".bold().with_style(StyleSet::default().with_underline(true));
        let output = node.render();

        // Should have underline on, then bold on
        let underline_on_pos = output.windows(3).position(|w| w == [ESC, b'-', 1]);
        let bold_on_pos = output.windows(3).position(|w| w == [ESC, b'E', 1]);

        assert!(underline_on_pos.is_some());
        assert!(bold_on_pos.is_some());
    }

    #[test]
    fn append_creates_siblings() {
        let node = "Hello ".bold().append("World".underlined());
        let output = node.render();

        // Both text segments should be present
        assert!(output.windows(6).any(|w| w == b"Hello "));
        assert!(output.windows(5).any(|w| w == b"World"));
    }

    #[test]
    fn render_line_appends_lf() {
        let node = StyledNode::text("Hello");
        let output = node.render_line();

        assert!(output.ends_with(&[0x0A])); // LF
    }

    #[test]
    fn complex_nesting_resolves_correctly() {
        // outer bold -> inner underline -> innermost: should have both bold AND underline
        let outer = StyleSet::default().with_bold(true);
        let inner = StyleSet::default().with_underline(true);

        let node = StyledNode::Styled {
            style: outer,
            children: vec![StyledNode::Styled {
                style: inner,
                children: vec![StyledNode::Text("test".to_string())],
            }],
        };

        let output = node.render();

        // Bold should be enabled
        assert!(output.windows(3).any(|w| w == [ESC, b'E', 1]));
        // Underline should be enabled
        assert!(output.windows(3).any(|w| w == [ESC, b'-', 1]));
        // Both should be disabled at the end
        assert!(output.windows(3).any(|w| w == [ESC, b'E', 0]));
        assert!(output.windows(3).any(|w| w == [ESC, b'-', 0]));
    }

    #[test]
    fn style_reset_after_pop() {
        // Test the "underline → bold → exit bold" case
        // After exiting bold, underline should still be active

        let underline_style = StyleSet::default().with_underline(true);
        let bold_style = StyleSet::default().with_bold(true);

        let node = StyledNode::Styled {
            style: underline_style,
            children: vec![
                StyledNode::Text("A".to_string()),
                StyledNode::Styled {
                    style: bold_style,
                    children: vec![StyledNode::Text("B".to_string())],
                },
                StyledNode::Text("C".to_string()),
            ],
        };

        let output = node.render();

        // Find positions
        let bytes: &[u8] = &output;

        // "A" comes after underline on
        // "B" comes after bold on (underline still on)
        // After "B", bold off but underline stays on
        // "C" printed with underline still on
        // At end, underline off

        // Count underline toggles
        let underline_on_count = bytes.windows(3).filter(|w| *w == [ESC, b'-', 1]).count();
        let underline_off_count = bytes.windows(3).filter(|w| *w == [ESC, b'-', 0]).count();

        // Should only turn underline on once at start and off once at end
        assert_eq!(underline_on_count, 1);
        assert_eq!(underline_off_count, 1);
    }
}
