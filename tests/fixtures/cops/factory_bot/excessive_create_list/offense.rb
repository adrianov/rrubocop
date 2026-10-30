create_list(:merge_requests, 11)
                             ^^ FactoryBot/ExcessiveCreateList: Avoid using `create_list` with more than 10 items.
create_list(:data_export, 11, :exported, kind: 'product_feed_for_gravity_field')
                          ^^ FactoryBot/ExcessiveCreateList: Avoid using `create_list` with more than 10 items.
FactoryBot.create_list('merge_requests', 1_000, state: :opened)
                                         ^^^^^ FactoryBot/ExcessiveCreateList: Avoid using `create_list` with more than 10 items.
::FactoryGirl.create_list :merge_requests, 0x20
                                           ^^^^ FactoryBot/ExcessiveCreateList: Avoid using `create_list` with more than 10 items.
