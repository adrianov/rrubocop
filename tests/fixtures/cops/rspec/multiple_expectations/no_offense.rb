RSpec.describe Foo do
  it 'one expect' do
    expect(1).to eq(1)
  end

  it 'allowed with metadata', :aggregate_failures do
    expect(1).to eq(1)
    expect(2).to eq(2)
  end

  it 'allowed with kw', aggregate_failures: true do
    expect(1).to eq(1)
    expect(2).to eq(2)
  end
end

RSpec.describe Bar, :aggregate_failures do
  it 'inherits from group' do
    expect(1).to eq(1)
    expect(2).to eq(2)
  end
end
