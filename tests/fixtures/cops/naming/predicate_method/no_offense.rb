def start_processing?
  previous_status = take_for_retry!
  return false unless previous_status

  enqueue_processing(previous_status)
  true
end

def foo?
  x == y
end

def ==(other)
  hash == other.hash
end

def initialize
  true
end

def call
  foo == bar
end

# unknown return type — conservative mode skips
def maybe?
  bar
end

def maybe
  bar
end

# Nested def returns must not make the outer method look boolean-only.
def outer
  def inner
    return false
    bar
  end
  bar
end

# case with unknown calls — conservative skip (not all-boolean)
def apply_access_policy
  return true if current_user&.admin?

  case access_policy
  when :public
    true
  when :authenticated
    verify_authenticated
  else
    verify_file_access
  end
end

# call+block is Opaque in RuboCop (not call_type?)
def check!(checks)
  checks.all? { |m| send(m) }
end

# bare `return` is nil — not all-boolean
def redeemed_by_issuer
  return unless used

  object.issuer_id == object.recipient_id
end

def write_ref_history
  return true if record.persisted?
  return if !defined?(Sentry) || Rails.env.test?

  false
end

def update_deposit_details(deposit)
  return false if completed?(deposit)
  return
  accept_deposit(deposit)
end
