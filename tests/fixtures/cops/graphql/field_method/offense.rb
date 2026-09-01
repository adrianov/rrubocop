# frozen_string_literal: true

class UserType < ::Types::BaseObject
  field :phone, String, null: true
  ^^^^^ GraphQL/FieldMethod: Use method: :home_phone

  def phone
    object.home_phone
  end
end
