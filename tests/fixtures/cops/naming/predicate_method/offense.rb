def start_processing
    ^^^^^^^^^^^^^^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  previous_status = take_for_retry!
  return false unless previous_status

  enqueue_processing(previous_status)
  true
end

def foo
    ^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  x == y
end

def bar
    ^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  !x
end

def baz
    ^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  qux?
end

def qux?
    ^^^^ Naming/PredicateMethod: Non-predicate method names should not end with `?`.
  5
end

def message(message)
    ^^^^^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  # trailing comments must not hide the boolean return
  message == payload # true
  # store_message(message['text'])
end

def cc_only_merchant = false
    ^^^^^^^^^^^^^^^^ Naming/PredicateMethod: Predicate method names should end with `?`.

def subscribe(params = {})
    ^^^^^^^^^ Naming/PredicateMethod: Predicate method names should end with `?`.
  response['error'].present? ? false : !!response.dig('result', 'person_id')
end
