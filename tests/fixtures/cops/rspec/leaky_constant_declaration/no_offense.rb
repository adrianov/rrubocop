FOO = 1

class Bar
end

RSpec.describe 'x' do
  let(:foo) { 1 }

  before do
    stub_const('FOO', 1)
  end
end
