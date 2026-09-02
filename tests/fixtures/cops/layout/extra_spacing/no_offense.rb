# AllowForAlignment: same column has a word start on another line.
name     = "RuboCop"
website  = "https://example.com"

@last_success_timestamp = register_gauge(:periodic_job_last_success_timestamp,
                                         'Unix timestamp of last successful run')
@last_run_timestamp     = register_gauge(:periodic_job_last_run_timestamp,
                                         'Unix timestamp of last start')
@last_finish_timestamp  = register_gauge(:periodic_job_last_finish_timestamp,
                                         'Unix timestamp of last finish')

allow(Ofd::FetchReceiptInfo).to          receive(:call).with(attrs: parsed)
allow(Ofd::ReceiptLink).to               receive(:call).with(params: parsed)

# Aligned with `=` of `||=` on the previous line (RuboCop equals-operator alignment).
self.content_type ||= 'application/csv'
self.response_body  = csv.respond_to?(:to_csv) ? csv.to_csv : csv

# Multiline hash key/value gaps are Layout/HashAlignment's job.
{
  images:  nil,
  to_time:    '30-12-2020 00:00:00',
  old_price:   99,
  price:       33
}

# Display-column word start on a UTF-8 line (RuboCop indexes by characters, not bytes).
it 'не проходит проверку, когда размер файла больше 5 МБ' do
  preview_card = build(:preview_card,  image_path: 'spec/fixtures/big_image.png')
end

# Single space before trailing comment is fine.
object.method(arg) # comment

# Aligned trailing comments (AllowForAlignment).
object.method(arg)         # comment one
another_object.method(arg) # comment two
