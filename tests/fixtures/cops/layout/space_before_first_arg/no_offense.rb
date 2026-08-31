expect(response.headers['X-Trace-Id']).to eq(expected_trace_id)
expect(response.headers['X-Span-Id']).to  eq(expected_span_id)
foo bar
