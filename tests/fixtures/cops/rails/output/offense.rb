puts 'no'
^^^^ Rails/Output: Do not write to stdout. Use Rails's logger if you want to log.
puts
^^^^ Rails/Output: Do not write to stdout. Use Rails's logger if you want to log.
print
^^^^^ Rails/Output: Do not write to stdout. Use Rails's logger if you want to log.
x = puts
    ^^^^ Rails/Output: Do not write to stdout. Use Rails's logger if you want to log.
next puts 'x'.red if y
     ^^^^ Rails/Output: Do not write to stdout. Use Rails's logger if you want to log.
