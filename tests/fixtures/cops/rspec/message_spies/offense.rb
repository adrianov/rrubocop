expect(foo).to receive(:bar)
               ^^^^^^^ RSpec/MessageSpies: Prefer `have_received` for setting message expectations. Setup `foo` as a spy using `allow` or `instance_spy`.

expect(Telemetry::Metrics::VirtualProducts.requests).not_to receive(:add)
                                                            ^^^^^^^ RSpec/MessageSpies: Prefer `have_received` for setting message expectations. Setup `Telemetry::Metrics::VirtualProducts.requests` as a spy using `allow` or `instance_spy`.

expect(cache_counter).to receive(:add).with(service:, force:, result:)
                         ^^^^^^^ RSpec/MessageSpies: Prefer `have_received` for setting message expectations. Setup `cache_counter` as a spy using `allow` or `instance_spy`.

expect(foo).to_not receive(:bar)
                   ^^^^^^^ RSpec/MessageSpies: Prefer `have_received` for setting message expectations. Setup `foo` as a spy using `allow` or `instance_spy`.
