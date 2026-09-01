RSpec.describe MyClass do
  it 'creates' do
    described_class.new
  end
end

# RuboCop `described_constant` matches only `describe`, not xdescribe/fdescribe
RSpec.xdescribe OrderAsk do
  it 'builds' do
    OrderAsk.new
  end
end

xdescribe Worker::Matching do
  subject { Worker::Matching.new }
end

# OnlyStaticConstants: true (default) — path prefixes are allowed
RSpec.describe Transfer do
  it 'reads constant' do
    Transfer::BLOCKCHAIN_EXPLORER_URL_TEMPLATES
  end
end

RSpec.describe KeycloakAuth do
  it 'raises' do
    raise KeycloakAuth::TokenExpiredSignatureError
  end
end
