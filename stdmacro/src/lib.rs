#![forbid(unsafe_code)]

#[macro_export]
macro_rules! deque {
    () => (
        __VecDeque::new()
    );
    ($elem:expr; $n:expr) => (
        __VecDeque::from(::std::vec::from_elem($elem, $n))
    );
    ($($x:expr),+ $(,)?) => (
        // __VecDeque::from(<[_]>::into_vec (
        //     ::std::boxed::Box::new([$($x),+])
        // ))
        __VecDeque::from([$($x),+])
    )
}

#[macro_export]
macro_rules! sorted_vec {
    () => (
        ::std::vec::Vec::new()
    );
    ($($x:expr),+ $(,)?) => {
        {
            let mut v = <[_]>::into_vec (
            ::std::boxed::Box::new([$($x),+])
            );
            v.sort();
            v
        }
    }
}

#[macro_export]
macro_rules! map {
    () => (
        __HashMap::new()
    );
    ($($x:expr => $y:expr),+ $(,)?) => (
        __HashMap::from([$(($x, $y)),+])
    )
}
