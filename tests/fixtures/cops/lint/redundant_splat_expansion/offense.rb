a = *[1, 2, 3]
    ^^^^^^^^^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.

a = *Array.new(3)
    ^^^^^^^^^^^^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.

link1 = fu_clean_components(*Array.new([n, 0].max, '..'), *srcdirs[i..-1])
                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.
