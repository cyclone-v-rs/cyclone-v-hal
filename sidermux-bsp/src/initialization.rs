/// A hardware initialization attempt must never be repeated after failure.
pub(crate) enum Initialization<T, E> {
    Pending,
    Ready(T),
    Failed(E),
}
impl<T, E: Copy> Initialization<T, E> {
    pub(crate) fn get_or_init(
        &mut self,
        initialize: impl FnOnce() -> Result<T, E>,
    ) -> Result<&mut T, E> {
        if matches!(self, Self::Pending) {
            *self = match initialize() {
                Ok(value) => Self::Ready(value),
                Err(error) => Self::Failed(error),
            };
        }
        match self {
            Self::Ready(value) => Ok(value),
            Self::Failed(error) => Err(*error),
            Self::Pending => unreachable!(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Initialization;
    #[test]
    fn success_reuses_the_initialized_value() {
        let mut state: Initialization<u32, ()> = Initialization::Pending;
        *state.get_or_init(|| Ok(41)).unwrap() += 1;
        assert_eq!(
            *state.get_or_init(|| panic!("initialized twice")).unwrap(),
            42
        );
    }
    #[test]
    fn failure_is_cached_without_retry() {
        let mut state: Initialization<(), u32> = Initialization::Pending;
        assert_eq!(state.get_or_init(|| Err(7)), Err(7));
        assert_eq!(state.get_or_init(|| panic!("retried hardware")), Err(7));
    }
}
