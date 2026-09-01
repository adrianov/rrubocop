RSpec.describe MyClass do
  it 'creates' do
    described_class.new
  end
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
