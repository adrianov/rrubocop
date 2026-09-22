RSpec.describe Order do
  let(:item2) { build(:item) }
  let(:visible_item) { build(:item, visible: true) }
  let!(:v2_beta) { build(:version, beta: true) }

  it 'works' do
    expect(item2).to be_present
    expect(visible_item).to be_present
    expect(v2_beta).to be_present
  end
end

RSpec.describe Deposit do
  let(:md5) { Digest::MD5.hexdigest('x') }
  let(:sha256) { Digest::SHA256.hexdigest('x') }

  it 'hashes' do
    expect(md5).to be_present
    expect(sha256).to be_present
  end
end
