use miette::{GraphicalReportHandler, GraphicalTheme};

pub fn display_miette_error(err: &impl miette::Diagnostic) -> String {
    let mut s = String::new();
    GraphicalReportHandler::new()
        .with_links(true)
        .with_theme(GraphicalTheme::unicode())
        .render_report(&mut s, err)
        .unwrap();
    s
}
