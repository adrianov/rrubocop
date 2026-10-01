def is_even(value)
    ^^^^^^^ Naming/PredicatePrefix: Rename `is_even` to `even?`.
end

def is_even?
    ^^^^^^^^ Naming/PredicatePrefix: Rename `is_even?` to `even?`.
end

def has_house_number? = short_address.match?(/,.*\d/)
    ^^^^^^^^^^^^^^^^^ Naming/PredicatePrefix: Rename `has_house_number?` to `house_number?`.

def have_items?
    ^^^^^^^^^^^ Naming/PredicatePrefix: Rename `have_items?` to `items?`.
end

def does_exist
    ^^^^^^^^^^ Naming/PredicatePrefix: Rename `does_exist` to `exist?`.
end

def self.is_ready?
         ^^^^^^^^^ Naming/PredicatePrefix: Rename `is_ready?` to `ready?`.
end

def is_even!
    ^^^^^^^^ Naming/PredicatePrefix: Rename `is_even!` to `even!?`.
end

define_method(:is_even) { |value| }
              ^^^^^^^^ Naming/PredicatePrefix: Rename `is_even` to `even?`.

define_singleton_method(:has_key) { }
                        ^^^^^^^^ Naming/PredicatePrefix: Rename `has_key` to `key?`.

define_method :is_even do
              ^^^^^^^^ Naming/PredicatePrefix: Rename `is_even` to `even?`.
end
