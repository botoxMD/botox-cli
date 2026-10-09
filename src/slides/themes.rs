pub struct SlideThemeStyle {
    pub bg_color: &'static str,
    pub text_color: &'static str,
    pub heading_color: &'static str,
    pub muted_color: &'static str,
}

pub fn get_slide_theme_style(theme: &str) -> SlideThemeStyle {
    match theme {
        "dark" => SlideThemeStyle {
            bg_color: "#0f172a",
            text_color: "#f8fafc",
            heading_color: "#f8fafc",
            muted_color: "#94a3b8",
        },
        "nord" => SlideThemeStyle {
            bg_color: "#2e3440",
            text_color: "#eceff4",
            heading_color: "#f8fafc",
            muted_color: "#94a3b8",
        },
        "academic" => SlideThemeStyle {
            bg_color: "#ffffff",
            text_color: "#1a1a1a",
            heading_color: "#0f172a",
            muted_color: "#475569",
        },
        "gaia" => SlideThemeStyle {
            bg_color: "#fbfbf8",
            text_color: "#333333",
            heading_color: "#903020",
            muted_color: "#85756c",
        },
        "uncover" => SlideThemeStyle {
            bg_color: "#fafafa",
            text_color: "#0f172a",
            heading_color: "#0284c7",
            muted_color: "#64748b",
        },
        _ => SlideThemeStyle {
            bg_color: "#f8fafc",
            text_color: "#0f172a",
            heading_color: "#0f172a",
            muted_color: "#475569",
        },
    }
}
