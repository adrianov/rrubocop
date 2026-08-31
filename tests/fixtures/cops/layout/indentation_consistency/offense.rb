def foo
  _x = 1
    _y = 2
    ^^^ Layout/IndentationConsistency: Inconsistent indentation detected.
end

class Bar
  _a = 1
      _b = 2
      ^^^ Layout/IndentationConsistency: Inconsistent indentation detected.
end

if cond
 func
  func
  ^^^^ Layout/IndentationConsistency: Inconsistent indentation detected.
end

# Class reopen / peatio-style bodies are covered in no_offense; keep a
# deliberately mis-indented method body here.
def greet
  a
    b
    ^ Layout/IndentationConsistency: Inconsistent indentation detected.
end
