foo unless !bar
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
foo unless x != y
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
foo unless x >= 10
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
foo unless x.even?
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
foo unless x != y || x.even?
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
unless ::Exchanger::MARKET_ORDER_TYPES.include?(type)
^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
  :price_not_allowed
end
foo unless (x.any?)
    ^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
unless CONST < OTHER
^^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.
  :skip
end
