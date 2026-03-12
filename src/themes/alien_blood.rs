use super::{DiagnosticStyles, HighlightName, Theme, UiStyles};
use crate::{
    style::{fg, Style},
    themes::{GitGutterStyles, SyntaxStyles},
};
use my_proc_macros::hex;

pub fn alien_blood() -> Theme {
    Theme {
        name: "Alien Blood".to_string(),
        syntax: SyntaxStyles::new({
            use HighlightName::*;
            &[
                (Variable, fg(hex!("#73fa91"))),
                (Keyword, fg(hex!("#bde000"))),
                (KeywordModifier, fg(hex!("#bde000"))),
                (Function, fg(hex!("#00aae0"))),
                (Type, fg(hex!("#00aae0"))),
                (TypeBuiltin, fg(hex!("#00e0c4"))),
                (String, fg(hex!("#18e000"))),
                (Comment, fg(hex!("#3c4812"))),
                (Tag, fg(hex!("#00e0c4"))),
                (TagAttribute, fg(hex!("#73fa91"))),
                (Number, fg(hex!("#e08009"))),
                (Constant, fg(hex!("#e08009"))),
                (Operator, fg(hex!("#637d75"))),
                (PunctuationDelimiter, fg(hex!("#647d75"))),
                (PunctuationBracket, fg(hex!("#647d75"))),
                (Boolean, fg(hex!("#e08009"))),
            ]
        }),
        ui: UiStyles {
            global_title: Style::new()
                .foreground_color(hex!("#112616"))
                .underline(hex!("#2f7e25")),
            window_title_focused: Style::new()
                .foreground_color(hex!("#112616"))
                .underline(hex!("#2f7e25")),
            window_title_unfocused: fg(hex!("#647d75")),
            focused_tab: fg(hex!("#73fa91")),
            parent_lines_background: hex!("#1a2b1e"),
            section_divider_background: hex!("#1a2b1e"),
            jump_mark_odd: Style::new()
                .background_color(hex!("#7f2b27"))
                .foreground_color(hex!("#73fa91")),
            jump_mark_even: Style::new()
                .background_color(hex!("#2f7e25"))
                .foreground_color(hex!("#0f1610")),
            background_color: None, // transparent
            text_foreground: hex!("#637d75"),
            primary_selection_background: hex!("#1d4125"),
            primary_selection_anchor_background: hex!("#1d4125"),
            primary_selection_secondary_cursor: Style::new()
                .background_color(hex!("#327f77"))
                .foreground_color(hex!("#0f1610")),
            secondary_selection_background: hex!("#152a1c"),
            secondary_selection_anchor_background: hex!("#1d4125"),
            secondary_selection_primary_cursor: Style::new()
                .background_color(hex!("#73fa91"))
                .foreground_color(hex!("#0f1610")),
            secondary_selection_secondary_cursor: Style::new()
                .background_color(hex!("#327f77"))
                .foreground_color(hex!("#0f1610")),
            line_number: Style::new().foreground_color(hex!("#3c4812")),
            border: Style::new()
                .background_color(hex!("#0f1610"))
                .foreground_color(hex!("#2f7e25")),
            mark: Style::new().background_color(hex!("#4a5a12")),
            possible_selection_background: hex!("#1d4125"),
            incremental_search_match_background: hex!("#2f7e25"),
            fuzzy_matched_char: Style::new().foreground_color(hex!("#18e000")),
        },
        diagnostic: DiagnosticStyles::default(),
        hunk: super::HunkStyles::dark(),
        git_gutter: GitGutterStyles::default(),
    }
}
