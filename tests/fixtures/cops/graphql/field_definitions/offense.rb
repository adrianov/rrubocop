# frozen_string_literal: true

class UserType < ::Types::BaseObject
  field :first_name, String, null: true

  def first_name
    object.contact_data.first_name
  end

  field :last_name, String, null: true
  ^^^^^ GraphQL/FieldDefinitions: Group all field definitions together.
end
