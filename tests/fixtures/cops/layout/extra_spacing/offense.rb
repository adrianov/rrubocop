# Unaligned double spaces — always an offense.
set_app(  "RuboCop")
        ^^ Layout/ExtraSpacing: Unnecessary spacing detected.
website  = "https://example.com"
       ^^ Layout/ExtraSpacing: Unnecessary spacing detected.
private_constant  :ONLY
                ^^ Layout/ExtraSpacing: Unnecessary spacing detected.
object.method(arg)  # unaligned extra space before comment
                  ^^ Layout/ExtraSpacing: Unnecessary spacing detected.
