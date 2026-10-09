#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTheme {
    Academic,
    Modern,
    Elegant,
    Technical,
    Compact,
    Minimal,
}

impl DocumentTheme {
    pub fn parse(name: &str) -> Self {
        match name.trim().to_lowercase().as_str() {
            "modern" => Self::Modern,
            "elegant" => Self::Elegant,
            "technical" | "tech" => Self::Technical,
            "compact" | "ieee" | "acm" => Self::Compact,
            "minimal" | "swiss" => Self::Minimal,
            _ => Self::Academic,
        }
    }

    pub fn default_font(&self) -> &'static str {
        match self {
            Self::Academic => "New Computer Modern",
            Self::Modern => "Liberation Sans",
            Self::Elegant => "Linux Libertine",
            Self::Technical => "Liberation Sans",
            Self::Compact => "Times New Roman",
            Self::Minimal => "Liberation Sans",
        }
    }

    pub fn default_fontsize(&self) -> &'static str {
        match self {
            Self::Academic => "11pt",
            Self::Modern => "10.5pt",
            Self::Elegant => "11pt",
            Self::Technical => "10pt",
            Self::Compact => "10pt",
            Self::Minimal => "11pt",
        }
    }

    pub fn default_margins(&self) -> (&'static str, &'static str) {
        match self {
            Self::Academic => ("2.5cm", "2.5cm"),
            Self::Modern => ("2.4cm", "2.4cm"),
            Self::Elegant => ("2.8cm", "2.8cm"),
            Self::Technical => ("2cm", "2cm"),
            Self::Compact => ("1.8cm", "1.8cm"),
            Self::Minimal => ("3cm", "3cm"),
        }
    }

    pub fn default_columns(&self) -> usize {
        match self {
            Self::Compact => 2,
            _ => 1,
        }
    }

    pub fn default_linkcolor(&self) -> &'static str {
        match self {
            Self::Academic => "#0284c7",
            Self::Modern => "#2563eb",
            Self::Elegant => "#9a3412",
            Self::Technical => "#0d9488",
            Self::Compact => "#0284c7",
            Self::Minimal => "#18181b",
        }
    }

    pub fn default_text_color(&self) -> Option<&'static str> {
        match self {
            Self::Academic => None,
            Self::Modern => Some("#1e293b"),
            Self::Elegant => Some("#262626"),
            Self::Technical => Some("#0f172a"),
            Self::Compact => None,
            Self::Minimal => Some("#18181b"),
        }
    }

    pub fn default_numbering(&self) -> bool {
        !matches!(self, Self::Minimal)
    }

    pub fn default_leading(&self) -> &'static str {
        match self {
            Self::Compact => "0.65em",
            Self::Technical => "0.68em",
            Self::Modern => "0.72em",
            Self::Academic => "0.7em",
            Self::Elegant => "0.8em",
            Self::Minimal => "0.8em",
        }
    }
}
