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
