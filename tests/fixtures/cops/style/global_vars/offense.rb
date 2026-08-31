$foo = 1
^^^^ Style/GlobalVars: Do not introduce global variables.

$bar = "hello"
^^^^ Style/GlobalVars: Do not introduce global variables.

x = $custom_global
    ^^^^^^^^^^^^^^ Style/GlobalVars: Do not introduce global variables.

$spec_a, $spec_b = 1, 2
^^^^^^^ Style/GlobalVars: Do not introduce global variables.
         ^^^^^^^ Style/GlobalVars: Do not introduce global variables.

$r, $w = IO.pipe
^^ Style/GlobalVars: Do not introduce global variables.
    ^^ Style/GlobalVars: Do not introduce global variables.
