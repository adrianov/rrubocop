if x != y
  do_something
end
do_something if x <= 0
unless condition
  do_something
end
if foo.odd?
  bar
end
unless complex_expression(a, b)
  bar
end
# Non-invertible method in a compound condition
foo unless x != y || x.awesome?
# Class inheritance check (mixed-case constant = inheritance)
foo unless x < Foo
foo unless x < Foo::Bar
# Non-invertible methods
foo unless x.empty?
# Safe-navigation calls are not invertible (csend, not send)
foo unless x&.include?(y)
foo unless order&.any?
# Calls with blocks are not invertible (block node, not send)
foo unless items.any? { |i| i.valid? }
foo unless @queue.none? { |_, s, r| s == "spec" }
# Invertible if — cop only applies to unless
foo if !condition
# Empty braces
foo unless ()
# Explicit begin...end conditions are not flagged
foo unless begin x != y end
# Pattern matching guards (case/in) — unless is a guard, not a conditional
case element.name
in "div" unless element.at("div").present?
  element.name = "p"
else
  nil
end
case [hunk, attributes]
in [{ path: path }, { after: }] unless created_at > after
  :skip
else
  :ok
end
foo unless x.empty?
foo unless x.nil?
foo unless x.is_a?(String)
foo unless defined?(x)
foo unless x.any? { |a| a.positive? }
foo unless x&.include?(y)
foo unless x.zero? && y.empty?
foo if x.exclude?(y)
unless Foo < Bar
  :skip
end
unless CONST < Other
  :skip
end
