use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::pango::{AttrColor, AttrFontDesc, AttrList, FontDescription};
use tree_sitter::{Node, Tree, TreeCursor};

#[derive(Clone)]
pub struct Highlight {
    theme: Theme,
    parser: Rc<RefCell<tree_sitter::Parser>>,
    render: Rc<Render>,
}

impl Highlight {
    pub fn new(font: FontDescription) -> Self {
        let theme = Theme::system_style();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_lua::language()).unwrap();

        let render = Render::new(theme.clone(), font);

        Highlight {
            theme,
            parser: Rc::new(RefCell::new(parser)),
            render: Rc::new(render),
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn process(&self, text: &str) -> AttrList {
        let attrs = AttrList::default();

        let Some(tree) = self.parser.borrow_mut().parse(text, None) else {
            println!("highlight");
            return attrs
        };

        let cursor = SyntaxCursor::new(&tree);

        let mut font = AttrFontDesc::new(&self.render.font);
        font.set_start_index(0);
        font.set_end_index(text.len() as u32);
        attrs.insert(font);

        for node in cursor {
            if node.child_count() > 0 {
                continue;
            }

            self.render.highlight(&node, &attrs);
        }

        attrs
    }
}

#[derive(Clone, Copy)]
struct Style {
    color: Color,
    bold: bool,
}

impl Style {
    pub fn color(color: Color) -> Style {
        Style { color, bold: false }
    }

    pub fn bold(mut self) -> Style {
        self.bold = true;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color(u32);

impl Color {
    pub fn red(&self) -> u16 {
        ((self.0 >> 16) * 256) as u16
    }

    pub fn green(&self) -> u16 {
        ((self.0 >> 8) * 256) as u16
    }

    pub fn blue(&self) -> u16 {
        ((self.0 >> 0) * 256) as u16
    }

    pub fn foreground(&self) -> AttrColor {
        AttrColor::new_foreground(self.red(), self.green(), self.blue())
    }
}

struct Render {
    font: FontDescription,
    bold: FontDescription,
    map: HashMap<&'static str, Style>,
}

impl Render {
    pub fn new(theme: Theme, font: FontDescription) -> Self {
        let mut map = HashMap::new();

        // default highlighting for whole thing
        map.insert("chunk", Style::color(theme.base05));

        let keyword = Style::color(theme.base0e).bold();
        map.insert("function", keyword);
        map.insert("return", keyword);
        map.insert("end", keyword);
        map.insert("local", keyword);
        map.insert("if", keyword);
        map.insert("then", keyword);
        map.insert("while", keyword);
        map.insert("do", keyword);

        let string = Style::color(theme.base0b);
        map.insert("string_content", string);
        map.insert("\"", string);

        let literal = Style::color(theme.base09);
        map.insert("number", literal);
        map.insert("true", literal);
        map.insert("false", literal);

        let operator = Style::color(theme.base05);
        map.insert("+", operator);
        map.insert("-", operator);
        map.insert("/", operator);
        map.insert("*", operator);
        map.insert(",", operator);
        map.insert("(", operator);
        map.insert(")", operator);
        map.insert(";", operator);

        map.insert("call.name", Style::color(theme.base0d));
        map.insert("call.access", operator.bold());

        let mut bold = font.clone();
        bold.set_weight(gtk::pango::Weight::Bold);

        Render { font, bold, map }
    }

    pub fn highlight(&self, node: &Node, attrs: &AttrList) {
        let last_child = node.next_sibling().is_none();
        let parent_kind = node.parent().map(|n| n.kind());
        let gparent_kind = node.parent().and_then(|n| n.parent()).map(|n| n.kind());

        let kind = match (node.kind(), parent_kind, gparent_kind) {
            ("identifier", Some("function_call"), _) => "call.name",
            ("identifier", Some("dot_index_expression"), _) if last_child => "call.name",
            ("identifier", Some("method_index_expression"), _) if last_child => "call.name",
            (".", Some("dot_index_expression"), _) => "call.access",
            (":", Some("method_index_expression"), _) => "call.access",
            (kind, _, _) => kind,
        };

        if node.kind() == "identifier" {
            if let Some(parent) = node.parent() {
                println!("identifier parent kind: {}", parent.kind());
            }
        }

        let Some(style) = self.map.get(kind) else {
            match node.kind() {
                kind => {
                    eprintln!("unknown node kind: {kind}");
                }
            }
            return;
        };

        let Some(start) = u32::try_from(node.start_byte()).ok() else { return };
        let Some(end) = u32::try_from(node.end_byte()).ok() else { return };

        if style.bold {
            let mut bold = AttrFontDesc::new(&self.bold);
            bold.set_start_index(start);
            bold.set_end_index(end);
            attrs.insert(bold);
        }

        let mut color = style.color.foreground();
        color.set_start_index(start);
        color.set_end_index(end);
        attrs.insert(color);
    }
}

/// Structure adapted from base16 theme
/// https://github.com/chriskempson/base16
#[derive(Clone)]
pub struct Theme {
    _base00: Color,
    _base01: Color,
    #[allow(unused)]
    pub base02: Color,
    _base03: Color,
    _base04: Color,
    pub base05: Color,
    _base06: Color,
    _base07: Color,
    _base08: Color,
    base09: Color,
    _base0a: Color,
    base0b: Color,
    _base0c: Color,
    base0d: Color,
    base0e: Color,
    pub base0f: Color,
}

impl Theme {
    pub fn system_style() -> Self {
        // TODO react to changes in system theme
        if adw::StyleManager::default().is_dark() {
            Self::one_dark()
        } else {
            Self::one_light()
        }
    }

    /// https://github.com/purpleKarrot/base16-one-light-scheme
    pub fn one_light() -> Self {
        Theme {
            _base00: Color(0xfafafa),
            _base01: Color(0xf0f0f1),
            base02: Color(0xe5e5e6),
            _base03: Color(0xa0a1a7),
            _base04: Color(0x696c77),
            base05: Color(0x383a42),
            _base06: Color(0x202227),
            _base07: Color(0x090a0b),
            _base08: Color(0xca1243),
            base09: Color(0xd75f00),
            _base0a: Color(0xc18401),
            base0b: Color(0x50a14f),
            _base0c: Color(0x0184bc),
            base0d: Color(0x4078f2),
            base0e: Color(0xa626a4),
            base0f: Color(0x986801),
        }
    }

    /// https://github.com/LalitMaganti/base16-onedark-scheme
    pub fn one_dark() -> Self {
        Theme {
            _base00: Color(0x282c34),
            _base01: Color(0x353b45),
            base02: Color(0x3e4451),
            _base03: Color(0x545862),
            _base04: Color(0x565c64),
            base05: Color(0xabb2bf),
            _base06: Color(0xb6bdca),
            _base07: Color(0xc8ccd4),
            _base08: Color(0xe06c75),
            base09: Color(0xd19a66),
            _base0a: Color(0xe5c07b),
            base0b: Color(0x98c379),
            _base0c: Color(0x56b6c2),
            base0d: Color(0x61afef),
            base0e: Color(0xc678dd),
            base0f: Color(0xbe5046),
        }
    }
}

struct SyntaxCursor<'a> {
    cursor: TreeCursor<'a>,
    finished: bool,
}

impl<'a> SyntaxCursor<'a> {
    pub fn new(tree: &'a Tree) -> Self {
        SyntaxCursor {
            cursor: tree.walk(),
            finished: false,
        }
    }

    fn move_next(&mut self) {
        if self.cursor.goto_first_child() {
            return;
        }

        if self.cursor.goto_next_sibling() {
            return;
        }

        loop {
            if !self.cursor.goto_parent() {
                self.finished = true;
                return;
            }

            if self.cursor.goto_next_sibling() {
                return;
            }
        }
    }
}

impl<'a> Iterator for SyntaxCursor<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let node = self.cursor.node();
        self.move_next();
        Some(node)
    }
}
