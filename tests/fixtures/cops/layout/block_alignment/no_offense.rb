items.each do |x|
  puts x
end

# do..rescue..end — end aligns with the `.each` line, not the `do` column
Authentication
  .where(provider: :barong)
  .each do |auth|
    next if auth.token.blank?
  rescue => e
    report_exception(e)
  end

# RSpec chain ending with `) do` — end aligns with the do-line indent
expect(AppNotifications::MarkAsViewedAndNotify)
  .to(receive(:call)
  .with(member: user)
  .and_wrap_original) do |*_args|
    run
  end

# Chained `.with {` block — `}` aligns with the do-line indent
expect(WebMock).to have_requested(:post, endpoint)
  .with { |request|
    expect(request.body).to be_present
  }

# Chained `.to raise_error do` — end aligns with the do-line indent
expect { service.create_address }
  .to raise_error do |error|
    expect(error.message).to include("invalid")
  end

# block as argument inside parentheses — end aligns with method inside parens
expect(arr.all? do |o|
         o.valid?
       end)

# Long method chain ending with `.each do` — end aligns with chain root, not `do` column
::Withdraws::Coin.where(currency_id: currency_id, aasm_state: 'succeed')
  .where("data->>'api' = 'local'")
  .each do |withdrawal|
  txid = withdrawal.txid
  process(txid)
end

# `.find_each` at end of long line — end aligns with receiver chain start
ChatMessage
  .where("deal_id IS NOT NULL AND message LIKE '#{message_like}' and kind = 'default'")
  .find_each(batch_size: 1000) do |chat_m|
  chat_m.update_columns(message: '', kind: kind)
end

# `.then do` chained with `.rescue` — end aligns with chain root
client.get_user(member.tgid)
  .then do |user|
  chat_users << user.id
end.rescue { |e| report_exception(e) }.wait

# `.yield_self do` on continuation line — end aligns with chain root
json_rpc(:wallet_propose, { passphrase: secret }).fetch('result')
  .yield_self do |result|
  result.slice('key_type', 'master_seed')
    .symbolize_keys
end

# Parenthesized expression ending with `.each do` — end aligns with `(`
(Consumers::Notifiers::Base.descendants - [Consumers::Notifiers::Mail]).each do |klass|
  expect_any_instance_of(klass).not_to(receive(:notify))
end

# RSpec `it` with multiline keyword args ending in `do`
it 'responds with Result object containing transaction ID',
  :aggregate_failures,
  vcr: 'tazapay/generate_deposit/success' do
  expect(result).to(be_success)
end
