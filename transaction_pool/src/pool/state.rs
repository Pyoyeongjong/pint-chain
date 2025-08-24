#[derive(Debug, Clone)]
pub enum SubPool {
    Pending,
    Parked,
}

impl SubPool {
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TxState {
    has_balance: bool,
    has_ancestor: bool,
}

impl TxState {
    pub fn new() -> Self {
        Self { has_balance: false, has_ancestor: false }
    }

    pub fn has_balance(&mut self) {
        self.has_balance = true;
    }

    pub fn has_ancestor(&mut self) {
        self.has_ancestor = true;
    }
}

impl From<TxState> for SubPool {
    fn from(value: TxState) -> Self {
        match value.has_balance && !value.has_ancestor {
            true => SubPool::Pending,
            false => SubPool::Parked,
        }
    }
}
