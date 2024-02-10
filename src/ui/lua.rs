use derive_more::Deref;
use gtk::pango::{AttrList, FontDescription};
use gtk::prelude::{BoxExt, EditableExt, EntryExt, WidgetExt};

use self::highlight::{Color, Highlight, Theme};

use super::application::DeviceContext;

mod entry;

mod highlight;

const PROMPT: &str = ">> ";
const RESULT: &str = "=> ";
const ERROR: &str = "!! ";

pub fn page(device: DeviceContext) -> adw::NavigationPage {
    let mut font = FontDescription::new();
    font.set_family("monospace");

    let highlight = Highlight::new(font);

    let console = Console::new();

    let header = adw::HeaderBar::new();
    let footer = Footer::new(highlight.clone());

    let view = adw::ToolbarView::builder()
        .content(&*console)
        .build();

    view.add_top_bar(&header);
    view.add_bottom_bar(&*footer);

    let page = adw::NavigationPage::builder()
        .child(&view)
        .title("Lua Console")
        .build();

    page.connect_root_notify({
        let entry = footer.entry.clone();
        move |_| { entry.grab_focus(); }
    });

    // for _ in 1..50 {
    //     console.append(input_line(&highlight, "hello"));
    // }

    footer.entry.connect_activate(move |entry| {
        let text = entry.text();
        entry.set_text("");

        console.append(input_line(&highlight, &text));

        let tangara = device.tangara.clone();
        let console = console.clone();
        let theme = highlight.theme().clone();

        glib::spawn_future_local(async move {
            let line = match tangara.connection().eval_lua(&text).await {
                Ok(result) => output_line(&theme, &result),
                Err(error) => error_line(&theme, &format!("{error}")),
            };
            console.append(line);
        });
    });

    page
}

fn input_line(highlight: &Highlight, line: &str) -> gtk::Box {
    let prompt = gtk::Label::builder()
        .label(PROMPT)
        .build();

    let attrs = highlight.process(line);

    let input = gtk::Label::builder()
        .label(line)
        .attributes(&attrs)
        .build();

    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    line.append(&prompt);
    line.append(&input);

    line
}

fn output_line(theme: &Theme, line: &str) -> gtk::Box {
    let prompt = colored_text(RESULT, theme.base03);

    let input = gtk::Label::builder()
        .label(line)
        .build();

    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(["output-line"])
        .build();

    line.append(&prompt);
    line.append(&input);

    line
}

fn error_line(theme: &Theme, line: &str) -> gtk::Box {
    let prompt = colored_text(ERROR, theme.base0f);
    let text = colored_text(line, theme.base0f);

    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(["output-line"])
        .build();

    line.append(&prompt);
    line.append(&text);

    line
}

fn colored_text(text: &str, color: Color) -> gtk::Label {
    let mut color = color.foreground();
    color.set_start_index(0);
    color.set_end_index(text.len() as u32);

    let attrs = AttrList::new();
    attrs.insert(color);

    gtk::Label::builder()
        .label(text)
        .attributes(&attrs)
        .build()
}

#[derive(Deref, Clone)]
struct Console {
    #[deref]
    scroll: gtk::ScrolledWindow,
    console: gtk::Box,
}

impl Console {
    pub fn new() -> Self {
        let console = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::End)
            .css_classes(["console"])
            .build();

        let scroll = gtk::ScrolledWindow::builder()
            .child(&console)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .css_classes(["console-scroll-container"])
            .build();

        Console { scroll, console }
    }

    pub fn append(&self, line: gtk::Box) {
        self.console.append(&line);
    }
}

#[derive(Deref)]
struct Footer {
    #[deref]
    footer: gtk::Box,
    entry: gtk::Entry,
}

impl Footer {
    pub fn new(highlight: Highlight) -> Self {
        let entry = entry::entry(highlight);

        let prompt = gtk::Label::builder()
            .label(PROMPT)
            .build();

        let footer = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["toolbar", "console-toolbar"])
            .build();

        footer.append(&prompt);
        footer.append(&entry);

        Footer { footer, entry }
    }
}
