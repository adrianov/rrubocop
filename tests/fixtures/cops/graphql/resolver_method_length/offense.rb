# frozen_string_literal: true

class UserType < ::Types::BaseObject
  field :bio, String, null: true

  def bio
  ^^^ GraphQL/ResolverMethodLength: ResolverMethod has too many lines. [11/10]
    line1
    line2
    line3
    line4
    line5
    line6
    line7
    line8
    line9
    line10
    line11
  end
end
