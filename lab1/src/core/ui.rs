
pub trait Ui{
    type Input;
    fn render(&self, input: &Self::Input);
}