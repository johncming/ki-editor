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
│  handle_mouse_event() ──> handle_mouse_click(col, row)         │
│                               │                               │
│                               ▼                               │
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
│  │ 7. Call set_cursor_position(buffer_row, buffer_col)      │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### File Modifications

#### 1. `src/components/editor.rs`

**Modify `handle_mouse_event`:**

```rust
fn handle_mouse_event(&mut self, mouse_event: crossterm::event::MouseEvent)
    -> anyhow::Result<Dispatches> {
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
            self.handle_mouse_click(mouse_event.column, mouse_event.row)
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
    if matches!(self.mode, Mode::MultiCursor) {
        self.mode = Mode::Normal;
    }

    // Set cursor position (existing function)
    self.set_cursor_position(buffer_row, buffer_col, &Context::default())
}
```

**Add `get_line_number_width`:**

```rust
fn get_line_number_width(&self) -> usize {
    let buffer = self.buffer.borrow();
    let line_count = buffer.len_lines();
    // Width = digits + vertical border character (│)
    let digits = if line_count == 0 {
        1
    } else {
        (line_count + 1).ilog10() as usize + 1
    };
    digits + 1
}
```

#### 2. `src/rectangle.rs`

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

    // Inside points
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
fn test_mouse_click_moves_cursor() {
    let mut editor = Editor::new(...);

    // Valid click should move cursor
    editor.handle_mouse_click(column, row)?;
    assert_eq!(editor.cursor_position(), expected_position);
}

#[test]
fn test_click_in_line_number_area_ignored() {
    // Clicking in line number area should not move cursor
}

#[test]
fn test_click_outside_file_bounds_ignored() {
    // Clicking beyond file content should be ignored
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

## Future Extensions

This design can be extended to support:

1. **Drag-to-select** - Track mouse drag state to create selection
2. **Double-click** - Select word under cursor
3. **Triple-click** - Select entire line
4. **Middle-click** - Paste from clipboard
5. **Right-click** - Context menu

These can be added by extending the `MouseEventKind` match in `handle_mouse_event` without requiring architectural changes.

## Implementation Notes

1. The `set_cursor_position` function currently uses `Context::default()` which may need adjustment depending on actual runtime context requirements.

2. The `get_line_number_width` calculation assumes the line number area is always the same width. If Ki supports variable line number formatting, this will need adjustment.

3. Coordinate calculation assumes 1:1 mapping between screen cells and buffer characters. If Ki implements visual character width handling (e.g., for Unicode), the column calculation will need to account for this.

4. The design assumes `scroll_offset` is available as a field on `Editor`. If the actual field name differs, the implementation will need adjustment.
