RSpec.describe Foo do
  let(:user) { 1 }
  it 'reads' do
    expect(user).to eq(1)
  end
end

# Helper class outside the example group is not searched (RuboCop TopLevelGroup)
class FirekassaTestLogger
  def info(message)
    @messages[:info].push(message)
  end
end

# Support helpers with no example group
class MemoryCache
  def write(key, value)
    @data[key] = value
  end

  def read(key)
    @data[key]
  end
end

# Class.new blocks are exempt (RuboCop valid_usage?)
Class.new do
  def call
    @x
  end
end

# Custom matcher blocks are exempt
RSpec::Matchers.define :have_attr do
  match do |actual|
    @stored = actual.attr
    @stored
  end
end

# Describe wrapped in `if` is not a top-level group
if defined?(SomeGem)
  describe ConditionalSpec do
    it { expect(@foo).to be_empty }
  end
end
