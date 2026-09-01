if x != nil
  y
end

def numeric_value?(value)
  Float(value.tr(',', '.'), exception: false) != nil
end

if nil == x
  y
end

if x.nil?
  y
end
