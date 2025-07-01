use std::fmt::Display;

pub enum BinOpKind {
    ADD,
    SUB,
    MUL,
    DIV,
}

impl Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            BinOpKind::ADD => "+",
            BinOpKind::SUB => "-",
            BinOpKind::MUL => "*",
            BinOpKind::DIV => "/",
        };
        write!(f, "{}", c)
    }
}
