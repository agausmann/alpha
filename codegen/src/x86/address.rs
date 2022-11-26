#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Index<I, B>(pub I, pub B);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Times1;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Times2;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Times4;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Times8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledIndex<S, I, B>(pub S, pub I, pub B);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Indirect<R>(pub R);
