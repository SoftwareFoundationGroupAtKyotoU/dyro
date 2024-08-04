use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialFunction {
    Alloc,
    Dealloc,
    Print,
    Panic,
}

impl FromStr for SpecialFunction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alloc" => Ok(SpecialFunction::Alloc),
            "dealloc" => Ok(SpecialFunction::Dealloc),
            "print" => Ok(SpecialFunction::Print),
            "panic" => Ok(SpecialFunction::Panic),
            _ => Err(()),
        }
    }
}
