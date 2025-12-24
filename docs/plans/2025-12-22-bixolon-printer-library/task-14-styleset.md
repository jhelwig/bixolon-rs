# Task 14: StyleSet and Resolution

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/style/mod.rs`
- Modify: `src/lib.rs` (add module)

**Important:** StyleSet uses `Option<T>` per property. Stack-based resolution applies styles from bottom to top, with later styles overriding earlier ones.

---

## Core Concepts

1. **StyleSet**: A set of optional style properties (not a delta)
2. **Stack resolution**: When nested styles exist, compute effective style by folding all StyleSets in the stack
3. **Command generation**: Generate commands to transition from one effective style to another

---

## Implementation

```rust
//! Style system for formatted text.
//!
//! Provides [`StyleSet`] for defining text formatting and stack-based
//! resolution for handling nested styles correctly.

pub mod text;

use crate::command::character::{
    CharacterSize, UnderlineMode, Justification,
};

/// A set of optional style properties.
///
/// Each property is `Option<T>` - `None` means "inherit from parent",
/// `Some(value)` means "override with this value".
///
/// # Stack Resolution
///
/// When styles are nested, use [`StyleSet::from_stack`] to compute the
/// effective style by folding all StyleSets from bottom to top:
///
/// ```
/// use bixolon::style::StyleSet;
///
/// let outer = StyleSet::default().with_bold(true);
/// let inner = StyleSet::default().with_underline(true);
/// let stack = [outer, inner];
///
/// let effective = StyleSet::from_stack(&stack);
/// assert_eq!(effective.bold, Some(true));      // from outer
/// assert_eq!(effective.underline, Some(true)); // from inner
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleSet {
    /// Bold/emphasized text.
    pub bold: Option<bool>,
    /// Underline mode.
    pub underline: Option<UnderlineMode>,
    /// Double-strike mode.
    pub double_strike: Option<bool>,
    /// Character size multiplier.
    pub size: Option<CharacterSize>,
    /// Reverse (white on black) mode.
    pub reverse: Option<bool>,
    /// Upside-down printing.
    pub upside_down: Option<bool>,
    /// 90-degree clockwise rotation.
    pub rotated: Option<bool>,
    /// Text justification.
    pub justification: Option<Justification>,
}

impl StyleSet {
    /// Create an empty style set (all properties inherit).
    pub const fn new() -> Self {
        Self {
            bold: None,
            underline: None,
            double_strike: None,
            size: None,
            reverse: None,
            upside_down: None,
            rotated: None,
            justification: None,
        }
    }

    /// Compute effective style by folding all StyleSets in the stack.
    ///
    /// Later (higher index) styles override earlier ones.
    pub fn from_stack(stack: &[StyleSet]) -> Self {
        let mut merged = StyleSet::new();
        for style in stack {
            if style.bold.is_some() {
                merged.bold = style.bold;
            }
            if style.underline.is_some() {
                merged.underline = style.underline;
            }
            if style.double_strike.is_some() {
                merged.double_strike = style.double_strike;
            }
            if style.size.is_some() {
                merged.size = style.size;
            }
            if style.reverse.is_some() {
                merged.reverse = style.reverse;
            }
            if style.upside_down.is_some() {
                merged.upside_down = style.upside_down;
            }
            if style.rotated.is_some() {
                merged.rotated = style.rotated;
            }
            if style.justification.is_some() {
                merged.justification = style.justification;
            }
        }
        merged
    }

    /// Set bold/emphasized.
    pub const fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    /// Set underline mode.
    pub const fn with_underline(mut self, mode: UnderlineMode) -> Self {
        self.underline = Some(mode);
        self
    }

    /// Convenience: enable single-line underline.
    pub const fn underlined(self) -> Self {
        self.with_underline(UnderlineMode::Single)
    }

    /// Set double-strike mode.
    pub const fn with_double_strike(mut self, enabled: bool) -> Self {
        self.double_strike = Some(enabled);
        self
    }

    /// Set character size.
    pub const fn with_size(mut self, size: CharacterSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Set reverse (white on black) mode.
    pub const fn with_reverse(mut self, enabled: bool) -> Self {
        self.reverse = Some(enabled);
        self
    }

    /// Set upside-down printing.
    pub const fn with_upside_down(mut self, enabled: bool) -> Self {
        self.upside_down = Some(enabled);
        self
    }

    /// Set 90-degree clockwise rotation.
    pub const fn with_rotated(mut self, enabled: bool) -> Self {
        self.rotated = Some(enabled);
        self
    }

    /// Set text justification.
    pub const fn with_justification(mut self, justification: Justification) -> Self {
        self.justification = Some(justification);
        self
    }

    /// Check if any property is set.
    pub fn is_empty(&self) -> bool {
        self.bold.is_none()
            && self.underline.is_none()
            && self.double_strike.is_none()
            && self.size.is_none()
            && self.reverse.is_none()
            && self.upside_down.is_none()
            && self.rotated.is_none()
            && self.justification.is_none()
    }
}

/// Compute commands needed to transition from one style to another.
///
/// This is used when rendering styled text to generate the minimal
/// set of ESC/POS commands needed between text segments.
pub fn style_transition_commands(from: &StyleSet, to: &StyleSet) -> Vec<Box<dyn crate::command::Command>> {
    use crate::command::character::{
        SetEmphasized, SetUnderline, SetDoubleStrike, SetCharacterSize,
        SetReverse, SetUpsideDown, SetRotation, SetJustification,
    };

    let mut commands: Vec<Box<dyn crate::command::Command>> = Vec::new();

    // Bold
    let from_bold = from.bold.unwrap_or(false);
    let to_bold = to.bold.unwrap_or(false);
    if from_bold != to_bold {
        commands.push(Box::new(SetEmphasized(to_bold)));
    }

    // Underline
    let from_underline = from.underline.unwrap_or(UnderlineMode::None);
    let to_underline = to.underline.unwrap_or(UnderlineMode::None);
    if from_underline != to_underline {
        commands.push(Box::new(SetUnderline(to_underline)));
    }

    // Double-strike
    let from_ds = from.double_strike.unwrap_or(false);
    let to_ds = to.double_strike.unwrap_or(false);
    if from_ds != to_ds {
        commands.push(Box::new(SetDoubleStrike(to_ds)));
    }

    // Size
    let from_size = from.size.unwrap_or_default();
    let to_size = to.size.unwrap_or_default();
    if from_size != to_size {
        commands.push(Box::new(SetCharacterSize(to_size)));
    }

    // Reverse
    let from_reverse = from.reverse.unwrap_or(false);
    let to_reverse = to.reverse.unwrap_or(false);
    if from_reverse != to_reverse {
        commands.push(Box::new(SetReverse(to_reverse)));
    }

    // Upside-down
    let from_upside = from.upside_down.unwrap_or(false);
    let to_upside = to.upside_down.unwrap_or(false);
    if from_upside != to_upside {
        commands.push(Box::new(SetUpsideDown(to_upside)));
    }

    // Rotated
    let from_rotated = from.rotated.unwrap_or(false);
    let to_rotated = to.rotated.unwrap_or(false);
    if from_rotated != to_rotated {
        commands.push(Box::new(SetRotation(to_rotated)));
    }

    // Justification
    let from_just = from.justification.unwrap_or(Justification::Left);
    let to_just = to.justification.unwrap_or(Justification::Left);
    if from_just != to_just {
        commands.push(Box::new(SetJustification(to_just)));
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styleset_default_is_empty() {
        let style = StyleSet::default();
        assert!(style.is_empty());
    }

    #[test]
    fn styleset_with_bold_not_empty() {
        let style = StyleSet::default().with_bold(true);
        assert!(!style.is_empty());
        assert_eq!(style.bold, Some(true));
    }

    #[test]
    fn from_stack_empty() {
        let result = StyleSet::from_stack(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn from_stack_single() {
        let style = StyleSet::default().with_bold(true);
        let result = StyleSet::from_stack(&[style]);
        assert_eq!(result.bold, Some(true));
    }

    #[test]
    fn from_stack_override() {
        let outer = StyleSet::default().with_bold(true);
        let inner = StyleSet::default().with_bold(false);
        let result = StyleSet::from_stack(&[outer, inner]);
        assert_eq!(result.bold, Some(false)); // inner overrides outer
    }

    #[test]
    fn from_stack_inherit() {
        let outer = StyleSet::default().with_bold(true);
        let inner = StyleSet::default().underlined();
        let result = StyleSet::from_stack(&[outer, inner]);
        assert_eq!(result.bold, Some(true)); // inherited from outer
        assert_eq!(result.underline, Some(UnderlineMode::Single)); // from inner
    }

    #[test]
    fn from_stack_nested_three_levels() {
        let level1 = StyleSet::default().with_bold(true).with_underline(UnderlineMode::Single);
        let level2 = StyleSet::default().with_bold(false); // override bold
        let level3 = StyleSet::default().with_reverse(true);

        let result = StyleSet::from_stack(&[level1, level2, level3]);
        assert_eq!(result.bold, Some(false)); // overridden at level2
        assert_eq!(result.underline, Some(UnderlineMode::Single)); // inherited from level1
        assert_eq!(result.reverse, Some(true)); // from level3
    }

    #[test]
    fn style_transition_no_change() {
        let style = StyleSet::default().with_bold(true);
        let commands = style_transition_commands(&style, &style);
        assert!(commands.is_empty());
    }

    #[test]
    fn style_transition_bold_on() {
        let from = StyleSet::default();
        let to = StyleSet::default().with_bold(true);
        let commands = style_transition_commands(&from, &to);
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn style_transition_multiple_changes() {
        let from = StyleSet::default();
        let to = StyleSet::default()
            .with_bold(true)
            .underlined()
            .with_reverse(true);
        let commands = style_transition_commands(&from, &to);
        assert_eq!(commands.len(), 3);
    }
}
```

---

## Commit

```bash
git add src/style/mod.rs src/lib.rs
git commit -m "feat: add StyleSet with stack-based resolution"
```
