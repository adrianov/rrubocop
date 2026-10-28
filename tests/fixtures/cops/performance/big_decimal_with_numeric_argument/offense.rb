BigDecimal(1.2, 3, exception: true)
           ^^^ Performance/BigDecimalWithNumericArgument: Convert float literal to string and pass it to `BigDecimal`.
4.5.to_d(6, exception: true)
^^^ Performance/BigDecimalWithNumericArgument: Convert float literal to string and pass it to `BigDecimal`.
BigDecimal('1', 2)
           ^^^ Performance/BigDecimalWithNumericArgument: Convert string literal to integer and pass it to `BigDecimal`.
BigDecimal("4", 6)
           ^^^ Performance/BigDecimalWithNumericArgument: Convert string literal to integer and pass it to `BigDecimal`.
'4'.to_d(6)
^^^ Performance/BigDecimalWithNumericArgument: Convert string literal to integer and pass it to `BigDecimal`.
let(:snapshot_entries) { { product_a.id => BigDecimal('0') } }
                                                      ^^^ Performance/BigDecimalWithNumericArgument: Convert string literal to integer and pass it to `BigDecimal`.
BigDecimal -1.5
           ^^^^ Performance/BigDecimalWithNumericArgument: Convert float literal to string and pass it to `BigDecimal`.
