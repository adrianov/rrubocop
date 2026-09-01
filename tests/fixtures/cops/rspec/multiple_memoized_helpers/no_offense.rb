RSpec.describe Foo do
  let(:a) { 1 }
  let(:b) { 2 }
  let(:c) { 3 }
  let(:d) { 4 }
  let(:e) { 5 }
end

# AllowSubject: true (default) — subject does not count toward Max 5
RSpec.describe Bar do
  subject(:transfer) { described_class.new }
  let(:a) { 1 }
  let(:b) { 2 }
  let(:c) { 3 }
  let(:d) { 4 }
  let(:e) { 5 }
end

# Redefining the same let name in a nested context does not increase unique count
RSpec.describe Baz do
  let(:a) { 1 }
  let(:b) { 2 }
  let(:c) { 3 }
  let(:d) { 4 }
  let(:e) { 5 }

  context 'nested' do
    let(:a) { 99 }
  end
end
