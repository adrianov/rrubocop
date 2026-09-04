RSpec.describe 'x' do
  FOO = 1
  ^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.

  class Bar
  ^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub class constant instead of declaring explicitly.
  end

  module Baz
  ^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub module constant instead of declaring explicitly.
  end
end
