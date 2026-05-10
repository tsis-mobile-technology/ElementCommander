use std::path::PathBuf;
use anyhow::Result;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style as SyntectStyle};
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;
use pulldown_cmark::{Parser, Event, Tag, TagEnd};
use ratatui::prelude::*;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FileFormat {
    Markdown,
    Log,
    Code(String),
    Text,
    Binary,
}

pub struct ViewerState {
    pub path: PathBuf,
    pub lines: Vec<LineContent>,
    pub scroll: usize,
    pub format: FileFormat,
    pub last_offset: u64,
    pub is_tail_active: bool,
    pub wrap: bool,
    pub viewport_height: usize,
}

#[derive(Clone)]
pub struct LineContent {
    pub raw: String,
    pub styled: Vec<(Color, String, bool)>, // (Color, Text, Bold)
}

impl ViewerState {
    pub fn new(path: PathBuf, width: usize, height: usize) -> Result<Self> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = match extension.as_str() {
            "md" | "markdown" => FileFormat::Markdown,
            "log" => FileFormat::Log,
            "rs" | "js" | "ts" | "py" | "c" | "cpp" | "h" | "hpp" | "go" | "toml" | "yaml" | "json" | "html" | "css" => {
                FileFormat::Code(extension)
            },
            _ => FileFormat::Text,
        };

        let mut state = ViewerState {
            path: path.clone(),
            lines: Vec::new(),
            scroll: 0,
            format,
            last_offset: 0,
            is_tail_active: false,
            wrap: true, // Default to wrap
            viewport_height: height,
        };

        state.load_content(width)?;
        
        if state.format == FileFormat::Log {
            state.is_tail_active = true;
            state.scroll_to_bottom();
        }

        Ok(state)
    }

    fn load_content(&mut self, width: usize) -> Result<()> {
        let mut file = File::open(&self.path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        self.last_offset = file.metadata()?.len();
        
        match self.format {
            FileFormat::Markdown => self.load_markdown_content(&content, width)?,
            FileFormat::Code(_) => self.load_code_content(&content)?,
            _ => {
                self.lines = content.lines()
                    .map(|l| LineContent {
                        raw: l.to_string(),
                        styled: vec![(Color::White, l.to_string(), false)],
                    })
                    .collect();
            }
        }

        if self.wrap {
            self.apply_word_wrap(width);
        }

        Ok(())
    }

    fn apply_word_wrap(&mut self, width: usize) {
        let mut wrapped_lines = Vec::new();
        let wrap_width = width.saturating_sub(4);
        for line in &self.lines {
            if line.raw.len() > wrap_width && wrap_width > 10 {
                let mut current_width = 0;
                let mut new_styled = Vec::new();
                for (color, text, is_bold) in &line.styled {
                    for word in text.split_inclusive(char::is_whitespace) {
                        let word_len = word.chars().count();
                        if current_width + word_len > wrap_width {
                            wrapped_lines.push(LineContent { raw: String::new(), styled: new_styled });
                            new_styled = Vec::new();
                            current_width = 0;
                        }
                        new_styled.push((*color, word.to_string(), *is_bold));
                        current_width += word_len;
                    }
                }
                wrapped_lines.push(LineContent { raw: String::new(), styled: new_styled });
            } else {
                wrapped_lines.push(line.clone());
            }
        }
        self.lines = wrapped_lines;
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.lines.len() > self.viewport_height {
            self.scroll = self.lines.len() - self.viewport_height;
        } else {
            self.scroll = 0;
        }
    }

    pub fn append_new_content(&mut self, new_text: &str, width: usize) {
        let mut new_state = ViewerState {
            path: self.path.clone(),
            lines: Vec::new(),
            scroll: 0,
            format: self.format.clone(),
            last_offset: 0,
            is_tail_active: false,
            wrap: self.wrap,
            viewport_height: self.viewport_height,
        };
        
        // Temporarily load only new text to get styled/wrapped lines
        new_state.lines = new_text.lines()
            .map(|l| LineContent {
                raw: l.to_string(),
                styled: vec![(Color::White, l.to_string(), false)],
            })
            .collect();
            
        if self.wrap {
            new_state.apply_word_wrap(width);
        }

        // Check if we were already at the bottom
        let is_at_bottom = self.scroll + self.viewport_height >= self.lines.len().saturating_sub(2);

        self.lines.extend(new_state.lines);

        if self.is_tail_active && is_at_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn toggle_wrap(&mut self, width: usize) -> Result<()> {
        self.wrap = !self.wrap;
        self.lines.clear();
        self.load_content(width)?;
        Ok(())
    }

    fn load_markdown_content(&mut self, content: &str, _width: usize) -> Result<()> {
        let parser = Parser::new(content);
        let mut current_line = Vec::new();
        let mut bold = false;
        let mut italic = false;
        let mut is_header = false;
        let mut header_level = 0;

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Strong => bold = true,
                    Tag::Emphasis => italic = true,
                    Tag::Heading { level, .. } => {
                        is_header = true;
                        header_level = level as u32;
                        current_line.push((Color::Cyan, "#".repeat(header_level as usize) + " ", true));
                    }
                    Tag::List(_) => current_line.push((Color::Yellow, "• ".to_string(), false)),
                    Tag::CodeBlock(_) => current_line.push((Color::Green, "```".to_string(), false)),
                    _ => {}
                },
                Event::End(tag) => match tag {
                    TagEnd::Strong => bold = false,
                    TagEnd::Emphasis => italic = false,
                    TagEnd::Heading(_) => {
                        is_header = false;
                        self.push_line(&mut current_line);
                    }
                    TagEnd::CodeBlock => {
                        current_line.push((Color::Green, "```".to_string(), false));
                        self.push_line(&mut current_line);
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    let color = if is_header {
                        match header_level {
                            1 => Color::Magenta,
                            2 => Color::Blue,
                            _ => Color::Cyan,
                        }
                    } else if bold {
                        Color::Yellow
                    } else if italic {
                        Color::Cyan
                    } else {
                        Color::White
                    };
                    current_line.push((color, text.to_string(), bold || is_header));
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.push_line(&mut current_line);
                }
                _ => {}
            }
        }
        self.push_line(&mut current_line);
        Ok(())
    }

    fn push_line(&mut self, current: &mut Vec<(Color, String, bool)>) {
        if !current.is_empty() {
            let raw: String = current.iter().map(|(_, t, _)| t.as_str()).collect();
            self.lines.push(LineContent {
                raw,
                styled: current.clone(),
            });
            current.clear();
        } else {
            self.lines.push(LineContent {
                raw: String::new(),
                styled: Vec::new(),
            });
        }
    }

    fn load_code_content(&mut self, content: &str) -> Result<()> {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        
        let syntax = if let FileFormat::Code(ref ext) = self.format {
            ps.find_syntax_by_extension(ext)
                .unwrap_or_else(|| ps.find_syntax_plain_text())
        } else {
            ps.find_syntax_plain_text()
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        
        for line in LinesWithEndings::from(content) {
            let ranges: Vec<(SyntectStyle, String)> = h.highlight_line(line, &ps)?
                .into_iter()
                .map(|(s, str)| (s, str.to_string()))
                .collect();
            
            let styled = ranges.into_iter().map(|(s, text)| {
                let color = Color::Rgb(s.foreground.r, s.foreground.g, s.foreground.b);
                (color, text.replace('\n', ""), false)
            }).collect();

            self.lines.push(LineContent {
                raw: line.to_string(),
                styled,
            });
        }

        Ok(())
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.lines.len().saturating_sub(self.viewport_height) {
            self.scroll += 1;
        } else {
            // If already at bottom, stay there
            self.scroll = self.lines.len().saturating_sub(self.viewport_height);
        }
    }

    pub fn page_up(&mut self, size: usize) {
        self.scroll = self.scroll.saturating_sub(size);
    }

    pub fn page_down(&mut self, size: usize) {
        let max = self.lines.len().saturating_sub(self.viewport_height);
        self.scroll = (self.scroll + size).min(max);
    }
}
