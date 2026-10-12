use crate::core::system::System;
pub mod parser;
pub mod problem;
pub mod ui;

pub struct EjudgeSystem;

impl System for EjudgeSystem {
    const NAME: &'static str = "ejudge";
    type Problem = problem::EjudgeProblem;
    type Parser = parser::EjudgeParser;
    type Ui = ui::EjudgeUI;

    fn parser() -> Self::Parser {
        parser::EjudgeParser
    }
    fn ui() -> Self::Ui {
        ui::EjudgeUI
    }
}