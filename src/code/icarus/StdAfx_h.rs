#![allow(non_snake_case)]

// pragma warning( disable : 4786 )  // identifier was truncated

// pragma warning (push, 3)
// #include <string>
// #include <list>
// #include <vector>
// #include <map>
// #include <algorithm>
// pragma warning (pop)

// using namespace std;

// #define STL_ITERATE( a, b )		for ( a = b.begin(); a != b.end(); a++ )
macro_rules! STL_ITERATE {
    ($iter:ident, $container:expr) => {
        for $iter in $container.iter()
    };
}

// #define STL_INSERT( a, b )		a.insert( a.end(), b );
macro_rules! STL_INSERT {
    ($container:expr, $item:expr) => {
        $container.push($item)
    };
}
