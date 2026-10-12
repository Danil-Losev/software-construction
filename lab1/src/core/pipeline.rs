use super::{system::System, parser::Parser, ui::Ui};

pub fn run<SystemType: System>(path: &std::path::Path) {
    let parser = SystemType::parser();
    match parser.parse(path) {
        Ok(problem) => {
            let ui = SystemType::ui();
            ui.render(&problem);
        }
        Err(e) => {
            eprintln!("Error parsing input: {:?}", e);
        }
    }
}
