use super::Node;

#[derive(PartialEq, Eq)]
pub(super) enum StackState {
    Left,
    Right,
    Handle,
}
pub(super) struct StackEntry<V: Ord + Sized + Default> {
    pub(super) ptr: *mut Node<V>,
    pub(super) state: StackState,
}
pub(super) struct CloneStackEntry<V: Ord + Sized + Default> {
    pub(super) new_ptr: *mut Node<V>,
    pub(super) ptr: *mut Node<V>,
    pub(super) state: StackState,
}