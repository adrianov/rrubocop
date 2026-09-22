RSpec.describe Order do
  let(:trigger_at_72) { create(:trigger) }
  ^^^ RSpec/IndexedLet: This `let` statement uses `72` in its name. Please give it a meaningful name.
  let(:trigger_at_73) { create(:trigger) }
  ^^^ RSpec/IndexedLet: This `let` statement uses `73` in its name. Please give it a meaningful name.
  let!(:item2) { create(:item) }
  ^^^^ RSpec/IndexedLet: This `let` statement uses `2` in its name. Please give it a meaningful name.
  let!(:item3) { create(:item) }
  ^^^^ RSpec/IndexedLet: This `let` statement uses `3` in its name. Please give it a meaningful name.

  it 'works' do
    expect(trigger_at_72).to be_present
    expect(item2).to be_present
  end

  context 'nested' do
    let(:step_10) { build(:step) }
    ^^^ RSpec/IndexedLet: This `let` statement uses `10` in its name. Please give it a meaningful name.
    let(:step_20) { build(:step) }
    ^^^ RSpec/IndexedLet: This `let` statement uses `20` in its name. Please give it a meaningful name.
  end
end
