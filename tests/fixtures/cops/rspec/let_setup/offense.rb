RSpec.describe Foo do
  let!(:unused) { create(:foo) }
  ^^^^ RSpec/LetSetup: Do not use `let!` to setup objects not referenced in tests.
  it 'works' do
    expect(1).to eq(1)
  end
end
