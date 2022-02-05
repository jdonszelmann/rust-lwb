use miette::{GraphicalReportHandler, GraphicalTheme};

pub fn display_miette_error(err: &impl miette::Diagnostic) -> String {
    let mut s = String::new();
    if let Err(e) = GraphicalReportHandler::new()
        .with_links(true)
        .with_theme(GraphicalTheme::unicode())
        .render_report(&mut s, err)
    {
        eprintln!("{}", e);
        panic!();
    }
    s
}
