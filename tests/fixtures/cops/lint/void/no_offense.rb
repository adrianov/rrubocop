# Last expression in block is return value (trailing disable comment is not a stmt)
FactoryBot.define do
  trait :tid do
    sequence :tid do |n|
      "#{n}" # rubocop:disable Style/RedundantInterpolation
    end
  end
end

def returns_literal
  42
end
