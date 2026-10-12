use crate::core::system::System;
pub mod parser;
pub mod problem;
pub mod ui;

pub struct Pcms2System;

impl System for Pcms2System {
    const NAME: &'static str = "pcms2";
    type Problem = problem::Pcms2Problem;
    type Parser = parser::Pcms2Parser;
    type Ui = ui::Pcms2UI;

    fn parser() -> Self::Parser {
        parser::Pcms2Parser
    }
    fn ui() -> Self::Ui {
        ui::Pcms2UI
    }
}