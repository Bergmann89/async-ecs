pub trait Split {
    type Left;
    type Right;

    fn split(self) -> (Self::Left, Self::Right);
}

macro_rules! for_each_prefix (
    ($m:ident, [$(($acc:tt),)*], []) => {
        $m!($($acc,)*);
    };
    ($m:ident, [$(($acc:tt),)*], [($arg0:tt), $(($arg:tt),)*]) => {
        $m!($($acc,)*);
        for_each_prefix!($m, [$(($acc),)* ($arg0),], [$(($arg),)*]);
    };
);

macro_rules! split_impl (
    ($(($a:ident, $b:ident),)*) => (
        impl<$($a,)* $($b,)*> Split for ($($a,)* $($b,)*) {
            type Left = ($($a,)*);
            type Right = ($($b,)*);
            #[allow(non_snake_case)]
            fn split(self) -> (Self::Left, Self::Right) {
                match self {
                    ($($a,)* $($b,)*) => (($($a,)*), ($($b,)*))
                }
            }
        }
        impl<$($a,)* $($b,)* TLast> Split for ($($a,)* $($b,)* TLast,) {
            type Left = ($($a,)*);
            type Right = ($($b,)* TLast,);
            #[allow(non_snake_case)]
            fn split(self) -> (Self::Left, Self::Right) {
                match self {
                    ($($a,)* $($b,)* t_last,) => (($($a,)*), ($($b,)* t_last,))
                }
            }
        }
    );
);

for_each_prefix! {
    split_impl,
    [],
    [((T0, T1)), ((T2, T3)), ((T4, T5)), ((T6, T7)), ((T8, T9)), ((T10, T11)), ((T12, T13)), ((T14, T15)),]
}
