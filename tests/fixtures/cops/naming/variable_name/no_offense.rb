good = 1
@good = 1
@@mock_order_id = 10_000
@@mock_order_id += 1

withdrawal => { id:, apply_time: }
_ = Integer(apply_time)
_ = apply_time.present?

annotate: "path #{__FILE__}"
