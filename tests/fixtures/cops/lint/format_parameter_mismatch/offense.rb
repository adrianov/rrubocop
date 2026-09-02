format('%s %s', 1)
^^^^^^^^^^^^^^^^^^ Lint/FormatParameterMismatch: Number of arguments (1) to `format` doesn't match the number of fields (2).
format('%*d', 1)
^^^^^^^^^^^^^^^^ Lint/FormatParameterMismatch: Number of arguments (1) to `format` doesn't match the number of fields (2).
format('%*.*f', 1, 2)
^^^^^^^^^^^^^^^^^^^^^ Lint/FormatParameterMismatch: Number of arguments (2) to `format` doesn't match the number of fields (3).
