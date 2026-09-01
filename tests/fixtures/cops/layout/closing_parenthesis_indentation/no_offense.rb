# First arg same line, later args not lined up — `)` at line indent
emit_debug_event("action_mailer.processed",
  mailer: event.payload[:mailer],
  action: event.payload[:action],
)

# First arg on next line — `)` outdented
some_method(
  a,
  b
)
