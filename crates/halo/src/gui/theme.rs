use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
use palette::Srgba;

pub struct ThemeColors {
    pub hovered: Srgba<f64>,
    pub running: Srgba<f64>,
    pub default: Srgba<f64>,
    pub center_circle: Srgba<f64>,
    pub broken: Srgba<f64>,
}

impl ThemeColors {
    pub fn from_context(context: &gtk::StyleContext) -> Self {
        Self {
            hovered: Self::lookup_color(
                context,
                "theme_selected_bg_color",
                Srgba::new(0.4, 0.4, 0.8, 0.9),
                Some(0.9),
            ),
            running: Self::lookup_color(
                context,
                "theme_fg_color",
                Srgba::new(0.25, 0.25, 0.25, 0.85),
                Some(0.3),
            ),
            broken: Self::lookup_color(
                context,
                "error_bg_color",
                Srgba::new(0.8, 0.2, 0.2, 0.5),
                Some(0.5),
            ),
            default: Self::lookup_color(
                context,
                "theme_bg_color",
                Srgba::new(0.15, 0.15, 0.15, 0.5),
                Some(0.5),
            ),
            center_circle: Self::lookup_color(
                context,
                "theme_fg_color",
                Srgba::new(0.2, 0.2, 0.2, 0.15),
                Some(0.1),
            ),
        }
    }

    fn lookup_color(
        context: &gtk::StyleContext,
        name: &str,
        fallback: Srgba<f64>,
        alpha_override: Option<f64>,
    ) -> Srgba<f64> {
        context
            .lookup_color(name)
            .map(|c| {
                let (r, g, b, a) = (
                    c.red() as f64,
                    c.green() as f64,
                    c.blue() as f64,
                    c.alpha() as f64,
                );
                Srgba::new(r, g, b, alpha_override.unwrap_or(a))
            })
            .unwrap_or(fallback)
    }
}

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    let css_data = "
.halo-window, .halo-drawing-area {
    background: none;
    background-color: transparent;
}
";
    provider.load_from_data(css_data);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
