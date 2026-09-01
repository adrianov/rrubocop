RSpec.describe Foo do
  let!(:used) { create(:foo) }
  it 'works' do
    expect(used).to be_present
  end
end

# Ruby 3.1 shorthand kwargs are method calls (RuboCop `(send nil? %)`)
RSpec.describe Deposit do
  let!(:funds_hold) { create(:funds_hold) }

  it 'releases' do
    expect(release).to have_received(:call).with(funds_hold:)
  end
end

RSpec.shared_context 'api login' do
  let!(:member) { create(:member) }
  let(:api_key) { create(:api_key, member:) }
end
