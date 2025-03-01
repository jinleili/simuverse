use alloc::string::String;

pub(crate) fn show_code(ui: &mut egui::Ui, code: &str) {
    let code = remove_leading_indentation(code.trim_start_matches('\n'));
    let language = "rs";
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, &code, language);
}

pub(crate) fn remove_leading_indentation(code: &str) -> String {
    fn is_indent(c: &u8) -> bool {
        matches!(*c, b' ' | b'\t')
    }
    let first_line_indent = code.bytes().take_while(is_indent).count();
    let mut out = String::new();

    let mut code = code;
    while !code.is_empty() {
        let indent = code.bytes().take_while(is_indent).count();
        let start = first_line_indent.min(indent);
        let end = code
            .find('\n')
            .map_or_else(|| code.len(), |endline| endline + 1);
        out += &code[start..end];
        code = &code[end..];
    }
    out
}
