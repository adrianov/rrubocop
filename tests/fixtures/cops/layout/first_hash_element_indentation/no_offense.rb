h = {
  a: 1
}

func({
       a: 1,
       b: 2
     })

func(x, {
       a: 1
     })

func({ a: 1
     })

func(x, { a: 1
     })

func(x: {
       a: 1,
       b: 2
     },
     y: {
       c: 1,
       d: 2
     })

func(:x, y: {
       a: 1,
       b: 2
     }, z: {
       c: 1,
       d: 2
     })

func(x:
       {
         a: 1,
         b: 2
       },
     y: {
       c: 1,
       d: 2
     })

func x, {
  a: 1, b: 2 }

func a: 1, b: 2

super({
  a: 1
})

yield({
  a: 1
})

return({
  a: 1
})

func x, { a: 1, b: 2 }

expect(result).to eq({
                       filters: [
                         { name: 'Price', type: 'range' }
                       ],
                       count: 2
                     })

stub_request(:get, /x/)
  .to_return(status: 200, body: {
    FreeChoiceLimit: 3, PaidChoiceLimit: 0
  }.to_json)
