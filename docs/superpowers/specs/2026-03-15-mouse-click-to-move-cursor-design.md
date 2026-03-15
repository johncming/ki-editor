# Mouse Click to Move Cursor Design

**Date:** 2026-03-15
**Author:** Claude Code
**Status:** Draft

## Overview

Add support for mouse click navigation in Ki editor. Users will be able to move the cursor to a specific position by left-clicking within the editor content area.

## Requirements

| Feature | Requirement |
|---------|-------------|
| **Mouse Operation** | Left-click only, no drag-to-select |
| **Boundary Handling** | Clicking outside text content is invalid |
| **Line Number Area** | Clicking line numbers is invalid |
| **Multi-Cursor Mode** | Clear all cursors when clicking |
| **Edit Mode** | Do not switch modes (e.g., Normal → Insert) |

## Architecture

### Component Changes

```
┌─────────────────────────────────────────────────────────────────┐
│                    Event Flow                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  MouseEvent::Down(Left)                                         │
│         │                                                       │
│         ▼                                                       │
│  handle_mouse_event(context, event) ──> handle_mouse_click(...)  │
│                                            │               │
│                                            ▼               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 1. Check if click is within editor rectangle           │   │
│  │    (Rectangle::contains)                              │   │
│  │                                                         │   │
│  │ 2. Calculate relative position                          │   │
│  │    relative_row = mouse_row - rectangle.origin.line      │   │
│  │    relative_col = mouse_col - rectangle.origin.column    │   │
│  │                                                         │   │
│  │ 3. Check if click is in line number area               │   │
│  │    if relative_col < line_number_width: return           │   │
│  │                                                         │   │
│  │ 4. Calculate buffer position                            │   │
│  │    buffer_row = relative_row + scroll_offset             │   │
│  │    buffer_col = relative_col - line_number_width         │   │
│  │                                                         │   │
│  │ 5. Validate buffer position                             │   │
│  │    - buffer_row < buffer.len_lines()                     │   │
│  │    - buffer_col <= line.len_chars()                      │   │
│  │                                                         │   │
│  │ 6. Clear multi-cursor if in MultiCursor mode             │   │
│  │                                                         │   │
│  │ 7. Call set_cursor_position(buffer_row, buffer_col, context)│
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### File Modifications

#### 1. `src/components/component.rs`

**Modify `handle_mouse_event` trait method signature to accept `Context`:**

```rust
fn handle_mouse_event(
    &mut self,
    event: crossterm::event::MouseEvent,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    self.editor_mut().handle_mouse_event(event, context)
}
```

#### 2. `src/components/editor.rs`

**Modify `handle_mouse_event` signature to accept `Context`:**

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

**Add `handle_mouse_click`:**

```rust
fn handle_mouse_click(
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

**Add `get_line_number_width`:**

```rust
fn get_line_number_width(&self) -> usize {
    let buffer = self.buffer.borrow();
    let line_count = buffer.len_lines();
    // Width = digits for last line number + vertical border character (│)
    // Matches the calculation in src/grid.rs:357
    let line_number_digits = line_count.max(1).to_string().len();
    line_number_digits + 1
}
```

#### 3. `src/rectangle.rs`

**Add `contains` method:**

```rust
impl Rectangle {
    pub fn contains(&self, position: &Position) -> bool {
        position.line >= self.origin.line
            && position.line < self.origin.line + self.height
            && position.column >= self.origin.column
            && position.column < self.origin.column + self.width
    }
}
```

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Click outside editor rectangle | Silently ignore (return empty Dispatches) |
| Click in line number area | Silently ignore |
| Row exceeds file line count | Silently ignore |
| Column exceeds line length | Silently ignore |
| Buffer access failure | Propagate error to caller |

### Testing

#### Unit Tests for `Rectangle::contains`

```rust
#[test]
fn test_rectangle_contains() {
    let rect = Rectangle {
        origin: Position::new(2, 3),
        width: 5,
        height: 4,
    };

    // Inside points (inclusive of origin, exclusive of bounds)
    assert!(rect.contains(&Position::new(2, 3)));
    assert!(rect.contains(&Position::new(5, 7)));

    // Outside points
    assert!(!rect.contains(&Position::new(1, 3)));
    assert!(!rect.contains(&Position::new(6, 3)));
    assert!(!rect.contains(&Position::new(2, 2)));
    assert!(!rect.contains(&Position::new(2, 8)));
}
```

#### Unit Tests for Mouse Click Handling

```rust
#[test]
fn test_valid_mouse_click_moves_cursor() {
    let mut editor = create_editor_with_content("hello\nworld\nfoo");
    editor.set_scroll_offset(0);
    editor.set_rectangle(Rectangle {
        origin: Position::new(0, 0),
        width: 20,
        height: 10,
    }, &context);

    // Click on 'o' in "hello" (line 0, column 2 in buffer)
    // After line number (3 digits for "100" + 1 for border = 4)
    let mouse_column = 4 + 2; // 6
    let mouse_row = 0;

    let dispatches = editor.handle_mouse_click(mouse_column, mouse_row, &context)?;

    // Cursor should move to position (0, 2)
    assert_eq!(editor.cursor_position(), Position::new(0, 2));
    assert!(dispatches.is_empty()); // No side dispatches expected
}

#[test]
fn test_click_in_line_number_area_ignored() {
    let mut editor = create_editor_with_content("hello\nworld");
    editor.set_rectangle(Rectangle {
        origin: Position::new(0, 0),
        width: 20,
        height: 10,
    }, &context);

    let initial_position = editor.cursor_position();

    // Click in line number area (column 0-3)
    let dispatches = editor.handle_mouse_click(2, 0, &context)?;

    // Cursor should not move
    assert_eq!(editor.cursor_position(), initial_position);
    assert!(dispatches.is_empty());
}

#[test]
fn test_click_outside_file_bounds_ignored() {
    let mut editor = create_editor_with_content("hello\nworld");
    editor.set_rectangle(Rectangle {
        origin: Position::new(0, 0),
        width: 20,
        height: 10,
    }, &context);

    let initial_position = editor.cursor_position();

    // Click beyond file content (line 10, column 10)
    let dispatches = editor.handle_mouse_click(15, 10, &context)?;

    // Cursor should not move
    assert_eq!(editor.cursor_position(), initial_position);
    assert!(dispatches.is_empty());
}

#[test]
fn test_click_clears_multi_cursor_mode() {
    let mut editor = create_editor_with_content("hello world");
    editor.mode = Mode::MultiCursor;
    // Set up multiple cursors (implementation depends on how MultiCursor works)

    let dispatches = editor.handle_mouse_click(8, 0, &context)?;

    // Should be in Normal mode now
    assert!(matches!(editor.mode, Mode::Normal));
}

#[test]
fn test_click_in_insert_mode_stays_in_insert() {
    let mut editor = create_editor_with_content("hello");
    editor.mode = Mode::Insert;

    let dispatches = editor.handle_mouse_click(8, 0, &context)?;

    // Should still be in Insert mode
    assert!(matches!(editor.mode, Mode::Insert));
}
```

## Existing Code Reuse

The design reuses existing Ki editor infrastructure:

| Component | Existing Functionality |
|-----------|----------------------|
| `set_cursor_position` | Already implemented, marked `#[allow(dead_code)]` |
| `Rectangle` | Represents editor area with origin, width, height |
| `Position` | Represents cursor/file position with line and column |
| `SelectionSet` | Manages cursor positions, supports clearing multi-cursor |
| `Buffer` | Provides `len_lines()` and `line()` for validation |

## Dependencies

No new external dependencies required. Uses existing:
- `crossterm::event` for mouse events
- `ropey` (via Buffer) for text operations
- Standard library for basic operations

## Implementation Details

### Coordinate System Assumptions

The design makes the following assumptions about Ki's coordinate system:

1. **Mouse coordinates** are 0-based (u16 from crossterm)
2. **Rectangle origin** is the top-left corner of the editor area on screen
3. **Buffer coordinates** are 0-based and represent character indices (not visual columns)
4. **Line number width** is calculated as `max_line_number.max(1).to_string().len() + 1`
   - Matches calculation in `src/grid.rs:357`
   - The `+ 1` accounts for the vertical border character (│)

### Visual Character Width

Ki uses the `unicode_width` crate for handling multi-width Unicode characters (e.g., emoji 🦀 has width 2). However, this design operates at the buffer coordinate level (character indices), not at the visual display level. The mouse click position is converted from screen coordinates to buffer coordinates, and this conversion uses the assumption that within the text area, the click column maps directly to the character column in the buffer.

If Ki implements visual column-to-buffer-column conversion (e.g., for properly positioning cursor after a multi-width character), this design will need to use that conversion instead of the simple offset calculation.

### Mode Behavior

| Current Mode | Click Behavior |
|-------------|---------------|
| Normal | Clear multi-cursor if active, stay in Normal |
| Insert | Clear multi-cursor if active, stay in Insert |
| MultiCursor | Clear all cursors, switch to Normal |
| FindOneChar | Clear multi-cursor if active, stay in FindOneChar |
| Swap | Clear multi-cursor if active, stay in Swap |
| Replace | Clear multi-cursor if active, stay in Replace |

## Future Extensions

This design can be extended to support:

1. **Drag-to-select** - Track mouse drag state to create selection
2. **Double-click** - Select word under cursor
3. **Triple-click** - Select entire line
4. **Middle-click** - Paste from clipboard
5. **Right-click** - Context menu

These can be added by extending the `MouseEventKind` match in `handle_mouse_event` without requiring architectural changes.
