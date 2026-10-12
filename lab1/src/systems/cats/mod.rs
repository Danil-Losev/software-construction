use crate::core::system::System;

pub mod parser;
pub mod ui;
pub mod problem;

pub struct CatsSystem;

impl System for CatsSystem {
    const NAME: &'static str = "cats";
    type Problem = problem::CatsProblem;
    type Parser = parser::CatsParser;
    type Ui = ui::CatsUI;

    fn parser() -> Self::Parser {
        parser::CatsParser
    }
    fn ui() -> Self::Ui {
        ui::CatsUI
    }
}
