# Mouse Click to Move Cursor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add mouse click navigation to Ki editor - users can move cursor by left-clicking within the editor content area.

**Architecture:** Modify Component trait to pass Context to mouse events, implement coordinate conversion from screen to buffer coordinates, validate click position, and move cursor. Reuse existing `set_cursor_position` function.

**Tech Stack:** Rust, crossterm (mouse events), ropey (buffer operations), existing Ki editor infrastructure.

---

## File Structure

| File | Responsibility |
|-------|---------------|
| `src/rectangle.rs` | Add `contains` method for point-in-rectangle check |
| `src/components/component.rs` | Add `context: &Context` parameter to `handle_mouse_event` trait |
| `src/components/editor.rs` | Implement `handle_mouse_click`, `get_line_number_width`, update `handle_mouse_event` signature |
| `src/components/test_editor.rs` | Add unit tests for mouse click behavior |

---

## Chunk 1: Rectangle::contains

### Task 1: Add Rectangle::contains method

**Files:**
- Modify: `src/rectangle.rs`

- [ ] **Step 1: Verify Rectangle structure**

Verify Rectangle struct has expected fields (origin: Position, width: usize, height: usize).
Reference: Rectangle is defined at src/rectangle.rs:11-17 with fields confirmed by reading the file.

- [ ] **Step 2: Write failing test for contains method**

Add test module at end of `src/rectangle.rs` (before any `#[cfg(test)]` modules, or after existing test modules):

```rust
#[cfg(test)]
mod test_contains {
    use super::*;

    #[test]
    fn test_rectangle_contains_inside_points() {
        let rect = Rectangle {
            origin: Position::new(2, 3),
            width: 5,
            height: 4,
        };

        // Inside points (inclusive of origin, exclusive of bounds)
        assert!(rect.contains(&Position::new(2, 3)));
        assert!(rect.contains(&Position::new(5, 7)));
        assert!(rect.contains(&Position::new(2, 4)));
        assert!(rect.contains(&Position::new(3, 3)));
    }

    #[test]
    fn test_rectangle_contains_outside_points() {
        let rect = Rectangle {
            origin: Position::new(2, 3),
            width: 5,
            height: 4,
        };

        // Outside points
        assert!(!rect.contains(&Position::new(1, 3)));
        assert!(!rect.contains(&Position::new(6, 3)));
        assert!(!rect.contains(&Position::new(2, 2)));
        assert!(!rect.contains(&Position::new(2, 8)));
        assert!(!rect.contains(&Position::new(1, 2)));
    }

    #[test]
    fn test_rectangle_contains_boundary() {
        let rect = Rectangle {
            origin: Position::new(0, 0),
            width: 10,
            height: 5,
        };

        // Origin is inside
        assert!(rect.contains(&Position::new(0, 0)));

        // Right edge (column 10) is outside
        assert!(!rect.contains(&Position::new(0, 10)));

        // Bottom edge (line 5) is outside
        assert!(!rect.contains(&Position::new(5, 0)));

        // Last valid column (9) is inside
        assert!(rect.contains(&Position::new(0, 9)));

        // Last valid line (4) is inside
        assert!(rect.contains(&Position::new(4, 9)));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test rectangle::test_contains`
Expected: FAIL with "no method named `contains`"

- [ ] **Step 4: Implement contains method**

Add to `impl Rectangle` block in `src/rectangle.rs`. Good location is after the Rectangle struct methods (before `fn generate_*` methods, approximately after line 120):

```rust
pub fn contains(&self, position: &Position) -> bool {
    position.line >= self.origin.line
        && position.line < self.origin.line + self.height
        && position.column >= self.origin.column
        && position.column < self.origin.column + self.width
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test rectangle::test_contains`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/rectangle.rs
git commit -m "feat(rectangle): add contains method for point-in-rectangle check

johncming@126.com
```

---

## Chunk 2: Component trait signature update

### Task 2: Update Component::handle_mouse_event signature

**Files:**
- Modify: `src/components/component.rs`
- Modify: `src/components/editor.rs`

- [ ] **Step 1: Find all implementations of handle_mouse_event**

Search for implementations:
```bash
rg "fn handle_mouse_event" --type rust
```
Expected results: component.rs trait definition and editor.rs implementation

- [ ] **Step 2: Update trait definition in component.rs**

Modify trait method in `src/components/component.rs` around line 143:

Change from:
```rust
fn handle_mouse_event(
    &mut self,
    event: crossterm::event::MouseEvent,
) -> anyhow::Result<Dispatches> {
    self.editor_mut().handle_mouse_event(event)
}
```

To:
```rust
fn handle_mouse_event(
    &mut self,
    event: crossterm::event::MouseEvent,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    self.editor_mut().handle_mouse_event(event, context)
}
```

- [ ] **Step 2.5: Verify trait signature change**

Run: `cargo check --package ki`
Expected: Compiler reports that Editor's `handle_mouse_event` signature doesn't match trait (missing `context` parameter)
This confirms the trait change is detected before we fix the implementation.

Change from:
```rust
fn handle_mouse_event(
    &mut self,
    event: crossterm::event::MouseEvent,
) -> anyhow::Result<Dispatches> {
    self.editor_mut().handle_mouse_event(event)
}
```

To:
```rust
fn handle_mouse_event(
    &mut self,
    event: crossterm::event::MouseEvent,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    self.editor_mut().handle_mouse_event(event, context)
}
```

- [ ] **Step 3: Update Editor implementation in editor.rs**

Modify implementation in `src/components/editor.rs` around line 142.

Change from:
```rust
fn handle_mouse_event(
    &mut self,
    mouse_event: crossterm::event::MouseEvent,
) -> anyhow::Result<Dispatches> {
    const SCROLL_HEIGHT: usize = 2;
    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            self.apply_scroll(Direction::Start, SCROLL_HEIGHT);
            Ok(Dispatches::default())
        }
        MouseEventKind::ScrollDown => {
            self.apply_scroll(Direction::End, SCROLL_HEIGHT);
            Ok(Dispatches::default())
        }
        MouseEventKind::Down(MouseButton::Left) => Ok(Dispatches::default()),
        _ => Ok(Dispatches::default()),
    }
}
```

To:
```rust
fn handle_mouse_event(
    &mut self,
    mouse_event: crossterm::event::MouseEvent,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    const SCROLL_HEIGHT: usize = 2;
    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            self.apply_scroll(Direction::Start, SCROLL_HEIGHT);
            Ok(Dispatches::default())
        }
        MouseEventKind::ScrollDown => {
            self.apply_scroll(Direction::End, SCROLL_HEIGHT);
            Ok(Dispatches::default())
        }
        MouseEventKind::Down(MouseButton::Left) => {
            self.handle_mouse_click(mouse_event.column, mouse_event.row, context)
        }
        _ => Ok(Dispatches::default()),
    }
}
```

- [ ] **Step 4: Check for other handle_mouse_event calls**

Search for calls to handle_mouse_event:
```bash
rg "handle_mouse_event" --type rust -A 2
```

Expected: Should find calls in:
- `Component::handle_event` in `src/components/component.rs` (event dispatcher)
- Possibly in test files that test mouse handling

If calls found, update them to pass context:
```rust
event::event::Event::Mouse(event) => self.handle_mouse_event(event, context),
```

Note: Test file calls may need updating, but for now focus on production code path.

- [ ] **Step 5: Run tests**

Run: `cargo test --package ki`

Expected:
- All existing tests should pass
- May get compilation error about missing `handle_mouse_click` (this is expected and will be resolved in Chunk 3)
- This step verifies the trait signature change is correct and propagates properly

- [ ] **Step 6: Commit**

```bash
git add src/components/component.rs src/components/editor.rs
git commit -m "refactor(mouse): add context parameter to handle_mouse_event

This allows mouse event handlers to access runtime context.

johncming@126.com
```

---

## Chunk 3: Editor mouse click implementation

### Task 3: Add get_line_number_width helper

**Files:**
- Modify: `src/components/editor.rs`

- [ ] **Step 1: Implement get_line_number_width**

Add private method to `impl Editor` block in `src/components/editor.rs`. Good location is after `handle_mouse_event` method (around line 160):

```rust
pub(crate) fn get_line_number_width(&self) -> usize {
    let buffer = self.buffer.borrow();
    let line_count = buffer.len_lines();
    // Width = digits for last line number + vertical border character (│)
    // Matches calculation in src/grid.rs:357
    let line_number_digits = line_count.max(1).to_string().len();
    line_number_digits + 1
}
```

- [ ] **Step 2: Compile check**

Run: `cargo check --package ki`
Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/components/editor.rs
git commit -m "feat(editor): add get_line_number_width helper

Calculates line number area width: digits for last line + 1 for border character.
Matches grid.rs calculation at line 357.

johncming@126.com
```

### Task 4: Implement handle_mouse_click

**Files:**
- Modify: `src/components/editor.rs`

- [ ] **Step 1: Write failing tests for mouse click behavior**

Add to `src/components/editor.rs` as `#[cfg(test)]` module (after existing test modules):

```rust
#[cfg(test)]
mod mouse_click_tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::components::editor::Mode;
    use crate::position::Position;
    use crate::rectangle::Rectangle;
    use crate::selection::SelectionSet;
    use crate::selection::Selection;
    use crate::selection::CharIndex;
    use crate::context::Context;
    use crate::shared::languages::Language;
    use ropey::Rope;
    use std::rc::Rc;
    use std::cell::RefCell;
    use nonempty::NonEmpty;

    fn create_test_editor(content: &str) -> Editor {
        let buffer = Rc::new(RefCell::new(
            Buffer::new(Language::Rust, Rope::from_str(content))
        ));
        Editor {
            mode: Mode::Normal,
            selection_set: SelectionSet::default(),
            cursor_direction: crate::components::editor::Direction::Start,
            scroll_offset: 0,
            rectangle: Rectangle::default(),
            buffer,
            title: None,
            id: crate::components::component::ComponentId::new(),
            current_view_alignment: None,
            regex_highlight_rules: Vec::new(),
            incremental_search_matches: None,
            jumps: None,
            char_index_range_highlight: None,
        }
    }

    #[test]
    fn test_valid_mouse_click_moves_cursor() {
        let mut editor = create_test_editor("hello\nworld\nfoo");
        editor.rectangle = Rectangle {
            origin: Position::new(0, 0),
            width: 20,
            height: 10,
        };

        let context = Context::default();
        // Line number width for 3 lines: "3" = 1 digit + 1 border = 2
        // Click on 'l' in "hello" - buffer column 2, screen column 4
        let result = editor.handle_mouse_click(4, 0, &context);

        assert!(result.is_ok());
        // Check cursor moved to expected position
        let selection = editor.selection_set.primary_selection();
        assert_eq!(selection.head.line, 0);
        assert_eq!(selection.head.column, 2);
    }

    #[test]
    fn test_click_in_line_number_area_ignored() {
        let mut editor = create_test_editor("hello");
        editor.rectangle = Rectangle {
            origin: Position::new(0, 0),
            width: 20,
            height: 10,
        };
        editor.selection_set = SelectionSet::default()
            .set_selections(NonEmpty::new(
                Selection::new(CharIndex(2)..=CharIndex(2))
            ));

        let context = Context::default();
        let initial_head = editor.selection_set.primary_selection().head;

        let result = editor.handle_mouse_click(0, 0, &context);

        assert!(result.is_ok());
        // Cursor should not have moved
        let current_head = editor.selection_set.primary_selection().head;
        assert_eq!(current_head, initial_head);
    }

    #[test]
    fn test_click_outside_rectangle_ignored() {
        let mut editor = create_test_editor("hello");
        editor.rectangle = Rectangle {
            origin: Position::new(5, 5),
            width: 20,
            height: 10,
        };

        let context = Context::default();
        let result = editor.handle_mouse_click(0, 0, &context);

        assert!(result.is_ok());
        // Click was outside rectangle, should be ignored
        // Cursor should remain at initial position (0,0)
        assert_eq!(editor.selection_set.primary_selection().head.line, 0);
    }

    #[test]
    fn test_click_in_insert_mode_stays_in_insert() {
        let mut editor = create_test_editor("hello");
        editor.mode = Mode::Insert;
        editor.rectangle = Rectangle {
            origin: Position::new(0, 0),
            width: 20,
            height: 10,
        };

        let context = Context::default();
        let result = editor.handle_mouse_click(4, 0, &context);

        assert!(result.is_ok());
        // Should still be in Insert mode
        assert!(matches!(editor.mode, Mode::Insert));
    }

    #[test]
    fn test_click_clears_multi_cursor_mode() {
        let mut editor = create_test_editor("hello world");
        editor.mode = Mode::MultiCursor;
        editor.rectangle = Rectangle {
            origin: Position::new(0, 0),
            width: 20,
            height: 10,
        };

        let context = Context::default();
        let result = editor.handle_mouse_click(4, 0, &context);

        assert!(result.is_ok());
        // Should be in Normal mode now
        assert!(matches!(editor.mode, Mode::Normal));
    }

    #[test]
    fn test_click_past_line_end_ignored() {
        let mut editor = create_test_editor("hello");
        editor.rectangle = Rectangle {
            origin: Position::new(0, 0),
            width: 20,
            height: 10,
        };

        let context = Context::default();
        let initial_head = editor.selection_set.primary_selection().head;

        // Click way past line end (column 100 when line is 5 chars)
        let result = editor.handle_mouse_click(100, 0, &context);

        assert!(result.is_ok());
        // Cursor should not move
        assert_eq!(editor.selection_set.primary_selection().head, initial_head);
    }

    #[test]
    fn test_get_line_number_width() {
        let editor = create_test_editor("line1\nline2\nline3");
        // 3 lines -> "3" is 1 digit + 1 border = 2
        assert_eq!(editor.get_line_number_width(), 2);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test mouse_click_tests`
Expected: FAIL with "no method named `handle_mouse_click`"

- [ ] **Step 3: Implement handle_mouse_click**

Add to `impl Editor` block in `src/components/editor.rs`. Good location is after `get_line_number_width` (around line 180):

```rust
pub(crate) fn handle_mouse_click(
    &mut self,
    mouse_column: u16,
    mouse_row: u16,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    // Check if click is within editor rectangle
    let mouse_pos = Position::new(mouse_row as usize, mouse_column as usize);

    if !self.rectangle.contains(&mouse_pos) {
        return Ok(Dispatches::default());
    }

    // Calculate relative position
    let relative_row = mouse_row as usize - self.rectangle.origin.line;
    let relative_col = mouse_column as usize - self.rectangle.origin.column;

    // Check if click is in line number area
    let line_number_width = self.get_line_number_width();
    if relative_col < line_number_width {
        return Ok(Dispatches::default());
    }

    // Calculate buffer position
    let buffer_row = relative_row + self.scroll_offset;
    let buffer_col = relative_col - line_number_width;

    // Validate buffer position
    let buffer = self.buffer.borrow();
    let total_lines = buffer.len_lines();
    if buffer_row >= total_lines {
        return Ok(Dispatches::default());
    }

    let line_content = buffer.line(buffer_row)?;
    if buffer_col > line_content.len_chars() {
        return Ok(Dispatches::default());
    }
    drop(buffer);

    // Clear multi-cursor if in MultiCursor mode
    // Note: In Normal, Insert, or other modes, clicking does not affect mode
    if matches!(self.mode, Mode::MultiCursor) {
        self.mode = Mode::Normal;
    }

    // Set cursor position (existing function)
    self.set_cursor_position(buffer_row, buffer_col, context)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test mouse_click_tests`
Expected: All tests should pass

- [ ] **Step 5: Commit**

```bash
git add src/components/editor.rs
git commit -m "feat(editor): implement handle_mouse_click for mouse navigation

- Validates click is within editor rectangle
- Excludes line number area
- Validates buffer position
- Clears multi-cursor mode
- Moves cursor to clicked position
- Includes unit tests for all scenarios

johncming@126.com
```

---

## Chunk 4: Integration cleanup

### Task 5: Remove dead_code attribute from set_cursor_position

**Files:**
- Modify: `src/components/editor.rs:2142`

- [ ] **Step 1: Find set_cursor_position function**

Search for the function around line 2142:
```bash
rg "pub fn set_cursor_position" src/components/editor.rs -A 5
```

- [ ] **Step 2: Remove #[allow(dead_code)] and TODO comment**

Find lines:
```rust
// TODO: handle mouse click
#[allow(dead_code)]
pub fn set_cursor_position(
```

Remove the `#[allow(dead_code)]` attribute and the `// TODO: handle mouse click` comment.

Should become:
```rust
pub fn set_cursor_position(
```

- [ ] **Step 3: Compile check**

Run: `cargo check --package ki`
Expected: Compiles without warnings about dead_code

- [ ] **Step 4: Commit**

```bash
git add src/components/editor.rs
git commit -m "refactor(editor): remove dead_code attribute from set_cursor_position

Function is now used by handle_mouse_click.

johncming@126.com
```

---

## Chunk 5: Final verification

### Task 6: Run all tests and manual verification

**Files:**
- Test: `src/components/editor.rs`, `src/rectangle.rs`

- [ ] **Step 1: Run all editor tests**

Run: `cargo test --package ki`
Expected: All tests pass

- [ ] **Step 2: Run rectangle tests**

Run: `cargo test rectangle::test_contains`
Expected: All tests pass

- [ ] **Step 3: Manual verification (optional but recommended)**

1. Run ki editor
2. Open a file with content (e.g., create a test file)
3. Try clicking at different positions:
   - Click in the middle of a line → cursor should move there
   - Click in line number area → nothing should happen
   - Click outside the editor rectangle → nothing should happen
   - Click past the end of a line → nothing should happen
   - Click in MultiCursor mode → mode should switch to Normal
   - Click in Insert mode → should stay in Insert mode

- [ ] **Step 4: Final commit if manual testing reveals issues**

If any issues found during manual testing, create additional commits to fix them.

```bash
git commit -m "fix: address manual testing feedback for mouse click

[Description of any fixes]

johncming@126.com
```

---

## Summary

This plan adds mouse click navigation in 5 chunks:

1. **Chunk 1**: `Rectangle::contains` - Foundation for position validation
2. **Chunk 2**: Component trait update - Pass Context through event chain
3. **Chunk 3**: Editor mouse click implementation - Core functionality
4. **Chunk 4**: Cleanup - Remove dead_code markers
5. **Chunk 5**: Final verification - Tests and manual check

Total estimated tasks: ~30 steps
Estimated time: 30-45 minutes for experienced developer

---

## References

- Spec: `docs/superpowers/specs/2026-03-15-mouse-click-to-move-cursor-design.md`
- Related code: `src/grid.rs:357` (line number width calculation)
- Existing function: `src/components/editor.rs:2143` (set_cursor_position)
