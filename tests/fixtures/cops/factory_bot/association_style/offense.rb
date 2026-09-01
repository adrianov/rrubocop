FactoryBot.define do
  factory :asset do
    trait :gateio do
      association :account, :gateio
      ^^^^^^^^^^^ FactoryBot/AssociationStyle: Use implicit style to define associations.
    end
  end
end
