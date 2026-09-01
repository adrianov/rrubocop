RSpec.describe Foo do
  let(:user) { 1 }
  it 'reads' do
    expect(user).to eq(1)
  end
end

# Class.new blocks are exempt (RuboCop valid_usage?)
Class.new do
  def call
    @x
  end
end
