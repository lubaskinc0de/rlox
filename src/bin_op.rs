use std::fmt::Display;

pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

impl Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            BinOpKind::Add => "+",
            BinOpKind::Sub => "-",
            BinOpKind::Mul => "*",
            BinOpKind::Div => "/",
        };
        write!(f, "{c}")
    }
}
