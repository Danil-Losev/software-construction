use super::{parser::Parser, ui::Ui};

pub trait System {
    const NAME: &'static str;
    
    type Problem;
    type Parser: Parser<Output = Self::Problem>;
    type Ui: Ui<Input = Self::Problem>;
    
    fn parser() -> Self::Parser;
    fn ui() -> Self::Ui;
}