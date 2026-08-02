#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Confirm,
    Cancel,
    CycleView,
    Digit(u8),
    Char(char),
    Erase,
    Tick,
}
