use crate::core::system::System;
pub mod parser;
pub mod problem;
pub mod ui;

pub struct KrsuSystem;

impl System for KrsuSystem {
    const NAME: &'static str = "krsu";
    type Problem = problem::KrsuProblem;
    type Parser = parser::KrsuParser;
    type Ui = ui::KrsuUI;

    fn parser() -> Self::Parser {
        parser::KrsuParser
    }
    fn ui() -> Self::Ui {
        ui::KrsuUI
    }
}