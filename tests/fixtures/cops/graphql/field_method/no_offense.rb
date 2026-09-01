# frozen_string_literal: true

class MarketOrder < ::Types::BaseObject
  field :id, ID, null: true # rubocop:disable GraphQL/FieldDescription

  def id
    object&.id
  end
end
