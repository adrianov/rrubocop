RSpec.describe Foo do
  let!(:used) { create(:foo) }
  it 'works' do
    expect(used).to be_present
  end
end
