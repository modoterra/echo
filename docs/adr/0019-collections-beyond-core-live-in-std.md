# Collections Beyond Core Live In Std

Echo keeps core collection syntax limited to PHP arrays, Echo arrays, fixed arrays, lists, tuples, and structural objects; maps, dictionaries, sets, ordered maps, and similar containers live in `std.containers` instead of gaining dedicated literal syntax. The runtime may provide efficient data structures that stdlib containers build on, but those implementation choices should not force new syntax or special semantic lowering into the language core.
