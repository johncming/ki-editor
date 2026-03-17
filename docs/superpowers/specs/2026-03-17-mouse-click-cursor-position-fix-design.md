# Mouse Click Cursor Position Fix Design

## Problem Summary

When using mouse clicks to position the cursor in the editor, the selected text extends to the next line instead of the expected exact position. This occurs specifically when using the **Line** SelectionMode.

## Root Cause

The `handle_mouse_click` function calls `set_cursor_position`, which uses `Current` movement to expand the selection according to the active SelectionMode:

```
handle_mouse_click → set_cursor_position → SelectionMode (Current)
                                              ↓
                                  Expands selection (Line mode includes \n)
                                              ↓
                                  Selection spans to next line
```

In Line mode, the selection includes the trailing newline character (`\n`), which belongs to the beginning of the next line. This causes the visual selection to extend to the next line even though the cursor is in the correct position.

## Solution

Add an `expand` parameter to `set_cursor_position` to control whether the SelectionMode should expand the selection.

## Architecture

### Before
```
┌────────────────────────────────────────────────────────────────┐
│ handle_mouse_click → set_cursor_position → SelectionMode       │
│                       ↓                                       │
│               Current movement → Expands selection              │
└────────────────────────────────────────────────────────────────┘
```

### After
```
┌────────────────────────────────────────────────────────────────┐
│ handle_mouse_click → set_cursor_position(row, col, false)     │
│                                    ↓                         │
│                   expand=false → Skip SelectionMode expansion    │
│                   expand=true  → Use existing SelectionMode     │
└────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### 1. Modify `set_cursor_position` Signature

**File:** `src/components/editor.rs` (line ~2144)

```rust
pub fn set_cursor_position(
    &mut self,
    row: usize,
    column: usize,
    expand: bool,           // NEW: Control selection expansion
    context: &Context,
) -> anyhow::Result<Dispatches> {
    let char_index = self.buffer.borrow().line_to_char(row)? + column;

    if expand {
        // Existing logic: Use SelectionMode's Current movement
        let temp_selection = self.selection_set
            .primary_selection()
            .clone()
            .set_range((char_index..char_index).into());

        let temp_selection_set = self.selection_set
            .clone()
            .set_selections(NonEmpty::new(temp_selection));

        let current_mode = self.selection_set.mode().clone();
        let new_selection_set = {
            let buffer = self.buffer.borrow();
            temp_selection_set.generate(
                &buffer,
                &current_mode,
                &MovementApplicandum::Current(IfCurrentNotFound::LookForward),
                &self.cursor_direction,
                context,
            )?
        };
        if let Some(new_selection_set) = new_selection_set {
            Ok(self.update_selection_set(new_selection_set, true, context))
        } else {
            Ok(Dispatches::default())
        }
    } else {
        // NEW: Use exact position without expansion
        let exact_selection = Selection::new((char_index..char_index).into());
        let new_selection_set = self.selection_set
            .clone()
            .set_selections(NonEmpty::new(exact_selection));
        Ok(self.update_selection_set(new_selection_set, true, context))
    }
}
```

### 2. Update `handle_mouse_click` Call

**File:** `src/components/editor.rs` (line ~2241)

```rust
pub(crate) fn handle_mouse_click(
    &mut self,
    mouse_column: u16,
    mouse_row: u16,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    // ... existing validation logic unchanged ...

    // Clear multi-cursor if in MultiCursor mode
    if matches!(self.mode, Mode::MultiCursor) {
        self.mode = Mode::Normal;
    }

    // CHANGE: Pass expand = false for exact positioning
    self.set_cursor_position(buffer_row, buffer_col, false, context)
}
```

### 3. Update Existing Call Sites

All other call sites of `set_cursor_position` should pass `expand = true` to maintain existing behavior:

- Keyboard navigation (`Movement::Up`, `Movement::Down`, etc.)
- Command operations (e.g., `:goto`)
- Programmatic cursor positioning that should respect SelectionMode

## Testing

### Test Matrix

| Operation | expand Value | Expected Behavior |
|-----------|--------------|------------------|
| Mouse click | false | Exact cursor position, no selection expansion |
| Keyboard navigation | true | Selection expands according to SelectionMode |
| `:goto` command | true | Selection expands according to SelectionMode |

### Existing Tests

**Tests that need updating:**

The following tests currently expect mouse clicks to expand selection according to SelectionMode. They need to be updated to expect exact positioning (empty selection):

1. `test_click_respects_line_mode` (line ~5108) - Currently expects line to be selected
2. `test_click_respects_word_mode` (line ~5139) - Currently expects word to be selected

**Updated expectation:** After clicking, the selection should be empty (cursor at exact position).

### New Tests Needed

Add tests for edge cases:

1. **Last line test** - Click on last line which has no trailing `\n`
2. **Line boundary test** - Click at exact position of line length (before `\n`)
3. **Different SelectionModes** - Verify exact positioning in Character, Word, Line, Subword modes
4. **Multi-cursor mode** - Verify multi-cursor is cleared on click

### Manual Testing

After implementation, verify:
1. Mouse click in Normal mode → exact positioning
2. Mouse click in Word mode → exact positioning
3. Mouse click in Line mode → exact positioning (no extension to next line)
4. Keyboard navigation still works correctly with SelectionMode expansion

## Files Modified

- `src/components/editor.rs`
  - `set_cursor_position` function signature (line ~2144)
  - `set_cursor_position` function body (add expand branch)
  - `handle_mouse_click` call (line ~2241)

## Implementation Steps

1. Modify `set_cursor_position` function signature to add `expand: bool` parameter
2. Add `if expand` branch in `set_cursor_position` function body
3. Update `handle_mouse_click` to pass `expand = false`
4. Find and update all other `set_cursor_position` call sites to pass `expand = true`
5. Run existing tests to verify no regressions
6. Manual testing of mouse click behavior

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missing call sites for `set_cursor_position` | Could break existing behavior | Use Grep to find all call sites and review each one |
| Users expect mouse click to expand selection | Behavior change | Mouse click is fundamentally a positioning operation, not a selection operation. This matches standard editor behavior (single click = position, double click = select) |
| Test updates overlooked | Tests may fail | Explicitly identify tests that need updates and verify they pass |
| Mouse drag selection affected | Could break text selection | Verify that mouse drag uses a different code path or is unaffected |

**Important UX Note:** In standard editors:
- Single mouse click = position cursor
- Double click = select word
- Triple click = select line

This fix aligns ki-editor with standard behavior.

## Success Criteria

- Mouse clicks position the cursor exactly where clicked, regardless of SelectionMode
- Updated tests (`test_click_respects_line_mode`, `test_click_respects_word_mode`) pass with exact positioning expectations
- New edge case tests (last line, line boundary) pass
- All other existing tests pass
- Keyboard navigation and other cursor movements still work correctly with SelectionMode expansion
- Mouse drag selection still works correctly (if using different code path)
