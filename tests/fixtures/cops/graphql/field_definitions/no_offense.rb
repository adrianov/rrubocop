# frozen_string_literal: true

class Queries < ::Types::BaseObject
  graphql_name 'OtcQueries'

  field :deal, resolver: ::Queries::Otc::Deals::FindQuery # rubocop:disable GraphQL/FieldDescription

  field( # rubocop:disable GraphQL/FieldDescription
    :deals,
    Types::Union::FieldWithAccessError.for(::Types::CustomTypes::Otc::Deals::Connection),
    null: false,
    connection: true,
    resolver: ::Queries::Otc::Deals::ListQuery,
  )

  field :announcement, resolver: ::Queries::Otc::Announcements::FindQuery # rubocop:disable GraphQL/FieldDescription
end
