
pub enum IterableOrSingle<T> {
    Iterable(T), 
    Single(T),
}
pub enum Iterable<T> {
    VecWrapper(Vec<T>),
    // You can add more variants here if you need to wrap T in different iterable types
}


// Example usage
pub fn use_iterable_wrapper<T>(wrapper: Iterable<T>) {
    match wrapper {
        Iterable::VecWrapper(vec) => {
            for item in vec {
                //TODO Process each item
            }
        },
    }
}