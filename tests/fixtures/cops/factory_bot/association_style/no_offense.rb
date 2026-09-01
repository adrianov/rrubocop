FactoryBot.define do
  factory :asset do
    trait :gateio do
      account
    end
    # Attributes with args/blocks are not implicit associations.
    name { 'x' }
    email 'a@b.c'
    status(:active)
  end
end
